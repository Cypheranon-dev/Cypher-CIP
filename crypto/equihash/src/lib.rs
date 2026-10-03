//! Equihash (Wagner / generalized birthday), Blake2b, Zcash verification order.
//!
//! Whitepaper §7 names Equihash(n=512, k=9). That pair is not a legal instance:
//! k+1 does not divide n, and n/(k+1)+1 does not fit the 32-bit index word
//! the algorithm uses. `Instance::paper()` returns that error.
//!
//! The implementation itself is the algorithm. Tests solve and verify
//! Equihash(48, 5), which satisfies both constraints and is byte-aligned.
//! That instance is not Equihash-512.

use blake2b_simd::Params;

pub const PAPER_N: u32 = 512;
pub const PAPER_K: u32 = 9;
/// Legal byte-aligned instance used by the local testnet. Not the whitepaper pair.
pub const TESTNET_N: u32 = 48;
pub const TESTNET_K: u32 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Instance {
    pub n: u32,
    pub k: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParamError {
    NotDivisible,
    IndexWord,
    Order,
}

impl Instance {
    pub fn new(n: u32, k: u32) -> Result<Self, ParamError> {
        if k == 0 || k >= n {
            return Err(ParamError::Order);
        }
        if !n.is_multiple_of(k + 1) {
            return Err(ParamError::NotDivisible);
        }
        let collision = n / (k + 1);
        if collision + 1 >= 32 {
            return Err(ParamError::IndexWord);
        }
        Ok(Self { n, k })
    }

    pub fn paper() -> Result<Self, ParamError> {
        Self::new(PAPER_N, PAPER_K)
    }
}

fn personal(n: u32, k: u32) -> [u8; 16] {
    let mut person = [0u8; 16];
    person[..8].copy_from_slice(b"CypherEH");
    person[8..12].copy_from_slice(&n.to_le_bytes());
    person[12..16].copy_from_slice(&k.to_le_bytes());
    person
}

fn blake(header: &[u8], n: u32, k: u32, xi: u32) -> [u8; 64] {
    let mut state = Params::new()
        .hash_length(64)
        .personal(&personal(n, k))
        .to_state();
    state.update(header);
    state.update(&xi.to_le_bytes());
    let hash = state.finalize();
    let mut out = [0u8; 64];
    out.copy_from_slice(hash.as_bytes());
    out
}

fn expand_array(inp: &[u8], out_len: usize, bit_len: usize) -> Vec<u8> {
    let word_mask: u64 = (1 << 32) - 1;
    let bit_len_mask: u64 = (1u64 << bit_len) - 1;
    let out_width = bit_len.div_ceil(8);
    assert_eq!(out_len, 8 * out_width * inp.len() / bit_len);
    let mut out = vec![0u8; out_len];
    let mut acc_bits: u32 = 0;
    let mut acc_value: u64 = 0;
    let mut j = 0;
    for &byte in inp {
        acc_value = ((acc_value << 8) & word_mask) | u64::from(byte);
        acc_bits += 8;
        if acc_bits >= bit_len as u32 {
            acc_bits -= bit_len as u32;
            for x in 0..out_width {
                let shift = u64::from(acc_bits) + 8 * (out_width - x - 1) as u64;
                let mask = (bit_len_mask >> (8 * (out_width - x - 1))) & 0xFF;
                out[j + x] = ((acc_value >> shift) as u8) & (mask as u8);
            }
            j += out_width;
        }
    }
    out
}

fn has_collision(ha: &[u8], hb: &[u8], i: usize, collision: usize) -> bool {
    let from = (i - 1) * collision / 8;
    let to = i * collision / 8;
    ha[from..to] == hb[from..to]
}

fn distinct(a: &[u32], b: &[u32]) -> bool {
    !a.iter().any(|x| b.contains(x))
}

fn xor_bytes(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().zip(b).map(|(x, y)| x ^ y).collect()
}

fn count_zeroes(h: &[u8]) -> usize {
    let mut n = 0;
    for byte in h {
        if *byte == 0 {
            n += 8;
        } else {
            n += byte.leading_zeros() as usize;
            break;
        }
    }
    n
}

fn row_hash(header: &[u8], inst: Instance, index: u32) -> Vec<u8> {
    let collision = (inst.n / (inst.k + 1)) as usize;
    let hash_len = (inst.k as usize + 1) * collision.div_ceil(8);
    let per = 512 / inst.n;
    let r = (index % per) as usize;
    let block = index / per;
    let digest = blake(header, inst.n, inst.k, block);
    let start = r * inst.n as usize / 8;
    let end = (r + 1) * inst.n as usize / 8;
    expand_array(&digest[start..end], hash_len, collision)
}

/// Solve Equihash for `header`. Returns index lists, each of length 2^k.
pub fn solve(header: &[u8], inst: Instance) -> Vec<Vec<u32>> {
    let collision = (inst.n / (inst.k + 1)) as usize;
    let hash_len = (inst.k as usize + 1) * collision.div_ceil(8);
    let count = 1usize << (collision + 1);
    let mut rows: Vec<(Vec<u8>, Vec<u32>)> = (0..count)
        .map(|i| (row_hash(header, inst, i as u32), vec![i as u32]))
        .collect();

    for round in 1..inst.k {
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        let mut next = Vec::new();
        let mut cursor = rows.len();
        while cursor > 0 {
            let mut j = 1;
            while j < cursor
                && has_collision(
                    &rows[cursor - 1].0,
                    &rows[cursor - 1 - j].0,
                    round as usize,
                    collision,
                )
            {
                j += 1;
            }
            for l in 0..j - 1 {
                for m in l + 1..j {
                    let left = &rows[cursor - 1 - l];
                    let right = &rows[cursor - 1 - m];
                    if distinct(&left.1, &right.1) {
                        let (first, second) = if left.1[0] < right.1[0] {
                            (left, right)
                        } else {
                            (right, left)
                        };
                        let mut indices = first.1.clone();
                        indices.extend_from_slice(&second.1);
                        next.push((xor_bytes(&left.0, &right.0), indices));
                    }
                }
            }
            cursor -= j;
        }
        rows = next;
        if rows.is_empty() {
            return Vec::new();
        }
    }

    rows.sort_by(|a, b| a.0.cmp(&b.0));
    let mut sols = Vec::new();
    let mut cursor = rows.len();
    let k = inst.k as usize;
    while cursor > 0 {
        let mut j = 1;
        while j < cursor
            && has_collision(&rows[cursor - 1].0, &rows[cursor - 1 - j].0, k, collision)
            && has_collision(
                &rows[cursor - 1].0,
                &rows[cursor - 1 - j].0,
                k + 1,
                collision,
            )
        {
            j += 1;
        }
        for l in 0..j - 1 {
            for m in l + 1..j {
                let left = &rows[cursor - 1 - l];
                let right = &rows[cursor - 1 - m];
                let mixed = xor_bytes(&left.0, &right.0);
                if count_zeroes(&mixed) == 8 * hash_len && distinct(&left.1, &right.1) {
                    let mut indices = if left.1[0] < right.1[0] {
                        left.1.clone()
                    } else {
                        right.1.clone()
                    };
                    let other = if left.1[0] < right.1[0] {
                        &right.1
                    } else {
                        &left.1
                    };
                    indices.extend_from_slice(other);
                    sols.push(indices);
                }
            }
        }
        cursor -= j;
    }
    sols
}

pub fn verify(header: &[u8], inst: Instance, indices: &[u32]) -> bool {
    if indices.len() != 1usize << inst.k {
        return false;
    }
    let collision = (inst.n / (inst.k + 1)) as usize;
    let hash_len = (inst.k as usize + 1) * collision.div_ceil(8);
    let mut rows: Vec<(Vec<u8>, Vec<u32>)> = indices
        .iter()
        .map(|i| (row_hash(header, inst, *i), vec![*i]))
        .collect();
    for round in 1..=inst.k {
        let mut next = Vec::new();
        if !rows.len().is_multiple_of(2) {
            return false;
        }
        for pair in rows.chunks(2) {
            if !has_collision(&pair[0].0, &pair[1].0, round as usize, collision) {
                return false;
            }
            if pair[1].1[0] < pair[0].1[0] || !distinct(&pair[0].1, &pair[1].1) {
                return false;
            }
            let mut indices = pair[0].1.clone();
            indices.extend_from_slice(&pair[1].1);
            next.push((xor_bytes(&pair[0].0, &pair[1].0), indices));
        }
        rows = next;
    }
    rows.len() == 1 && count_zeroes(&rows[0].0) == 8 * hash_len
}

pub fn leading_zero_bits(bytes: &[u8]) -> u32 {
    let mut n = 0;
    for byte in bytes {
        if *byte == 0 {
            n += 8;
        } else {
            n += byte.leading_zeros();
            break;
        }
    }
    n
}

pub fn commit(header: &[u8], indices: &[u32]) -> [u8; 32] {
    let mut state = Params::new()
        .hash_length(32)
        .personal(b"CypherPowCommit")
        .to_state();
    state.update(header);
    for index in indices {
        state.update(&index.to_le_bytes());
    }
    let hash = state.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(hash.as_bytes());
    out
}

/// Find a nonce whose Equihash solution also meets `difficulty_bits` on `commit`.
pub fn mine(prefix: &[u8], inst: Instance, difficulty_bits: u32) -> Option<(u32, Vec<u32>)> {
    for nonce in 0..20_000u32 {
        let mut header = prefix.to_vec();
        header.extend_from_slice(&nonce.to_le_bytes());
        for indices in solve(&header, inst) {
            if leading_zero_bits(&commit(&header, &indices)) >= difficulty_bits {
                return Some((nonce, indices));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paper_parameters_are_rejected() {
        assert_eq!(Instance::paper(), Err(ParamError::NotDivisible));
        // 512 is divisible by 16, but the collision word does not fit in 31 bits.
        assert_eq!(Instance::new(512, 15), Err(ParamError::IndexWord));
        assert_eq!(Instance::new(200, 9).unwrap().n, 200);
    }

    #[test]
    fn solve_then_verify_and_reject_a_swap() {
        let inst = Instance::new(TESTNET_N, TESTNET_K).unwrap();
        let mut found = None;
        for salt in 0..32u32 {
            let header = format!("cypher-testnet-header-{salt}");
            let sols = solve(header.as_bytes(), inst);
            if !sols.is_empty() {
                found = Some((header, sols));
                break;
            }
        }
        let (header, sols) = found.expect("solver found no solution");
        assert!(verify(header.as_bytes(), inst, &sols[0]));
        let mut bad = sols[0].clone();
        bad.swap(0, 1);
        assert!(!verify(header.as_bytes(), inst, &bad));
    }
}
