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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_raw_key_uses_live_prefix_and_random_payload() {
        let key = generate_raw_key();

        assert!(key.starts_with(KEY_PREFIX));
        assert_eq!(key.len(), KEY_PREFIX.len() + 48);
    }

    #[test]
    fn hash_key_is_stable_sha256_hex() {
        assert_eq!(
            hash_key("wi_live_test"),
            "dc70286d8f6be08623176329c8c001be4ab2f07dd0baffa7f6701bbcc273437b"
        );
        assert_eq!(hash_key("wi_live_test").len(), 64);
    }

    #[test]
    fn extract_prefix_handles_short_and_long_keys() {
        assert_eq!(extract_prefix("short"), "short");
        assert_eq!(extract_prefix("wi_live_1234567890"), "wi_live_12345678...");
    }
}
