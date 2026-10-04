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

pub const HELLO: u8 = 1;
pub const GET_BLOCKS: u8 = 2;
pub const BLOCKS: u8 = 3;

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
    let remote_height = hello.u64().map_err(|err| err.to_string())?;
    let _remote_work = hello.u128().map_err(|err| err.to_string())?;
    if remote_height > chain.height() {
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
        for block in blocks {
            chain.append(block).map_err(|err| err.to_string())?;
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
    let mut hello = Input::new(&payload);
    let remote_height = hello.u64().map_err(|err| err.to_string())?;
    let _remote_work = hello.u128().map_err(|err| err.to_string())?;
    send_hello(stream, chain)?;
    if remote_height < chain.height() {
        let (tag, payload) = recv(stream)?;
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
