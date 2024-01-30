use clap::{Parser, command};
use rpassword::prompt_password;
use std::fs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    name: Option<String>,
}

fn setup_vodka() -> std::io::Result<()> {
    println!("Welcome! You have no idea about the greatness that you are in for!\n");

    if let Some(home_dir) = dirs::home_dir() {
        let dir_name = ".vodka";
        let dir_path = home_dir.join(dir_name);
        
        if dir_path.exists() {
            println!("Warning: vodka already set up at {:?}", dir_path);
            return Ok(());
        }

        let master_pass = prompt_password("Enter master password: ").unwrap();
        if master_pass != prompt_password("Confirm master password: ").unwrap() {
            eprintln!("Error: Master password does not match!");
            return Ok(());
        }
        
        fs::create_dir_all(dir_path)?;
    }
    
    Ok(())
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    if let Some(str) = args.name {
        println!("{}", str);
    }
    
    if let Err(e) = setup_vodka() {
        eprintln!("Error creating vodka folder: {}", e);
        std::process::exit(1);
    }
    
    Ok(())
}
