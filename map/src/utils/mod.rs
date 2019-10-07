pub fn seed_from_str(seed: &str) -> Vec<u8> {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.input(seed.as_bytes());
    hasher.result().to_vec()
}
