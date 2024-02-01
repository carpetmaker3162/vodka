use crate::crypto;
use crate::store;
use rpassword::prompt_password;
use std::fs;
use std::path::PathBuf;

pub fn setup_vodka() -> std::io::Result<()> {
    println!("Welcome! You have no idea about the greatness that you are in for!\n");
    
    let vodka_dir = ".vodka";
    let mut dir_path = PathBuf::new();

    if let Some(home_dir) = dirs::home_dir() {
        dir_path = home_dir.join(vodka_dir);
    }

    if dir_path.exists() {
        println!("Warning: vodka already set up at {:?}. Aborting.", dir_path);
        return Ok(());
    }

    let master_pass = prompt_password("Enter master password: ").unwrap();
    if master_pass != prompt_password("Confirm master password: ").unwrap() {
        eprintln!("Error: Master password does not match!");
        return Ok(());
    }
    
    fs::create_dir_all(dir_path)?;
    
    if let Some(hashed) = crypto::hash_bcrypt(master_pass.as_bytes())
    {
        store::write_to_file(".master_key", hashed, false);
    } else {
        println!("Something weird happened.");
    }
    
    Ok(())
}