use crate::crypto;
use crate::store;
use rpassword::prompt_password;
use std::fs;
use std::path::PathBuf;

pub fn setup_vodka() -> std::io::Result<()> {
    let vodka_dir = ".vodka";
    let mut dir_path = PathBuf::new();
    
    if let Some(home_dir) = dirs::home_dir() {
        dir_path = home_dir.join(vodka_dir);
    }
    
    if dir_path.exists() {
        eprintln!("Warning: vodka already set up at {:?}. Aborting.", dir_path);
        return Ok(());
    }

    eprintln!("Welcome to vodka! You have no idea about the greatness that you are in for!\n\nPlease enter a master key, which will be used for adding and retrieving passwords.\n");

    let master_pass = prompt_password("Enter master key: ").unwrap();
    if master_pass != prompt_password("Confirm master key: ").unwrap() {
        eprintln!("Error: Please enter the same master key!");
        return Ok(());
    }
    
    fs::create_dir_all(dir_path)?;
    
    if let Some(hashed) = crypto::hash_bcrypt(master_pass.as_bytes())
    {
        store::write_to_file(".master_key", hashed, false).expect("Error: something weird happened while writing master key.");
    }
    
    Ok(())
}