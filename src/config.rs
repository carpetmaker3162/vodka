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
            _ => panic!("Expected a bool")
        }
    }
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

pub fn config() -> Table {
    let content = read_file("config.toml").unwrap();

    content.parse::<Table>().unwrap()
}

pub fn get<T>(path: &str, default: T) -> T 
where
    T: FromValue
{
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
                return default;
            }
        } else {
            return default;
        }
    }

    if let Some(v) = current_value {
        T::from_value(v)
    } else {
        default
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