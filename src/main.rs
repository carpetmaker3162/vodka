#![allow(unused_imports)]

use clap::{arg, Arg, ArgAction, Command};
use vodka::{config, crypto, display, setup, store, transport};
use vodka::{Entry, SearchResult};

fn cli() -> Command {
    Command::new("vodka")
        .about("Password Manager")
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
                .arg(arg!(<FULLNAME>).required(false).conflicts_with("id"))
                .arg(arg!(-i --id <ID>).required_unless_present("FULLNAME").num_args(1))
        )
        .subcommand(
            Command::new("search")
                .about("Search for an entry with fullname")
                .arg(arg!(<FULLNAME>).required(true))
        )
        .subcommand(
            Command::new("delete")
                .about("Delete an entry with its ID")
                .arg(arg!(<ID>).required(true))
        )
        .subcommand(
            Command::new("list")
                .about("List all existing entries")
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
        .subcommand(
            Command::new("change-master")
                .about("Change the master key")
        )
        .subcommand(
            Command::new("erase")
                .about("Erase all existing passwords")
        )
}

fn main() -> Result<(), vodka::Error> {
    let matches = cli().get_matches();
    
    match matches.subcommand() {
        Some(("setup", _)) => {
            if let Err(e) = setup::setup_vodka() {
                eprintln!("Error while setting up vodka: {:?}", e);
                std::process::exit(1);
            }
        },
        Some(("add", sub_matches)) => {
            let master_key_sha256 = vodka::unlock();

            let fullname = sub_matches.get_one::<String>("FULLNAME").unwrap().to_string();
            let (login, name) = vodka::parse_fullname(fullname);
            let comment = sub_matches.get_one::<String>("comment").unwrap_or(&String::new()).to_string();
            let password_unencrypted: String;
            
            if sub_matches.get_flag("random") {
                password_unencrypted = crypto::get_random_password();
            } else {
                password_unencrypted = sub_matches.get_one::<String>("password").unwrap().to_string()
            }

            let entry = Entry::new(name, login, password_unencrypted, comment, &master_key_sha256);
            if let Err(e) = vodka::add_entry(entry) {
                eprintln!("Error while adding password: {:?}", e);
            }
        },
        Some(("copy", sub_matches)) => {
            let master_key_sha256 = vodka::unlock();
            
            // search by id
            if sub_matches.contains_id("id") {
                let id = match sub_matches.get_one::<String>("id").unwrap().parse::<i32>() {
                    Ok(value) => value,
                    Err(e) => {
                        eprintln!("Error while parsing command arguments: {}", e);
                        std::process::exit(1);
                    }
                };
                
                if let Some(entry) = store::get_entry_by_id(id) {
                    let password = entry.get_password(&master_key_sha256);
                    vodka::copy_to_clipboard(password);
                } else {
                    eprintln!("No such entry found!");
                }
            } else { // search by fullname
                let fullname = sub_matches.get_one::<String>("FULLNAME").unwrap().to_string();
                let (login, name) = vodka::parse_fullname(fullname);

                // strict search
                match vodka::get_entry(name, login, true) {
                    SearchResult::OneResult(entry) => {
                        let password = entry.get_password(&master_key_sha256);
                        vodka::copy_to_clipboard(password);
                    },
                    SearchResult::NoResults => { eprintln!("No entries found!"); },
                    SearchResult::ManyResults(_) => { eprintln!("Several possible entries found. Try searching?"); }
                }
            }
        },
        Some(("search", sub_matches)) => {
            vodka::unlock();

            let fullname = sub_matches.get_one::<String>("FULLNAME").unwrap().to_string();
            let (login, name) = vodka::parse_fullname(fullname);

            match vodka::get_entry(name, login, false) {
                SearchResult::OneResult(entry) => {
                    display::display(vec![entry]);
                },
                SearchResult::NoResults => { eprintln!("No entries found!") },
                SearchResult::ManyResults(entries) => {
                    display::display(entries);
                }
            }
        },
        Some(("delete", sub_matches)) => {
            vodka::unlock();
            
            let id = match sub_matches.get_one::<String>("ID").unwrap().parse::<i32>() {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("Error while parsing command arguments: {}", e);
                    std::process::exit(1);
                }
            };

            if let Err(e) = store::delete_entry(id) {
                eprintln!("Error while deleting entry {}: {:?}", id, e);
            }
        },
        Some(("list", _)) => {
            vodka::unlock();
            display::display_all();
        },
        Some(("export", sub_matches)) => {
            let master_key_sha256 = vodka::unlock();

            let file_path = sub_matches.get_one::<String>("FILE").unwrap().as_str();
            if let Err(e) = transport::export(file_path, &master_key_sha256, false) {
                match e {
                    vodka::Error::ExportFileExists(_) => {
                        let confirmed = vodka::ask_for_confirmation(
                            format!("{} already exists. This will overwrite the existing file.", file_path)
                        );

                        if confirmed {
                            transport::export(file_path, &master_key_sha256, true)?;
                        }
                    },
                    _ => { eprintln!("Error during exporting: {:?}", e) },
                }
            }
        },
        Some(("import", sub_matches)) => {
            let master_key_sha256 = vodka::unlock();

            let file_path = sub_matches.get_one::<String>("FILE").unwrap().as_str();
            if let Err(e) = transport::import(file_path, &master_key_sha256, false) {
                match e {
                    vodka::Error::ImportFileExists(_) => {
                        let confirmed = vodka::ask_for_confirmation(
                            String::from("cellar.sqlite already exists. This will overwrite the existing file.")
                        );
                        
                        if confirmed {
                            transport::import(file_path, &master_key_sha256, true)?;
                        }
                    },
                    _ => { eprintln!("Error during importing: {:?}", e) },
                }
            }
        },
        Some(("change-master", _)) => {
            vodka::unlock_with_prompt("Enter old master key: ");
            
            let new_master_key = rpassword::prompt_password("Enter new master key: ").unwrap();
            if new_master_key != rpassword::prompt_password("Confirm new master key: ").unwrap() {
                eprintln!("Error: Please enter the same master key! (No changes were made)");
                std::process::exit(1);
            }

            setup::set_master(new_master_key, true)?;
        },
        Some(("erase", _)) => {
            let entry_count = store::get_all_rows().len();
            
            let confirmed = vodka::ask_for_confirmation(format!("{} entries will be erased.", entry_count));
            if !confirmed {
                std::process::exit(0);
            }
            
            vodka::unlock();
            
            store::erase_all()?;
        },
        None => eprintln!("what do you want?"),
        _ => unreachable!(),
    }
    
    Ok(())
}
