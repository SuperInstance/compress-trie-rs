//! Generic trie data structure for dictionary management.

use std::collections::HashMap;

/// A node in the trie.
#[derive(Debug, Clone)]
pub struct TrieNode {
    /// Children indexed by byte value.
    children: HashMap<u8, TrieNode>,
    /// Optional code/value associated with this node.
    code: Option<u16>,
    /// Whether this node represents a complete entry.
    is_terminal: bool,
}

impl TrieNode {
    /// Create a new empty trie node.
    pub fn new() -> Self {
        TrieNode {
            children: HashMap::new(),
            code: None,
            is_terminal: false,
        }
    }

    /// Get a child node by byte key.
    pub fn get_child(&self, key: u8) -> Option<&TrieNode> {
        self.children.get(&key)
    }

    /// Get a mutable child by byte key, creating if needed.
    pub fn get_or_create_child(&mut self, key: u8) -> &mut TrieNode {
        self.children.entry(key).or_default()
    }

    /// Set the code for this node.
    pub fn set_code(&mut self, code: u16) {
        self.code = Some(code);
        self.is_terminal = true;
    }

    /// Get the code for this node.
    pub fn code(&self) -> Option<u16> {
        self.code
    }

    /// Whether this is a terminal node.
    pub fn is_terminal(&self) -> bool {
        self.is_terminal
    }

    /// Number of children.
    pub fn child_count(&self) -> usize {
        self.children.len()
    }
}

impl Default for TrieNode {
    fn default() -> Self {
        Self::new()
    }
}

/// A trie that maps byte sequences to codes.
///
/// Used as the dictionary for LZ78/LZW compression.
#[derive(Debug, Clone)]
pub struct Trie {
    root: TrieNode,
    /// Next code to assign.
    next_code: u16,
    /// Maximum code value before reset.
    max_code: u16,
}

impl Trie {
    /// Create a new trie initialized with single-byte entries (0–255).
    ///
    /// Codes 0–255 are reserved for literal bytes. New entries start at 256.
    pub fn new() -> Self {
        let mut root = TrieNode::new();
        for i in 0u16..256 {
            let byte = i as u8;
            let mut child = TrieNode::new();
            child.set_code(i);
            root.children.insert(byte, child);
        }
        Trie {
            root,
            next_code: 256,
            max_code: 65535,
        }
    }

    /// Create a trie with a custom maximum code.
    pub fn with_max_code(max_code: u16) -> Self {
        let mut t = Trie::new();
        t.max_code = max_code;
        t
    }

    /// Look up the longest prefix of `data` starting at `offset` in the trie.
    ///
    /// Returns `(code, match_length)` where `code` is the trie code for the
    /// longest match and `match_length` is how many bytes matched.
    pub fn longest_prefix(&self, data: &[u8], offset: usize) -> (Option<u16>, usize) {
        let mut node = &self.root;
        let mut last_code: Option<u16> = None;
        let mut matched = 0;

        for &byte in &data[offset..] {
            match node.get_child(byte) {
                Some(child) => {
                    matched += 1;
                    last_code = child.code();
                    node = child;
                }
                None => break,
            }
        }

        (last_code, matched)
    }

    /// Insert a new sequence: the existing prefix (code `prefix_code`) + `next_byte`.
    ///
    /// Returns the new code assigned, or `None` if the dictionary is full.
    pub fn insert(&mut self, prefix_code: u16, next_byte: u8) -> Option<u16> {
        // Find the node for prefix_code
        let node = self.find_node_by_code(prefix_code)?;
        // Build the path bytes
        let path = self.get_path_to(node)?;

        // Navigate to the prefix node in self
        let mut current = &mut self.root;
        for &b in &path {
            current = current.children.get_mut(&b)?;
        }

        // Add the new child
        if current.children.contains_key(&next_byte) {
            return current.children.get(&next_byte).and_then(|c| c.code);
        }

        if self.next_code > self.max_code {
            return None;
        }

        let new_code = self.next_code;
        let mut child = TrieNode::new();
        child.set_code(new_code);
        current.children.insert(next_byte, child);
        self.next_code += 1;
        Some(new_code)
    }

    /// Insert a byte sequence directly.
    pub fn insert_sequence(&mut self, seq: &[u8]) -> Option<u16> {
        let mut current = &mut self.root;
        for &b in seq {
            current = current.get_or_create_child(b);
        }
        if current.code().is_some() {
            return current.code();
        }
        if self.next_code > self.max_code {
            return None;
        }
        let code = self.next_code;
        current.set_code(code);
        self.next_code += 1;
        Some(code)
    }

    /// Get the root node.
    pub fn root(&self) -> &TrieNode {
        &self.root
    }

    /// Next code to be assigned.
    pub fn next_code(&self) -> u16 {
        self.next_code
    }

    /// Number of entries in the trie (including initial 256).
    pub fn len(&self) -> usize {
        self.next_code as usize
    }

    /// Whether the trie has only initial entries.
    pub fn is_empty(&self) -> bool {
        false // always has initial 256 entries
    }
    pub fn is_only_initial(&self) -> bool {
        self.next_code == 256
    }

    /// Reset the trie to initial state (only single-byte entries).
    pub fn reset(&mut self) {
        self.root = TrieNode::new();
        for i in 0u16..256 {
            let byte = i as u8;
            let mut child = TrieNode::new();
            child.set_code(i);
            self.root.children.insert(byte, child);
        }
        self.next_code = 256;
    }

    fn find_node_by_code(&self, code: u16) -> Option<*const TrieNode> {
        // For codes 0-255, we know exactly where they are
        if code < 256 {
            return self.root.children.get(&(code as u8)).map(|n| n as *const _);
        }
        // For higher codes, search
        self.search_node(&self.root, code)
    }

    fn search_node(&self, node: &TrieNode, code: u16) -> Option<*const TrieNode> {
        if node.code() == Some(code) {
            return Some(node as *const _);
        }
        for child in node.children.values() {
            if let Some(n) = self.search_node(child, code) {
                return Some(n);
            }
        }
        None
    }

    fn get_path_to(&self, target: *const TrieNode) -> Option<Vec<u8>> {
        let mut path = Vec::new();
        self.find_path(&self.root, target, &mut path)
    }

    fn find_path(
        &self,
        node: &TrieNode,
        target: *const TrieNode,
        path: &mut Vec<u8>,
    ) -> Option<Vec<u8>> {
        for (&key, child) in &node.children {
            if std::ptr::eq(child, target) {
                path.push(key);
                return Some(path.clone());
            }
            path.push(key);
            if let Some(result) = self.find_path(child, target, path) {
                return Some(result);
            }
            path.pop();
        }
        None
    }
}

impl Default for Trie {
    fn default() -> Self {
        Self::new()
    }
}
