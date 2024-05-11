#![allow(dead_code)]

use rusqlite::{Connection, params, params_from_iter};
use crate::{Entry, get_path, get_cellar_path};
use std::fs;
use std::io::Read;
use std::io::Write;

pub fn read_file(file_name: &str) -> std::io::Result<String> {
    let file_path = get_path(file_name);
    
    let mut file_content = String::new();
    let mut file = fs::File::open(&file_path)?;
    if let Err(err) = file.read_to_string(&mut file_content)
    {
        eprintln!("Error: failed to open file {:?} ({})", file_path, err);
    }
    
    Ok(file_content)
}

pub fn write_to_file(file_name: &str, content: String, overwrite: bool) -> std::io::Result<()> {
    let file_path = get_path(file_name);

    if file_path.exists() && !overwrite {
        eprintln!("Error: file {:?} already exists", file_path);
        return Ok(());
    }

    let mut file = fs::File::create(&file_path)?;
    if let Err(err) = file.write_all(content.as_bytes())
    {
        eprintln!("Error: failed to write to file {:?} ({})", file_path, err);
    }

    Ok(())
}

pub fn add_entry(name: String, login: String, password: &[u8], comment: String) -> Result<(), rusqlite::Error> {
    let file_path = get_cellar_path();

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
    connection.close().expect("Error: something weird happened while closing sqlite connection.");

    Ok(())
}

pub fn search_entries(name: String, login: String) -> Vec<Entry> {
    let file_path = get_cellar_path();
    let mut connection = Connection::open(file_path).unwrap();
    let transaction = connection.transaction().unwrap();
    let mut query_command = String::from("SELECT name, login, password, comment FROM passwords");
    let mut query_params = Vec::new();

    if !name.is_empty() {
        query_command.push_str(" WHERE name = ?");
        query_params.push(name.clone());
    }

    if !login.is_empty() {
        if name.is_empty() {
            query_command.push_str(" WHERE login = ?");
        } else {
            query_command.push_str(" AND login = ?");
        }
        query_params.push(login);
    }

    let mut stmt = transaction
        .prepare(&query_command)
        .unwrap();
    let query_match = stmt
        .query_map(params_from_iter(query_params), |row| {
            Ok(Entry {
                name: row.get(0)?,
                login: row.get(1)?,
                password: row.get(2)?,
                comment: row.get(3)?,
            })
        });

    let entries: Result<Vec<Entry>, rusqlite::Error> = query_match.unwrap().collect();

    entries.unwrap()
}