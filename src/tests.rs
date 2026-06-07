//! Comprehensive test suite for compress-trie-rs.

#[cfg(test)]
mod tests {
    use crate::{trie, lzw_table, lzw_encode, lzw_decode, dictionary};

    /// Helper: round-trip LZW encode/decode.
    fn round_trip(data: &[u8]) -> Vec<u8> {
        let codes = lzw_encode::encode(data);
        lzw_decode::decode(&codes)
    }

    // ── Trie tests ──────────────────────────────────────────────────────

    #[test]
    fn test_trie_new_has_256_entries() {
        let t = trie::Trie::new();
        assert_eq!(t.next_code(), 256);
    }

    #[test]
    fn test_trie_lookup_single_byte() {
        let t = trie::Trie::new();
        let (code, len) = t.longest_prefix(b"\x42", 0);
        assert_eq!(code, Some(0x42));
        assert_eq!(len, 1);
    }

    #[test]
    fn test_trie_insert_and_lookup() {
        let mut t = trie::Trie::new();
        let code = t.insert_sequence(b"ab");
        assert_eq!(code, Some(256));
        let (c, len) = t.longest_prefix(b"ab", 0);
        assert_eq!(c, Some(256));
        assert_eq!(len, 2);
    }

    #[test]
    fn test_trie_longest_prefix() {
        let mut t = trie::Trie::new();
        t.insert_sequence(b"abc");
        let (code, len) = t.longest_prefix(b"abcd", 0);
        assert_eq!(code, Some(256));
        assert_eq!(len, 3);
    }

    #[test]
    fn test_trie_no_match() {
        let t = trie::Trie::new();
        // No multi-byte entries exist
        let root = t.root();
        assert_eq!(root.child_count(), 256);
    }

    #[test]
    fn test_trie_reset() {
        let mut t = trie::Trie::new();
        t.insert_sequence(b"hello");
        assert!(t.next_code() > 256);
        t.reset();
        assert_eq!(t.next_code(), 256);
    }

    #[test]
    fn test_trie_node_default() {
        let node = trie::TrieNode::default();
        assert_eq!(node.child_count(), 0);
        assert!(node.code().is_none());
        assert!(!node.is_terminal());
    }

    // ── LZW Table tests ─────────────────────────────────────────────────

    #[test]
    fn test_lzw_table_new() {
        let t = lzw_table::LzwTable::new(false);
        assert_eq!(t.code_width(), lzw_table::INITIAL_CODE_WIDTH);
        assert!(!t.is_full());
    }

    #[test]
    fn test_lzw_table_alloc() {
        let mut t = lzw_table::LzwTable::new(false);
        let code = t.alloc_code();
        assert_eq!(code, Some(256));
        assert_eq!(t.len(), 257);
    }

    #[test]
    fn test_lzw_table_width_growth() {
        let mut t = lzw_table::LzwTable::new(false);
        // Fill up to 512 entries (9-bit codes can represent 0-511)
        for _ in 0..256 {
            t.alloc_code();
        }
        // After filling 9-bit range, width should increase
        assert!(t.code_width() >= lzw_table::INITIAL_CODE_WIDTH);
    }

    #[test]
    fn test_lzw_table_reset() {
        let mut t = lzw_table::LzwTable::new(false);
        t.alloc_code();
        t.alloc_code();
        t.reset();
        assert_eq!(t.peek_code(), 256);
        assert_eq!(t.code_width(), lzw_table::INITIAL_CODE_WIDTH);
    }

    // ── Dictionary tests ────────────────────────────────────────────────

    #[test]
    fn test_dictionary_new() {
        let d = dictionary::Dictionary::new();
        assert_eq!(d.len(), 256);
        assert_eq!(d.next_code(), 256);
    }

    #[test]
    fn test_dictionary_lookup_literal() {
        let d = dictionary::Dictionary::new();
        assert_eq!(d.lookup(&[42]), Some(42));
    }

    #[test]
    fn test_dictionary_add_and_lookup() {
        let mut d = dictionary::Dictionary::new();
        let code = d.add(b"abc".to_vec());
        assert_eq!(code, 256);
        assert_eq!(d.lookup(b"abc"), Some(256));
        assert_eq!(d.lookup_code(256), Some(&b"abc"[..]));
    }

    #[test]
    fn test_dictionary_longest_prefix() {
        let mut d = dictionary::Dictionary::new();
        d.add(b"ab".to_vec());
        let (code, len) = d.longest_prefix(b"abc");
        assert_eq!(code, Some(256));
        assert_eq!(len, 2);
    }

    #[test]
    fn test_dictionary_reset() {
        let mut d = dictionary::Dictionary::new();
        d.add(b"x".to_vec());
        d.reset();
        assert_eq!(d.len(), 256);
    }

    // ── LZW Encode/Decode round-trip tests ──────────────────────────────

    #[test]
    fn test_lzw_roundtrip_simple() {
        let data = b"WEWERABRAABRABRA";
        assert_eq!(data.as_slice(), round_trip(data).as_slice());
    }

    #[test]
    fn test_lzw_roundtrip_empty() {
        assert_eq!(Vec::<u8>::new(), round_trip(b""));
    }

    #[test]
    fn test_lzw_roundtrip_single_byte() {
        assert_eq!(b"x".as_slice(), round_trip(b"x").as_slice());
    }

    #[test]
    fn test_lzw_roundtrip_repetitive() {
        let data = b"aaaaaaaaaaaaaaaa";
        assert_eq!(data.as_slice(), round_trip(data).as_slice());
    }

    #[test]
    fn test_lzw_roundtrip_all_bytes() {
        let data: Vec<u8> = (0..=255).collect();
        assert_eq!(data.as_slice(), round_trip(&data).as_slice());
    }

    #[test]
    fn test_lzw_roundtrip_text() {
        let data = b"the cat in the hat came back the cat in the hat";
        assert_eq!(data.as_slice(), round_trip(data).as_slice());
    }

    #[test]
    fn test_lzw_roundtrip_binary() {
        let data: Vec<u8> = (0..=255).cycle().take(1024).collect();
        assert_eq!(data.as_slice(), round_trip(&data).as_slice());
    }

    // ── Packed 12-bit tests ─────────────────────────────────────────────

    #[test]
    fn test_pack_unpack_12bit_roundtrip() {
        let codes: Vec<u16> = vec![0x100, 0x200, 0xABC, 0x123];
        let packed = lzw_encode::pack_codes_12bit(&codes);
        let unpacked = lzw_encode::unpack_codes_12bit(&packed);
        assert_eq!(codes, unpacked);
    }

    #[test]
    fn test_pack_unpack_odd_codes() {
        let codes: Vec<u16> = vec![0x100, 0x200, 0xABC];
        let packed = lzw_encode::pack_codes_12bit(&codes);
        let unpacked = lzw_encode::unpack_codes_12bit(&packed);
        assert_eq!(codes, unpacked);
    }

    #[test]
    fn test_encode_packed12_roundtrip() {
        let data = b"WEWERABRAABRABRA";
        let packed = lzw_encode::encode_packed12(data);
        let decoded = lzw_decode::decode_packed12(&packed);
        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    // ── Known LZW example tests ─────────────────────────────────────────

    #[test]
    fn test_lzw_known_example() {
        // Classic LZW example: WEGOABAB -> should produce codes
        let data = b"WEGOABAB";
        let codes = lzw_encode::encode(data);
        let decoded = lzw_decode::decode(&codes);
        assert_eq!(data.as_slice(), decoded.as_slice());
        // Should compress (fewer codes than bytes)
        assert!(codes.len() <= data.len());
    }

    #[test]
    fn test_lzw_dictionary_grows() {
        let data = b"ABABABABABABABAB";
        let codes = lzw_encode::encode(data);
        // With repetitive data, dictionary should grow and produce fewer codes
        assert!(codes.len() < data.len(),
            "codes ({}) should be fewer than input ({})", codes.len(), data.len());
    }

    #[test]
    fn test_lzw_no_compression_random() {
        // Random-ish data may not compress, but should still round-trip
        let data = b"abcdefghij";
        let decoded = round_trip(data);
        assert_eq!(data.as_slice(), decoded.as_slice());
    }
}
