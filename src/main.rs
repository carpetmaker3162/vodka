use arboard::Clipboard;
use clap::{arg, Command};
use rpassword::prompt_password;
use vodka::setup;

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
            let master_key_sha256 = vodka::unlock();
            
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