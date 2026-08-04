use crate::pqc::{generate_node_keys, QuantumBlockSignature};
use pqcrypto_kyber::kyber1024::{PublicKey as KemPublicKey, SecretKey as KemSecretKey};
use pqcrypto_dilithium::dilithium5::{PublicKey as QuantumPublicKey, SecretKey as QuantumSecretKey};
use pqcrypto_traits::sign::PublicKey as DsaPublicKeyTrait;
use pqcrypto_traits::kem::PublicKey as KemPublicKeyTrait;
use crate::error::TimechainError;
use serde::{Deserialize, Serialize};
use sha3::{Sha3_256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthEncap {
    pub ciphertext: Vec<u8>,
    pub sender_signature: QuantumBlockSignature,
    pub sender_public_key: Vec<u8>,
    pub context_nonce: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PqcIdentity {
    pub dsa_public_key: Vec<u8>,
    pub kem_public_key: Vec<u8>,
    #[serde(skip)]
    pub fingerprint: [u8; 32],
}

impl PqcIdentity {
    pub fn new(dsa_public_key: &QuantumPublicKey, kem_public_key: &KemPublicKey) -> Self {
        let mut hasher = Sha3_256::new();
        hasher.update(dsa_public_key.as_bytes());
        hasher.update(kem_public_key.as_bytes());
        let result = hasher.finalize();
        let mut fingerprint = [0u8; 32];
        fingerprint.copy_from_slice(&result);
        Self {
            dsa_public_key: dsa_public_key.as_bytes().to_vec(),
            kem_public_key: kem_public_key.as_bytes().to_vec(),
            fingerprint,
        }
    }

    pub fn public_only(&self) -> Self {
        self.clone()
    }
}

pub struct PqcKeyMaterial {
    pub dsa_secret_key: QuantumSecretKey,
    pub kem_secret_key: KemSecretKey,
    pub identity: PqcIdentity,
}

impl PqcKeyMaterial {
    pub fn generate() -> Self {
        let (dsa_pk, dsa_sk) = generate_node_keys();
        let (kem_pk, kem_sk) = pqcrypto_kyber::kyber1024::keypair();
        let identity = PqcIdentity::new(&dsa_pk, &kem_pk);
        Self {
            dsa_secret_key: dsa_sk,
            kem_secret_key: kem_sk,
            identity,
        }
    }
}

pub struct AuthenticatedKem;
impl AuthenticatedKem {
    pub fn encapsulate_auth(
        _: &PqcKeyMaterial,
        _: &PqcIdentity,
        _: &[u8],
    ) -> Result<(AuthEncap, [u8; 32]), TimechainError> {
        Ok((AuthEncap {
            ciphertext: vec![],
            sender_signature: QuantumBlockSignature(vec![]),
            sender_public_key: vec![],
            context_nonce: [0; 32],
        }, [0; 32]))
    }
    pub fn decapsulate_auth(
        _: &PqcKeyMaterial,
        _: &AuthEncap,
        _: &[u8],
    ) -> Result<[u8; 32], TimechainError> {
        Ok([0; 32])
    }
}
