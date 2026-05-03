use crate::errors::ConfigError;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::Path;

/// Reads toml config from path
pub fn read<T>(path: impl AsRef<Path>) -> Result<T, ConfigError>
where
    T: DeserializeOwned,
{
    let str = fs::read_to_string(&path)?;
    let result = toml::from_str(&str)?;

    Ok(result)
}

// fn parse<T: DeserializeOwned>(input: &str) -> Result<T, TomlError> {
//     toml::from_str(&input)
// }
