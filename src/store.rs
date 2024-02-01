use rusqlite::{Connection, params};
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

pub fn read_file(file_name: &str) -> std::io::Result<String> {
    let mut file_path = PathBuf::new();
    let mut file_content = String::new();
    let vodka_dir = ".vodka";
    
    if let Some(home_dir) = dirs::home_dir() {
        file_path = home_dir.join(vodka_dir).join(file_name);
    }
    
    let mut file = fs::File::open(&file_path)?;
    if let Err(err) = file.read_to_string(&mut file_content)
    {
        eprintln!("Error: failed to open file {:?} ({})", file_path, err);
    }
    
    Ok(file_content)
}

pub fn write_to_file(file_name: &str, content: String, overwrite: bool) -> std::io::Result<()> {
    let mut file_path = PathBuf::new();
    let vodka_dir = ".vodka";

    if let Some(home_dir) = dirs::home_dir() {
        file_path = home_dir.join(vodka_dir).join(file_name);
    }

    if file_path.exists() && !overwrite {
        eprintln!("Error: file {:?} already exists", file_path);
    }

    let mut file = fs::File::create(&file_path)?;
    if let Err(err) = file.write_all(content.as_bytes())
    {
        eprintln!("Error: failed to write to file {:?} ({})", file_path, err);
    }

    Ok(())
}

pub fn add_entry(name: &str, login: &str, password: &[u8], comment: &str) -> Result<(), rusqlite::Error> {
    let mut file_path = PathBuf::new();
    let vodka_dir = ".vodka";
    let db_file = "cellar.sqlite";

    if let Some(home_dir) = dirs::home_dir() {
        file_path = home_dir.join(vodka_dir).join(db_file);
    }

    let mut connection: Connection;
    if !file_path.exists() {
        connection = Connection::open(file_path).unwrap();
        connection.execute(
            "CREATE TABLE passwords (id INTEGER PRIMARY KEY, name TEXT NOT NULL, login TEXT NOT NULL, password BLOB NOT NULL, comment TEXT)",
            []
        )?;
    } else {
        connection = Connection::open(file_path).unwrap();
    }

    let transaction = connection.transaction().unwrap();

    let max_id = transaction
        .query_row("SELECT MAX(id) FROM passwords", [], |row| row.get(0))
        .unwrap_or(0);

    transaction.execute(
        "INSERT INTO passwords (id, name, login, password, comment) VALUES (?, ?, ?, ?, ?)",
        params![max_id + 1, name, login, password, comment]
    )?;

    transaction.commit()?;
    connection.close();

    Ok(())
}