use arboard::Clipboard;
use clap::{arg, Command};
use rpassword::prompt_password;

mod setup;
mod crypto;
mod store;

fn cli() -> Command {
    Command::new("vodka")
        .about("Password Manager")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("setup")
                .about("Sets up vodka")
        )
        .subcommand(
            Command::new("add")
                .about("Add a new password")
                .arg(arg!(<LOGIN>).required(true))
                .arg(arg!(-p --password <PASSWORD>).required(true))
                .arg(arg!(-c --comment <COMMENT>).required(false))
        )
        .subcommand(
            Command::new("copy")
                .about("Copy an existing password to clipboard")
                .arg(arg!(<NAME>).required(true))
        )
        .subcommand(
            Command::new("change-master")
                .about("Change the master key")
        )
}

// Ask the user for the master key. Once verified, returns the SHA-256 of the password
fn unlock() -> Vec<u8> {
    unlock_with_prompt("Enter master key: ")
}

fn unlock_with_prompt(prompt: &str) -> Vec<u8> {
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

fn main() -> std::io::Result<()> {
    let matches = cli().get_matches();
    
    match matches.subcommand() {
        Some(("setup", _)) => {
            if let Err(e) = setup::setup_vodka() {
                eprintln!("Error while setting up vodka: {}", e);
                std::process::exit(1);
            }
        },
        Some(("add", sub_matches)) => {
            let master_key_sha256 = unlock();

            let login_arg = sub_matches.get_one::<String>("LOGIN").unwrap();
            let password = sub_matches.get_one::<String>("password").unwrap();
            let default_comment = String::new();
            let comment = sub_matches.get_one::<String>("comment").unwrap_or(&default_comment);
            
            let (login, name) = match login_arg.rfind("@") {
                Some(index) => {
                    let login = login_arg[..index].to_string();
                    let name = login_arg[index + 1..].to_string();
                    (login, name)
                },
                _ => (String::new(), login_arg.clone())
            };

            match vodka::add_password(&name, &login, password, comment, &master_key_sha256) {
                Ok(_) => {},
                Err(e) => { eprintln!("Error while adding password: {}", e); }
            }
        },
        Some(("copy", sub_matches)) => {
            let master_key_sha256 = unlock();
            
            // TODO: search for passwords using login as well. (currently the login field is useless)
            let name = sub_matches.get_one::<String>("NAME").unwrap();

            if let Some(password) = vodka::get_password(&name, &master_key_sha256) {
                let mut clipboard = Clipboard::new().unwrap();
                clipboard.set_text(password).unwrap();
            } else {
                eprintln!("Error: Failed to fetch password (likely because .vodka folder doesn't exist)")
            }
        },
        Some(("change-master", _)) => {
            unlock_with_prompt("Enter old master key: ");
            
            let new_master_key = prompt_password("Enter new master key: ").unwrap();
            if new_master_key != prompt_password("Confirm new master key: ").unwrap() {
                eprintln!("Error: Please enter the same master key! (No changes were made)");
                std::process::exit(1);
            }

            setup::set_master(new_master_key, true)?;
        },
        _ => {}
    }
    
    Ok(())
}