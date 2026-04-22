use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::pbkdf2;
use ring::rand::{SecureRandom, SystemRandom};
use std::num::NonZeroU32;

use crate::machine_id::get_machine_id;

const SALT_LEN: usize = 32;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
const ITERATIONS: u32 = 100_000;

fn get_salt() -> Vec<u8> {
    // 使用固定盐值结合机器码派生
    let mut salt = vec![0u8; SALT_LEN];
    let machine_id = get_machine_id().unwrap_or_else(|_| "default-salt".to_string());
    let machine_bytes = machine_id.as_bytes();

    // 用机器码填充盐的前部分
    for (i, byte) in machine_bytes.iter().enumerate() {
        salt[i % SALT_LEN] ^= byte;
    }
    // 固定后部分（8字节）
    salt[SALT_LEN - 8..].copy_from_slice(b"excl-tol");

    salt
}

pub fn get_machine_key() -> Result<Vec<u8>, String> {
    let salt = get_salt();
    let machine_id = get_machine_id().map_err(|e| e.to_string())?;

    let mut key = vec![0u8; KEY_LEN];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        NonZeroU32::new(ITERATIONS).unwrap(),
        &salt,
        machine_id.as_bytes(),
        &mut key,
    );

    Ok(key)
}

pub fn encrypt_vault(plaintext: &str, key: &[u8]) -> Result<Vec<u8>, String> {
    let unbound_key = UnboundKey::new(&AES_256_GCM, key)
        .map_err(|e| format!("Failed to create key: {}", e))?;
    let less_safe_key = LessSafeKey::new(unbound_key);

    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill(&mut nonce_bytes)
        .map_err(|e| format!("RNG error: {}", e))?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    let mut in_out = plaintext.as_bytes().to_vec();
    less_safe_key
        .seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // 格式: nonce || ciphertext+tag
    let mut result = Vec::with_capacity(NONCE_LEN + in_out.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&in_out);

    Ok(result)
}

pub fn decrypt_vault(encrypted: &[u8], key: &[u8]) -> Result<String, String> {
    if encrypted.len() < NONCE_LEN + 16 {
        return Err("Invalid encrypted data".to_string());
    }

    let unbound_key = UnboundKey::new(&AES_256_GCM, key)
        .map_err(|e| format!("Failed to create key: {}", e))?;
    let less_safe_key = LessSafeKey::new(unbound_key);

    let (nonce_bytes, ciphertext) = encrypted.split_at(NONCE_LEN);
    // Convert &[u8] to [u8; 12]
    let mut nonce_arr = [0u8; NONCE_LEN];
    nonce_arr.copy_from_slice(nonce_bytes);
    let nonce = Nonce::assume_unique_for_key(nonce_arr);

    let mut in_out = ciphertext.to_vec();
    let plaintext = less_safe_key
        .open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext.to_vec())
        .map_err(|e| format!("Invalid UTF-8: {}", e))
}
