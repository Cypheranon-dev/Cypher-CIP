//! Public-key commitment helper. Not an in-circuit ML-DSA verifier.
//!
//! `pk_commitment` is a Poseidon2 digest of key bytes. It does not constrain
//! NTT, the challenge polynomial, hints, or infinity norms. The node still
//! runs FIPS 204 verification outside the proof.

use winterfell::math::fields::f64::BaseElement;

use crate::v2::pk_commitment;

/// Check that `pk` opens `expected_commit`. This is not signature verification.
pub fn circuit_bind_check(
    pk: &[u8],
    expected_commit: [BaseElement; 4],
) -> Result<(), &'static str> {
    if pk.len() < 32 {
        return Err("pk too short");
    }
    let commit = pk_commitment(pk)?;
    if commit != expected_commit {
        return Err("commitment mismatch");
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
    fn matching_key_opens_commitment() {
        let pk = vec![7u8; 40];
        assert!(circuit_bind_check(&pk, fe_commit(&pk)).is_ok());
    }

    #[test]
    fn wrong_commit_fails() {
        let pk = vec![7u8; 40];
        let bad = [BaseElement::new(1); 4];
        assert!(circuit_bind_check(&pk, bad).is_err());
    }

    #[test]
    fn short_key_fails() {
        assert!(circuit_bind_check(&[1u8; 8], [BaseElement::new(0); 4]).is_err());
    }
}
