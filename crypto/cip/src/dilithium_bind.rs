//! In-circuit Dilithium binding.
//!
//! Full ML-DSA verification inside a STARK is a large AIR (thousands of
//! constraints). This module is the binding the circuit enforces now:
//! the spend proof publicly commits to the Dilithium public key, and the
//! node rejects a spend whose signature key does not open that commitment.
//!
//! The next AIR replaces this with in-circuit signature verification.

use crate::v2::{pk_commitment, verify_binding, PublicV2};

/// Check that a Dilithium public key opens the commitment carried with a spend.
/// This is the in-circuit binding check: the proof commits to the key, and
/// only that key may sign.
pub fn check_spend_key(pk: &[u8], public: &PublicV2) -> Result<(), &'static str> {
    if pk.len() < 32 {
        return Err("dilithium pk too short");
    }
    verify_binding(pk, public)
}

/// Derive the public commitment the circuit would assert.
pub fn circuit_pk_commit(pk: &[u8]) -> Result<[u64; 4], &'static str> {
    let c = pk_commitment(pk)?;
    Ok([c[0].as_int(), c[1].as_int(), c[2].as_int(), c[3].as_int()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::bind_spend;

    #[test]
    fn matching_key_passes() {
        let pk = vec![3u8; 40];
        let public = bind_spend(&pk, [1; 4], [2; 4], [3; 4]).unwrap();
        assert!(check_spend_key(&pk, &public).is_ok());
    }

    #[test]
    fn wrong_key_rejected() {
        let pk = vec![3u8; 40];
        let public = bind_spend(&pk, [1; 4], [2; 4], [3; 4]).unwrap();
        let other = vec![4u8; 40];
        assert!(check_spend_key(&other, &public).is_err());
    }
}
