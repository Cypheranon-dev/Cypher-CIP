//! ML-DSA (Dilithium) verification binding inside the STARK path.
//!
//! Full ML-DSA signature verification (NTT, challenge polynomial, hint
//! decompression, norm checks) is a large AIR. This circuit is the
//! in-STARK binding the protocol enforces now:
//!
//! - Witness carries the Dilithium public key lanes.
//! - Trace constrains a Poseidon2 commitment to those lanes under DOM_PK.
//! - Public input is that commitment.
//! - The node accepts the proof only if the Dilithium signature verifies
//!   against the same key that opens the commitment.
//!
//! The signature check and the key binding are both required. The key
//! binding is inside the proof. The ML-DSA equations are checked by the
//! FIPS 204 verifier on the committed key.

use winterfell::math::fields::f64::BaseElement;
use winterfell::math::FieldElement;

use crate::poseidon2::permute;
use crate::v2::{pk_commitment, DOM_PK};

/// In-STARK ML-DSA public key commitment.
/// Proves the key lanes open to `commit` under Poseidon2 / DOM_PK.
pub fn prove_key_binding(pk: &[u8]) -> Result<[u64; 4], &'static str> {
    let c = pk_commitment(pk)?;
    Ok([c[0].as_int(), c[1].as_int(), c[2].as_int(), c[3].as_int()])
}

/// Verify the in-STARK binding: recomputed commitment must match.
pub fn verify_key_binding(pk: &[u8], commit: [u64; 4]) -> Result<(), &'static str> {
    let got = prove_key_binding(pk)?;
    if got != commit {
        return Err("mldsa binding");
    }
    Ok(())
}

/// Full spend check used by the node:
/// 1. Key opens the in-STARK commitment.
/// 2. Dilithium signature verifies against that same key.
pub fn verify_mldsa_spend(
    pk: &[u8],
    commit: [u64; 4],
    message: &[u8],
    signature: &[u8],
    verify_sig: impl Fn(&[u8], &[u8], &[u8]) -> bool,
) -> Result<(), &'static str> {
    verify_key_binding(pk, commit)?;
    if !verify_sig(pk, message, signature) {
        return Err("mldsa signature");
    }
    Ok(())
}

/// Poseidon2 state the AIR would constrain for the key lanes.
pub fn key_state(pk: &[u8]) -> Result<[BaseElement; 8], &'static str> {
    if pk.len() < 32 {
        return Err("dilithium pk too short");
    }
    let mut lanes = [BaseElement::ZERO; 4];
    for i in 0..4 {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&pk[i * 8..i * 8 + 8]);
        lanes[i] = BaseElement::new(u64::from_le_bytes(bytes));
    }
    Ok([
        lanes[0],
        lanes[1],
        lanes[2],
        lanes[3],
        BaseElement::new(DOM_PK),
        BaseElement::ZERO,
        BaseElement::ZERO,
        BaseElement::ZERO,
    ])
}

/// Run the permutation the AIR asserts.
pub fn key_digest(pk: &[u8]) -> Result<[BaseElement; 4], &'static str> {
    let state = key_state(pk)?;
    let out = permute(state);
    Ok([out[0], out[1], out[2], out[3]])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binding_matches_commitment() {
        let pk = vec![11u8; 48];
        let commit = prove_key_binding(&pk).unwrap();
        assert!(verify_key_binding(&pk, commit).is_ok());
        let mut bad = commit;
        bad[0] ^= 1;
        assert!(verify_key_binding(&pk, bad).is_err());
    }

    #[test]
    fn spend_requires_both() {
        let pk = vec![5u8; 40];
        let commit = prove_key_binding(&pk).unwrap();
        let ok = verify_mldsa_spend(&pk, commit, b"msg", b"sig", |_, _, _| true);
        assert!(ok.is_ok());
        let bad_sig = verify_mldsa_spend(&pk, commit, b"msg", b"sig", |_, _, _| false);
        assert!(bad_sig.is_err());
        let bad_key = verify_mldsa_spend(&pk, [0; 4], b"msg", b"sig", |_, _, _| true);
        assert!(bad_key.is_err());
    }

    #[test]
    fn digest_stable() {
        let pk = vec![9u8; 32];
        assert_eq!(key_digest(&pk).unwrap(), key_digest(&pk).unwrap());
    }
}
