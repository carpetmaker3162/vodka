use arboard::Clipboard;
use cli_table::{Cell, CellStruct};
use rpassword::prompt_password;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

pub mod config;
pub mod crypto;
pub mod display;
pub mod setup;
pub mod store;
pub mod transport;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Entry {
    pub id: i32,
    pub name: String,
    pub login: String,
    pub password: Vec<u8>, // encrypted
    pub comment: String
}

impl Entry {
    // create an Entry with plaintext password. need master key
    pub fn new(name: String, login: String, password: String, comment: String, master_key: &[u8]) -> Entry {
        // might be a problem with assuming the next id?
        Entry {
            id: store::get_next_id(),
            name,
            login,
            password: crypto::encrypt_aes256(password.as_bytes(), master_key),
            comment
        }
    }

    // get decrypted password. need master key
    pub fn get_password(&self, master_key: &[u8]) -> String {
        let decrypted_password_bytes = crypto::decrypt_aes256(&self.password, master_key);
        String::from_utf8(decrypted_password_bytes).unwrap()
    }

    // for csv exporting (serialization)
    pub fn decrypted(&self, master_key: &[u8]) -> DecryptedEntry {
        DecryptedEntry {
            id: self.id,
            name: self.name.clone(),
            login: self.login.clone(),
            password: self.get_password(master_key),
            comment: self.comment.clone()
        }
    }

    pub fn as_table_row(&self) -> Vec<CellStruct> {
        vec![
            self.id.cell(),
            (&self.name).cell(),
            (&self.login).cell(),
            "********".cell(),
            (&self.comment).cell()
        ]
    }
}

// for csv exporting (serialization)
#[derive(Debug, Deserialize, Serialize)]
pub struct DecryptedEntry {
    pub id: i32,
    pub name: String,
    pub login: String,
    pub password: String,
    pub comment: String
}

#[derive(Debug)]
pub enum Error {
    ExportFileExists(PathBuf),
    ImportFileExists(PathBuf),
    VodkaFolderNotFound,
    MasterKeyFileNotFound,
    CellarFileNotFound,
    ConfigKeyNotFound(String),
    CsvError(csv::Error),
    RusqliteError(rusqlite::Error),
    IOError(std::io::Error),
}

#[derive(Debug)]
pub enum ErrorKind {
    ExportFileExists,
    ImportFileExists,
    VodkaFolderNotFound,
    MasterKeyFileNotFound,
    CellarFileNotFound,
    ConfigKeyNotFound,
    CsvError,
    RusqliteError,
    IOError,
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        match self {
            Error::ExportFileExists(_) => ErrorKind::ExportFileExists,
            Error::ImportFileExists(_) => ErrorKind::ImportFileExists,
            Error::VodkaFolderNotFound => ErrorKind::VodkaFolderNotFound,
            Error::MasterKeyFileNotFound => ErrorKind::MasterKeyFileNotFound,
            Error::CellarFileNotFound => ErrorKind::CellarFileNotFound,
            Error::ConfigKeyNotFound(_) => ErrorKind::ConfigKeyNotFound,
            Error::CsvError(_) => ErrorKind::CsvError,
            Error::RusqliteError(_) => ErrorKind::RusqliteError,
            Error::IOError(_) => ErrorKind::IOError,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ExportFileExists(path) => write!(f, "{} already exists", path.display()),
            Error::ImportFileExists(path) => write!(f, "cellar already exists at {}", path.display()),
            Error::VodkaFolderNotFound => write!(f, "{} folder not found", get_vodka_path("").display()),
            Error::MasterKeyFileNotFound => write!(f, "{} file not found", get_vodka_path(".master_key").display()),
            Error::CellarFileNotFound => write!(f, "{} file not found", get_cellar_path().display()),
            Error::ConfigKeyNotFound(s) => write!(f, "{} file not found", s),
            Error::CsvError(err) => write!(f, "CSV error: {}", err),
            Error::RusqliteError(err) => write!(f, "SQLite error: {}", err),
            Error::IOError(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Self {
        Error::CsvError(err)
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Error::RusqliteError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IOError(err)
    }
}

pub enum SearchResult {
    OneResult(Entry),
    NoResults,
    ManyResults(Vec<Entry>),
}

pub fn get_cellar_path() -> PathBuf {
    get_vodka_path("cellar.sqlite")
}

pub fn get_db() -> Connection {
    let cellar_path = get_cellar_path();
    Connection::open(cellar_path).unwrap()
}

// get absolute path of a file in .vodka folder
pub fn get_vodka_path(file_name: &str) -> PathBuf {
    let mut file_path = PathBuf::new();
    let vodka_dir = ".vodka";

    if let Some(home_dir) = dirs::home_dir() {
        if file_name.is_empty() {
            return home_dir.join(vodka_dir);
        }

        file_path = home_dir.join(vodka_dir).join(file_name);
    }
    
    /* if !file_path.exists() {
        eprintln!("Error: file {} not found!", file_name);
        std::process::exit(1);
    } */

    file_path
}

pub fn get_absolute_path(path: &str) -> PathBuf {
    std::env::current_dir().unwrap().join(path)
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

pub fn add_entry(entry: Entry) -> Result<(), Error> {
    store::add_entry(entry.name, entry.login, &entry.password, entry.comment)?;
    
    Ok(())
}

// will always return a single entry (for now?)
pub fn get_entry(name: String, login: String, strict: bool) -> SearchResult {
    let result_entries: Vec<Entry> = store::search_entries(name, login.clone());
    
    if result_entries.len() == 1 {
        return SearchResult::OneResult(result_entries[0].clone());
    } else if result_entries.len() == 0 {
        return SearchResult::NoResults;
    }

    // if looking for a single entry, and no login is provided, return the entry with no login
    if strict && login.is_empty() {
        if let Some(entry) = result_entries.iter().find(|&entry| entry.login.is_empty()) {
            return SearchResult::OneResult(entry.clone());
        }
    }
    
    SearchResult::ManyResults(result_entries)
}

// Ask the user for the master key. Once verified, returns the SHA-256 of the password
pub fn unlock() -> Vec<u8> {
    unlock_with_prompt("Enter master key: ")
}

pub fn unlock_with_prompt(prompt: &str) -> Vec<u8> {
    let master_key_plaintext = prompt_password(prompt).unwrap();
    
    if let Some(verified) = crypto::verify_password(master_key_plaintext.as_bytes()) {
        if verified {
            return crypto::hash_sha256(master_key_plaintext.as_bytes());
        }
    }
    
    eprintln!("Error: Failed to verify!");
    std::process::exit(1);
}

pub fn vodka_is_setup() -> bool {
    if let Err(e) = setup::check_setup() {
        eprintln!("Error with vodka's setup: {}", e);
        return false;
    }

    true
}

pub fn ask_for_confirmation(message: String) -> bool {
    eprintln!("{}\n", message);
    eprint!("Proceed? [y/N]: ");

    let mut selection = String::new();
    std::io::stdin().read_line(&mut selection).expect("Failed to read line");
    
    if selection.trim().to_lowercase().chars().next().unwrap() == 'y' {
        return true;
    }

    false
}

pub fn copy_to_clipboard(content: String) {
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text(content).unwrap();
}