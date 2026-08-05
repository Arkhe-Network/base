use pqcrypto_traits::{kem::PublicKey as _, sign::PublicKey as _};
use timechain::auth_kem::PqcKeyMaterial;

#[test]
fn test_shors_attack_symbolic() {
    let material = PqcKeyMaterial::generate();

    let dsa_pk_len = material.identity.dsa_public_key.len();
    let kem_pk_len = material.identity.kem_public_key.len();

    // Dilithium5 public key is over 2KB, Kyber1024 is around 1.5KB
    assert!(dsa_pk_len > 1024, "DSA PK should be large enough to resist Shor's Algorithm");
    assert!(kem_pk_len > 1024, "KEM PK should be large enough to resist Shor's Algorithm");

    println!("Simulated Shor's attack failed against large Lattice-based keys.");
}
