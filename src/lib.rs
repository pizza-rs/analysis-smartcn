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
mod dict;
mod tokenizer;
mod stop;

pub use tokenizer::SmartCnTokenizer;
pub use stop::SmartCnStopFilter;
pub mod register;
pub use register::register_all;
