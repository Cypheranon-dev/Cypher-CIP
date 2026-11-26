//! Minimal in-circuit ML-DSA binding (research).
//!
//! Full FIPS 204 verification inside a STARK is not implemented.
//! This module adds a constrained commitment to the public key and a
//! simple challenge-binding check that the node already performs natively.
//!
//! The production spend still verifies the signature outside the proof.

use winterfell::math::fields::f64::BaseElement;
use winterfell::math::FieldElement;

use crate::v2::pk_commitment;

/// In-circuit style check: the public key must open the commitment,
/// and a simple challenge binding must match.
pub fn circuit_bind_check(
    pk: &[u8],
    challenge: [u8; 32],
    expected_commit: [BaseElement; 4],
) -> Result<(), &'static str> {
    if pk.len() < 32 {
        return Err("pk too short");
    }
    let commit = pk_commitment(pk)?;
    if commit != expected_commit {
        return Err("commitment mismatch");
    }
    // Simple challenge binding: first byte of pk must equal first byte of challenge
    // for this research prototype. Not a real ML-DSA challenge.
    if pk[0] != challenge[0] {
        return Err("challenge bind");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fe_commit(pk: &[u8]) -> [BaseElement; 4] {
        crate::v2::pk_commitment(pk).unwrap()
    }

    #[test]
    fn matching_pk_and_challenge_pass() {
        let pk = vec![7u8; 40];
        let challenge = [7u8; 32];
        let commit = fe_commit(&pk);
        assert!(circuit_bind_check(&pk, challenge, commit).is_ok());
    }

    #[test]
    fn wrong_challenge_fails() {
        let pk = vec![7u8; 40];
        let challenge = [8u8; 32];
        let commit = fe_commit(&pk);
        assert!(circuit_bind_check(&pk, challenge, commit).is_err());
    }

    #[test]
    fn wrong_commit_fails() {
        let pk = vec![7u8; 40];
        let challenge = [7u8; 32];
        let bad = [BaseElement::ONE; 4];
        assert!(circuit_bind_check(&pk, challenge, bad).is_err());
    }
}
