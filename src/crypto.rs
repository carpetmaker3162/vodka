#![allow(dead_code)]

use aes::Aes256;
use aes::cipher::{BlockEncrypt, BlockDecrypt, KeyInit};
use aes::cipher::generic_array::GenericArray;
use argon2::Argon2;
use argon2::password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString
};
use crate::store;
use rand_core::OsRng;
use sha2::{Sha256, Digest};

pub fn hash_bcrypt(password: &[u8]) -> Option<String> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    
    if let Ok(password_hashed) = argon2.hash_password(password, &salt)
    {
        return Some(password_hashed.to_string());
    }

    None
}

pub fn hash_sha256(password: &[u8]) -> Vec<u8> {
    let mut sha256 = Sha256::new();
    
    sha256.update(password);
    
    sha256.finalize().to_vec()
}

pub fn verify_password(password: &[u8]) -> Option<bool> {
    let hash = match store::read_file(".master_key") {
        Ok(hash) => hash.to_string(),
        Err(_) => { return None; }
    };
    
    let argon2 = Argon2::default();
    if let Ok(parsed_hash) = PasswordHash::new(&hash)
    {
        return Some(argon2.verify_password(password, &parsed_hash).is_ok());
    }
    
    None
}

pub fn encrypt_aes256(data: &[u8], key: &[u8]) -> Vec<u8> {
    assert_eq!(key.len(), 32, "Key length must be 32 bytes (256 bits)");

    let cipher = Aes256::new(GenericArray::from_slice(key));

    let mut padded_data = data.to_vec();
    let padding_length = 16 - (data.len() % 16);
    padded_data.extend(vec![padding_length as u8; padding_length]);

    let mut encrypted_data = Vec::new();
    for chunk in padded_data.chunks_exact(16) {
        let mut block = GenericArray::clone_from_slice(chunk);

        cipher.encrypt_block(&mut block);
        encrypted_data.extend_from_slice(block.as_slice());
    }

    encrypted_data
}

pub fn decrypt_aes256(encrypted_data: &[u8], key: &[u8]) -> Vec<u8> {
    assert_eq!(key.len(), 32, "Key length must be 32 bytes (256 bits)");

    let cipher = Aes256::new(GenericArray::from_slice(key));

    let mut decrypted_data = Vec::new();
    for chunk in encrypted_data.chunks_exact(16) {
        let mut block = GenericArray::clone_from_slice(chunk);

        cipher.decrypt_block(&mut block);
        decrypted_data.extend_from_slice(block.as_slice());
    }

    if let Some(&padding_length) = decrypted_data.last() {
        let padding_length = padding_length as usize;
        if padding_length <= decrypted_data.len() {
            decrypted_data.truncate(decrypted_data.len() - padding_length);
        }
    }

    decrypted_data
}