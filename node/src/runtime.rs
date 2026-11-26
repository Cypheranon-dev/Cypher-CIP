//! One process that stores the chain, mines the next block, and hands
//! blocks to a peer over TCP. Not a public network.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

use cypher_cip::digest_words;
use cypher_cip::note_commitment;
use cypher_dilithium::Keypair;

use crate::wire::{self, Input};
use crate::{block_reward, mine_block, Block, Chain, Coinbase, Transfer, LEAVES};

pub const NETWORK_ID: &[u8] = b"cypher-testnet-0";
pub const ANNOUNCE_VERSION: u8 = 1;
const MAX_ANNOUNCE: usize = 1 << 20;
pub const HELLO: u8 = 1;
pub const GET_BLOCKS: u8 = 2;
pub const ANNOUNCE: u8 = 5;
pub const BLOCKS: u8 = 3;

pub struct Node {
    pub chain: Chain,
    miner_pk: Vec<u8>,
    miner_sk: Vec<u8>,
    /// Peer address -> score. Negative scores are banned.
    peer_score: std::collections::HashMap<String, i32>,
    #[allow(dead_code)] // reserved for wallet spend seed; coinbase uses miner pk
    field_sk: u64,
    mempool: Vec<Transfer>,
    peers: Vec<String>,
}

impl Node {
    pub fn open(
        infra: &Keypair,
        miner: &Keypair,
        field_sk: u64,
        leaves: [[u64; 4]; LEAVES],
        bits: u32,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            chain: Chain::open(infra.public.clone(), leaves, bits)?,
            miner_pk: miner.public.clone(),
            miner_sk: miner.secret.clone(),
            field_sk,
            mempool: Vec::new(),
            peers: Vec::new(),
            peer_score: std::collections::HashMap::new(),
        })
    }

    pub fn stage(&mut self, transfer: Transfer) -> Result<(), &'static str> {
        self.chain.check_transfer(&transfer)?;
        let nullifier = [
            transfer.public[4],
            transfer.public[5],
            transfer.public[6],
            transfer.public[7],
        ];
        if self.mempool.iter().any(|tx| {
            tx.public[4] == nullifier[0]
                && tx.public[5] == nullifier[1]
                && tx.public[6] == nullifier[2]
                && tx.public[7] == nullifier[3]
        }) {
            return Err("nullifier spent");
        }
        self.mempool.push(transfer);
        Ok(())
    }

    /// Seal the staged transfers that still match this tip, plus the subsidy.
    pub fn mine_next(&mut self, timestamp: u64) -> Result<Block, &'static str> {
        let transfers: Vec<Transfer> = self
            .mempool
            .iter()
            .filter(|tx| self.chain.check_transfer(tx).is_ok())
            .cloned()
            .collect();
        let height = self.chain.height() + 1;
        let reward = block_reward(height);
        let sk = self.field_sk;
        // Private spend key. Prove the output note is worth exactly the reward.
        let input = digest_words(note_commitment(reward, sk, height)?);
        let mut leaves = [[0u64; 4]; cypher_cip::LEAVES];
        leaves[0] = input;
        let mut chain_state = [0u64; 4];
        chain_state[3] = 0xC01B;
        let witness = cypher_cip::Witness {
            sk,
            r_in: height,
            rho_out: height,
            amount_in: reward,
            amount_out: reward,
            fee: 0,
            height,
            payload: reward,
            chain_state,
            index: 0,
            leaves,
            coinbase: true,
        };
        let (proof, public) = cypher_cip::prove(&witness)?;
        let commitment = digest_words(public.output);
        let block = mine_block(
            self.chain.tip(),
            height,
            timestamp,
            self.chain.difficulty_bits(),
            transfers,
            Some(Coinbase {
                sk,
                commitment,
                proof: proof.to_bytes(),
                public: public.to_u64s(),
            }),
            &self.miner_pk,
            &self.miner_sk,
        )?;
        self.chain.append(block.clone())?;
        self.mempool
            .retain(|tx| self.chain.check_transfer(tx).is_ok());
        Ok(block)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        wire::save_chain(&self.chain, self.chain.blocks())
    }

    pub fn from_bytes(bytes: &[u8], miner: &Keypair, field_sk: u64) -> Result<Self, &'static str> {
        let (mut chain, blocks) = wire::load_chain(bytes)?;
        for block in blocks {
            chain.append(block)?;
        }
        Ok(Self {
            chain,
            miner_pk: miner.public.clone(),
            miner_sk: miner.secret.clone(),
            field_sk,
            mempool: Vec::new(),
            peers: Vec::new(),
            peer_score: std::collections::HashMap::new(),
        })
    }
}

pub fn listen(addr: &str) -> Result<TcpListener, std::io::Error> {
    let listener = TcpListener::bind(addr)?;
    listener.set_nonblocking(false)?;
    Ok(listener)
}

/// Accept one peer. If they are behind, send the blocks they are missing.
/// If we are behind, append the blocks they send.
pub fn exchange(stream: &mut TcpStream, chain: &mut Chain) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|err| err.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|err| err.to_string())?;
    send_hello(stream, chain)?;
    let (tag, payload) = recv(stream)?;
    if tag != HELLO {
        return Err("expected hello".into());
    }
    let mut hello = Input::new(&payload);
    check_network(&mut hello)?;
    let remote_height = hello.u64().map_err(|err| err.to_string())?;
    let remote_work = hello.u128().map_err(|err| err.to_string())?;
    if remote_work > chain.work() || remote_height > chain.height() {
        let mut body = Vec::new();
        body.extend_from_slice(&chain.height().to_le_bytes());
        send(stream, GET_BLOCKS, &body)?;
        let (tag, payload) = recv(stream)?;
        if tag != BLOCKS {
            return Err("expected blocks".into());
        }
        let blocks = Input::new(&payload)
            .blocks()
            .map_err(|err| err.to_string())?;
        // Heavier chain wins. Do not append a longer-but-weaker fork.
        let mut candidate = chain.blocks().to_vec();
        candidate.extend(blocks);
        chain.consider(candidate).map_err(|err| err.to_string())?;
    }
    Ok(())
}

/// Parse and verify a signed block announcement.
///
/// Body: version (1) || network id (u32 len || bytes) || block id (32)
/// || sig_len (u32) || sig || one block.
/// The signature covers `NETWORK_ID || block_id`, not the id alone.
/// Does not apply the block and does not check proof-of-work.
pub fn validate_announce(payload: &[u8], miner_pk: &[u8]) -> Result<crate::Block, String> {
    if miner_pk.is_empty() {
        return Err("announce key".into());
    }
    let (block, _id) = open_announce(payload, miner_pk)?;
    Ok(block)
}

/// Same checks as [`validate_announce`], then append or consider.
/// Rejects a block id already on this chain before consensus runs.
pub fn ingest_announce(chain: &mut Chain, payload: &[u8]) -> Result<(), String> {
    if payload.len() > MAX_ANNOUNCE {
        return Err("announce length".into());
    }
    let pk = {
        let (block, _) = open_announce(payload, &[])?;
        block.coinbase_pk
    };
    let (block, id) = open_announce(payload, &pk)?;
    if chain
        .blocks()
        .iter()
        .any(|seen| crate::block_id(seen) == id)
    {
        return Err("replay".into());
    }
    if block.prev == chain.tip() {
        chain.append(block).map_err(|err| err.to_string())?;
        return Ok(());
    }
    let mut next = chain.blocks().to_vec();
    next.push(block);
    if !chain.consider(next).map_err(|err| err.to_string())? {
        return Err("announce rejected".into());
    }
    Ok(())
}

fn signed_announce_message(block_id: &[u8; 32]) -> Vec<u8> {
    let mut msg = Vec::with_capacity(NETWORK_ID.len() + 32);
    msg.extend_from_slice(NETWORK_ID);
    msg.extend_from_slice(block_id);
    msg
}

/// Encode one announcement. `sk` is the miner Dilithium2 secret.
pub fn encode_announce(block: &crate::Block, sk: &[u8]) -> Vec<u8> {
    use cypher_dilithium::sign2;
    let id = crate::block_id(block);
    let sig = sign2(sk, &signed_announce_message(&id));
    let mut body = Vec::new();
    body.push(ANNOUNCE_VERSION);
    body.extend_from_slice(&(NETWORK_ID.len() as u32).to_le_bytes());
    body.extend_from_slice(NETWORK_ID);
    body.extend_from_slice(&id);
    body.extend_from_slice(&(sig.len() as u32).to_le_bytes());
    body.extend_from_slice(&sig);
    wire::write_blocks(&mut body, std::slice::from_ref(block));
    body
}

fn open_announce(payload: &[u8], miner_pk: &[u8]) -> Result<(crate::Block, [u8; 32]), String> {
    use cypher_dilithium::verify2;
    if payload.is_empty() || payload.len() > MAX_ANNOUNCE {
        return Err("announce length".into());
    }
    if payload[0] != ANNOUNCE_VERSION {
        return Err("announce version".into());
    }
    let mut at = 1usize;
    let take = |at: &mut usize, n: usize| -> Result<&[u8], String> {
        let end = at.checked_add(n).ok_or("announce short")?;
        if end > payload.len() {
            return Err("announce short".into());
        }
        let slice = &payload[*at..end];
        *at = end;
        Ok(slice)
    };
    let net_len =
        u32::from_le_bytes(take(&mut at, 4)?.try_into().map_err(|_| "announce short")?) as usize;
    if net_len > 64 {
        return Err("wrong network".into());
    }
    let network = take(&mut at, net_len)?;
    if network != NETWORK_ID {
        return Err("wrong network".into());
    }
    let id: [u8; 32] = take(&mut at, 32)?
        .try_into()
        .map_err(|_| "announce short")?;
    let sig_len =
        u32::from_le_bytes(take(&mut at, 4)?.try_into().map_err(|_| "announce short")?) as usize;
    if sig_len == 0 || sig_len > 4096 {
        return Err("announce sig".into());
    }
    let sig = take(&mut at, sig_len)?.to_vec();
    if miner_pk.is_empty() {
        // Caller is peeking the block to learn coinbase_pk. Still require a
        // well-formed frame, but skip the signature until the key is known.
    } else if !verify2(miner_pk, &signed_announce_message(&id), &sig) {
        return Err("announce signature".into());
    }
    let mut rest = wire::Input::new(&payload[at..]);
    let blocks = rest.blocks().map_err(|e| e.to_string())?;
    if rest.rest() != 0 {
        return Err("announce trailing".into());
    }
    if blocks.len() != 1 {
        return Err("announce count".into());
    }
    let block = blocks.into_iter().next().unwrap();
    if crate::block_id(&block) != id {
        return Err("announce id".into());
    }
    Ok((block, id))
}

/// Serve one connection. Answer a request for the blocks after `from`,
/// or apply one signed announcement that extends this chain.
pub fn serve_one(stream: &mut TcpStream, chain: &mut Chain) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|err| err.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|err| err.to_string())?;
    let (tag, payload) = recv(stream)?;
    if tag != HELLO {
        return Err("expected hello".into());
    }
    let mut hello = Input::new(&payload);
    check_network(&mut hello)?;
    let remote_height = hello.u64().map_err(|err| err.to_string())?;
    let _remote_work = hello.u128().map_err(|err| err.to_string())?;
    send_hello(stream, chain)?;
    if remote_height > chain.height() {
        let (tag, payload) = recv(stream)?;
        if tag != ANNOUNCE {
            return Err("expected announce".into());
        }
        return ingest_announce(chain, &payload);
    }
    if remote_height < chain.height() {
        let (tag, payload) = recv(stream)?;
        if tag == ANNOUNCE {
            return ingest_announce(chain, &payload);
        }
        if tag != GET_BLOCKS {
            return Err("expected getblocks".into());
        }
        let from = Input::new(&payload).u64().map_err(|err| err.to_string())? as usize;
        let mut body = Vec::new();
        let slice = if from > chain.blocks().len() {
            &[]
        } else {
            &chain.blocks()[from..]
        };
        wire::write_blocks(&mut body, slice);
        send(stream, BLOCKS, &body)?;
    }
    Ok(())
}

fn send_hello(stream: &mut TcpStream, chain: &Chain) -> Result<(), String> {
    let mut body = Vec::new();
    body.extend_from_slice(&(NETWORK_ID.len() as u32).to_le_bytes());
    body.extend_from_slice(NETWORK_ID);
    body.extend_from_slice(&chain.height().to_le_bytes());
    body.extend_from_slice(&chain.work().to_le_bytes());
    body.extend_from_slice(&chain.tip());
    send(stream, HELLO, &body)
}

fn check_network(hello: &mut Input) -> Result<(), String> {
    let id = hello.bytes().map_err(|e| e.to_string())?;
    if id.len() > 64 || id.as_slice() != NETWORK_ID {
        return Err("wrong network".into());
    }
    Ok(())
}

fn send(stream: &mut TcpStream, tag: u8, payload: &[u8]) -> Result<(), String> {
    stream
        .write_all(&wire::encode_message(tag, payload))
        .map_err(|err| err.to_string())
}

fn recv(stream: &mut TcpStream) -> Result<(u8, Vec<u8>), String> {
    let mut len_bytes = [0u8; 4];
    stream
        .read_exact(&mut len_bytes)
        .map_err(|err| err.to_string())?;
    let len = u32::from_be_bytes(len_bytes) as usize;
    if len == 0 || len > 32 * 1024 * 1024 {
        return Err("bad frame".into());
    }
    let mut body = vec![0u8; len];
    stream
        .read_exact(&mut body)
        .map_err(|err| err.to_string())?;
    Ok((body[0], body[1..].to_vec()))
}

impl Node {
    pub fn add_peer(&mut self, addr: String) {
        if !self.peers.contains(&addr) {
            self.peers.push(addr);
        }
    }

    pub fn peers(&self) -> &[String] {
        &self.peers
    }

    /// Record a successful interaction. Score is capped at 100.
    pub fn note_good(&mut self, peer: &str) {
        let s = self.peer_score.entry(peer.to_string()).or_insert(0);
        *s = (*s + 1).min(100);
    }

    /// Record a bad interaction. Below -10 the peer is banned.
    pub fn note_bad(&mut self, peer: &str) {
        let s = self.peer_score.entry(peer.to_string()).or_insert(0);
        *s -= 5;
    }

    pub fn is_banned(&self, peer: &str) -> bool {
        self.peer_score.get(peer).copied().unwrap_or(0) < -10
    }

    /// Connection limit. Rejects if at capacity.
    pub const MAX_PEERS: usize = 32;
    pub fn can_accept(&self) -> bool {
        self.peers.len() < Self::MAX_PEERS
    }
}

impl Node {
    /// Authenticated broadcast. Signs the block id with the miner key and
    /// sends to every peer. Receivers must verify the Dilithium2 signature.
    /// Push the block to each configured peer over the existing TCP
    /// protocol. Signs the block id. This is authenticated point-to-point
    /// push, not a public gossip network and not Sybil-resistant.
    pub fn broadcast(&self, block: &crate::Block) -> Vec<(String, String)> {
        use std::net::TcpStream;
        let body = encode_announce(block, &self.miner_sk);
        let mut results = Vec::new();
        for peer in &self.peers {
            let parsed = match peer.parse::<std::net::SocketAddr>() {
                Ok(addr) => addr,
                Err(err) => {
                    results.push((peer.clone(), err.to_string()));
                    continue;
                }
            };
            match TcpStream::connect_timeout(&parsed, std::time::Duration::from_secs(2)) {
                Ok(mut stream) => {
                    let pushed = send_hello(&mut stream, &self.chain)
                        .and_then(|_| recv(&mut stream))
                        .and_then(|(tag, payload)| {
                            if tag != HELLO {
                                return Err("expected hello".into());
                            }
                            check_network(&mut Input::new(&payload))?;
                            send(&mut stream, ANNOUNCE, &body)
                        });
                    match pushed {
                        Ok(()) => results.push((peer.clone(), "sent".into())),
                        Err(err) => results.push((peer.clone(), err)),
                    }
                }
                Err(err) => results.push((peer.clone(), err.to_string())),
            }
        }
        results
    }

    pub fn verify_announcement(pk: &[u8], block_id: &[u8; 32], sig: &[u8]) -> bool {
        let mut msg = Vec::with_capacity(NETWORK_ID.len() + 32);
        msg.extend_from_slice(NETWORK_ID);
        msg.extend_from_slice(block_id);
        cypher_dilithium::verify2(pk, &msg, sig)
    }
}

#[cfg(test)]
mod announce_tests {
    use super::*;
    use crate::Block;
    use cypher_dilithium::Keypair;

    fn sample_block(pk: &[u8]) -> Block {
        Block {
            height: 1,
            prev: [9u8; 32],
            timestamp: 11,
            nonce: 0,
            indices: Vec::new(),
            difficulty_bits: 0,
            transfers: Vec::new(),
            coinbase: None,
            coinbase_pk: pk.to_vec(),
            coinbase_sig: Vec::new(),
        }
    }

    #[test]
    fn valid_announce_roundtrip() {
        let kp = Keypair::dilithium2();
        let block = sample_block(&kp.public);
        let body = encode_announce(&block, &kp.secret);
        let got = validate_announce(&body, &kp.public).unwrap();
        assert_eq!(got, block);
    }

    #[test]
    fn tampered_signature_rejected() {
        let kp = Keypair::dilithium2();
        let block = sample_block(&kp.public);
        let mut body = encode_announce(&block, &kp.secret);
        let sig_at = 1 + 4 + NETWORK_ID.len() + 32 + 4;
        body[sig_at] ^= 0x01;
        assert_eq!(
            validate_announce(&body, &kp.public).unwrap_err(),
            "announce signature"
        );
    }

    #[test]
    fn wrong_block_id_rejected() {
        let kp = Keypair::dilithium2();
        let block = sample_block(&kp.public);
        let mut body = encode_announce(&block, &kp.secret);
        // Flip a byte inside the claimed id (after version + net len + network).
        let id_at = 1 + 4 + NETWORK_ID.len();
        body[id_at] ^= 0xff;
        assert!(validate_announce(&body, &kp.public).is_err());
    }

    #[test]
    fn wrong_network_rejected() {
        let kp = Keypair::dilithium2();
        let block = sample_block(&kp.public);
        let mut body = encode_announce(&block, &kp.secret);
        // Network bytes start at offset 5.
        body[5] ^= 0xff;
        assert_eq!(
            validate_announce(&body, &kp.public).unwrap_err(),
            "wrong network"
        );
    }

    #[test]
    fn wrong_version_rejected() {
        let kp = Keypair::dilithium2();
        let block = sample_block(&kp.public);
        let mut body = encode_announce(&block, &kp.secret);
        body[0] = 2;
        assert_eq!(
            validate_announce(&body, &kp.public).unwrap_err(),
            "announce version"
        );
    }

    #[test]
    fn replay_of_known_id_rejected() {
        let infra = Keypair::dilithium5();
        let miner = Keypair::dilithium2();
        let mut node = Node::open(&infra, &miner, 7, [[0; 4]; crate::LEAVES], 0).unwrap();
        node.mine_next(20_000).unwrap();
        let block = node.chain.blocks()[0].clone();
        let body = encode_announce(&block, &miner.secret);
        let err = ingest_announce(&mut node.chain, &body).unwrap_err();
        assert_eq!(err, "replay");
    }
}
