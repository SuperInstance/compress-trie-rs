//! # compress-trie-rs
//!
//! A pure-Rust trie-based compression library implementing LZ78 and LZW style
//! dictionary compression.
//!
//! # Modules
//!
//! - [`trie`] — Generic trie data structure for dictionary management.
//! - [`lzw_table`] — LZW code table with dynamic code width.
//! - [`lzw_encode`] — LZW encoding (compress).
//! - [`lzw_decode`] — LZW decoding (decompress).
//! - [`dictionary`] — Dictionary management utilities.
//!
//! # Quick Start
//!
//! ```
//! use compress_trie_rs::{lzw_encode, lzw_decode};
//!
//! let data = b"WEWERABRAABRABRA";
//! let compressed = lzw_encode::encode(data);
//! let decompressed = lzw_decode::decode(&compressed);
//! assert_eq!(data.as_slice(), decompressed.as_slice());
//! ```

pub mod trie;
pub mod lzw_table;
pub mod lzw_encode;
pub mod lzw_decode;
pub mod dictionary;

#[cfg(test)]
mod tests;
