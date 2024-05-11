use arboard::Clipboard;
use clap::{arg, Command};
use rpassword::prompt_password;
use vodka::{setup, Entry};

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
                .arg(arg!(<FULLNAME>).required(true))
                .arg(arg!(-p --password <PASSWORD>).required(true))
                .arg(arg!(-c --comment <COMMENT>).required(false))
        )
        .subcommand(
            Command::new("copy")
                .about("Copy an existing password to clipboard")
                .arg(arg!(<FULLNAME>).required(true))
        )
        .subcommand(
            Command::new("change-master")
                .about("Change the master key")
        )
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
            let master_key_sha256 = vodka::unlock();

            let fullname = sub_matches.get_one::<String>("FULLNAME").unwrap().to_string();
            let (login, name) = vodka::parse_fullname(fullname);
            let password_unencrypted = sub_matches.get_one::<String>("password").unwrap().to_string();
            let comment = sub_matches.get_one::<String>("comment").unwrap_or(&String::new()).to_string();

            let entry = Entry::new(name, login, password_unencrypted, comment, &master_key_sha256);
            match vodka::add_password(entry) {
                Ok(_) => {},
                Err(e) => { eprintln!("Error while adding password: {}", e); }
            }
        },
        Some(("copy", sub_matches)) => {
            let master_key_sha256 = vodka::unlock();

            let fullname = sub_matches.get_one::<String>("FULLNAME").unwrap().to_string();
            let (login, name) = vodka::parse_fullname(fullname);

            // strict search
            if let Some(password) = vodka::get_password(name, login, &master_key_sha256, true) {
                let mut clipboard = Clipboard::new().unwrap();
                clipboard.set_text(password).unwrap();
            }
        },
        Some(("change-master", _)) => {
            vodka::unlock_with_prompt("Enter old master key: ");
            
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
