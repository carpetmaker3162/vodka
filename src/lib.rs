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

    let query_match = transaction.query_row(
        "SELECT password FROM passwords WHERE name = ?",
        params![name],
        |row| {
            row.get::<usize, Vec<u8>>(0)
        }
    ).unwrap();

    let decrypted_password_bytes = crypto::decrypt_aes256(&query_match, master_key);
    
    String::from_utf8(decrypted_password_bytes).ok()
}