//! SysBot API - REST API for Nintendo Switch Pokemon automation
//!
//! This crate provides HTTP endpoints for controlling Pokemon bots,
//! allowing any client (Discord bot, web frontend, mobile app) to
//! interact with the trading system.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
//! │  Discord    │  │  Web App    │  │  Mobile     │
//! │  Bot        │  │  (React)    │  │  App        │
//! └──────┬──────┘  └──────┬──────┘  └──────┬──────┘
//!        │                │                │
//!        └────────────────┼────────────────┘
//!                         │
//!                    HTTP/WebSocket
//!                         │
//!                ┌────────▼────────┐
//!                │   SysBot API    │
//!                │   (This crate)  │
//!                └────────┬────────┘
//!                         │
//!           ┌─────────────┼─────────────┐
//!           │             │             │
//!    ┌──────▼──────┐ ┌────▼────┐ ┌──────▼──────┐
//!    │   Switch    │ │  PKHeX  │ │   Trade     │
//!    │ Connection  │ │ Service │ │   Queue     │
//!    └─────────────┘ └─────────┘ └─────────────┘
//! ```

pub mod routes;
pub mod state;

pub use routes::create_router;
pub use state::AppState;
