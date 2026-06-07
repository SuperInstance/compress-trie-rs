//! LZW encoding (compress).

use crate::lzw_table::LzwTable;

/// LZW-encode a byte slice into a vector of codes.
///
/// Uses variable-width codes starting at 9 bits, growing up to 16 bits.
/// No special clear/EOF codes are emitted (simplified LZW).
///
/// # Algorithm
///
/// 1. Initialize dictionary with single-byte entries (0–255).
/// 2. Read input bytes, building the longest match in the dictionary.
/// 3. When no match, emit the code for the current prefix, add the new
///    prefix+byte to the dictionary, and start fresh from the current byte.
pub fn encode(data: &[u8]) -> Vec<u16> {
    if data.is_empty() {
        return Vec::new();
    }

    let mut table = LzwTable::new(false);
    // Build a simple dictionary: map byte sequences to codes
    // Using a Vec<(Vec<u8>, u16)> for the dictionary
    let mut dict: Vec<(Vec<u8>, u16)> = (0..256)
        .map(|i| (vec![i as u8], i as u16))
        .collect();

    let mut codes = Vec::new();
    let mut pos = 0;

    while pos < data.len() {
        // Find longest match in dictionary
        let mut best_idx = None;
        let mut best_len = 0;

        for (idx, (entry, _)) in dict.iter().enumerate() {
            if entry.len() <= best_len {
                continue;
            }
            if pos + entry.len() > data.len() {
                continue;
            }
            if &data[pos..pos + entry.len()] == entry.as_slice() {
                best_idx = Some(idx);
                best_len = entry.len();
            }
        }

        match best_idx {
            Some(idx) => {
                codes.push(dict[idx].1);
                let match_end = pos + best_len;
                if match_end < data.len() {
                    // Add new entry: matched sequence + next byte
                    let mut new_entry = dict[idx].0.clone();
                    new_entry.push(data[match_end]);
                    if let Some(code) = table.alloc_code() {
                        dict.push((new_entry, code));
                    }
                }
                pos += best_len;
            }
            None => {
                // Single byte (should always match from initial dict)
                codes.push(data[pos] as u16);
                pos += 1;
            }
        }
    }

    codes
}

/// Encode to packed bytes (12-bit fixed-width codes).
///
/// Each pair of codes is packed into 3 bytes (2 × 12 bits).
/// If odd number of codes, the last code is stored in the remaining 12 bits
/// of a 2-byte group (padded).
pub fn encode_packed12(data: &[u8]) -> Vec<u8> {
    let codes = encode(data);
    pack_codes_12bit(&codes)
}

/// Pack u16 codes (assumed ≤ 4095) into bytes using 12-bit packing.
pub fn pack_codes_12bit(codes: &[u16]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < codes.len() {
        let a = codes[i] & 0x0FFF;
        let b = codes[i + 1] & 0x0FFF;
        out.push((a >> 4) as u8);
        out.push((((a & 0x0F) << 4) | (b >> 8)) as u8);
        out.push((b & 0xFF) as u8);
        i += 2;
    }
    if i < codes.len() {
        let a = codes[i] & 0x0FFF;
        out.push((a >> 4) as u8);
        out.push(((a & 0x0F) << 4) as u8);
    }
    out
}

/// Unpack 12-bit packed bytes back to codes.
pub fn unpack_codes_12bit(data: &[u8]) -> Vec<u16> {
    let mut codes = Vec::new();
    let mut i = 0;
    while i + 2 < data.len() {
        let a = ((data[i] as u16) << 4) | ((data[i + 1] as u16) >> 4);
        let b = ((data[i + 1] as u16 & 0x0F) << 8) | (data[i + 2] as u16);
        codes.push(a);
        codes.push(b);
        i += 3;
    }
    // Handle remaining bytes (single code)
    if i + 1 < data.len() {
        let a = ((data[i] as u16) << 4) | ((data[i + 1] as u16) >> 4);
        codes.push(a);
    }
    codes
}
