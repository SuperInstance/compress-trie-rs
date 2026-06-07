//! LZW code table with dynamic code width.

/// The initial code width for LZW (9 bits, since 256 entries used).
pub const INITIAL_CODE_WIDTH: u8 = 9;

/// The maximum code width.
pub const MAX_CODE_WIDTH: u8 = 16;

/// An LZW code table that tracks codes and manages code width.
#[derive(Debug, Clone)]
pub struct LzwTable {
    /// Current next code to assign.
    next_code: u16,
    /// Current bit width for codes.
    code_width: u8,
    /// Maximum code value before reset.
    max_code: u16,
}

impl LzwTable {
    /// Create a new LZW table initialized after the 256 literal entries.
    ///
    /// Optionally reserves codes for clear (256) and end-of-information (257).
    pub fn new(reserve_special: bool) -> Self {
        let first_code = if reserve_special { 258 } else { 256 };
        LzwTable {
            next_code: first_code,
            code_width: if reserve_special { 10 } else { INITIAL_CODE_WIDTH },
            max_code: 65535, // (1u32 << MAX_CODE_WIDTH) - 1
        }
    }

    /// Create with a custom max code width.
    pub fn with_max_width(max_width: u8) -> Self {
        LzwTable {
            next_code: 256,
            code_width: INITIAL_CODE_WIDTH,
            max_code: (1u16 << max_width) - 1,
        }
    }

    /// Get the next available code and advance the counter.
    ///
    /// Returns `None` if the table is full.
    pub fn alloc_code(&mut self) -> Option<u16> {
        if self.next_code > self.max_code {
            return None;
        }
        let code = self.next_code;
        self.next_code += 1;
        // Check if we need to increase code width
        if self.next_code > (1u16 << self.code_width) {
            self.code_width = (self.code_width + 1).min(MAX_CODE_WIDTH);
        }
        Some(code)
    }

    /// Peek at the next code without allocating.
    pub fn peek_code(&self) -> u16 {
        self.next_code
    }

    /// Current code width in bits.
    pub fn code_width(&self) -> u8 {
        self.code_width
    }

    /// Number of entries allocated so far.
    pub fn len(&self) -> usize {
        self.next_code as usize
    }

    /// Whether the table is empty (always false — has 256 initial entries).
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Whether the table is full.
    pub fn is_full(&self) -> bool {
        self.next_code > self.max_code
    }

    /// Reset the table.
    pub fn reset(&mut self) {
        self.next_code = 256;
        self.code_width = INITIAL_CODE_WIDTH;
    }

    /// Maximum code value.
    pub fn max_code(&self) -> u16 {
        self.max_code
    }
}
