use clap::{Parser, command};

mod setup;
mod crypt;
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
    
    if let Err(e) = setup::setup_vodka() {
        eprintln!("Error setting up vodka: {}", e);
        std::process::exit(1);
    }
    
    Ok(())
}
