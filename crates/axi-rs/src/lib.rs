//! axi-rs — a Rust implementation of the AXI agent-facing CLI protocol.
//!
//! AXI is a convention for command-line tools built to be driven by AI agents:
//! command-first dispatch (`<bin> <command> [args]`), a home/ambient view for
//! discovery, structured errors carrying actionable suggestions, and TOON output
//! the agent reads.
//!
//! This crate is the protocol layer. It is a clean-room Rust reimplementation of
//! the AXI convention (originally `axi-sdk-js` by Kun Chen); we keep the protocol
//! and drop the JS dependency and its self-update command.

pub mod cli;
pub mod error;
pub mod hooks;
pub mod toon;

pub use cli::App;
pub use error::{exit_code_for, AxiError, ErrorCode};
