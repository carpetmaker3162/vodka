use rpassword::prompt_password;
use rusqlite::{Connection, params};
use std::path::PathBuf;

pub mod setup;
pub mod crypto;
pub mod store;

pub fn add_password(name: &str, login: &str, password: &str, comment: &str, master_key: &[u8]) -> Result<(), rusqlite::Error> {
    let encrypted = crypto::encrypt_aes256(password.as_bytes(), master_key);
    store::add_entry(name, login, &encrypted, comment)?;
    
    Ok(())
}

pub fn get_password(name: &str, master_key: &[u8]) -> Option<String> {
    let mut file_path = PathBuf::new();
    let vodka_dir = ".vodka";
    let db_file = "cellar.sqlite";

    if let Some(home_dir) = dirs::home_dir() {
        file_path = home_dir.join(vodka_dir).join(db_file);
    }
    
    if !file_path.exists() {
        return None;
    }

    let mut connection = Connection::open(file_path).unwrap();
    let transaction = connection.transaction().unwrap();

    let query_match = match transaction.query_row(
        "SELECT password FROM passwords WHERE name = ?",
        params![name],
        |row| {
            row.get::<usize, Vec<u8>>(0)
        }
    ) {
        Ok(query_match) => { query_match },
        Err(err) => match err {
            rusqlite::Error::QueryReturnedNoRows => {
                eprintln!("No such entry found!");
                std::process::exit(1);
            }
            _ => {
                eprintln!("Some other error occurred: {}", err);
                std::process::exit(1);
            }
        }
    };

    let decrypted_password_bytes = crypto::decrypt_aes256(&query_match, master_key);
    
    String::from_utf8(decrypted_password_bytes).ok()
}

// Ask the user for the master key. Once verified, returns the SHA-256 of the password
pub fn unlock() -> Vec<u8> {
    unlock_with_prompt("Enter master key: ")
}

pub fn unlock_with_prompt(prompt: &str) -> Vec<u8> {
    if !setup::vodka_is_setup() {
        eprintln!("Vodka is not set up!");
        eprintln!("To set up: `vodka setup`");
        std::process::exit(1);
    }

    let master_key_plaintext = prompt_password(prompt).unwrap();
    
    if let Some(verified) = crypto::verify_password(master_key_plaintext.as_bytes()) {
        if verified {
            return crypto::hash_sha256(master_key_plaintext.as_bytes());
        }
    }
    
    eprintln!("Error: Failed to verify!");
    std::process::exit(1);
}