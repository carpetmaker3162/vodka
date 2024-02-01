use clap::{Parser, command};

mod setup;
mod crypto;
mod store;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    name: Option<String>,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    if let Some(str) = args.name {
        println!("{}", str);
    }

    use rpassword::prompt_password;
    
    if let Err(e) = setup::setup_vodka() {
        eprintln!("Error setting up vodka: {}", e);
        std::process::exit(1);
    }

    let master_key_plaintext = prompt_password("yo: ").unwrap();
    let master_key_sha256: Vec<u8>;
    if crypto::verify_password(master_key_plaintext.as_bytes()).unwrap() {
        master_key_sha256 = crypto::hash_sha256(master_key_plaintext.as_bytes());
        println!("{}", master_key_sha256.len());
        vodka::add_password("Google", "needmoney33", "goodbye987", "", &master_key_sha256);
        
        if let Some(string) = vodka::get_password("Google", &master_key_sha256) {
            println!("{}", string);
        }
    }
    
    Ok(())
}