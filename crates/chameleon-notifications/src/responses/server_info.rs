use crate::SPECIFICATION_VERSION;
use serde::Serialize;
use zbus::zvariant::Type;

#[derive(Debug, Serialize, Type)]
pub struct ServerInfo {
    /// The product name of the server.
    pub name: String,

    /// The vendor name. For example "KDE," "GNOME," "freedesktop.org" or
    /// "Microsoft".
    pub vendor: String,

    /// The server's version number.
    pub version: String,

    /// The specification version the server is compliant with.
    pub spec_version: String,
}

impl Default for ServerInfo {
    fn default() -> Self {
        Self {
            name: env!("CARGO_PKG_NAME").to_string(),
            vendor: env!("CARGO_PKG_AUTHORS").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            spec_version: SPECIFICATION_VERSION.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::responses::ServerInfo;
    use zbus::zvariant::Type;

    #[test]
    fn check_signature() {
        // Array of strings
        assert_eq!(ServerInfo::SIGNATURE, "(ssss)");
    }
}
