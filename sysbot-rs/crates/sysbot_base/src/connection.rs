//! Switch connection implementations
//!
//! This module provides traits and implementations for connecting to
//! Nintendo Switch consoles via different protocols.

mod config;
mod traits;
mod wifi;

pub use config::ConnectionConfig;
pub use traits::SwitchConnection;
pub use wifi::WifiConnection;

// Re-export for testing
#[cfg(test)]
pub use wifi::MockConnection;
