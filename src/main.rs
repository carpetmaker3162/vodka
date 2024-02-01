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
}

// Ask the user for the master key. Once verified, returns the SHA-256 of the password
fn unlock() -> Vec<u8> {
    let master_key_plaintext = prompt_password("Enter master key: ").unwrap();
    
    if crypto::verify_password(master_key_plaintext.as_bytes()).unwrap() {
        return crypto::hash_sha256(master_key_plaintext.as_bytes());
    } else {
        eprintln!("Error: Failed to verify!");
        std::process::exit(1);
    }
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

            let master_key_sha256 = unlock();
            match vodka::add_password(&name, &login, password, comment, &master_key_sha256) {
                Ok(_) => {},
                Err(e) => { eprintln!("Error while adding password: {}", e); }
            }
        },
        Some(("copy", sub_matches)) => {
            let name = sub_matches.get_one::<String>("NAME").unwrap();
            
            // TODO: search for passwords using login as well. (currently the login field is useless)
            let master_key_sha256 = unlock();

            if let Some(password) = vodka::get_password(&name, &master_key_sha256) {
                let mut clipboard = Clipboard::new().unwrap();
                clipboard.set_text(password).unwrap();
            } else {
                eprintln!("Error: Failed to fetch password (likely because .vodka folder doesn't exist)")
            }
        },
        _ => {}
    }
    
    Ok(())
}