use aes::cipher::generic_array::GenericArray;
use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, OsRng, rand_core::RngCore},
};

pub fn encrypt(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let key_str = std::env::var("SECRET_KEY")?;
    let key_raw = key_str.as_bytes();

    if key_raw.len() != 32 {
        return Err("Key must be 32 bytes long for AES-256".into());
    }

    let key = GenericArray::clone_from_slice(key_raw);
    let cipher = Aes256Gcm::new(&key);

    let mut nonce_bytes = [0u8; 12];

    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let mut cipher_text = cipher
        .encrypt(nonce, data)
        .map_err(|_| "Encryption failed")?;

    cipher_text.extend_from_slice(&nonce_bytes);

    Ok(cipher_text)
}

pub fn decrypt(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let key_str = std::env::var("SECRET_KEY")?;
    let key_raw = key_str.as_bytes();

    if key_raw.len() != 32 {
        return Err("Key must be 32 bytes long for AES-256".into());
    }

    let key = GenericArray::clone_from_slice(key_raw);
    let cipher = Aes256Gcm::new(&key);

    let (cipher_text, nonce) = data.split_at(data.len() - 12);
    let nonce = Nonce::from_slice(nonce);
    let decrypted_data = cipher
        .decrypt(nonce, cipher_text)
        .map_err(|_| "Decryption failed")?;

    Ok(decrypted_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let data = b"Hello, world!";
        let encrypted_data = encrypt(data).unwrap();

        let decrypted_data = decrypt(&encrypted_data).unwrap();

        assert_eq!(data.to_vec(), decrypted_data);
    }
}
