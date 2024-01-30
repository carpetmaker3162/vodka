use argon2::{
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2,
};
use rand_core::OsRng;

pub fn get_hashed(password: &[u8]) -> Option<String> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    
    if let Ok(password_hashed) = argon2.hash_password(password, &salt)
    {
        return Some(password_hashed.to_string());
    }

    None
}

pub fn verify(password: &[u8], hash: String) -> Option<bool> {
    let argon2 = Argon2::default();
    if let Ok(parsed_hash) = PasswordHash::new(&hash)
    {
        return Some(argon2.verify_password(password, &parsed_hash).is_ok());
    }
    
    None
}

