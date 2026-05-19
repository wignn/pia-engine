use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, AeadCore,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use sha2::{Digest, Sha256};

fn derive_key(secret: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.finalize().into()
}


pub fn encrypt(plaintext: &str, secret: &str) -> Result<String, String> {
    let key = derive_key(secret);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("cipher init: {e}"))?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| format!("encrypt: {e}"))?;

    let mut combined = nonce.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(B64.encode(&combined))
}

pub fn decrypt(encoded: &str, secret: &str) -> Result<String, String> {
    let key = derive_key(secret);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("cipher init: {e}"))?;

    let combined = B64.decode(encoded).map_err(|e| format!("base64 decode: {e}"))?;
    if combined.len() < 12 {
        return Err("ciphertext too short".into());
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = aes_gcm::Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("decrypt: {e}"))?;

    String::from_utf8(plaintext).map_err(|e| format!("utf8: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_encrypt_decrypt() {
        let secret = "test-encryption-key-with-enough-entropy";
        let plaintext = "ya29.a0AfH6SMA_access_token_example";
        let encrypted = encrypt(plaintext, secret).unwrap();
        assert_ne!(encrypted, plaintext);
        let decrypted = decrypt(&encrypted, secret).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn wrong_key_fails() {
        let encrypted = encrypt("secret-data", "key1").unwrap();
        assert!(decrypt(&encrypted, "key2").is_err());
    }
}
