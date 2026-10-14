use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

use cypher_cip::digest_words;
use cypher_cip::note_commitment;
use cypher_dilithium::Keypair;

use crate::wire::{self, Input};
use crate::{block_reward, mine_block, Block, Chain, Coinbase, Transfer, LEAVES};

pub const HELLO: u8 = 1;
pub const GET_BLOCKS: u8 = 2;
pub const BLOCKS: u8 = 3;

fn transfer_nullifier(transfer: &Transfer) -> [u64; 4] {
    [
        transfer.public[4],
        transfer.public[5],
        transfer.public[6],
        transfer.public[7],
    ]
}

fn transfer_output(transfer: &Transfer) -> [u64; 4] {
    [
        transfer.public[8],
        transfer.public[9],
        transfer.public[10],
        transfer.public[11],
    ]
}

fn read_hello(payload: &[u8]) -> Result<(u64, u128, [u8; 32]), String> {
    if payload.len() != 8 + 16 + 32 {
        return Err("bad hello payload".into());
    }
    let height = u64::from_le_bytes(payload[0..8].try_into().map_err(|_| "bad hello".to_string())?);
    let work = u128::from_le_bytes(payload[8..24].try_into().map_err(|_| "bad hello".to_string())?);
    let mut tip = [0u8; 32];
    tip.copy_from_slice(&payload[24..32]);
    Ok((height, work, tip))
}

pub struct Node {
    pub chain: Chain,
    miner_pk: Vec<u8>,
    miner_sk: Vec<u8>,
    field_sk: u64,
    mempool: Vec<Transfer>,
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
        })
    }

    pub fn stage(&mut self, transfer: Transfer) -> Result<(), &'static str> {
        self.chain.check_transfer(&transfer)?;
        let nullifier = transfer_nullifier(&transfer);
        let output = transfer_output(&transfer);
        if self.mempool.iter().any(|tx| {
            transfer_nullifier(tx) == nullifier || transfer_output(tx) == output
        }) {
            return Err("duplicate transfer");
        }
        if self.chain.nullifiers().contains(&nullifier) || self.chain.contains_note(&output) {
            return Err("duplicate transfer");
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
        let commitment = digest_words(note_commitment(
            block_reward(self.chain.height() + 1),
            self.field_sk,
            self.chain.height() + 1,
        )?);
        let block = mine_block(
            self.chain.tip(),
            self.chain.height() + 1,
            timestamp,
            self.chain.difficulty_bits(),
            transfers,
            Some(Coinbase {
                sk: self.field_sk,
                commitment,
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
        })
    }
}

pub fn listen(addr: &str) -> Result<TcpListener, std::io::Error> {
    let listener = TcpListener::bind(addr)?;
    listener.set_nonblocking(false)?;
    Ok(listener)
}

/// Accept one peer. If they are behind, send the blocks they are missing.
/// If we are behind, append the blocks they send only when the candidate chain
/// has strictly greater cumulative work.
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
    let (remote_height, remote_work, remote_tip) = read_hello(&payload)?;

    if remote_height > chain.height() || (remote_height == chain.height() && remote_work > chain.work()) {
        let mut body = Vec::new();
        let from = if remote_height > chain.height() {
            chain.height() as usize
        } else {
            0
        };
        body.extend_from_slice(&(from as u64).to_le_bytes());
        send(stream, GET_BLOCKS, &body)?;

        let (tag, payload) = recv(stream)?;
        if tag != BLOCKS {
            return Err("expected blocks".into());
        }
        let blocks = Input::new(&payload)
            .blocks()
            .map_err(|err| err.to_string())?;
        let accepted = chain
            .consider(blocks)
            .map_err(|err| err.to_string())?;
        if !accepted && remote_tip != chain.tip() && remote_height >= chain.height() {
            return Err("rejected weaker peer chain".into());
        }
    }
    Ok(())
}

/// Serve one connection. Answer a request for the blocks after `from`.
pub fn serve_one(stream: &mut TcpStream, chain: &Chain) -> Result<(), String> {
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

    let (remote_height, _remote_work, remote_tip) = read_hello(&payload)?;
    send_hello(stream, chain)?;

    if remote_height < chain.height()
        || (remote_height == chain.height() && remote_tip != chain.tip())
    {
        let from = if remote_height < chain.height() {
            remote_height as usize
        } else {
            0
        };
        let (tag, payload) = recv(stream)?;
        if tag != GET_BLOCKS {
            return Err("expected getblocks".into());
        }
        let request = Input::new(&payload).u64().map_err(|err| err.to_string())? as usize;
        let offset = if request > 0 { request } else { from };
        let mut body = Vec::new();
        let slice = if offset > chain.blocks().len() {
            &[]
        } else {
            &chain.blocks()[offset..]
        };
        wire::write_blocks(&mut body, slice);
        send(stream, BLOCKS, &body)?;
    }
    Ok(())
}

fn send_hello(stream: &mut TcpStream, chain: &Chain) -> Result<(), String> {
    let mut body = Vec::new();
    body.extend_from_slice(&chain.height().to_le_bytes());
    body.extend_from_slice(&chain.work().to_le_bytes());
    body.extend_from_slice(&chain.tip());
    send(stream, HELLO, &body)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{block_id, chain_words, mine_block};
    use cypher_cip::{digest_words, note_commitment, prove, Witness, LEAVES};
    use cypher_dilithium::{sign2, sign5, Keypair};

    fn funded(chain: [u64; 4], rho_out: u64) -> Witness {
        let note = digest_words(note_commitment(10, 11, 22).unwrap());
        let mut leaves = [[0u64; 4]; LEAVES];
        leaves[3] = note;
        Witness {
            sk: 11,
            r_in: 22,
            rho_out,
            amount_in: 10,
            amount_out: 9,
            fee: 1,
            height: 1,
            payload: 99,
            chain_state: chain,
            index: 3,
            leaves,
        }
    }

    fn signed_transfer(
        infra: &Keypair,
        user: &Keypair,
        witness: &Witness,
    ) -> (Transfer, [[u64; 4]; LEAVES]) {
        let (proof, public) = prove(witness).expect("prove");
        let mut transfer = Transfer {
            proof: proof.to_bytes(),
            public: public.to_u64s(),
            user_pk: user.public.clone(),
            user_sig: Vec::new(),
            envelope_sig: Vec::new(),
        };
        transfer.user_sig = sign2(&user.secret, &crate::user_message(&transfer));
        transfer.envelope_sig = sign5(&infra.secret, &crate::envelope_message(&transfer));
        (transfer, witness.leaves)
    }

    #[test]
    fn stage_rejects_duplicate_nullifiers_and_outputs() {
        let infra = Keypair::dilithium5();
        let user = Keypair::dilithium2();
        let miner = Keypair::dilithium2();
        let witness = funded(chain_words(&[0u8; 32]), 77);
        let (transfer, leaves) = signed_transfer(&infra, &user, &witness);
        let mut node = Node::open(&infra, &miner, 7, leaves, 0).unwrap();

        node.stage(transfer.clone()).unwrap();
        assert_eq!(node.stage(transfer.clone()).err(), Some("duplicate transfer"));

        let mut replay = transfer.clone();
        replay.public[8] ^= 1;
        replay.user_sig = sign2(&user.secret, &crate::user_message(&replay));
        replay.envelope_sig = sign5(&infra.secret, &crate::envelope_message(&replay));
        assert_eq!(node.stage(replay).err(), Some("duplicate transfer"));
    }

    #[test]
    fn longer_lower_work_forks_are_rejected() {
        let infra = Keypair::dilithium5();
        let miner = Keypair::dilithium2();
        let mut chain = Chain::open(infra.public.clone(), [[0; 4]; LEAVES], 4).unwrap();

        let first = mine_block(
            [0; 32],
            1,
            1_000,
            4,
            Vec::new(),
            None,
            &miner.public,
            &miner.secret,
        )
        .unwrap();
        chain.append(first.clone()).unwrap();

        let weak_one = mine_block(
            [0; 32],
            1,
            2_000,
            0,
            Vec::new(),
            None,
            &miner.public,
            &miner.secret,
        )
        .unwrap();
        let weak_two = mine_block(
            block_id(&weak_one),
            2,
            2_100,
            0,
            Vec::new(),
            None,
            &miner.public,
            &miner.secret,
        )
        .unwrap();

        assert!(!chain.consider(vec![weak_one, weak_two]).unwrap());
    }
}

