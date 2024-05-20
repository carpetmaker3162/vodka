use crate::Error;
use crate::store::{read_file, write_to_file};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    default_cmd: Option<String>,
    hash_cost: Option<u8>,
    requires_key: Option<RequiresKey>,
}

#[derive(Debug, Deserialize)]
struct RequiresKey {
    search: Option<bool>,
    delete: Option<bool>,
    list: Option<bool>,
}

pub fn create_default_config() -> Result<(), Error> {
    let default = r#"
        default_cmd = "help"
        hash_cost = 2

        [requires-key]
        search = true
        delete = true
        list = true
    "#
        .lines()
        .map(|line| line.trim_start())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    write_to_file("config.toml", default, true)
}

pub fn config() {
    let content = read_file("config.toml");

    toml::from_str(&content.unwrap()).unwrap()
}