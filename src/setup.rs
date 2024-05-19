use crate::{crypto, store, get_cellar_path};
use rpassword::prompt_password;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

pub fn set_master(master_key: String, overwrite: bool) -> std::io::Result<()> {
    if let Some(hashed) = crypto::hash_argon2(master_key.as_bytes())
    {
        store::write_to_file(".master_key", hashed, overwrite)?;
    }
    
    Ok(())
}

pub fn vodka_is_setup() -> bool {
    let mut vodka_path = PathBuf::new();
    let vodka_dir = ".vodka";
    if let Some(home_dir) = dirs::home_dir() {
        vodka_path = home_dir.join(vodka_dir);
    }

    if !vodka_path.exists() {
        return false;
    }

    let master_key_path = vodka_path.join(".master_key");
    
    if !master_key_path.exists() {
        return false;
    }

    true
}

pub fn setup_db() -> Result<(), crate::Error> {
    let cellar_path = get_cellar_path();
    let connection = Connection::open(&cellar_path).unwrap();
    
    connection.execute(
        "CREATE TABLE IF NOT EXISTS passwords (
            id INTEGER PRIMARY KEY AUTOINCREMENT, 
            name TEXT NOT NULL, 
            login TEXT NOT NULL, 
            password BLOB NOT NULL, 
            comment TEXT
        )",
        [],
    )?;

    Ok(())
}

pub fn setup_vodka() -> Result<(), crate::Error> {
    let vodka_dir = ".vodka";
    let mut vodka_path = PathBuf::new();
    
    if let Some(home_dir) = dirs::home_dir() {
        vodka_path = home_dir.join(vodka_dir);
    }
    
    if vodka_path.exists() {
        eprintln!("Warning: vodka already set up at {:?}. Aborting.", vodka_path);
        return Ok(());
    }

    eprintln!("Welcome to vodka! You have no idea about the greatness that you are in for!\n\nPlease enter a master key, which will be used for adding and retrieving passwords.\n");

    let master_key = prompt_password("Enter master key: ").unwrap();
    if master_key != prompt_password("Confirm master key: ").unwrap() {
        eprintln!("Error: Please enter the same master key!");
        return Ok(());
    }

    fs::create_dir_all(vodka_path)?;
    setup_db()?;
    set_master(master_key, false)?;
    
    Ok(())
}