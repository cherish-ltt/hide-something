//! Error handling for the hide-something crate.

use thiserror::Error;

/// Possible errors that can occur during hiding or extraction.
#[derive(Error, Debug)]
pub enum HideSomethingError {
    /// The carrier template contains no alphabetic characters.
    #[error("carrier template contains no alphabetic characters")]
    CarrierTemplateEmpty,

    /// The extracted data is too short to contain the length prefix.
    #[error("extracted data is too short")]
    TooShortData,

    /// An I/O or compression error occurred.
    #[error("I/O or compression error: {0}")]
    IoError(#[from] std::io::Error),

    /// The decompressed data is not valid UTF‑8.
    #[error("decoded bytes are not valid UTF‑8: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}
