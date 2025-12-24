//! Connection configuration

use std::net::SocketAddr;

/// Configuration for connecting to a Nintendo Switch
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// IP address and port (for WiFi connections)
    pub address: SocketAddr,

    /// Connection name/label for identification
    pub name: String,

    /// Maximum bytes per transfer packet
    pub max_transfer_size: usize,

    /// Base delay between commands in milliseconds
    pub base_delay_ms: u64,

    /// Connection timeout in milliseconds
    pub timeout_ms: u64,

    /// Whether to use CRLF line endings (required for sys-botbase)
    pub use_crlf: bool,
}

impl ConnectionConfig {
    /// Create a new WiFi connection config with default settings
    pub fn wifi(ip: &str, port: u16) -> crate::Result<Self> {
        let address = format!("{}:{}", ip, port)
            .parse()
            .map_err(|e| crate::Error::InvalidCommand(format!("Invalid address: {}", e)))?;

        Ok(Self {
            address,
            name: format!("Switch@{}", ip),
            max_transfer_size: 0x1C0, // 448 bytes, same as C#
            base_delay_ms: 64,
            timeout_ms: 5000,
            use_crlf: true,
        })
    }

    /// Create config with a custom name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set maximum transfer size
    pub fn with_max_transfer_size(mut self, size: usize) -> Self {
        self.max_transfer_size = size;
        self
    }

    /// Set base delay between commands
    pub fn with_base_delay(mut self, delay_ms: u64) -> Self {
        self.base_delay_ms = delay_ms;
        self
    }

    /// Set connection timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            address: "0.0.0.0:6000".parse().unwrap(),
            name: "Switch".to_string(),
            max_transfer_size: 0x1C0,
            base_delay_ms: 64,
            timeout_ms: 5000,
            use_crlf: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_wifi() {
        let config = ConnectionConfig::wifi("192.168.1.100", 6000).unwrap();
        assert_eq!(config.address.to_string(), "192.168.1.100:6000");
        assert!(config.name.contains("192.168.1.100"));
    }

    #[test]
    fn test_config_with_name() {
        let config = ConnectionConfig::wifi("192.168.1.100", 6000)
            .unwrap()
            .with_name("MySwitch");
        assert_eq!(config.name, "MySwitch");
    }

    #[test]
    fn test_config_builder_chain() {
        let config = ConnectionConfig::wifi("10.0.0.1", 6000)
            .unwrap()
            .with_name("TestSwitch")
            .with_max_transfer_size(1024)
            .with_base_delay(100)
            .with_timeout(10000);

        assert_eq!(config.name, "TestSwitch");
        assert_eq!(config.max_transfer_size, 1024);
        assert_eq!(config.base_delay_ms, 100);
        assert_eq!(config.timeout_ms, 10000);
    }

    #[test]
    fn test_config_invalid_ip() {
        let result = ConnectionConfig::wifi("not.an.ip", 6000);
        assert!(result.is_err());
    }

    #[test]
    fn test_default_config() {
        let config = ConnectionConfig::default();
        assert_eq!(config.max_transfer_size, 0x1C0);
        assert!(config.use_crlf);
    }
}
