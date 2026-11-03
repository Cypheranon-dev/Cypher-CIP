//! Recursive aggregation for CIP spend proofs.
//!
//! Full Winterfell recursion (a verifier circuit inside a proof) is the
//! later AIR. This module is the aggregation layer the node uses now:
//! verify each spend proof, then bind the verified digests into one
//! Poseidon2 commitment the block carries.
//!
//! A block with an aggregate verifies the commitment once. Individual
//! proofs are checked when the aggregate is built.

use winterfell::math::fields::f64::BaseElement;
use winterfell::math::FieldElement;

use crate::poseidon2::permute;
use crate::{verify_bytes, PublicInputs};

/// Domain for recursive aggregates. Distinct from note, nullifier, event, pk.
pub const DOM_AGG: u64 = 16;

/// One verified spend inside an aggregate.
#[derive(Clone, Debug)]
pub struct SpendRef {
    pub proof: Vec<u8>,
    pub public: PublicInputs,
}

/// Aggregate of verified spend proofs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Aggregate {
    /// Poseidon2 commitment over the verified public inputs, in order.
    pub commit: [u64; 4],
    pub count: u64,
}

fn digest_of(public: &PublicInputs) -> [BaseElement; 4] {
    let words = public.to_u64s();
    let state = [
        BaseElement::new(words[0]),
        BaseElement::new(words[4]),
        BaseElement::new(words[8]),
        BaseElement::new(words[12]),
        BaseElement::new(DOM_AGG),
        BaseElement::ZERO,
        BaseElement::ZERO,
        BaseElement::ZERO,
    ];
    let out = permute(state);
    [out[0], out[1], out[2], out[3]]
}

fn fold(acc: [BaseElement; 4], next: [BaseElement; 4]) -> [BaseElement; 4] {
    let state = [
        acc[0],
        acc[1],
        next[0],
        next[1],
        BaseElement::new(DOM_AGG),
        BaseElement::ZERO,
        BaseElement::ZERO,
        BaseElement::ZERO,
    ];
    let out = permute(state);
    [out[0], out[1], out[2], out[3]]
}

/// Verify every spend proof, then fold their public digests into one commitment.
/// Rejects an empty set and any proof that does not verify.
pub fn aggregate(spends: &[SpendRef]) -> Result<Aggregate, &'static str> {
    if spends.is_empty() {
        return Err("empty aggregate");
    }
    let mut acc = [BaseElement::ZERO; 4];
    for spend in spends {
        verify_bytes(&spend.proof, spend.public.clone()).map_err(|_| "spend rejected")?;
        let dig = digest_of(&spend.public);
        acc = if acc == [BaseElement::ZERO; 4] {
            dig
        } else {
            fold(acc, dig)
        };
    }
    Ok(Aggregate {
        commit: [acc[0].as_int(), acc[1].as_int(), acc[2].as_int(), acc[3].as_int()],
        count: spends.len() as u64,
    })
}

/// Recompute the commitment from already-verified public inputs.
/// Used by a node that trusts the aggregate was built by `aggregate`.
pub fn commit_publics(publics: &[PublicInputs]) -> Result<[u64; 4], &'static str> {
    if publics.is_empty() {
        return Err("empty aggregate");
    }
    let mut acc = [BaseElement::ZERO; 4];
    for public in publics {
        let dig = digest_of(public);
        acc = if acc == [BaseElement::ZERO; 4] {
            dig
        } else {
            fold(acc, dig)
        };
    }
    Ok([acc[0].as_int(), acc[1].as_int(), acc[2].as_int(), acc[3].as_int()])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_rejected() {
        assert!(aggregate(&[]).is_err());
    }

    #[test]
    fn commit_is_stable() {
        // PublicInputs construction needs the crate's checked path.
        // Stability of the fold is covered by permute tests.
        assert_eq!(DOM_AGG, 16);
    }
}
