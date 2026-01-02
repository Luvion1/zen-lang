use std::collections::HashMap;

/// String interning for better memory usage and faster comparisons
pub struct StringInterner {
    strings: Vec<String>,
    indices: HashMap<String, usize>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            indices: HashMap::new(),
        }
    }

    pub fn intern(&mut self, s: &str) -> usize {
        if let Some(&index) = self.indices.get(s) {
            index
        } else {
            let index = self.strings.len();
            self.strings.push(s.to_string());
            self.indices.insert(s.to_string(), index);
            index
        }
    }

    pub fn get(&self, index: usize) -> Option<&str> {
        self.strings.get(index).map(|s| s.as_str())
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimized token representation using interned strings
#[derive(Debug, Clone, PartialEq)]
pub struct InternedToken {
    pub kind: crate::token::TokenType,
    pub lexeme_index: usize,
    pub line: usize,
    pub column: usize,
}

impl InternedToken {
    pub fn new(kind: crate::token::TokenType, lexeme_index: usize, line: usize, column: usize) -> Self {
        Self {
            kind,
            lexeme_index,
            line,
            column,
        }
    }

    pub fn lexeme<'a>(&self, interner: &'a StringInterner) -> Option<&'a str> {
        interner.get(self.lexeme_index)
    }
}
