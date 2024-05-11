use rpassword::prompt_password;
use std::path::PathBuf;

pub mod setup;
pub mod crypto;
pub mod store;

pub struct Entry {
    pub name: String,
    pub login: String,
    pub password: Vec<u8>, // encrypted
    pub comment: String
}

impl Entry {
    // create an Entry with unencrypted password and master key
    pub fn new(name: String, login: String, password: String, comment: String, master_key: &[u8]) -> Entry {
        Entry {
            name,
            login,
            password: crypto::encrypt_aes256(password.as_bytes(), master_key),
            comment
        }
    }

    pub fn decrypted_password(&self, master_key: &[u8]) -> String {
        let decrypted_password_bytes = crypto::decrypt_aes256(&self.password, master_key);
        String::from_utf8(decrypted_password_bytes).unwrap()
    }
}

pub fn get_cellar_path() -> PathBuf {
    get_path("cellar.sqlite")
}

pub fn get_path(file_name: &str) -> PathBuf {
    let mut file_path = PathBuf::new();
    let vodka_dir = ".vodka";

    if let Some(home_dir) = dirs::home_dir() {
        file_path = home_dir.join(vodka_dir).join(file_name);
    }
    
    if !file_path.exists() {
        eprintln!("Error: file {} not found!", file_name);
        std::process::exit(1);
    }

    file_path
}

// `login@name` or `name`
pub fn parse_fullname(fullname: String) -> (String, String) {
    match fullname.rfind("@") {
        Some(index) => {
            let login = fullname[..index].to_string();
            let name = fullname[index + 1..].to_string();
            (login, name)
        },
        _ => (String::new(), fullname.clone())
    }
}

pub fn add_password(entry: Entry) -> Result<(), rusqlite::Error> {
    store::add_entry(entry.name, entry.login, &entry.password, entry.comment)?;
    
    Ok(())
}

pub fn get_password(name: String, login: String, master_key: &[u8], strict: bool) -> Option<String> {
    let result_entries: Vec<Entry> = store::search_entries(name, login.clone());
    
    if result_entries.len() == 1 {
        return Some(result_entries[0].decrypted_password(master_key));
    } else if result_entries.len() == 0 {
        eprintln!("No entries found!");
        std::process::exit(1);
    }

    // if looking for a single entry, and no login is provided, return the entry with no login
    if strict && login.is_empty() {
        if let Some(entry) = result_entries.iter().find(|&entry| entry.login.is_empty()) {
            return Some(entry.decrypted_password(master_key));
        }
    }
    
    eprintln!("Multiple entries found! (not implemented yet)");
    std::process::exit(0);
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