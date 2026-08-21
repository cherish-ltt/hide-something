# hide‑something

[![GitHub Actions](https://img.shields.io/github/actions/workflow/status/cherish-ltt/hide-something/rust-ci.yml?branch=main)](https://github.com/cherish-ltt/hide-something/actions/workflows/rust-ci.yml)
[![Crates.io](https://img.shields.io/crates/v/hide-something.svg)](https://crates.io/crates/hide-something)
[![Docs.rs](https://docs.rs/hide-something/badge.svg)](https://docs.rs/hide-something)
[![MIT license](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

A lightweight steganography library written in Rust that hides arbitrary textual data inside a carrier text by altering the case of alphabetic characters. The data is first compressed using the **DEFLATE** algorithm (raw format) to reduce size, then embedded as a bit stream.

---

## 📚 Table of Contents

- [How it works](#how-it-works)
- [Installation](#installation)
- [Usage](#usage)
- [API Overview](#api-overview)
- [Error Handling](#error-handling)
- [Minimum Supported Rust Version](#minimum-supported-rust-version-msrv)
- [License](#license)

---

## How it works

1. **Compress** the input string with raw DEFLATE (using `flate2`).
2. **Prepend** a 4‑byte big‑endian length prefix.
3. **Convert** the byte sequence into a bit stream (MSB first).
4. **Repeat** the carrier template until it contains enough alphabetic characters to hold all bits.
5. **Encode** each bit by setting the corresponding alphabetic character to **uppercase** (bit = 1) or **lowercase** (bit = 0). Non‑alphabetic characters (digits, punctuation, spaces) remain untouched.

To decode, the process is reversed:
- Extract bits from the case of letters.
- Group bits into bytes (ignoring trailing partial bits).
- Read the length prefix.
- Extract and decompress the payload.
- Convert back to UTF‑8.

The use of DEFLATE compression ensures that even long messages can fit into relatively short carrier texts.

---

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
hide-something = "0.1.0"
```

Or, if you want to use the latest development version from GitHub:

```toml
[dependencies]
hide-something = { git = "https://github.com/cherish-ltt/hide-something.git" }
```

---

## Usage

### Basic example

```rust
use hide_something::{hide_encrypt, hide_decrypt};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let original = "Hello, world!";
    let template = "This is a public message.";

    // Hide the original text inside the template
    let hidden = hide_encrypt(original, template)?;
    println!("Hidden text: {}", hidden);

    // Recover the original text
    let recovered = hide_decrypt(&hidden)?;
    assert_eq!(recovered, original);

    Ok(())
}
```

### More examples

- **Short message** – even a single character can be hidden.
- **Long text** – compression helps keep the payload small.
- **Non‑alphabetic characters** – they are preserved exactly as in the template.

---

## API Overview

The library exposes two public functions:

### `hide_encrypt<T>(info: T, carrier_template: &str) -> Result<String, HideSomethingError>`

- **Parameters**:
  - `info`: any type that implements `ToString` (e.g., `&str`, `String`).
  - `carrier_template`: a string that will be repeated if needed to provide enough alphabetic characters.
- **Returns**: a new `String` with the hidden data encoded in the case of letters.
- **Errors**:
  - `CarrierTemplateEmpty` – if the template contains no alphabetic characters.
  - `IoError` – if compression fails (wrapped `std::io::Error`).

### `hide_decrypt<T>(info: T) -> Result<String, HideSomethingError>`

- **Parameters**:
  - `info`: any type that implements `ToString` (the hidden text produced by `hide_encrypt`).
- **Returns**: the original hidden string.
- **Errors**:
  - `TooShortData` – if the extracted payload is less than 4 bytes.
  - `IoError` – if decompression fails.
  - `Utf8Error` – if the decompressed bytes are not valid UTF‑8.

---

## Error Handling

All errors are collected under the `HideSomethingError` enum:

| Variant | Description |
|---------|-------------|
| `CarrierTemplateEmpty` | The provided carrier template contains no alphabetic characters (a..z, A..Z). |
| `TooShortData` | The extracted bit stream could not produce at least 4 bytes for the length prefix. |
| `IoError` | Underlying I/O or compression/decompression error (from `std::io::Error`). |
| `Utf8Error` | Decompressed bytes are not valid UTF‑8 (from `std::string::FromUtf8Error`). |

The library uses `thiserror` for ergonomic error handling; you can use `?` to propagate errors or match on the enum for custom handling.

---

## Minimum Supported Rust Version (MSRV)

The MSRV is **1.98.0** as specified in `Cargo.toml`. This version is required for edition 2024 and for some language features used. We may update the MSRV in future minor releases, but we will do so only when necessary and with a clear notice.

---

## License

This project is licensed under the **MIT License** – see the [LICENSE](LICENSE) file for details.

---

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests on [GitHub](https://github.com/cherish-ltt/hide-something).

---

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/).
- Compression powered by [flate2](https://crates.io/crates/flate2).
- Error handling with [thiserror](https://crates.io/crates/thiserror).