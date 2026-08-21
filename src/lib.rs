//! A steganography tool that hides arbitrary data into a carrier text.
//!
//! The core implementation is in the [`core`] module. This crate exposes
//! [`hide_encrypt`] and [`hide_decrypt`] as the main public API.

pub mod core;
pub mod error;

// Re‑export the most commonly used functions for convenience.
pub use core::{hide_decrypt, hide_encrypt};
