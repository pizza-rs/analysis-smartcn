//! Word frequency dictionary for SmartCN segmentation.
//!
//! Uses cedarwood (Double-Array Trie) for O(n) prefix scanning during DAG
//! construction. This is significantly faster than HashMap-based lookup because
//! we can scan all dictionary words starting at a given position in a single
//! trie traversal, rather than probing individual substrings.
//!
//! The dictionary is loaded from Lucene SmartCN's coredict (85,607 entries)
//! at compile time via `include_str!`.

use alloc::string::String;
use alloc::vec::Vec;
use cedarwood::Cedar;

/// Word frequency dictionary backed by a DARTS (Double-Array Trie Structure).
///
/// Provides two lookup modes:
/// 1. `prefix_match` - find all dictionary words that are prefixes of input
///    (used for building the segmentation DAG)
/// 2. `word_prob` - log-probability lookup for scoring
pub struct WordDict {
    /// DARTS trie for prefix scanning. Values are indices into `freqs`.
    trie: Cedar,
    /// Frequency array indexed by trie value (word_id → frequency).
    freqs: Vec<f64>,
    /// Total frequency mass (for log-probability normalization).
    total_freq: f64,
    /// Maximum word length in characters.
    max_word_len: usize,
    /// Number of entries.
    count: usize,
}

impl WordDict {
    /// Build dictionary from embedded Lucene SmartCN frequency data.
    ///
    /// Constructs a DARTS trie from ~85,000 entries for O(1) prefix scanning.
    pub fn new() -> Self {
        let data = include_str!("data/word_freq.txt");

        // First pass: parse entries
        let mut entries_vec: Vec<(String, f64)> = Vec::with_capacity(90000);
        let mut total_freq: f64 = 0.0;
        let mut max_word_len: usize = 1;

        for line in data.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // Format: "word freq" — use rsplit_once to handle words with spaces
            if let Some((word, freq_str)) = line.rsplit_once(' ') {
                if let Ok(freq) = freq_str.parse::<f64>() {
                    if freq <= 0.0 {
                        continue;
                    }
                    let word_len = word.chars().count();
                    if word_len > max_word_len {
                        max_word_len = word_len;
                    }
                    total_freq += freq;
                    entries_vec.push((String::from(word), freq));
                }
            }
        }

        // Sort by bytes for cedarwood (requires sorted insertion)
        entries_vec.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

        let mut freqs: Vec<f64> = Vec::with_capacity(entries_vec.len());
        let mut kv_pairs: Vec<(&str, i32)> = Vec::with_capacity(entries_vec.len());

        for (i, (word, freq)) in entries_vec.iter().enumerate() {
            kv_pairs.push((word.as_str(), i as i32));
            freqs.push(*freq);
        }

        let count = kv_pairs.len();
        let mut trie = Cedar::new();
        trie.build(&kv_pairs);

        Self {
            trie,
            freqs,
            total_freq,
            max_word_len,
            count,
        }
    }

    /// Find all dictionary words that are prefixes of `text`.
    ///
    /// Returns vec of (byte_length_of_match, word_id).
    /// This is the key operation for DAG building — finds all possible
    /// words starting at a given text position in a single trie traversal.
    #[inline]
    pub fn prefix_match(&self, text: &str) -> Vec<(usize, i32)> {
        match self.trie.common_prefix_search(text) {
            Some(matches) => matches.into_iter().map(|(id, len)| (len, id)).collect(),
            None => Vec::new(),
        }
    }

    /// Get the log-probability of a word by its ID (from prefix_match).
    #[inline]
    pub fn word_prob_by_id(&self, word_id: i32) -> f64 {
        let freq = self.freqs[word_id as usize];
        (freq / self.total_freq).ln()
    }

    /// Get the log-probability of a word string.
    pub fn word_prob(&self, word: &str) -> f64 {
        if let Some((word_id, _, _)) = self.trie.exact_match_search(word) {
            (self.freqs[word_id as usize] / self.total_freq).ln()
        } else {
            // Unknown word: very low probability
            (1.0 / self.total_freq).ln()
        }
    }

    /// Check if a word exists in the dictionary.
    #[inline]
    pub fn contains(&self, word: &str) -> bool {
        self.trie.exact_match_search(word).is_some()
    }

    /// Get raw frequency of a word (or None).
    pub fn get_freq(&self, word: &str) -> Option<f64> {
        self.trie
            .exact_match_search(word)
            .map(|(id, _, _)| self.freqs[id as usize])
    }

    /// Maximum word length in the dictionary (in chars).
    pub fn max_word_len(&self) -> usize {
        self.max_word_len
    }

    /// Number of entries in the dictionary.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Whether the dictionary is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Total frequency mass.
    pub fn total_freq(&self) -> f64 {
        self.total_freq
    }
}
