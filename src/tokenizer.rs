//! SmartCN Chinese tokenizer using word frequency dictionary + dynamic programming.
//!
//! Implements the same algorithmic approach as Lucene's SmartChineseAnalyzer:
//! uses a word frequency dictionary with Viterbi-like dynamic programming
//! to find the maximum probability segmentation of Chinese text.
//!
//! # Algorithm
//!
//! 1. Split text into sentences (on punctuation/whitespace boundaries)
//! 2. For each sentence containing CJK characters, find optimal segmentation:
//!    - Build a DAG (Directed Acyclic Graph) of all possible words
//!    - Use dynamic programming to find the path with maximum total probability
//! 3. Non-CJK sequences (ASCII, digits) are grouped normally

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use pizza_engine::analysis::{Token, Tokenizer};

use crate::dict::WordDict;

/// SmartCN Chinese tokenizer.
///
/// Segments Chinese text using statistical word segmentation with a frequency
/// dictionary and dynamic programming (maximum probability path).
///
/// Equivalent to Elasticsearch's `smartcn_tokenizer`.
#[derive(Clone)]
pub struct SmartCnTokenizer {
    dict: Arc<WordDict>,
}

impl SmartCnTokenizer {
    /// Create with the embedded word frequency dictionary.
    pub fn new() -> Self {
        Self {
            dict: Arc::new(WordDict::new()),
        }
    }
}

impl Default for SmartCnTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Character type classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CharType {
    Cjk,
    AsciiLetter,
    Digit,
    Space,
    Punctuation,
    Other,
}

impl Tokenizer for SmartCnTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let chars: Vec<(usize, char)> = text.char_indices().collect();

        if chars.is_empty() {
            return tokens;
        }

        let mut position = 0u32;
        let mut i = 0;

        while i < chars.len() {
            let (_, ch) = chars[i];
            let char_type = classify_char(ch);

            match char_type {
                CharType::Space | CharType::Punctuation => {
                    i += 1;
                }
                CharType::AsciiLetter => {
                    // Collect consecutive ASCII letters
                    let start_char = i;
                    let start_byte = chars[i].0;
                    while i < chars.len() && classify_char(chars[i].1) == CharType::AsciiLetter {
                        i += 1;
                    }
                    let end_byte = if i < chars.len() { chars[i].0 } else { text.len() };
                    tokens.push(Token {
                        term: Cow::Borrowed(&text[start_byte..end_byte]),
                        start_offset: start_char as u32,
                        end_offset: i as u32,
                        position,
                    });
                    position += 1;
                }
                CharType::Digit => {
                    // Collect consecutive digits
                    let start_char = i;
                    let start_byte = chars[i].0;
                    while i < chars.len() && classify_char(chars[i].1) == CharType::Digit {
                        i += 1;
                    }
                    let end_byte = if i < chars.len() { chars[i].0 } else { text.len() };
                    tokens.push(Token {
                        term: Cow::Borrowed(&text[start_byte..end_byte]),
                        start_offset: start_char as u32,
                        end_offset: i as u32,
                        position,
                    });
                    position += 1;
                }
                CharType::Cjk => {
                    // Collect consecutive CJK characters
                    let cjk_start = i;
                    while i < chars.len() && classify_char(chars[i].1) == CharType::Cjk {
                        i += 1;
                    }
                    let cjk_end = i;

                    // Segment CJK sequence using dynamic programming
                    let words = self.segment_cjk(text, &chars[cjk_start..cjk_end], cjk_start);
                    for (word_text, start_offset, end_offset) in words {
                        tokens.push(Token {
                            term: Cow::Owned(word_text),
                            start_offset: start_offset as u32,
                            end_offset: end_offset as u32,
                            position,
                        });
                        position += 1;
                    }
                }
                CharType::Other => {
                    i += 1;
                }
            }
        }

        tokens
    }
}

impl SmartCnTokenizer {
    /// Segment a CJK character sequence using dynamic programming.
    ///
    /// Uses DARTS prefix scanning + Viterbi to find the maximum probability
    /// segmentation path through all possible word boundaries.
    fn segment_cjk(
        &self,
        text: &str,
        chars: &[(usize, char)],
        base_offset: usize,
    ) -> Vec<(String, usize, usize)> {
        let n = chars.len();
        if n == 0 {
            return Vec::new();
        }

        // Single character: just return it
        if n == 1 {
            let (byte_pos, ch) = chars[0];
            let end_byte = byte_pos + ch.len_utf8();
            return vec![(
                String::from(&text[byte_pos..end_byte]),
                base_offset,
                base_offset + 1,
            )];
        }

        // Build DAG using DARTS prefix scanning.
        // dag[i] = list of (end_char_pos, log_prob)
        // For each position, we use common_prefix_search to find ALL dictionary
        // words starting at that position in a single trie traversal.
        let mut dag: Vec<Vec<(usize, f64)>> = Vec::with_capacity(n);

        // Pre-compute byte positions for char boundaries
        let mut char_byte_ends: Vec<usize> = Vec::with_capacity(n + 1);
        for &(byte_pos, ch) in chars.iter() {
            char_byte_ends.push(byte_pos);
        }
        // Append the end of the last char
        let last_end = chars[n - 1].0 + chars[n - 1].1.len_utf8();
        char_byte_ends.push(last_end);

        // Build a byte-offset to char-index map for the CJK segment
        // We need this to convert byte-length matches from DARTS back to char positions
        let segment_start_byte = chars[0].0;

        for start in 0..n {
            let start_byte = chars[start].0;
            let segment_text = &text[start_byte..last_end];

            let mut edges = Vec::new();

            // Use DARTS common_prefix_search: returns all dict words that are
            // prefixes of segment_text (i.e., words starting at position `start`)
            let matches = self.dict.prefix_match(segment_text);
            for (match_byte_len, word_id) in &matches {
                // Convert byte length to char count
                let match_end_byte = start_byte + match_byte_len;
                // Find which char index this byte offset corresponds to
                if let Ok(char_end) = char_byte_ends[start..].binary_search(&match_end_byte) {
                    let end = start + char_end;
                    if end > start && end <= n {
                        let prob = self.dict.word_prob_by_id(*word_id);
                        edges.push((end, prob));
                    }
                }
            }

            // Always ensure single-char edge exists (fallback)
            let has_single = edges.iter().any(|&(end, _)| end == start + 1);
            if !has_single {
                let single_byte_end = chars[start].0 + chars[start].1.len_utf8();
                let single_word = &text[chars[start].0..single_byte_end];
                let prob = self.dict.word_prob(single_word);
                edges.push((start + 1, prob));
            }

            dag.push(edges);
        }

        // Viterbi: find max probability path from 0 to n
        // best[i] = (best_log_prob_to_reach_i, previous_position)
        let mut best: Vec<(f64, usize)> = vec![(f64::NEG_INFINITY, 0); n + 1];
        best[0] = (0.0, 0);

        for pos in 0..n {
            if best[pos].0 == f64::NEG_INFINITY {
                continue;
            }
            for &(end, prob) in &dag[pos] {
                let new_prob = best[pos].0 + prob;
                if new_prob > best[end].0 {
                    best[end] = (new_prob, pos);
                }
            }
        }

        // Backtrack to find the optimal segmentation
        let mut boundaries = Vec::new();
        let mut pos = n;
        while pos > 0 {
            let prev = best[pos].1;
            boundaries.push((prev, pos));
            pos = prev;
        }
        boundaries.reverse();

        // Convert boundaries to word strings with offsets
        let mut result = Vec::with_capacity(boundaries.len());
        for (start, end) in boundaries {
            let word = Self::extract_word(text, chars, start, end);
            result.push((word, base_offset + start, base_offset + end));
        }

        result
    }

    /// Extract a word substring from char positions.
    fn extract_word(text: &str, chars: &[(usize, char)], start: usize, end: usize) -> String {
        let start_byte = chars[start].0;
        let end_byte = if end < chars.len() {
            chars[end].0
        } else {
            let (last_byte, last_char) = chars[chars.len() - 1];
            last_byte + last_char.len_utf8()
        };
        String::from(&text[start_byte..end_byte])
    }
}

/// Classify a character into its type for segmentation.
fn classify_char(ch: char) -> CharType {
    let c = ch as u32;

    // CJK Unified Ideographs
    if (0x4E00..=0x9FFF).contains(&c)
        || (0x3400..=0x4DBF).contains(&c)
        || (0x20000..=0x2A6DF).contains(&c)
        || (0x2A700..=0x2B73F).contains(&c)
        || (0xF900..=0xFAFF).contains(&c)
    {
        return CharType::Cjk;
    }

    // ASCII letters
    if ch.is_ascii_alphabetic() {
        return CharType::AsciiLetter;
    }

    // Digits (ASCII and fullwidth)
    if ch.is_ascii_digit() || ('０'..='９').contains(&ch) {
        return CharType::Digit;
    }

    // Whitespace
    if ch.is_whitespace() {
        return CharType::Space;
    }

    // Punctuation
    if ch.is_ascii_punctuation()
        || (0x3000..=0x303F).contains(&c)
        || (0xFF01..=0xFF0F).contains(&c)
        || (0xFF1A..=0xFF20).contains(&c)
        || (0xFF3B..=0xFF40).contains(&c)
        || (0xFF5B..=0xFF65).contains(&c)
    {
        return CharType::Punctuation;
    }

    CharType::Other
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_tokens() {
        let t = SmartCnTokenizer::new();
        let tokens = t.tokenize("hello world");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].term.as_ref(), "hello");
        assert_eq!(tokens[1].term.as_ref(), "world");
    }

    #[test]
    fn test_chinese_segmentation() {
        let t = SmartCnTokenizer::new();
        let tokens = t.tokenize("中华人民共和国");
        // Should produce multi-char words, not single chars
        assert!(!tokens.is_empty());
        // The exact segmentation depends on dictionary
        let combined: String = tokens.iter().map(|t| t.term.as_ref()).collect();
        assert_eq!(combined, "中华人民共和国");
    }

    #[test]
    fn test_mixed_text() {
        let t = SmartCnTokenizer::new();
        let tokens = t.tokenize("我爱Python编程");
        assert!(!tokens.is_empty());
        let has_python = tokens.iter().any(|t| t.term.as_ref() == "Python");
        assert!(has_python);
    }
}

