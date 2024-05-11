use arboard::Clipboard;
use clap::{arg, Command};
use rpassword::prompt_password;
use vodka::{crypto, setup, transport};
use vodka::{Entry, ask_for_confirmation};

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
                .arg(arg!(-p --password <PASSWORD>).required_unless_present("random"))
                .arg(arg!(-c --comment <COMMENT>).required(false))
                .arg(arg!(-r --random).num_args(0))
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
        .subcommand(
            Command::new("export")
                .about("Export your passwords to a CSV file. Warning: They will be unencrypted so delete the file after you're done with it.")
                .arg(arg!(<FILE>).required(true))
        )
        .subcommand(
            Command::new("import")
                .about("Import passwords from a CSV file")
                .arg(arg!(<FILE>).required(true))
        )
}

fn main() -> Result<(), vodka::Error> {
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
            let comment = sub_matches.get_one::<String>("comment").unwrap_or(&String::new()).to_string();
            let password_unencrypted: String;
            if sub_matches.contains_id("random") {
                password_unencrypted = crypto::get_random_password();
            } else {
                password_unencrypted = sub_matches.get_one::<String>("password").unwrap().to_string()
            }

            let entry = Entry::new(name, login, password_unencrypted, comment, &master_key_sha256);
            match vodka::add_password(entry) {
                Ok(_) => {},
                Err(e) => { eprintln!("Error while adding password: {:?}", e); }
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
        /* Some(("search", sub_matches)) => {
            
        },
        Some(("list", sub_matches)) => {

        }, */
        Some(("export", sub_matches)) => {
            let master_key_sha256 = vodka::unlock();

            let file_path = sub_matches.get_one::<String>("FILE").unwrap().as_str();
            if let Err(e) = transport::export(file_path, &master_key_sha256, false) {
                match e.kind() {
                    vodka::ErrorKind::ExportFileExists => if ask_for_confirmation(format!("{} already exists. This will overwrite the existing file.", file_path)) {
                            transport::export(file_path, &master_key_sha256, true)?;
                        },
                    _ => { eprintln!("Error during exporting: {:?}", e) },
                }
            }
        },
        Some(("import", sub_matches)) => {
            let master_key_sha256 = vodka::unlock();

            let file_path = sub_matches.get_one::<String>("FILE").unwrap().as_str();
            if let Err(e) = transport::import(file_path, &master_key_sha256, false) {
                match e.kind() {
                    vodka::ErrorKind::ImportFileExists => if ask_for_confirmation(String::from("cellar.sqlite already exists. This will overwrite the existing file.")) {
                            transport::import(file_path, &master_key_sha256, true)?;
                        },
                    _ => { eprintln!("Error during importing: {:?}", e) },
                }
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
