//! Libsodium-compatible encryption wrapper.
//!
//! Provides sealed boxes (crypto_box_seal) used by GitHub API for secrets,
//! and secret boxes (crypto_secretbox) for local symmetric encryption.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use crypto_box::{aead::OsRng, PublicKey};
use crypto_secretbox::{
    aead::{Aead, AeadCore, KeyInit},
    Key, XSalsa20Poly1305,
};

/// Encrypt a secret for GitHub Actions using their public key.
/// Returns the base64-encoded sealed box.
pub fn encrypt_github_secret(
    public_key_b64: &str,
    secret_value: &str,
) -> Result<String, crate::error::Error> {
    let pk_bytes = BASE64
        .decode(public_key_b64)
        .map_err(|_| crate::error::Error::InternalError)?;

    if pk_bytes.len() != 32 {
        return Err(crate::error::Error::InternalError);
    }

    let mut pk_array = [0u8; 32];
    pk_array.copy_from_slice(&pk_bytes);
    let public_key = PublicKey::from(pk_array);

    let ciphertext = public_key
        .seal(&mut OsRng, secret_value.as_bytes())
        .map_err(|_| crate::error::Error::InternalError)?;
    Ok(BASE64.encode(ciphertext))
}

/// Encrypt a token locally using a symmetric key.
pub fn encrypt_local_secret(
    master_key: &str,
    plaintext: &str,
) -> Result<String, crate::error::Error> {
    let mut key_bytes = [0u8; 32];
    let mk_bytes = master_key.as_bytes();
    let len = std::cmp::min(mk_bytes.len(), 32);
    key_bytes[..len].copy_from_slice(&mk_bytes[..len]);

    let key = Key::from(key_bytes);
    let cipher = XSalsa20Poly1305::new(&key);
    let nonce = XSalsa20Poly1305::generate_nonce(&mut OsRng); // 24 bytes

    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|_| crate::error::Error::InternalError)?;

    let mut combined = nonce.to_vec();
    combined.extend(ciphertext);
    Ok(BASE64.encode(combined))
}

/// Decrypt a token locally using a symmetric key.
pub fn decrypt_local_secret(
    master_key: &str,
    combined_b64: &str,
) -> Result<String, crate::error::Error> {
    let mut key_bytes = [0u8; 32];
    let mk_bytes = master_key.as_bytes();
    let len = std::cmp::min(mk_bytes.len(), 32);
    key_bytes[..len].copy_from_slice(&mk_bytes[..len]);

    let key = Key::from(key_bytes);
    let cipher = XSalsa20Poly1305::new(&key);

    let combined = BASE64
        .decode(combined_b64)
        .map_err(|_| crate::error::Error::InternalError)?;
    if combined.len() < 24 {
        return Err(crate::error::Error::InternalError);
    }

    let nonce = crypto_secretbox::Nonce::clone_from_slice(&combined[..24]);
    let plaintext = cipher
        .decrypt(&nonce, &combined[24..])
        .map_err(|_| crate::error::Error::InternalError)?;

    String::from_utf8(plaintext).map_err(|_| crate::error::Error::InternalError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_secret_encryption_decryption() -> Result<(), Box<dyn std::error::Error>> {
        let master_key = "my-super-secret-master-key-that-is-long-enough";
        let plaintext = "my-local-secret";

        let encrypted = encrypt_local_secret(master_key, plaintext)?;
        assert_ne!(encrypted, plaintext);

        let decrypted = decrypt_local_secret(master_key, &encrypted)?;
        assert_eq!(decrypted, plaintext);

        let invalid_decrypt = decrypt_local_secret(master_key, "invalid");
        assert!(invalid_decrypt.is_err());
        Ok(())
    }

    #[test]
    fn test_github_secret() -> Result<(), Box<dyn std::error::Error>> {
        // Need a valid 32 byte base64 key
        let key_b64 = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        let result = encrypt_github_secret(key_b64, "my_secret")?;
        assert!(!result.is_empty());

        let invalid = encrypt_github_secret("invalid", "my_secret");
        assert!(invalid.is_err());

        let invalid_len = encrypt_github_secret("AAA=", "my_secret");
        assert!(invalid_len.is_err());
        Ok(())
    }
}

#[test]
fn test_local_secret_decryption_too_short() -> Result<(), Box<dyn std::error::Error>> {
    let master_key = "my-super-secret-master-key-that-is-long-enough";
    let invalid = decrypt_local_secret(master_key, "AAA=");
    assert!(invalid.is_err());
    Ok(())
}
