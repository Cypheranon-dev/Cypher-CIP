//! Length-prefixed chain encoding. Disk and the peer socket use the same bytes.

use crate::{Block, Chain, Coinbase, Transfer, LEAVES, PUBLIC_LEN};

const MAGIC: &[u8] = b"CYPH1";
const MAX_BLOCKS: u64 = 10_000;
const MAX_TRANSFERS: u32 = 1_000;
const MAX_BYTES: u32 = 1_048_576;
const MAX_INDICES: u32 = 10_000;
const MAX_EXTRA_SIGS: u32 = 64;

pub fn save_chain(chain: &Chain, blocks: &[Block]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    let keys = chain.infra_pks();
    out.extend_from_slice(&(keys.len() as u32).to_le_bytes());
    for key in keys {
        write_bytes(&mut out, key);
    }
    for leaf in chain.genesis_leaves() {
        for word in leaf {
            out.extend_from_slice(&word.to_le_bytes());
        }
    }
    out.extend_from_slice(&chain.initial_bits().to_le_bytes());
    out.extend_from_slice(&(chain.threshold() as u32).to_le_bytes());
    out.push(if chain.production() { 1 } else { 0 });
    write_blocks(&mut out, blocks);
    out
}

pub fn load_chain(bytes: &[u8]) -> Result<(Chain, Vec<Block>), &'static str> {
    let mut input = Input::new(bytes);
    if input.take(MAGIC.len())? != MAGIC {
        return Err("not a chain file");
    }
    let nkeys = input.u32()? as usize;
    if nkeys == 0 || nkeys > 64 {
        return Err("infra keys");
    }
    let mut infra_pks = Vec::with_capacity(nkeys);
    for _ in 0..nkeys {
        infra_pks.push(input.bytes()?);
    }
    let mut leaves = [[0u64; 4]; LEAVES];
    for leaf in &mut leaves {
        for word in leaf.iter_mut() {
            *word = input.u64()?;
        }
    }
    let bits = input.u32()?;
    let threshold = input.u32()? as usize;
    let production = input.take(1)?[0] == 1;
    let blocks = input.blocks()?;
    if input.rest() != 0 {
        return Err("trailing chain bytes");
    }
    let mut chain = Chain::open_threshold(infra_pks, leaves, bits)?;
    if threshold > 1 {
        chain.set_threshold(threshold).map_err(|_| "threshold")?;
    }
    if production {
        chain.enable_production().map_err(|_| "production")?;
    }
    Ok((chain, blocks))
}

pub fn write_blocks(out: &mut Vec<u8>, blocks: &[Block]) {
    out.extend_from_slice(&(blocks.len() as u64).to_le_bytes());
    for block in blocks {
        write_block(out, block);
    }
}

pub fn encode_message(tag: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(5 + payload.len());
    let len = (1 + payload.len()) as u32;
    out.extend_from_slice(&len.to_be_bytes());
    out.push(tag);
    out.extend_from_slice(payload);
    out
}

pub struct Input<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Input<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }

    pub fn rest(&self) -> usize {
        self.bytes.len() - self.at
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], &'static str> {
        let end = self.at.checked_add(len).ok_or("short message")?;
        if end > self.bytes.len() {
            return Err("short message");
        }
        let slice = &self.bytes[self.at..end];
        self.at = end;
        Ok(slice)
    }

    pub fn u32(&mut self) -> Result<u32, &'static str> {
        let bytes: [u8; 4] = self.take(4)?.try_into().map_err(|_| "short message")?;
        Ok(u32::from_le_bytes(bytes))
    }

    pub fn u64(&mut self) -> Result<u64, &'static str> {
        let bytes: [u8; 8] = self.take(8)?.try_into().map_err(|_| "short message")?;
        Ok(u64::from_le_bytes(bytes))
    }

    pub fn u128(&mut self) -> Result<u128, &'static str> {
        let bytes: [u8; 16] = self.take(16)?.try_into().map_err(|_| "short message")?;
        Ok(u128::from_le_bytes(bytes))
    }

    pub fn bytes(&mut self) -> Result<Vec<u8>, &'static str> {
        let len = self.u32()?;
        if len > MAX_BYTES {
            return Err("oversize");
        }
        Ok(self.take(len as usize)?.to_vec())
    }

    pub fn blocks(&mut self) -> Result<Vec<Block>, &'static str> {
        let count = self.u64()?;
        if count > MAX_BLOCKS {
            return Err("too many blocks");
        }
        let count = count as usize;
        let mut blocks = Vec::with_capacity(count);
        for _ in 0..count {
            blocks.push(self.block()?);
        }
        Ok(blocks)
    }

    fn block(&mut self) -> Result<Block, &'static str> {
        let height = self.u64()?;
        let mut prev = [0u8; 32];
        prev.copy_from_slice(self.take(32)?);
        let timestamp = self.u64()?;
        let nonce = self.u32()?;
        let index_count = self.u32()?;
        if index_count > MAX_INDICES {
            return Err("too many indices");
        }
        let index_count = index_count as usize;
        let mut indices = Vec::with_capacity(index_count);
        for _ in 0..index_count {
            indices.push(self.u32()?);
        }
        let difficulty_bits = self.u32()?;
        let tx_count = self.u32()?;
        if tx_count > MAX_TRANSFERS {
            return Err("too many transfers");
        }
        let tx_count = tx_count as usize;
        let mut transfers = Vec::with_capacity(tx_count);
        for _ in 0..tx_count {
            transfers.push(self.transfer()?);
        }
        let coinbase = if self.take(1)?[0] == 1 {
            Some(Coinbase {
                sk: 0,
                commitment: [self.u64()?, self.u64()?, self.u64()?, self.u64()?],
                proof: self.bytes()?,
                public: {
                    let mut p = [0u64; crate::PUBLIC_LEN];
                    for word in &mut p {
                        *word = self.u64()?;
                    }
                    p
                },
            })
        } else {
            None
        };
        Ok(Block {
            height,
            prev,
            timestamp,
            nonce,
            indices,
            difficulty_bits,
            transfers,
            coinbase,
            coinbase_pk: self.bytes()?,
            coinbase_sig: self.bytes()?,
        })
    }

    fn transfer(&mut self) -> Result<Transfer, &'static str> {
        let proof = self.bytes()?;
        let mut public = [0u64; PUBLIC_LEN];
        for word in &mut public {
            *word = self.u64()?;
        }
        Ok(Transfer {
            proof,
            public,
            user_pk: self.bytes()?,
            user_sig: self.bytes()?,
            envelope_sig: self.bytes()?,
            extra_envelope_sigs: {
                let n = self.u32()?;
                if n > MAX_EXTRA_SIGS {
                    return Err("too many signatures");
                }
                let n = n as usize;
                let mut extras = Vec::with_capacity(n);
                for _ in 0..n {
                    extras.push(self.bytes()?);
                }
                extras
            },
        })
    }
}

fn write_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(bytes);
}

fn write_block(out: &mut Vec<u8>, block: &Block) {
    out.extend_from_slice(&block.height.to_le_bytes());
    out.extend_from_slice(&block.prev);
    out.extend_from_slice(&block.timestamp.to_le_bytes());
    out.extend_from_slice(&block.nonce.to_le_bytes());
    out.extend_from_slice(&(block.indices.len() as u32).to_le_bytes());
    for index in &block.indices {
        out.extend_from_slice(&index.to_le_bytes());
    }
    out.extend_from_slice(&block.difficulty_bits.to_le_bytes());
    out.extend_from_slice(&(block.transfers.len() as u32).to_le_bytes());
    for transfer in &block.transfers {
        write_bytes(out, &transfer.proof);
        for word in transfer.public {
            out.extend_from_slice(&word.to_le_bytes());
        }
        write_bytes(out, &transfer.user_pk);
        write_bytes(out, &transfer.user_sig);
        write_bytes(out, &transfer.envelope_sig);
        out.extend_from_slice(&(transfer.extra_envelope_sigs.len() as u32).to_le_bytes());
        for sig in &transfer.extra_envelope_sigs {
            write_bytes(out, sig);
        }
    }
    match &block.coinbase {
        Some(coinbase) => {
            out.push(1);
            for word in coinbase.commitment {
                out.extend_from_slice(&word.to_le_bytes());
            }
            write_bytes(out, &coinbase.proof);
            for word in coinbase.public {
                out.extend_from_slice(&word.to_le_bytes());
            }
        }
        None => out.push(0),
    }
    write_bytes(out, &block.coinbase_pk);
    write_bytes(out, &block.coinbase_sig);
}
