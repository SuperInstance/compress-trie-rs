//! Dictionary management utilities.

/// A simple dictionary mapping byte sequences to codes and back.
#[derive(Debug, Clone)]
pub struct Dictionary {
    /// Forward map: byte sequence → code.
    entries: Vec<(Vec<u8>, u16)>,
    /// Next code.
    next_code: u16,
}

impl Dictionary {
    /// Create a new dictionary initialized with single-byte entries.
    pub fn new() -> Self {
        let entries: Vec<(Vec<u8>, u16)> = (0..256)
            .map(|i| (vec![i as u8], i as u16))
            .collect();
        Dictionary {
            entries,
            next_code: 256,
        }
    }

    /// Look up a code by byte sequence.
    pub fn lookup(&self, seq: &[u8]) -> Option<u16> {
        self.entries
            .iter()
            .find(|(entry, _)| entry.as_slice() == seq)
            .map(|(_, code)| *code)
    }

    /// Look up a byte sequence by code.
    pub fn lookup_code(&self, code: u16) -> Option<&[u8]> {
        self.entries
            .iter()
            .find(|(_, c)| *c == code)
            .map(|(entry, _)| entry.as_slice())
    }

    /// Add a new entry. Returns the assigned code.
    pub fn add(&mut self, seq: Vec<u8>) -> u16 {
        let code = self.next_code;
        self.entries.push((seq, code));
        self.next_code += 1;
        code
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the dictionary is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Next code to be assigned.
    pub fn next_code(&self) -> u16 {
        self.next_code
    }

    /// Reset to initial state.
    pub fn reset(&mut self) {
        self.entries = (0..256)
            .map(|i| (vec![i as u8], i as u16))
            .collect();
        self.next_code = 256;
    }

    /// Find the longest prefix of `data` in the dictionary.
    ///
    /// Returns `(code, match_length)`.
    pub fn longest_prefix(&self, data: &[u8]) -> (Option<u16>, usize) {
        let mut best_code = None;
        let mut best_len = 0;

        for (entry, code) in &self.entries {
            if entry.len() <= best_len || entry.len() > data.len() {
                continue;
            }
            if &data[..entry.len()] == entry.as_slice() {
                best_code = Some(*code);
                best_len = entry.len();
            }
        }

        (best_code, best_len)
    }
}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new()
    }
}
