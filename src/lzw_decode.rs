//! LZW decoding (decompress).

use crate::lzw_table::LzwTable;

/// LZW-decode a vector of codes back into the original byte data.
///
/// Implements the standard LZW decoding algorithm:
/// 1. Initialize dictionary with single-byte entries.
/// 2. For each code, look up the corresponding string.
/// 3. Add the previous string + first byte of current string to the dictionary.
pub fn decode(codes: &[u16]) -> Vec<u8> {
    if codes.is_empty() {
        return Vec::new();
    }

    let mut table: Vec<Vec<u8>> = (0..256).map(|i| vec![i as u8]).collect();
    let mut lzw_table = LzwTable::new(false);
    let mut output = Vec::new();

    // First code must be a literal
    let first_code = codes[0] as usize;
    if first_code >= table.len() {
        return output;
    }
    output.extend_from_slice(&table[first_code]);
    let mut prev = table[first_code].clone();

    for &code in &codes[1..] {
        let entry = if (code as usize) < table.len() {
            table[code as usize].clone()
        } else if (code as usize) == table.len() {
            // Special case: code equals next dictionary entry
            let mut e = prev.clone();
            if !e.is_empty() {
                e.push(e[0]);
            }
            e
        } else {
            // Invalid code
            break;
        };

        output.extend_from_slice(&entry);

        // Add prev + first byte of entry to dictionary
        if let Some(_new_code) = lzw_table.alloc_code() {
            let mut new_entry = prev.clone();
            if !entry.is_empty() {
                new_entry.push(entry[0]);
            }
            table.push(new_entry);
        }

        prev = entry;
    }

    output
}

/// Decode from 12-bit packed bytes.
pub fn decode_packed12(data: &[u8]) -> Vec<u8> {
    let codes = crate::lzw_encode::unpack_codes_12bit(data);
    decode(&codes)
}
