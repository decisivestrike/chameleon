use crate::errors::ConfigError;
use resolve_path::PathResolveExt;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::{Path, PathBuf};

/// For argh
pub fn resolve_path(value: &str) -> Result<PathBuf, String> {
    Ok(value
        .try_resolve()
        .map_err(|e| e.to_string())?
        .to_path_buf())
}

/// Reads toml config from path
pub fn read_config<T>(path: impl AsRef<Path>) -> Result<T, ConfigError>
where
    T: DeserializeOwned,
{
    let str = fs::read_to_string(&path)?;
    let result = toml::from_str(&str)?;

    Ok(result)
}
