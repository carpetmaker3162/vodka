#![allow(dead_code)]

use rusqlite::{params, params_from_iter};
use crate::{Entry, Error, get_vodka_path, get_db};
use std::fs;
use std::io::Read;
use std::io::Write;

pub fn read_file(file_name: &str) -> Result<String, Error> {
    let file_path = get_vodka_path(file_name);
    
    let mut file_content = String::new();
    let mut file = fs::File::open(&file_path)?;
    if let Err(err) = file.read_to_string(&mut file_content)
    {
        eprintln!("Error: failed to open file {:?} ({})", file_path, err);
    }
    
    Ok(file_content)
}

// will abort if file already exists
pub fn write_to_file(file_name: &str, content: String, overwrite: bool) -> Result<(), Error> {
    let file_path = get_vodka_path(file_name);

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

pub fn get_next_id() -> i32 {
    let connection = get_db();

    match connection.query_row(
        "SELECT seq FROM sqlite_sequence WHERE name = 'passwords'",
        [],
        |row| Ok(row.get::<usize, i32>(0)?)
    ) {
        Ok(id) => id + 1,
        Err(e) => {
            match e {
                rusqlite::Error::QueryReturnedNoRows => 1,
                _ => {
                    eprintln!("Error (get_next_id): {:?}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

pub fn add_entry(name: String, login: String, password: &[u8], comment: String) -> Result<(), Error> {
    let connection = get_db();

    connection.execute(
        "INSERT INTO passwords (name, login, password, comment) VALUES (?, ?, ?, ?)",
        params![name, login, password, comment]
    )?;

    Ok(())
}

// if a parameter is an empty string, will search w/o the parameter
pub fn search_entries(name: String, login: String) -> Vec<Entry> {
    let connection = get_db();
    let mut query_command = String::from("SELECT id, name, login, password, comment FROM passwords");
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

    let mut stmt = connection
        .prepare(&query_command)
        .unwrap();
    
    let query_match = stmt
        .query_map(params_from_iter(query_params), |row| {
            Ok(Entry {
                id: row.get(0)?,
                name: row.get(1)?,
                login: row.get(2)?,
                password: row.get(3)?,
                comment: row.get(4)?,
            })
        });

    let entries: Result<Vec<Entry>, rusqlite::Error> = query_match.unwrap().collect();

    entries.unwrap()
}

// uses search_entries. change, if the behaviour of search_entries ever changes (empty string parameter behaviour)
pub fn get_entry_by_id(id: i32) -> Option<Entry> {
    let connection = get_db();

    let query_result = connection.query_row(
        "SELECT id, name, login, password, comment FROM passwords WHERE id = ?",
        [id],
        |row| {
            Ok(Entry {
                id: row.get(0)?,
                name: row.get(1)?,
                login: row.get(2)?,
                password: row.get(3)?,
                comment: row.get(4)?,
            })
        }
    );

    if let Err(e) = query_result {
        match e {
            rusqlite::Error::QueryReturnedNoRows => None,
            _ => { 
                eprintln!("Error while querying by ID: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        Some(query_result.unwrap())
    }
}

pub fn delete_entry(id: i32) -> Result<(), Error> {
    let connection = get_db();
    
    connection.execute(
        "DELETE FROM passwords WHERE id = ?",
        [id]
    )?;

    Ok(())
}

pub fn get_all_rows() -> Vec<Entry> {
    let connection = get_db();

    let mut stmt = connection
        .prepare("SELECT id, name, login, password, comment FROM passwords")
        .unwrap();
    
    let query_match = stmt
        .query_map([], |row| {
            Ok(Entry {
                id: row.get(0)?,
                name: row.get(1)?,
                login: row.get(2)?,
                password: row.get(3)?,
                comment: row.get(4)?,
            })
        });

    let entries: Result<Vec<Entry>, rusqlite::Error> = query_match.unwrap().collect();

    entries.unwrap()
}

pub fn erase_all() -> Result<(), Error> {
    let connection = get_db();

    // resets sqlite_sequence as well?
    connection.execute("DROP TABLE IF EXISTS passwords", [])?;

    crate::setup::setup_db()?;

    Ok(())
}