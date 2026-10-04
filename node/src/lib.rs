//! In-process chain. One process, not a public P2P network, and not
//! cypheranon.com. Callers supply timestamps. Nothing here waits 120 seconds.
//!
//! A block is accepted only when Equihash(48, 5) verifies, the solution
//! commitment meets the bit target, every transfer's Poseidon2 spend proof
//! verifies, the note root and event match this chain, both Dilithium
//! signatures verify, and the nullifier is new. The output note is then
//! appended. Equihash(48, 5) is not Equihash-512.
//!
//! Coinbase, when present, pays `block_reward(height)` into a note the
//! miner can spend. The bit target is retargeted from the paper's formula
//! once 500 intervals exist, and clamped at 12 bits so this process cannot
//! stall. The paper's `T_target` is 120 seconds. That number is the formula
//! input. It is not a block interval this process produces.

use std::collections::BTreeSet;

use blake2b_simd::Params;
pub use cypher_cip::LEAVES;
use cypher_cip::{
    digest_words, event_digest, note_commitment, root_words, verify_bytes, PublicInputs, PUBLIC_LEN,
};
use cypher_dilithium::{verify2, verify5};
use cypher_equihash::{commit, leading_zero_bits, verify, Instance, SOLVED_K, SOLVED_N};

pub mod runtime;
pub mod wire;

const WINDOW: usize = 500;
const TARGET_SECONDS: u64 = 120;
const MAX_BITS: u32 = 12;
pub const BLOCKS_PER_YEAR: u64 = 262_800;
pub const INITIAL_REWARD: u64 = 1_522_069;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transfer {
    pub proof: Vec<u8>,
    pub public: [u64; PUBLIC_LEN],
    pub user_pk: Vec<u8>,
    pub user_sig: Vec<u8>,
    pub envelope_sig: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Coinbase {
    pub sk: u64,
    pub commitment: [u64; 4],
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
    pub coinbase: Option<Coinbase>,
    pub coinbase_pk: Vec<u8>,
    pub coinbase_sig: Vec<u8>,
}

struct Ledger {
    trees: Vec<[[u64; 4]; LEAVES]>,
    nullifiers: BTreeSet<[u64; 4]>,
    bits: u32,
    prev: [u8; 32],
    prev_time: Option<u64>,
    timestamps: Vec<u64>,
}

#[derive(Clone, Debug)]
pub struct Chain {
    infra_pk: Vec<u8>,
    genesis_leaves: [[u64; 4]; LEAVES],
    initial_bits: u32,
    blocks: Vec<Block>,
    trees: Vec<[[u64; 4]; LEAVES]>,
    nullifiers: BTreeSet<[u64; 4]>,
    bits: u32,
    timestamps: Vec<u64>,
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

fn words4(words: &[u64], at: usize) -> [u64; 4] {
    [words[at], words[at + 1], words[at + 2], words[at + 3]]
}

pub fn chain_words(block_id: &[u8; 32]) -> [u64; 4] {
    let mut out = [0u64; 4];
    for lane in 0..4 {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&block_id[lane * 4..lane * 4 + 4]);
        out[lane] = u32::from_le_bytes(bytes).into();
    }
    out
}

/// Per-block subsidy. Year 0 is `1,522,069`. Later years take
/// `floor(previous * 5 / 8)`, which is the integer form of §12.2.
pub fn block_reward(height: u64) -> u64 {
    if height == 0 {
        return 0;
    }
    let year = (height - 1) / BLOCKS_PER_YEAR;
    let mut reward = u128::from(INITIAL_REWARD);
    for _ in 0..year {
        reward = reward * 5 / 8;
    }
    u64::try_from(reward).unwrap_or(0)
}

fn encode_public(public: &[u64; PUBLIC_LEN]) -> Vec<u8> {
    let mut out = Vec::with_capacity(PUBLIC_LEN * 8);
    for value in public {
        out.extend_from_slice(&value.to_le_bytes());
    }
    out
}

pub fn envelope_message(transfer: &Transfer) -> Vec<u8> {
    let mut message = b"Cypher-CIP/envelope/v1".to_vec();
    message.extend_from_slice(&transfer.proof);
    message.extend_from_slice(&encode_public(&transfer.public));
    message
}

pub fn user_message(transfer: &Transfer) -> Vec<u8> {
    let mut message = b"Cypher-CIP/user/v1".to_vec();
    message.extend_from_slice(&transfer.proof);
    message.extend_from_slice(&encode_public(&transfer.public));
    message
}

fn coinbase_bytes(coinbase: &Option<Coinbase>) -> Vec<u8> {
    let mut out = Vec::new();
    if let Some(coinbase) = coinbase {
        out.extend_from_slice(&coinbase.sk.to_le_bytes());
        for word in coinbase.commitment {
            out.extend_from_slice(&word.to_le_bytes());
        }
    }
    out
}

pub fn pow_prefix(
    prev: &[u8; 32],
    height: u64,
    timestamp: u64,
    difficulty_bits: u32,
    transfers: &[Transfer],
    coinbase: &Option<Coinbase>,
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
    prefix.extend_from_slice(&coinbase_bytes(coinbase));
    prefix
}

pub fn block_id(block: &Block) -> [u8; 32] {
    let prefix = pow_prefix(
        &block.prev,
        block.height,
        block.timestamp,
        block.difficulty_bits,
        &block.transfers,
        &block.coinbase,
    );
    let mut header = prefix;
    header.extend_from_slice(&block.nonce.to_le_bytes());
    let indices = encode_indices(&block.indices);
    blake32(&[
        b"Cypher-CIP/block/v1",
        &header,
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

fn bits_of(difficulty: u64) -> u32 {
    if difficulty <= 1 {
        0
    } else {
        (63 - difficulty.leading_zeros()).min(MAX_BITS)
    }
}

/// Paper §7 formula: `D_new = D_old * (T_target * 500) / sum(t_i)`.
/// Higher `D` is harder. `target_seconds` is the paper's 120 when the
/// chain calls this. The chain stores the bit length of `D`, clamped.
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
    if target_seconds == 0 {
        return Err("zero target");
    }
    let next = u128::from(old)
        .checked_mul(u128::from(target_seconds))
        .and_then(|value| value.checked_mul(WINDOW as u128))
        .ok_or("difficulty overflow")?
        / sum;
    u64::try_from(next).map_err(|_| "difficulty overflow")
}

fn retarget(bits: u32, timestamps: &[u64]) -> Result<u32, &'static str> {
    if timestamps.len() < WINDOW + 1 || !(timestamps.len() - 1).is_multiple_of(WINDOW) {
        return Ok(bits);
    }
    let start = timestamps.len() - (WINDOW + 1);
    let mut intervals = [0u64; WINDOW];
    for i in 0..WINDOW {
        let elapsed = timestamps[start + i + 1]
            .checked_sub(timestamps[start + i])
            .ok_or("timestamp")?;
        if elapsed == 0 {
            return Err("timestamp");
        }
        intervals[i] = elapsed;
    }
    let old = 1u64 << bits;
    Ok(bits_of(adjust_difficulty(old, TARGET_SECONDS, &intervals)?))
}

fn first_empty(leaves: &[[u64; 4]; LEAVES]) -> Option<usize> {
    leaves.iter().position(|leaf| *leaf == [0; 4])
}

fn note_exists(trees: &[[[u64; 4]; LEAVES]], note: &[u64; 4]) -> bool {
    trees.iter().any(|tree| tree.contains(note))
}

fn tree_roots(trees: &[[[u64; 4]; LEAVES]]) -> Result<Vec<[u64; 4]>, &'static str> {
    trees.iter().map(root_words).collect()
}

/// Fill the open tree. When it is full, open another eight-leaf tree.
/// Old roots stay valid because a full tree is not modified again.
fn insert_note(trees: &mut Vec<[[u64; 4]; LEAVES]>, note: [u64; 4]) {
    if trees.last().and_then(first_empty).is_none() {
        trees.push([[0; 4]; LEAVES]);
    }
    let tree = trees.last_mut().expect("a note tree");
    let slot = first_empty(tree).expect("an empty leaf");
    tree[slot] = note;
}

impl Chain {
    pub fn open(
        infra_pk: Vec<u8>,
        leaves: [[u64; 4]; LEAVES],
        difficulty_bits: u32,
    ) -> Result<Self, &'static str> {
        if difficulty_bits > MAX_BITS {
            return Err("difficulty range");
        }
        let _ = root_words(&leaves)?;
        Ok(Self {
            infra_pk,
            genesis_leaves: leaves,
            initial_bits: difficulty_bits,
            blocks: Vec::new(),
            trees: vec![leaves],
            nullifiers: BTreeSet::new(),
            bits: difficulty_bits,
            timestamps: Vec::new(),
        })
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

    pub fn nullifiers(&self) -> &BTreeSet<[u64; 4]> {
        &self.nullifiers
    }

    pub fn contains_note(&self, note: &[u64; 4]) -> bool {
        self.trees.iter().any(|tree| tree.contains(note))
    }

    pub fn note_trees(&self) -> usize {
        self.trees.len()
    }

    pub fn difficulty_bits(&self) -> u32 {
        self.bits
    }

    pub fn work(&self) -> u128 {
        self.blocks
            .iter()
            .map(|block| work_of(block.difficulty_bits).expect("stored difficulty"))
            .sum()
    }

    pub fn append(&mut self, block: Block) -> Result<(), &'static str> {
        let mut ledger = self.snapshot();
        self.apply(&mut ledger, &block)?;
        self.blocks.push(block);
        self.install(ledger);
        Ok(())
    }

    /// Replace the chain only when the candidate has strictly more work.
    pub fn consider(&mut self, blocks: Vec<Block>) -> Result<bool, &'static str> {
        let mut trees = vec![self.genesis_leaves];
        let mut nullifiers = BTreeSet::new();
        let mut bits = self.initial_bits;
        let mut timestamps = Vec::new();
        let mut prev = [0u8; 32];
        let mut prev_time = None;
        let mut work = 0u128;
        for block in &blocks {
            let mut ledger = Ledger {
                trees,
                nullifiers,
                bits,
                prev,
                prev_time,
                timestamps,
            };
            self.apply(&mut ledger, block)?;
            work = work
                .checked_add(work_of(block.difficulty_bits)?)
                .ok_or("work overflow")?;
            trees = ledger.trees;
            nullifiers = ledger.nullifiers;
            bits = ledger.bits;
            timestamps = ledger.timestamps;
            prev = block_id(block);
            prev_time = Some(block.timestamp);
        }
        if work <= self.work() {
            return Ok(false);
        }
        self.blocks = blocks;
        self.trees = trees;
        self.nullifiers = nullifiers;
        self.bits = bits;
        self.timestamps = timestamps;
        Ok(true)
    }

    fn snapshot(&self) -> Ledger {
        Ledger {
            trees: self.trees.clone(),
            nullifiers: self.nullifiers.clone(),
            bits: self.bits,
            prev: self.tip(),
            prev_time: self.blocks.last().map(|block| block.timestamp),
            timestamps: self.timestamps.clone(),
        }
    }

    fn install(&mut self, ledger: Ledger) {
        self.trees = ledger.trees;
        self.nullifiers = ledger.nullifiers;
        self.bits = ledger.bits;
        self.timestamps = ledger.timestamps;
    }

    pub fn infra_pk(&self) -> &[u8] {
        &self.infra_pk
    }

    pub fn genesis_leaves(&self) -> [[u64; 4]; LEAVES] {
        self.genesis_leaves
    }

    pub fn initial_bits(&self) -> u32 {
        self.initial_bits
    }

    /// A transfer the next block can seal: proof, both signatures, an
    /// existing note root, and an event bound to this tip and the next height.
    pub fn check_transfer(&self, transfer: &Transfer) -> Result<(), &'static str> {
        let roots = tree_roots(&self.trees)?;
        let mut seen = BTreeSet::new();
        let mut outputs = Vec::new();
        self.take_transfer(
            &Ledger {
                trees: self.trees.clone(),
                nullifiers: self.nullifiers.clone(),
                bits: self.bits,
                prev: self.tip(),
                prev_time: None,
                timestamps: Vec::new(),
            },
            self.height() + 1,
            &roots,
            &chain_words(&self.tip()),
            transfer,
            &mut seen,
            &mut outputs,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn take_transfer(
        &self,
        ledger: &Ledger,
        height: u64,
        roots: &[[u64; 4]],
        chain: &[u64; 4],
        transfer: &Transfer,
        seen: &mut BTreeSet<[u64; 4]>,
        outputs: &mut Vec<[u64; 4]>,
    ) -> Result<(), &'static str> {
        let public = PublicInputs::try_from_u64s(transfer.public)?;
        let nullifier = words4(&transfer.public, 4);
        if ledger.nullifiers.contains(&nullifier) || !seen.insert(nullifier) {
            return Err("nullifier spent");
        }
        if !roots.contains(&digest_words(public.note_root)) {
            return Err("note root");
        }
        if public.height.as_int() != height {
            return Err("proof height");
        }
        let expected = event_digest(public.payload.as_int(), height, *chain, public.fee.as_int())?;
        if public.event_id != expected {
            return Err("event");
        }
        let output = words4(&transfer.public, 8);
        if output == [0; 4] || note_exists(&ledger.trees, &output) || outputs.contains(&output) {
            return Err("duplicate note");
        }
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
        outputs.push(output);
        Ok(())
    }

    fn apply(&self, ledger: &mut Ledger, block: &Block) -> Result<(), &'static str> {
        if block.height != ledger.timestamps.len() as u64 + 1 {
            return Err("height");
        }
        if block.prev != ledger.prev {
            return Err("prev");
        }
        if block.difficulty_bits != ledger.bits || block.difficulty_bits > MAX_BITS {
            return Err("difficulty");
        }
        if let Some(previous) = ledger.prev_time {
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
            &block.coinbase,
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
        let roots = tree_roots(&ledger.trees)?;
        let chain = chain_words(&ledger.prev);
        let mut seen = BTreeSet::new();
        let mut outputs = Vec::with_capacity(block.transfers.len() + 1);
        for transfer in &block.transfers {
            self.take_transfer(
                ledger,
                block.height,
                &roots,
                &chain,
                transfer,
                &mut seen,
                &mut outputs,
            )?;
        }
        if let Some(coinbase) = &block.coinbase {
            let expected = digest_words(note_commitment(
                block_reward(block.height),
                coinbase.sk,
                block.height,
            )?);
            if coinbase.commitment != expected {
                return Err("coinbase");
            }
            if note_exists(&ledger.trees, &coinbase.commitment)
                || outputs.contains(&coinbase.commitment)
            {
                return Err("duplicate note");
            }
            outputs.push(coinbase.commitment);
        }
        for output in outputs {
            insert_note(&mut ledger.trees, output);
        }
        for nullifier in seen {
            ledger.nullifiers.insert(nullifier);
        }
        ledger.timestamps.push(block.timestamp);
        ledger.bits = retarget(ledger.bits, &ledger.timestamps)?;
        ledger.prev = block_id(block);
        ledger.prev_time = Some(block.timestamp);
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
#[allow(clippy::too_many_arguments)]
pub fn mine_block(
    prev: [u8; 32],
    height: u64,
    timestamp: u64,
    difficulty_bits: u32,
    transfers: Vec<Transfer>,
    coinbase: Option<Coinbase>,
    coinbase_pk: &[u8],
    coinbase_sk: &[u8],
) -> Result<Block, &'static str> {
    if difficulty_bits > MAX_BITS {
        return Err("difficulty range");
    }
    let prefix = pow_prefix(
        &prev,
        height,
        timestamp,
        difficulty_bits,
        &transfers,
        &coinbase,
    );
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
        coinbase,
        coinbase_pk: coinbase_pk.to_vec(),
        coinbase_sig,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
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
        transfer.user_sig = sign2(&user.secret, &user_message(&transfer));
        transfer.envelope_sig = sign5(&infra.secret, &envelope_message(&transfer));
        (transfer, witness.leaves)
    }

    #[test]
    fn retarget_matches_the_formula() {
        let slow = vec![120u64; 500];
        assert_eq!(adjust_difficulty(1_000_000, 120, &slow).unwrap(), 1_000_000);
        let fast = vec![60u64; 500];
        assert_eq!(adjust_difficulty(1_000_000, 120, &fast).unwrap(), 2_000_000);
        assert!(adjust_difficulty(1_000_000, 120, &[120u64; 499]).is_err());
    }

    #[test]
    fn subsidy_follows_the_integer_schedule() {
        assert_eq!(block_reward(1), INITIAL_REWARD);
        assert_eq!(block_reward(BLOCKS_PER_YEAR), INITIAL_REWARD);
        assert_eq!(block_reward(BLOCKS_PER_YEAR + 1), INITIAL_REWARD * 5 / 8);
    }

    #[test]
    fn retarget_runs_after_five_hundred_intervals() {
        let infra = Keypair::dilithium5();
        let miner = Keypair::dilithium2();
        let mut chain = Chain::open(infra.public.clone(), [[0; 4]; LEAVES], 0).unwrap();
        for height in 1..=501 {
            let block = mine_block(
                chain.tip(),
                height,
                1_000 + height * 60,
                chain.difficulty_bits(),
                Vec::new(),
                None,
                &miner.public,
                &miner.secret,
            )
            .expect("mine");
            chain.append(block).expect("append");
        }
        assert_eq!(chain.height(), 501);
        assert_eq!(chain.difficulty_bits(), 1);
        let rejected = mine_block(
            chain.tip(),
            502,
            1_000 + 502 * 60,
            0,
            Vec::new(),
            None,
            &miner.public,
            &miner.secret,
        )
        .unwrap();
        assert_eq!(chain.append(rejected).err(), Some("difficulty"));
    }

    #[test]
    fn local_chain_accepts_and_rejects() {
        let infra = Keypair::dilithium5();
        let user = Keypair::dilithium2();
        let miner = Keypair::dilithium2();
        let witness = funded(chain_words(&[0u8; 32]), 77);
        let (transfer, leaves) = signed_transfer(&infra, &user, &witness);
        let mut chain = Chain::open(infra.public.clone(), leaves, 4).unwrap();

        let mut flipped = transfer.clone();
        flipped.envelope_sig[0] ^= 1;
        let bad_sig = mine_block(
            chain.tip(),
            1,
            900,
            4,
            vec![flipped],
            None,
            &miner.public,
            &miner.secret,
        )
        .unwrap();
        assert_eq!(chain.append(bad_sig).err(), Some("envelope signature"));

        let mut wrong_event = transfer.clone();
        wrong_event.public[18] ^= 1;
        wrong_event.user_sig = sign2(&user.secret, &user_message(&wrong_event));
        wrong_event.envelope_sig = sign5(&infra.secret, &envelope_message(&wrong_event));
        let bad_event = mine_block(
            chain.tip(),
            1,
            950,
            4,
            vec![wrong_event],
            None,
            &miner.public,
            &miner.secret,
        )
        .unwrap();
        assert_eq!(chain.append(bad_event).err(), Some("event"));

        let block1 = mine_block(
            chain.tip(),
            1,
            1_000,
            4,
            vec![transfer.clone()],
            None,
            &miner.public,
            &miner.secret,
        )
        .unwrap();
        chain.append(block1.clone()).unwrap();
        assert_eq!(chain.height(), 1);
        let nullifier = words4(&transfer.public, 4);
        assert!(chain.nullifiers().contains(&nullifier));
        let output = words4(&transfer.public, 8);
        assert!(chain.contains_note(&output));

        let mut other = Chain::open(infra.public, leaves, 4).unwrap();
        other.append(block1).unwrap();
        assert!(other.nullifiers().contains(&nullifier));

        let replay = mine_block(
            chain.tip(),
            2,
            1_100,
            4,
            vec![transfer],
            None,
            &miner.public,
            &miner.secret,
        )
        .unwrap();
        assert_eq!(chain.append(replay).err(), Some("nullifier spent"));

        let bad_coin = Coinbase {
            sk: 5,
            commitment: [1, 2, 3, 4],
        };
        let bad_mint = mine_block(
            chain.tip(),
            2,
            1_200,
            4,
            Vec::new(),
            Some(bad_coin),
            &miner.public,
            &miner.secret,
        )
        .unwrap();
        assert_eq!(chain.append(bad_mint).err(), Some("coinbase"));

        let reward = block_reward(2);
        let minted = digest_words(note_commitment(reward, 5, 2).unwrap());
        let good_mint = mine_block(
            chain.tip(),
            2,
            1_400,
            4,
            Vec::new(),
            Some(Coinbase {
                sk: 5,
                commitment: minted,
            }),
            &miner.public,
            &miner.secret,
        )
        .unwrap();
        chain.append(good_mint).unwrap();
        assert!(chain.contains_note(&minted));
        assert_eq!(chain.height(), 2);

        let mut broken = mine_block(
            chain.tip(),
            3,
            1_500,
            4,
            Vec::new(),
            None,
            &miner.public,
            &miner.secret,
        )
        .unwrap();
        broken.indices.swap(0, 1);
        assert_eq!(chain.append(broken).err(), Some("equihash rejected"));

        let alt1 = mine_block(
            [0; 32],
            1,
            2_000,
            4,
            Vec::new(),
            None,
            &miner.public,
            &miner.secret,
        )
        .unwrap();
        let alt2 = mine_block(
            block_id(&alt1),
            2,
            2_100,
            4,
            Vec::new(),
            None,
            &miner.public,
            &miner.secret,
        )
        .unwrap();
        assert!(!chain.consider(vec![alt1.clone(), alt2.clone()]).unwrap());
        assert!(chain.nullifiers().contains(&nullifier));
        let alt3 = mine_block(
            block_id(&alt2),
            3,
            2_200,
            4,
            Vec::new(),
            None,
            &miner.public,
            &miner.secret,
        )
        .unwrap();
        assert!(chain.consider(vec![alt1, alt2, alt3]).unwrap());
        assert_eq!(chain.height(), 3);
        assert!(chain.nullifiers().is_empty());
        assert!(!chain.contains_note(&output));
        assert_eq!(chain.work(), 48);
    }

    #[test]
    fn a_full_tree_opens_the_next_one_and_reloads() {
        use crate::runtime::Node;
        let infra = Keypair::dilithium5();
        let miner = Keypair::dilithium2();
        let mut node = Node::open(&infra, &miner, 7, [[0; 4]; LEAVES], 0).unwrap();
        for step in 1..=9 {
            node.mine_next(10_000 + step * 10).unwrap();
        }
        assert_eq!(node.chain.height(), 9);
        assert_eq!(node.chain.note_trees(), 2);
        let loaded = Node::from_bytes(&node.to_bytes(), &miner, 7).unwrap();
        assert_eq!(loaded.chain.tip(), node.chain.tip());
        assert_eq!(loaded.chain.note_trees(), 2);
        assert_eq!(loaded.chain.height(), 9);
    }

    #[test]
    fn a_peer_receives_the_longer_chain() {
        use std::net::TcpStream;

        use crate::runtime::{exchange, listen, serve_one, Node};
        let infra = Keypair::dilithium5();
        let miner = Keypair::dilithium2();
        let mut ahead = Node::open(&infra, &miner, 7, [[0; 4]; LEAVES], 0).unwrap();
        for step in 1..=3 {
            ahead.mine_next(20_000 + step * 10).unwrap();
        }
        let listener = listen("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let serving = ahead.chain.clone();
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            serve_one(&mut socket, &serving).unwrap();
        });
        let mut behind = Node::open(&infra, &miner, 9, [[0; 4]; LEAVES], 0).unwrap();
        let mut socket = TcpStream::connect(address).unwrap();
        exchange(&mut socket, &mut behind.chain).unwrap();
        server.join().unwrap();
        assert_eq!(behind.chain.height(), 3);
        assert_eq!(behind.chain.tip(), ahead.chain.tip());

        behind.mine_next(30_000).unwrap();
        let listener = listen("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let serving = behind.chain.clone();
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            serve_one(&mut socket, &serving).unwrap();
        });
        let mut socket = TcpStream::connect(address).unwrap();
        exchange(&mut socket, &mut ahead.chain).unwrap();
        server.join().unwrap();
        assert_eq!(ahead.chain.height(), 4);
        assert_eq!(ahead.chain.tip(), behind.chain.tip());
    }
}
