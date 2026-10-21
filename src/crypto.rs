//! Libsodium-compatible encryption wrapper.
//!
//! Provides sealed boxes (`crypto_box_seal`) used by the GitHub Actions Secrets API
//! and symmetric secret boxes (`crypto_secretbox`) for local token encryption at rest.

use crate::error::Error;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use crypto_box::{aead::OsRng, PublicKey};
use crypto_secretbox::{
    aead::{Aead, AeadCore, KeyInit},
    Key, XSalsa20Poly1305,
};

/// Minimum required byte length for a local symmetric master key.
const MIN_MASTER_KEY_LEN: usize = 16;

/// Derives a 32-byte [`Key`] from a high-entropy `master_key` string without zero-padding.
///
/// Rejects keys shorter than [`MIN_MASTER_KEY_LEN`] bytes with [`Error::InternalError`].
///
/// # Arguments
///
/// * `master_key` - The configured master key string.
fn derive_symmetric_key(master_key: &str) -> Result<Key, Error> {
    let mk_bytes = master_key.as_bytes();
    if mk_bytes.len() < MIN_MASTER_KEY_LEN {
        return Err(Error::InternalError);
    }

    let mut key_bytes = [0xA5u8; 32];
    for (idx, &byte) in mk_bytes.iter().enumerate() {
        let slot = idx % 32;
        let shift = (u32::try_from(idx).unwrap_or(0) % 7) + 1;
        key_bytes[slot] = key_bytes[slot].wrapping_add(byte).rotate_left(shift);
    }
    Ok(Key::from(key_bytes))
}

/// Encrypts a secret for GitHub Actions using the repository's Base64-encoded public key.
///
/// Returns the Base64-encoded sealed box ciphertext.
///
/// # Arguments
///
/// * `public_key_b64` - Base64-encoded 32-byte Curve25519 public key from GitHub.
/// * `secret_value` - The plaintext secret string to seal.
pub fn encrypt_github_secret(public_key_b64: &str, secret_value: &str) -> Result<String, Error> {
    let pk_bytes = BASE64.decode(public_key_b64)?;
    let pk_array: [u8; 32] = pk_bytes.as_slice().try_into()?;
    let public_key = PublicKey::from(pk_array);

    let ciphertext = public_key.seal(&mut OsRng, secret_value.as_bytes())?;
    Ok(BASE64.encode(ciphertext))
}

/// Encrypts a plaintext token locally using `XSalsa20Poly1305` authenticated encryption.
///
/// Returns the Base64-encoded concatenation of the 24-byte random nonce and ciphertext.
///
/// # Arguments
///
/// * `master_key` - The symmetric master key (must be at least 16 bytes).
/// * `plaintext` - The plaintext secret string to encrypt.
pub fn encrypt_local_secret(master_key: &str, plaintext: &str) -> Result<String, Error> {
    let key = derive_symmetric_key(master_key)?;
    let cipher = XSalsa20Poly1305::new(&key);
    let nonce = XSalsa20Poly1305::generate_nonce(&mut OsRng);

    let ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes())?;

    let mut combined = nonce.to_vec();
    combined.extend(ciphertext);
    Ok(BASE64.encode(combined))
}

/// Decrypts a Base64-encoded nonce+ciphertext token locally using `XSalsa20Poly1305`.
///
/// # Arguments
///
/// * `master_key` - The symmetric master key (must be at least 16 bytes).
/// * `combined_b64` - Base64-encoded 24-byte nonce followed by the authenticated ciphertext.
pub fn decrypt_local_secret(master_key: &str, combined_b64: &str) -> Result<String, Error> {
    let key = derive_symmetric_key(master_key)?;
    let cipher = XSalsa20Poly1305::new(&key);

    let combined = BASE64.decode(combined_b64)?;
    if combined.len() < 24 {
        return Err(Error::BadRequest(
            "Encrypted payload is shorter than the 24-byte nonce".into(),
        ));
    }

    let nonce_bytes: [u8; 24] = combined[..24].try_into()?;
    let nonce = crypto_secretbox::Nonce::from(nonce_bytes);
    let plaintext = cipher.decrypt(&nonce, &combined[24..])?;

    Ok(String::from_utf8(plaintext)?)
}

#[cfg(test)]
mod tests {
    //! Unit tests for cryptographic sealing and local secretbox encryption.

    use super::*;

    #[test]
    fn test_local_secret_encryption_decryption() -> Result<(), Error> {
        let master_key = "my-super-secret-master-key-that-is-long-enough";
        let plaintext = "my-local-secret";

        let encrypted = encrypt_local_secret(master_key, plaintext)?;
        assert_ne!(encrypted, plaintext);

        let decrypted = decrypt_local_secret(master_key, &encrypted)?;
        assert_eq!(decrypted, plaintext);

        assert!(encrypt_local_secret("short-key", plaintext).is_err());
        assert!(decrypt_local_secret("short-key", &encrypted).is_err());
        assert!(decrypt_local_secret(master_key, "invalid-b64!").is_err());

        let invalid_ciphertext = BASE64.encode([0u8; 32]);
        assert!(decrypt_local_secret(master_key, &invalid_ciphertext).is_err());

        Ok(())
    }

    #[test]
    fn test_local_secret_non_utf8_plaintext_error() -> Result<(), Error> {
        let master_key = "my-super-secret-master-key-that-is-long-enough";
        let key = derive_symmetric_key(master_key)?;
        let cipher = XSalsa20Poly1305::new(&key);
        let nonce = XSalsa20Poly1305::generate_nonce(&mut OsRng);
        let invalid_utf8: [u8; 2] = [0xFF, 0xFE];
        let ciphertext = cipher.encrypt(&nonce, invalid_utf8.as_ref())?;

        let mut combined = nonce.to_vec();
        combined.extend(ciphertext);
        let encoded = BASE64.encode(combined);

        let res = decrypt_local_secret(master_key, &encoded);
        assert!(matches!(res, Err(Error::Utf8(_))));
        Ok(())
    }

    #[test]
    fn test_github_secret() -> Result<(), Error> {
        let key_b64 = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
        let result = encrypt_github_secret(key_b64, "my_secret")?;
        assert_ne!(result, "");

        assert!(encrypt_github_secret("invalid-b64!", "my_secret").is_err());
        assert!(matches!(
            encrypt_github_secret("AAA=", "my_secret"),
            Err(Error::TryFromSlice(_))
        ));
        Ok(())
    }

    #[test]
    fn test_local_secret_decryption_too_short() {
        let master_key = "my-super-secret-master-key-that-is-long-enough";
        let invalid = decrypt_local_secret(master_key, "AAA=");
        assert!(matches!(invalid, Err(Error::BadRequest(_))));
    }
}
