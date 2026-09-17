#![cfg_attr(not(feature = "std"), no_std)]
//! SmartCN Chinese word segmentation for Pizza search engine.
//!
//! Implements a word-frequency-based Chinese word segmenter using the same
//! algorithmic approach as Lucene's SmartChineseAnalyzer: a word frequency
//! dictionary with Viterbi-like dynamic programming to find the maximum
//! probability segmentation path.
//!
//! # Algorithm
//!
//! The segmenter uses dynamic programming (Viterbi) over a DAG of possible
//! words built from a frequency dictionary:
//! 1. Build a DAG where edges represent dictionary words at each position
//! 2. Find the path maximizing total log-probability
//! 3. Non-CJK sequences (ASCII, digits) are grouped normally
//!
//! # Components
//!
//! - [`SmartCnTokenizer`] — Chinese word segmentation tokenizer
//! - [`SmartCnStopFilter`] — Chinese stop words filter
extern crate alloc;
#[cfg(feature = "std")]
#[doc(hidden)]
/// Point the analysis dictionary directory at this crate's `src/data/`
/// copy so tests can construct the tokenizer under any feature selection
/// (the external file must be laid out as `<dict_dir>/smartcn/word_freq.txt`).
pub fn init_test_dict_dir() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let dir = std::env::temp_dir().join(format!(
            "pizza-smartcn-test-dict-{}",
            std::process::id()
        ));
        let ns = dir.join("smartcn");
        if std::fs::create_dir_all(&ns).is_ok() {
            let _ = std::fs::copy(
                concat!(env!("CARGO_MANIFEST_DIR"), "/src/data/word_freq.txt"),
                ns.join("word_freq.txt"),
            );
        }
        pizza_engine::analysis::dict::set_dict_dir(&dir);
    });
}

mod dict;
mod stop;
mod tokenizer;

pub use stop::SmartCnStopFilter;
pub use tokenizer::SmartCnTokenizer;
pub mod register;
pub use register::register_all;
