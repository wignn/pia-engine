use rand::Rng;
use sha2::{Digest, Sha256};

const KEY_PREFIX: &str = "wi_live_";

pub fn generate_raw_key() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 24] = rng.gen();
    format!("{}{}", KEY_PREFIX, hex::encode(bytes))
}

pub fn hash_key(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn extract_prefix(raw: &str) -> String {
    if raw.len() >= 16 {
        format!("{}...", &raw[..16])
    } else {
        raw.to_string()
    }
}
