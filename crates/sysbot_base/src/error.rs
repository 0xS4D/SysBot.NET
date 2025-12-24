//! Error types for SysBot Base

use thiserror::Error;

/// Result type alias using our Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during Switch communication
#[derive(Error, Debug)]
pub enum Error {
    /// Connection to the Switch failed
    #[error("Failed to connect to Switch at {address}: {source}")]
    ConnectionFailed {
        address: String,
        #[source]
        source: std::io::Error,
    },

    /// Connection was lost during operation
    #[error("Connection lost: {0}")]
    ConnectionLost(String),

    /// The Switch is not connected
    #[error("Not connected to Switch")]
    NotConnected,

    /// Timeout waiting for response
    #[error("Timeout waiting for response after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },

    /// Invalid response from Switch
    #[error("Invalid response from Switch: {message}")]
    InvalidResponse { message: String },

    /// Failed to decode hex response
    #[error("Failed to decode hex response: {0}")]
    HexDecodeError(String),

    /// Invalid command
    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Memory read/write error
    #[error("Memory operation failed at offset 0x{offset:X}: {message}")]
    MemoryError { offset: u64, message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::NotConnected;
        assert_eq!(err.to_string(), "Not connected to Switch");

        let err = Error::Timeout { timeout_ms: 5000 };
        assert_eq!(
            err.to_string(),
            "Timeout waiting for response after 5000ms"
        );

        let err = Error::MemoryError {
            offset: 0xDEADBEEF,
            message: "read failed".to_string(),
        };
        assert!(err.to_string().contains("0xDEADBEEF"));
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Error>();
    }
}
