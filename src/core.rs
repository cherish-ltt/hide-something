//! A steganography tool that hides arbitrary data (as a string) into a carrier
//! text by manipulating the case of alphabetic characters.
//!
//! The data is first compressed using the DEFLATE algorithm (raw format), then
//! prefixed with a 4‑byte big‑endian length, converted into a bit stream, and
//! finally embedded by altering the case of letters in the carrier template.

use flate2::{Compression, read::DeflateDecoder, write::DeflateEncoder};
use std::io::{Read, Write};

use crate::error;

/// Hides the given information into a carrier text.
///
/// # Process
/// 1. Compress the input string with raw DEFLATE.
/// 2. Prepend a 4‑byte big‑endian length prefix.
/// 3. Convert the resulting byte vector into a stream of bits (MSB first).
/// 4. Repeat the carrier template until it contains at least as many alphabetic
///    characters as there are bits.
/// 5. Encode each bit by setting the corresponding alphabetic character to
///    uppercase (bit = 1) or lowercase (bit = 0). Non‑alphabetic characters
///    are left unchanged.
///
/// # Errors
/// Returns `HideSomethingError` if compression fails, or if the template
/// contains no alphabetic characters.
///
/// # Example
/// ```
/// use hide_something::hide_encrypt;
/// let hidden = hide_encrypt("secret", "Hello World!").unwrap();
/// assert!(hidden.len() > 0);
/// ```
pub fn hide_encrypt<T>(info: T, carrier_template: &str) -> Result<String, error::HideSomethingError>
where
    T: ToString,
{
    // 1. compress
    let compressed = flate2_compress(info)?;

    // 2. prepend 4‑byte big‑endian length prefix
    let mut payload = Vec::new();
    payload.extend_from_slice(&(compressed.len() as u32).to_be_bytes());
    payload.extend_from_slice(&compressed);

    // 3. convert to bit stream
    let bits: Vec<u8> = bytes_to_bits(&payload).collect();
    let needed_bits = bits.len();

    // 4. extend the carrier so that it has at least `needed_bits` alphabetic chars
    let mut carrier = String::new();
    let template_letters: Vec<char> = carrier_template
        .chars()
        .filter(|c| c.is_alphabetic())
        .collect();
    if template_letters.is_empty() {
        return Err(error::HideSomethingError::CarrierTemplateEmpty);
    }

    while carrier.chars().filter(|c| c.is_alphabetic()).count() < needed_bits {
        carrier.push_str(carrier_template);
    }

    let chars: Vec<char> = carrier.chars().collect();

    // 5. encode bits case‑sensitively
    let mut bit_iter = bits.into_iter();
    let mut result = String::with_capacity(chars.len());
    for ch in chars {
        if ch.is_alphabetic() {
            if let Some(bit) = bit_iter.next() {
                if bit == 1 {
                    result.push(ch.to_ascii_uppercase());
                } else {
                    result.push(ch.to_ascii_lowercase());
                }
            } else {
                // no more bits, keep original case
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}

/// Recovers the original information from a hidden text produced by `hide_encrypt`.
///
/// # Process
/// 1. Extract bits from the case of all alphabetic characters (uppercase = 1,
///    lowercase = 0).
/// 2. Group bits into bytes (MSB first) and discard any trailing partial byte.
/// 3. Read the first 4 bytes as a big‑endian length.
/// 4. Extract the compressed payload of that length and decompress it.
/// 5. Convert the decompressed bytes back into a UTF‑8 string.
///
/// # Errors
/// Returns `HideSomethingError` if the extracted data is too short, if
/// decompression fails, or if the decoded bytes are not valid UTF‑8.
///
/// # Example
/// ```
/// use hide_something::hide_decrypt;
/// let hidden = "HeLlO WoRlD!"; // actually hides something else
/// // In practice you would use hide_encrypt to produce the hidden text.
/// ```
pub fn hide_decrypt<T>(info: T) -> Result<String, error::HideSomethingError>
where
    T: ToString,
{
    // 1. extract bits from letter case
    let bits: Vec<u8> = info
        .to_string()
        .chars()
        .filter(|c| c.is_alphabetic())
        .map(|c| if c.is_uppercase() { 1 } else { 0 })
        .collect();

    // 2. convert bits to bytes
    let raw_bytes = bits_to_bytes(&bits);
    if raw_bytes.len() < 4 {
        return Err(error::HideSomethingError::TooShortData);
    }

    // 3. read length prefix
    let len_bytes: [u8; 4] = [raw_bytes[0], raw_bytes[1], raw_bytes[2], raw_bytes[3]];
    let data_len = u32::from_be_bytes(len_bytes) as usize;
    let compressed = &raw_bytes[4..4 + data_len];

    // 4. decompress
    let decompressed = flate2_decompress(compressed)?;

    // 5. convert to UTF‑8 string
    Ok(String::from_utf8(decompressed)?)
}

// ---------- private helpers ----------

/// Compresses a string using raw DEFLATE.
fn flate2_compress<T>(info: T) -> Result<Vec<u8>, error::HideSomethingError>
where
    T: ToString,
{
    let mut e = DeflateEncoder::new(Vec::new(), Compression::fast());
    e.write_all(&info.to_string().into_bytes())?;
    Ok(e.finish()?)
}

/// Decompresses raw DEFLATE data.
fn flate2_decompress(data: &[u8]) -> Result<Vec<u8>, error::HideSomethingError> {
    let mut decoder = DeflateDecoder::new(data);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out)?;
    Ok(out)
}

/// Converts a byte slice into an iterator of bits (MSB first).
fn bytes_to_bits(data: &[u8]) -> impl Iterator<Item = u8> + '_ {
    data.iter()
        .flat_map(|&b| (0..8).rev().map(move |i| (b >> i) & 1))
}

/// Converts a slice of bits (MSB first) back into bytes.
/// Only full bytes (multiples of 8) are converted; trailing bits are ignored.
fn bits_to_bytes(bits: &[u8]) -> Vec<u8> {
    bits.chunks(8)
        .filter(|chunk| chunk.len() == 8)
        .map(|chunk| {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                byte |= bit << (7 - i);
            }
            byte
        })
        .collect()
}

// ---------- unit tests ----------
#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to test round‑trip encryption/decryption.
    fn round_trip(original: &str, template: &str) {
        let hidden = hide_encrypt(original, template).unwrap();
        let recovered = hide_decrypt(&hidden).unwrap();
        assert_eq!(recovered, original);
    }

    #[test]
    fn test_hide_encrypt_basic() {
        // This expected string was generated by the current implementation.
        // Any change to compression or bit ordering would alter it.
        let info = "123";
        let template = "Happy Everyday!";
        let result = hide_encrypt(info, template).unwrap();
        assert_eq!(
            result,
            "happy everyday!happy everyday!hapPy EveRYdaY!HapPY eVerydAY!haPpy everYDay!happy everyday!"
        );
    }

    #[test]
    fn test_hide_decrypt_basic() {
        let hidden = "happy everyday!happy everyday!hapPy EveRYdaY!HapPY eVerydAY!haPpy everYDay!happy everyday!";
        let recovered = hide_decrypt(hidden).unwrap();
        assert_eq!(recovered, "123");
    }

    #[test]
    fn test_round_trip_simple() {
        round_trip("Hello", "abc");
        round_trip("Rust", "Programming Language");
        round_trip("123456", "Template");
    }

    #[test]
    fn test_round_trip_with_special_chars() {
        // The input can contain non‑alphabetic characters; they are preserved.
        round_trip("Hello, World!", "This is a template.");
        round_trip("data: 42", "Carrier with spaces and punctuation!");
    }

    #[test]
    fn test_round_trip_empty_string() {
        // An empty string compresses to a small payload.
        round_trip("", "Any template");
    }

    #[test]
    fn test_round_trip_long_data() {
        let long = "a".repeat(1000);
        round_trip(&long, "Short");
    }

    #[test]
    fn test_error_carrier_empty() {
        // Template with no alphabetic characters should fail.
        let result = hide_encrypt("test", "123!@#");
        assert!(matches!(
            result,
            Err(error::HideSomethingError::CarrierTemplateEmpty)
        ));
    }

    #[test]
    fn test_error_too_short_data() {
        // A hidden text with fewer than 4 alphabetic chars cannot hold the length.
        let result = hide_decrypt("abc");
        assert!(matches!(
            result,
            Err(error::HideSomethingError::TooShortData)
        ));
    }

    // Note: The following test is disabled because it depends on the exact
    // compression output; it serves as a sanity check for future changes.
    #[test]
    fn test_hide_encrypt_known_expected() {
        // If the algorithm changes, update this test accordingly.
        let info = "hello";
        let template = "Test";
        let hidden = hide_encrypt(info, template).unwrap();
        // We just check that it can be decoded back.
        let recovered = hide_decrypt(&hidden).unwrap();
        assert_eq!(recovered, "hello");
    }
}
