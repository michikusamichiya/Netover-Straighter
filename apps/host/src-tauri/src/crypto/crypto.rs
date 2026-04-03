use sha2::{Sha256, Digest};
use rand::rngs::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey};
use hkdf::Hkdf;

pub fn derive_salt(pub1: &[u8; 32], pub2: &[u8; 32]) -> [u8; 32] {
    let mut combined = Vec::with_capacity(64);

    if pub1 < pub2 {
        combined.extend_from_slice(pub1);
        combined.extend_from_slice(pub2);
    } else {
        combined.extend_from_slice(pub2);
        combined.extend_from_slice(pub1);
    }

    let hash = Sha256::digest(&combined);

    let mut result = [0u8; 32];
    result.copy_from_slice(&hash);
    result
}

pub fn generate_x25519() -> (EphemeralSecret, PublicKey) {
    let secret = EphemeralSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);
    (secret, public)
}
pub fn compute_shared(secret: EphemeralSecret, public: PublicKey) -> [u8; 32] {
    let shared = secret.diffie_hellman(&public);
    shared.to_bytes()
}
pub fn bytes_to_string(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    // JavaScript implementation: combined.set(byteArr, 0); combined.set(context, byteArr.length);
    // So: bytes (shared secret) THEN context string.
    hasher.update(bytes);
    hasher.update(b"Netover v1 shared-secret hash");
    let result = hasher.finalize();

    hex::encode(result)
}

pub fn derive_hkdf_key(shared_secret: &[u8; 32], salt: &[u8; 32], context: &[u8]) -> [u8; 32] {
    let hkdf = Hkdf::<Sha256>::new(Some(salt), shared_secret);
    let mut okm = [0u8; 32];
    hkdf.expand(context, &mut okm).expect("Failed to expand key");
    okm
}