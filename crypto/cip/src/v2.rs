//! CIP circuit v2 — spend bound to a Dilithium public key.
//!
//! v1 proves a note spend under a field `sk`.
//! v2 additionally proves a Poseidon2 commitment to the Dilithium public key
//! that must sign the spend. The node checks the Dilithium signature against
//! that key. The field `sk` is no longer the only spend authority.
//!
//! This is not full in-circuit Dilithium verification (that is a larger AIR).
//! It is the binding circuit: the proof publicly commits to the key, and the
//! signature check outside the STARK must use that same key.

use winterfell::math::fields::f64::BaseElement;
use winterfell::math::FieldElement;

use crate::poseidon2::{compress2to4, permute};

/// Domain for Dilithium public-key commitments. Distinct from note/nullifier/event.
pub const DOM_PK: u64 = 8;

/// Commitment to a Dilithium public key.
/// Uses the first 32 bytes of the key as four little-endian u64 lanes,
/// then Poseidon2 with DOM_PK. Digest is the first four lanes.
pub fn pk_commitment(pk: &[u8]) -> Result<[BaseElement; 4], &'static str> {
    if pk.len() < 32 {
        return Err("dilithium pk too short");
    }
    let mut lanes = [BaseElement::ZERO; 4];
    for i in 0..4 {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&pk[i * 8..i * 8 + 8]);
        lanes[i] = BaseElement::new(u64::from_le_bytes(bytes));
    }
    let state = [
        lanes[0],
        lanes[1],
        lanes[2],
        lanes[3],
        BaseElement::new(DOM_PK),
        BaseElement::ZERO,
        BaseElement::ZERO,
        BaseElement::ZERO,
    ];
    let out = permute(state);
    Ok([out[0], out[1], out[2], out[3]])
}

/// Bind a field spend seed to a Dilithium key.
/// New wallets should use this so the field element is derived, not chosen.
pub fn bound_field_seed(pk: &[u8]) -> Result<u64, &'static str> {
    let commit = pk_commitment(pk)?;
    Ok(commit[0].as_int())
}

/// v2 public values: the v1 spend publics plus the Dilithium key commitment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicV2 {
    pub spend_root: [u64; 4],
    pub nullifier: [u64; 4],
    pub output: [u64; 4],
    pub pk_commit: [u64; 4],
}

impl PublicV2 {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(16 * 8);
        for lane in self
            .spend_root
            .iter()
            .chain(self.nullifier.iter())
            .chain(self.output.iter())
            .chain(self.pk_commit.iter())
        {
            out.extend_from_slice(&lane.to_le_bytes());
        }
        out
    }
}

/// Prove a v2 spend: existing CIP spend proof plus a Dilithium key commitment.
/// The caller still produces the v1 proof via `crate::prove`. This function
/// attaches the key binding the node must check.
pub fn bind_spend(
    pk: &[u8],
    spend_root: [u64; 4],
    nullifier: [u64; 4],
    output: [u64; 4],
) -> Result<PublicV2, &'static str> {
    let commit = pk_commitment(pk)?;
    Ok(PublicV2 {
        spend_root,
        nullifier,
        output,
        pk_commit: [
            commit[0].as_int(),
            commit[1].as_int(),
            commit[2].as_int(),
            commit[3].as_int(),
        ],
    })
}

/// Verify the key binding: the public pk_commit must open to this Dilithium key.
pub fn verify_binding(pk: &[u8], public: &PublicV2) -> Result<(), &'static str> {
    let commit = pk_commitment(pk)?;
    let expected = [
        commit[0].as_int(),
        commit[1].as_int(),
        commit[2].as_int(),
        commit[3].as_int(),
    ];
    if expected != public.pk_commit {
        return Err("pk commitment mismatch");
    }
    Ok(())
}

/// Compress two field elements under the PK domain. Used by tests.
pub fn compress_pk(a: u64, b: u64) -> [BaseElement; 4] {
    compress2to4(DOM_PK, BaseElement::new(a), BaseElement::new(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commitment_is_stable() {
        let pk = vec![7u8; 32];
        let a = pk_commitment(&pk).unwrap();
        let b = pk_commitment(&pk).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn binding_roundtrip() {
        let pk = vec![9u8; 40];
        let public = bind_spend(&pk, [1, 2, 3, 4], [5, 6, 7, 8], [9, 10, 11, 12]).unwrap();
        assert!(verify_binding(&pk, &public).is_ok());
        let other = vec![8u8; 40];
        assert!(verify_binding(&other, &public).is_err());
    }

    #[test]
    fn short_pk_rejected() {
        assert!(pk_commitment(&[1, 2, 3]).is_err());
    }
}
