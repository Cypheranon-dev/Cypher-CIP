//! CRYSTALS-Dilithium via FIPS 204 ML-DSA.
//!
//! Dilithium5 is ML-DSA-87. Dilithium2 is ML-DSA-44.
//! Public keys match the whitepaper (2,592 and 1,312 bytes). The FIPS 204
//! Dilithium5 signature is 4,627 bytes; the paper's 4,595 is the round-3 draft.

use fips204::ml_dsa_44::{self, PK_LEN as D2_PK, SIG_LEN as D2_SIG, SK_LEN as D2_SK};
use fips204::ml_dsa_87::{self, PK_LEN as D5_PK, SIG_LEN as D5_SIG, SK_LEN as D5_SK};
use fips204::traits::{SerDes, Signer, Verifier};

pub const DILITHIUM5_PK: usize = D5_PK;
pub const DILITHIUM5_SK: usize = D5_SK;
pub const DILITHIUM5_SIG: usize = D5_SIG;
pub const DILITHIUM2_PK: usize = D2_PK;
pub const DILITHIUM2_SK: usize = D2_SK;
pub const DILITHIUM2_SIG: usize = D2_SIG;

#[derive(Clone)]
pub struct Keypair {
    pub public: Vec<u8>,
    pub secret: Vec<u8>,
}

fn as_array<const N: usize>(bytes: &[u8]) -> Option<[u8; N]> {
    bytes.try_into().ok()
}

impl Keypair {
    pub fn dilithium5() -> Self {
        let (pk, sk) = ml_dsa_87::try_keygen().expect("dilithium5 keygen");
        Self {
            public: pk.into_bytes().to_vec(),
            secret: sk.into_bytes().to_vec(),
        }
    }

    pub fn dilithium2() -> Self {
        let (pk, sk) = ml_dsa_44::try_keygen().expect("dilithium2 keygen");
        Self {
            public: pk.into_bytes().to_vec(),
            secret: sk.into_bytes().to_vec(),
        }
    }
}

pub fn sign5(secret: &[u8], message: &[u8]) -> Vec<u8> {
    let sk =
        ml_dsa_87::PrivateKey::try_from_bytes(as_array(secret).expect("dilithium5 secret length"))
            .expect("dilithium5 secret");
    sk.try_sign(message, b"").expect("dilithium5 sign").to_vec()
}

pub fn verify5(public: &[u8], message: &[u8], signature: &[u8]) -> bool {
    let (Some(pk_bytes), Some(sig_bytes)) = (as_array(public), as_array(signature)) else {
        return false;
    };
    let Ok(pk) = ml_dsa_87::PublicKey::try_from_bytes(pk_bytes) else {
        return false;
    };
    pk.verify(message, &sig_bytes, b"")
}

pub fn sign2(secret: &[u8], message: &[u8]) -> Vec<u8> {
    let sk =
        ml_dsa_44::PrivateKey::try_from_bytes(as_array(secret).expect("dilithium2 secret length"))
            .expect("dilithium2 secret");
    sk.try_sign(message, b"").expect("dilithium2 sign").to_vec()
}

pub fn verify2(public: &[u8], message: &[u8], signature: &[u8]) -> bool {
    let (Some(pk_bytes), Some(sig_bytes)) = (as_array(public), as_array(signature)) else {
        return false;
    };
    let Ok(pk) = ml_dsa_44::PublicKey::try_from_bytes(pk_bytes) else {
        return false;
    };
    pk.verify(message, &sig_bytes, b"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dilithium5_accepts_and_rejects() {
        let keys = Keypair::dilithium5();
        let message = b"CIP_proof||event_id";
        let sig = sign5(&keys.secret, message);
        assert_eq!(sig.len(), DILITHIUM5_SIG);
        assert_eq!(keys.public.len(), DILITHIUM5_PK);
        assert!(verify5(&keys.public, message, &sig));
        let mut flipped = message.to_vec();
        flipped[0] ^= 1;
        assert!(!verify5(&keys.public, &flipped, &sig));
        let mut bad = sig.clone();
        bad[0] ^= 1;
        assert!(!verify5(&keys.public, message, &bad));
        assert_eq!(DILITHIUM5_PK, 2592);
        assert_eq!(DILITHIUM5_SIG, 4627);
    }

    #[test]
    fn dilithium2_accepts_and_rejects() {
        let keys = Keypair::dilithium2();
        let message = b"coinbase";
        let sig = sign2(&keys.secret, message);
        assert_eq!(sig.len(), DILITHIUM2_SIG);
        assert!(verify2(&keys.public, message, &sig));
        assert!(!verify2(&keys.public, b"other", &sig));
        assert_eq!(DILITHIUM2_PK, 1312);
        assert_eq!(DILITHIUM2_SIG, 2420);
    }
}
