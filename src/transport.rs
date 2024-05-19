use crate::{get_cellar_path, get_absolute_path, Entry, Error};
use crate::store;
use csv::{Writer, ReaderBuilder};
use std::path::PathBuf;

// provide path of export csv
pub fn export(export_file: &str, master_key: &[u8], overwrite: bool) -> Result<(), Error> {
    let path: PathBuf = get_absolute_path(export_file);
    let entries: Vec<Entry> = store::get_all_rows();

    if !overwrite && path.exists() {
        return Err(Error::ExportFileExists);
    }

    let mut writer = Writer::from_path(path).unwrap();
    for entry in &entries {
        writer.serialize(entry.decrypted(master_key))?;
    }
    writer.flush()?;

    Ok(())
}

// provide path of csv imported
// erases existing db
pub fn import(import_file: &str, master_key: &[u8], overwrite: bool) -> Result<(), crate::Error> {
    let import_path: PathBuf = get_absolute_path(import_file);
    let cellar_path: PathBuf = get_cellar_path();

    if !overwrite && cellar_path.exists() {
        return Err(Error::ImportFileExists);
    }

    store::erase_all()?;
    
    let mut reader = ReaderBuilder::new().from_path(import_path)?;

    for record in reader.records() {
        let record = record?;
        let name = record.get(0).unwrap_or("").to_string();
        let login = record.get(1).unwrap_or("").to_string();
        let password: String = record.get(2).unwrap_or("").to_string();
        let comment = record.get(3).unwrap_or("").to_string();

        crate::add_entry(Entry::new(
            name,
            login,
            password,
            comment,
            master_key
        ))?;
    }
    
    Ok(())
}