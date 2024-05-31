use crate::Error;
use crate::store::{read_file, write_to_file};
use toml::{Table, Value};

pub trait FromValue {
    fn from_value(value: &Value) -> Self;
}

impl FromValue for String {
    fn from_value(value: &Value) -> Self {
        match value {
            Value::String(s) => s.to_string(),
            _ => panic!("Expected a string")
        }
    }
}

impl FromValue for i32 {
    fn from_value(value: &Value) -> Self {
        match *value {
            Value::Integer(i) => i.try_into().unwrap(),
            _ => panic!("Expected an integer")
        }
    }
}

impl FromValue for bool {
    fn from_value(value: &Value) -> Self {
        match *value {
            Value::Boolean(b) => b,
            _ => panic!("Expected a boolean")
        }
    }
}

pub fn parse(s: &str) -> Value {
    if s.parse::<u32>().is_ok() {
        return Value::from(s.parse::<u32>().unwrap());
    }

    match s {
        "true" => Value::from(true),
        "false" => Value::from(false),
        _ => Value::from(s)
    }
}

pub fn create_default_config() -> Result<(), Error> {
    let default = r#"
        default-cmd = "help"
        hash-memory = 19456
        hash-iterations = 2
        hash-parallelism = 1
        hash = "Argon2id"
        
        [requires-key]
        search = true
        delete = true
        list = true
        config = false
    "#
        .lines()
        .map(|line| line.trim_start())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    write_to_file("config.toml", default, true)
}

pub fn config_str() -> String {
    read_file("config.toml").unwrap()
}

fn config() -> Table {
    config_str().parse::<Table>().unwrap()
}

// returns value as str regardless of toml type
pub fn get_as_str(path: &str) -> Option<String> {
    let split_path: Vec<String> = path
        .split(".")
        .map(|s| s.to_string())
        .collect();
    
    let config = config();

    let mut current_value = config.get(&split_path[0]);

    for component in split_path.iter().skip(1) {
        if let Some(v) = current_value {
            if v.is_table() {
                current_value = v.get(component);
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    if let Some(v) = current_value {
        Some(v.to_string())
    } else {
        None
    }
}

pub fn get<T>(path: &str) -> Option<T> 
where
    T: FromValue
{
    if let Some(val) = get_as_str(path) {
        return Some(T::from_value(&parse(&val)));
    }
    
    None
}

pub fn get_or<T>(path: &str, default: T) -> T 
where
    T: FromValue
{
    match get(path) {
        Some(val) => val,
        None => default
    }
}

pub fn set(path: &str, value: Value) -> Result<(), Error> {
    let split_path: Vec<String> = path
        .split(".")
        .map(|s| s.to_string())
        .collect();
    
    let mut config = config();

    let mut current_value = config.get_mut(&split_path[0]);

    for component in split_path.iter().skip(1) {
        if let Some(v) = current_value {
            if v.is_table() {
                current_value = v.get_mut(component);
            } else {
                return Err(Error::ConfigKeyNotFound(path.to_string()));
            }
        } else {
            return Err(Error::ConfigKeyNotFound(path.to_string()));
        }
    }

    if let Some(v) = current_value {
        *v = value;
        let toml_content = toml::to_string(&config).unwrap();
        write_to_file("config.toml", toml_content, true)?;
        return Ok(())
    }

    Err(Error::ConfigKeyNotFound(path.to_string()))
}