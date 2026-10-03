//! In-process chain. One process, not a public P2P network, and not
//! cypheranon.com. Block timestamps are caller-supplied. This process
//! does not produce a block every 120 seconds.
//!
//! A block is accepted only when Equihash(48, 5) verifies, the solution
//! commitment meets the bit target, every transfer's CIP proof verifies,
//! the Dilithium2 user signature and the Dilithium5 envelope verify, and
//! the nullifier is new. Equihash(48, 5) is not Equihash-512. The in-circuit
//! hash is a^3 + 3b^3 + 7, not Poseidon2. The field secret inside the
//! circuit is not the Dilithium key.

use std::collections::BTreeSet;

use blake2b_simd::Params;
use cypher_cip::{verify_bytes, PublicInputs};
use cypher_dilithium::{verify2, verify5};
use cypher_equihash::{commit, leading_zero_bits, verify, Instance, SOLVED_K, SOLVED_N};

const WINDOW: usize = 500;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transfer {
    pub proof: Vec<u8>,
    pub public: [u64; 4],
    pub user_pk: Vec<u8>,
    pub user_sig: Vec<u8>,
    pub envelope_sig: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub height: u64,
    pub prev: [u8; 32],
    pub timestamp: u64,
    pub nonce: u32,
    pub indices: Vec<u32>,
    pub difficulty_bits: u32,
    pub transfers: Vec<Transfer>,
    pub coinbase_pk: Vec<u8>,
    pub coinbase_sig: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Chain {
    pub infra_pk: Vec<u8>,
    pub miner_root: u64,
    pub difficulty_bits: u32,
    blocks: Vec<Block>,
    nullifiers: BTreeSet<u64>,
}

fn blake32(parts: &[&[u8]]) -> [u8; 32] {
    let mut state = Params::new()
        .hash_length(32)
        .personal(b"CypherNode")
        .to_state();
    for part in parts {
        state.update(part);
    }
    let hash = state.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(hash.as_bytes());
    out
}

fn encode_public(public: &[u64; 4]) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, value) in public.iter().enumerate() {
        out[i * 8..i * 8 + 8].copy_from_slice(&value.to_le_bytes());
    }
    out
}

pub fn envelope_message(transfer: &Transfer) -> Vec<u8> {
    let mut message = transfer.proof.clone();
    message.extend_from_slice(&transfer.public[2].to_le_bytes());
    message
}

pub fn user_message(transfer: &Transfer) -> Vec<u8> {
    let mut message = transfer.proof.clone();
    message.extend_from_slice(&encode_public(&transfer.public));
    message
}

pub fn pow_prefix(
    prev: &[u8; 32],
    height: u64,
    timestamp: u64,
    difficulty_bits: u32,
    transfers: &[Transfer],
) -> Vec<u8> {
    let mut prefix = Vec::new();
    prefix.extend_from_slice(prev);
    prefix.extend_from_slice(&height.to_le_bytes());
    prefix.extend_from_slice(&timestamp.to_le_bytes());
    prefix.extend_from_slice(&difficulty_bits.to_le_bytes());
    for transfer in transfers {
        let tx = blake32(&[&transfer.proof, &encode_public(&transfer.public)]);
        prefix.extend_from_slice(&tx);
    }
    prefix
}

pub fn block_id(block: &Block) -> [u8; 32] {
    let mut indices = Vec::with_capacity(block.indices.len() * 4);
    for index in &block.indices {
        indices.extend_from_slice(&index.to_le_bytes());
    }
    blake32(&[
        &block.prev,
        &block.height.to_le_bytes(),
        &block.timestamp.to_le_bytes(),
        &block.nonce.to_le_bytes(),
        &indices,
        &block.coinbase_pk,
        &block.coinbase_sig,
    ])
}

fn solved_pow() -> Instance {
    Instance::new(SOLVED_N, SOLVED_K).expect("equihash(48, 5)")
}

fn work_of(bits: u32) -> Result<u128, &'static str> {
    if bits >= 127 {
        return Err("difficulty shift");
    }
    Ok(1u128 << bits)
}

/// Paper §7 formula: `D_new = D_old * (T_target * 500) / sum(t_i)`.
/// Higher `D` is harder. The paper's `T_target` is 120 seconds. This
/// function is not applied by the in-process chain.
pub fn adjust_difficulty(
    old: u64,
    target_seconds: u64,
    intervals: &[u64],
) -> Result<u64, &'static str> {
    if intervals.len() != WINDOW {
        return Err("difficulty window is 500 intervals");
    }
    let sum: u128 = intervals.iter().map(|t| u128::from(*t)).sum();
    if sum == 0 {
        return Err("zero elapsed");
    }
    let next = u128::from(old) * u128::from(target_seconds) * (WINDOW as u128) / sum;
    u64::try_from(next).map_err(|_| "difficulty overflow")
}

impl Chain {
    pub fn open(infra_pk: Vec<u8>, miner_root: u64, difficulty_bits: u32) -> Self {
        Self {
            infra_pk,
            miner_root,
            difficulty_bits,
            blocks: Vec::new(),
            nullifiers: BTreeSet::new(),
        }
    }

    pub fn height(&self) -> u64 {
        self.blocks.len() as u64
    }

    pub fn tip(&self) -> [u8; 32] {
        self.blocks.last().map(block_id).unwrap_or([0u8; 32])
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn nullifiers(&self) -> &BTreeSet<u64> {
        &self.nullifiers
    }

    pub fn work(&self) -> u128 {
        self.blocks
            .iter()
            .map(|block| work_of(block.difficulty_bits).expect("stored difficulty"))
            .sum()
    }

    pub fn append(&mut self, block: Block) -> Result<(), &'static str> {
        self.check_block(
            &block,
            &self.nullifiers,
            self.tip(),
            self.height() + 1,
            self.blocks.last().map(|b| b.timestamp),
        )?;
        self.adopt_one(&block);
        Ok(())
    }

    /// Replace the chain only when the candidate has strictly more work.
    pub fn consider(&mut self, blocks: Vec<Block>) -> Result<bool, &'static str> {
        let mut nullifiers = BTreeSet::new();
        let mut prev = [0u8; 32];
        let mut prev_time = None;
        let mut work = 0u128;
        for (i, block) in blocks.iter().enumerate() {
            self.check_block(block, &nullifiers, prev, (i as u64) + 1, prev_time)?;
            work = work
                .checked_add(work_of(block.difficulty_bits)?)
                .ok_or("work overflow")?;
            for transfer in &block.transfers {
                nullifiers.insert(transfer.public[1]);
            }
            prev = block_id(block);
            prev_time = Some(block.timestamp);
        }
        if work <= self.work() {
            return Ok(false);
        }
        self.blocks = blocks;
        self.nullifiers = nullifiers;
        Ok(true)
    }

    fn adopt_one(&mut self, block: &Block) {
        for transfer in &block.transfers {
            self.nullifiers.insert(transfer.public[1]);
        }
        self.blocks.push(block.clone());
    }

    fn check_block(
        &self,
        block: &Block,
        spent: &BTreeSet<u64>,
        prev: [u8; 32],
        height: u64,
        prev_time: Option<u64>,
    ) -> Result<(), &'static str> {
        if block.height != height {
            return Err("height");
        }
        if block.prev != prev {
            return Err("prev");
        }
        if block.difficulty_bits != self.difficulty_bits {
            return Err("difficulty");
        }
        if let Some(previous) = prev_time {
            if block.timestamp <= previous {
                return Err("timestamp");
            }
        }
        let prefix = pow_prefix(
            &block.prev,
            block.height,
            block.timestamp,
            block.difficulty_bits,
            &block.transfers,
        );
        let mut header = prefix.clone();
        header.extend_from_slice(&block.nonce.to_le_bytes());
        let inst = solved_pow();
        if !verify(&header, inst, &block.indices) {
            return Err("equihash rejected");
        }
        if leading_zero_bits(&commit(&header, &block.indices)) < block.difficulty_bits {
            return Err("below target");
        }
        let coinbase_msg = blake32(&[&header, &encode_indices(&block.indices)]);
        if !verify2(&block.coinbase_pk, &coinbase_msg, &block.coinbase_sig) {
            return Err("coinbase signature");
        }
        let mut seen = BTreeSet::new();
        for transfer in &block.transfers {
            let nullifier = transfer.public[1];
            if spent.contains(&nullifier) || !seen.insert(nullifier) {
                return Err("nullifier spent");
            }
            if transfer.public[0] != self.miner_root {
                return Err("miner root");
            }
            if transfer.public[3] != block.height {
                return Err("proof height");
            }
            let public = PublicInputs::from_u64s(transfer.public);
            verify_bytes(&transfer.proof, public)?;
            if !verify2(
                &transfer.user_pk,
                &user_message(transfer),
                &transfer.user_sig,
            ) {
                return Err("user signature");
            }
            if !verify5(
                &self.infra_pk,
                &envelope_message(transfer),
                &transfer.envelope_sig,
            ) {
                return Err("envelope signature");
            }
        }
        Ok(())
    }
}

fn encode_indices(indices: &[u32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(indices.len() * 4);
    for index in indices {
        out.extend_from_slice(&index.to_le_bytes());
    }
    out
}

/// Mine a block on `prev`. The caller has already signed each transfer.
pub fn mine_block(
    prev: [u8; 32],
    height: u64,
    timestamp: u64,
    difficulty_bits: u32,
    transfers: Vec<Transfer>,
    coinbase_pk: &[u8],
    coinbase_sk: &[u8],
) -> Result<Block, &'static str> {
    let prefix = pow_prefix(&prev, height, timestamp, difficulty_bits, &transfers);
    let inst = solved_pow();
    let (nonce, indices) = cypher_equihash::mine(&prefix, inst, difficulty_bits)
        .ok_or("no equihash solution in range")?;
    let mut header = prefix;
    header.extend_from_slice(&nonce.to_le_bytes());
    let coinbase_msg = blake32(&[&header, &encode_indices(&indices)]);
    let coinbase_sig = cypher_dilithium::sign2(coinbase_sk, &coinbase_msg);
    Ok(Block {
        height,
        prev,
        timestamp,
        nonce,
        indices,
        difficulty_bits,
        transfers,
        coinbase_pk: coinbase_pk.to_vec(),
        coinbase_sig,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cypher_cip::{prepare, prove, public_key, Witness};
    use cypher_dilithium::{sign2, sign5, Keypair};

    fn witness(height: u64, r_in: u64) -> Witness {
        let mut leaves = [1u64, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        leaves[3] = public_key(11).as_int();
        Witness {
            sk: 11,
            r_in,
            amount_in: 10,
            amount_out: 9,
            fee: 1,
            height,
            payload: 99,
            chain_state: 5,
            index: 3,
            leaves,
        }
    }

    fn signed_transfer(infra: &Keypair, user: &Keypair, height: u64, r_in: u64) -> (Transfer, u64) {
        let prepared = prepare(&witness(height, r_in)).expect("prepare");
        let root = prepared.public.miner_root.as_int();
        let (proof, public) = prove(&witness(height, r_in)).expect("prove");
        assert_eq!(public.to_u64s()[0], root);
        let proof = proof.to_bytes();
        let mut transfer = Transfer {
            proof,
            public: public.to_u64s(),
            user_pk: user.public.clone(),
            user_sig: Vec::new(),
            envelope_sig: Vec::new(),
        };
        transfer.user_sig = sign2(&user.secret, &user_message(&transfer));
        transfer.envelope_sig = sign5(&infra.secret, &envelope_message(&transfer));
        (transfer, root)
    }

    #[test]
    fn retarget_matches_the_formula() {
        // 120 is the paper's T_target, not an interval this chain runs.
        let slow = vec![120u64; 500];
        assert_eq!(adjust_difficulty(1_000_000, 120, &slow).unwrap(), 1_000_000);
        let fast = vec![60u64; 500];
        assert_eq!(adjust_difficulty(1_000_000, 120, &fast).unwrap(), 2_000_000);
        assert!(adjust_difficulty(1_000_000, 120, &[120u64; 499]).is_err());
    }

    #[test]
    fn local_chain_accepts_and_rejects() {
        let infra = Keypair::dilithium5();
        let user = Keypair::dilithium2();
        let miner = Keypair::dilithium2();
        let started = std::time::Instant::now();
        let (transfer, root) = signed_transfer(&infra, &user, 1, 22);
        let prove_ms = started.elapsed().as_millis();
        let mut chain = Chain::open(infra.public.clone(), root, 4);

        let mut flipped = transfer.clone();
        flipped.envelope_sig[0] ^= 1;
        let bad_sig = mine_block(
            chain.tip(),
            1,
            900,
            4,
            vec![flipped],
            &miner.public,
            &miner.secret,
        )
        .expect("mine flipped");
        assert_eq!(chain.append(bad_sig).err(), Some("envelope signature"));

        let mut wrong = transfer.clone();
        wrong.public[1] ^= 1;
        wrong.user_sig = sign2(&user.secret, &user_message(&wrong));
        wrong.envelope_sig = sign5(&infra.secret, &envelope_message(&wrong));
        let bad = mine_block(
            chain.tip(),
            1,
            950,
            4,
            vec![wrong],
            &miner.public,
            &miner.secret,
        )
        .expect("mine bad proof");
        assert_eq!(chain.append(bad).err(), Some("proof rejected"));

        let block1 = mine_block(
            chain.tip(),
            1,
            1_000,
            4,
            vec![transfer.clone()],
            &miner.public,
            &miner.secret,
        )
        .expect("mine 1");
        chain.append(block1.clone()).expect("append 1");
        assert_eq!(chain.height(), 1);
        assert!(chain.nullifiers().contains(&transfer.public[1]));

        let replay = mine_block(
            chain.tip(),
            2,
            1_100,
            4,
            vec![transfer.clone()],
            &miner.public,
            &miner.secret,
        )
        .expect("mine replay");
        assert_eq!(chain.append(replay).err(), Some("nullifier spent"));

        let block2 = mine_block(
            chain.tip(),
            2,
            1_400,
            4,
            Vec::new(),
            &miner.public,
            &miner.secret,
        )
        .expect("mine 2");
        chain.append(block2).expect("append 2");
        assert_eq!(chain.height(), 2);

        let mut broken = mine_block(
            chain.tip(),
            3,
            1_500,
            4,
            Vec::new(),
            &miner.public,
            &miner.secret,
        )
        .expect("mine broken");
        broken.indices.swap(0, 1);
        assert_eq!(chain.append(broken).err(), Some("equihash rejected"));
        let honest_height = chain.height();
        let honest_work = chain.work();

        let alt1 = mine_block(
            [0u8; 32],
            1,
            2_000,
            4,
            Vec::new(),
            &miner.public,
            &miner.secret,
        )
        .expect("alt 1");
        let alt2 = mine_block(
            block_id(&alt1),
            2,
            2_100,
            4,
            Vec::new(),
            &miner.public,
            &miner.secret,
        )
        .expect("alt 2");
        assert!(!chain.consider(vec![alt1.clone(), alt2.clone()]).unwrap());
        assert_eq!(chain.height(), 2);
        assert!(chain.nullifiers().contains(&transfer.public[1]));

        let alt3 = mine_block(
            block_id(&alt2),
            3,
            2_200,
            4,
            Vec::new(),
            &miner.public,
            &miner.secret,
        )
        .expect("alt 3");
        assert!(chain.consider(vec![alt1, alt2, alt3]).unwrap());
        assert_eq!(chain.height(), 3);
        assert!(chain.nullifiers().is_empty());
        eprintln!(
            "report prove_ms={prove_ms} proof_bytes={} nullifier={} event={} proof_height={} root={} d5_pk={} d5_sig={} d2_pk={} d2_sig={} honest_height={honest_height} honest_work={honest_work} reorg_height={} reorg_work={} eq={}x{}",
            transfer.proof.len(),
            transfer.public[1],
            transfer.public[2],
            transfer.public[3],
            transfer.public[0],
            cypher_dilithium::DILITHIUM5_PK,
            cypher_dilithium::DILITHIUM5_SIG,
            cypher_dilithium::DILITHIUM2_PK,
            cypher_dilithium::DILITHIUM2_SIG,
            chain.height(),
            chain.work(),
            SOLVED_N,
            SOLVED_K,
        );
    }
}
