use thiserror::Error;

#[derive(Error, Debug)]
pub enum MonitorError {
    #[error("Connector not available for this monitor")]
    NoConnector,
}
