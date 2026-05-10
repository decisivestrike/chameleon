use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MonitorError {
    #[error("Connector not available for this monitor")]
    NoConnector,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(transparent)]
    DeserializeError(#[from] toml::de::Error),

    #[error(transparent)]
    IoError(#[from] io::Error),
}
