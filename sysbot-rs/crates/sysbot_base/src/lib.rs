//! SysBot Base - Core library for Nintendo Switch connection and bot control
//!
//! This crate provides the foundational infrastructure for connecting to
//! Nintendo Switch consoles via sys-botbase (WiFi) or usb-botbase (USB).
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           SwitchConnection              │
//! │  (trait for all connection types)       │
//! └─────────────────┬───────────────────────┘
//!                   │
//!       ┌───────────┴───────────┐
//!       │                       │
//! ┌─────▼─────┐           ┌─────▼─────┐
//! │  WiFi     │           │   USB     │
//! │ Connection│           │ Connection│
//! └───────────┘           └───────────┘
//! ```

pub mod command;
pub mod connection;
pub mod error;

pub use command::{OffsetType, ResponseDecoder, SwitchButton, SwitchCommand, SwitchStick};
pub use connection::{ConnectionConfig, SwitchConnection, WifiConnection};
pub use error::{Error, Result};
