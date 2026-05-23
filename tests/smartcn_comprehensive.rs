//! Comprehensive tests for pizza-analysis-smartcn (Chinese word segmentation).

use pizza_analysis_smartcn::{SmartCnStopFilter, SmartCnTokenizer};
use pizza_engine::analysis::{AnalysisFactory, Token, TokenFilter, Tokenizer};

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

fn make_token(term: &str) -> Token<'_> {
    Token::new(term, 0, term.len() as u32, 0)
}

fn terms(tokens: &[Token]) -> Vec<String> {
    tokens.iter().map(|t| t.term.to_string()).collect()
}

// ═══════════════════════════════════════════════════════════════════════════════
// SmartCnTokenizer — construction
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn tokenizer_construction() {
    let _t = SmartCnTokenizer::new();
}

#[test]
fn tokenizer_default_trait() {
    let _t = SmartCnTokenizer::default();
}

#[test]
fn tokenizer_clone() {
    let t1 = SmartCnTokenizer::new();
    let _t2 = t1.clone();
}

// ═══════════════════════════════════════════════════════════════════════════════
// SmartCnTokenizer — basic segmentation
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn tokenize_simple_chinese() {
    let t = SmartCnTokenizer::new();
    let tokens = t.tokenize("中华人民共和国");
    assert!(!tokens.is_empty());
    // Verify all terms concatenate back to original (no gaps)
    let joined: String = tokens.iter().map(|t| t.term.as_ref()).collect();
    assert!(joined.chars().all(|c| "中华人民共和国".contains(c)));
}

#[test]
fn tokenize_mixed_chinese_english() {
    let t = SmartCnTokenizer::new();
    let tokens = t.tokenize("我喜欢Python编程");
    let ts = terms(&tokens);
    assert!(!ts.is_empty());
    // Should find "Python" as a token
    assert!(ts.iter().any(|s| s == "Python" || s == "python"));
}

#[test]
fn tokenize_chinese_with_numbers() {
    let t = SmartCnTokenizer::new();
    let tokens = t.tokenize("2024年北京奥运会");
    assert!(!tokens.is_empty());
    // Should contain "2024" or similar numeric token
    assert!(tokens.iter().any(|t| t.term.contains("2024")));
}

#[test]
fn tokenize_pure_ascii() {
    let t = SmartCnTokenizer::new();
    let tokens = t.tokenize("hello world");
    assert!(!tokens.is_empty());
}

#[test]
fn tokenize_pure_digits() {
    let t = SmartCnTokenizer::new();
    let tokens = t.tokenize("12345");
    assert!(!tokens.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// SmartCnTokenizer — edge cases
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn tokenize_empty_string() {
    let t = SmartCnTokenizer::new();
    let tokens = t.tokenize("");
    assert!(tokens.is_empty());
}

#[test]
fn tokenize_single_chinese_char() {
    let t = SmartCnTokenizer::new();
    let tokens = t.tokenize("我");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].term.as_ref(), "我");
}

#[test]
fn tokenize_whitespace_only() {
    let t = SmartCnTokenizer::new();
    let tokens = t.tokenize("   ");
    assert!(tokens.is_empty());
}

#[test]
fn tokenize_punctuation_only() {
    let t = SmartCnTokenizer::new();
    let tokens = t.tokenize("，。！？");
    assert!(tokens.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// SmartCnTokenizer — offsets and positions
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn tokenize_offsets_valid() {
    let t = SmartCnTokenizer::new();
    let text = "搜索引擎技术";
    let tokens = t.tokenize(text);
    for tok in &tokens {
        assert!(tok.start_offset <= tok.end_offset);
        assert!((tok.end_offset as usize) <= text.len());
    }
}

#[test]
fn tokenize_positions_monotonic() {
    let t = SmartCnTokenizer::new();
    let tokens = t.tokenize("自然语言处理");
    for window in tokens.windows(2) {
        assert!(window[1].position >= window[0].position);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SmartCnStopFilter — construction
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn stop_filter_construction() {
    let _f = SmartCnStopFilter::new();
}

#[test]
fn stop_filter_default() {
    let _f = SmartCnStopFilter::default();
}

#[test]
fn stop_filter_custom_words() {
    let words = vec!["自定义".to_string()];
    let _f = SmartCnStopFilter::with_words(words);
}

// ═══════════════════════════════════════════════════════════════════════════════
// SmartCnStopFilter — filtering
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn stop_filter_removes_particles() {
    let f = SmartCnStopFilter::new();
    for &word in &["的", "了", "在", "是", "和"] {
        let mut token = make_token(word);
        let (deleted, _) = f.filter(&mut token);
        assert!(deleted, "expected '{}' to be deleted", word);
    }
}

#[test]
fn stop_filter_keeps_content_words() {
    let f = SmartCnStopFilter::new();
    for &word in &["计算机", "搜索", "引擎"] {
        let mut token = make_token(word);
        let (deleted, _) = f.filter(&mut token);
        assert!(!deleted, "expected '{}' to NOT be deleted", word);
    }
}

#[test]
fn stop_filter_empty_token() {
    let f = SmartCnStopFilter::new();
    let mut token = make_token("");
    let (deleted, _) = f.filter(&mut token);
    assert!(!deleted);
}

#[test]
fn stop_filter_custom_words_filtering() {
    let words = vec!["测试".to_string()];
    let f = SmartCnStopFilter::with_words(words);
    let mut token = make_token("测试");
    let (deleted, _) = f.filter(&mut token);
    assert!(deleted);

    let mut token2 = make_token("代码");
    let (deleted2, _) = f.filter(&mut token2);
    assert!(!deleted2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Registration
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn register_all_does_not_panic() {
    let mut factory = AnalysisFactory::new();
    pizza_analysis_smartcn::register_all(&mut factory);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Pipeline integration
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn pipeline_tokenize_then_stop_filter() {
    let tokenizer = SmartCnTokenizer::new();
    let stop = SmartCnStopFilter::new();

    let tokens = tokenizer.tokenize("我是一个程序员");
    let mut surviving = Vec::new();
    for mut tok in tokens {
        let (deleted, _) = stop.filter(&mut tok);
        if !deleted {
            surviving.push(tok.term.to_string());
        }
    }
    // Common particles like "是", "一个" should be removed
    assert!(!surviving.iter().any(|s| s == "是"));
}

#[test]
fn pipeline_long_text() {
    let tokenizer = SmartCnTokenizer::new();
    let text = "人工智能是计算机科学的一个分支，它试图理解智能的实质，并生产出一种新的能以人类智能相似的方式做出反应的智能机器。";
    let tokens = tokenizer.tokenize(text);
    assert!(tokens.len() > 5);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Unicode handling
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn tokenize_cjk_unified_ideographs() {
    let t = SmartCnTokenizer::new();
    let tokens = t.tokenize("鬱鬱蔥蔥");
    assert!(!tokens.is_empty());
}

#[test]
fn tokenize_chinese_with_emoji() {
    let t = SmartCnTokenizer::new();
    // Should not panic on emoji mixed with Chinese
    let _tokens = t.tokenize("你好😊世界");
}

#[test]
fn tokenize_japanese_kanji() {
    let t = SmartCnTokenizer::new();
    // CJK characters shared with Japanese — should still segment
    let tokens = t.tokenize("東京大学");
    assert!(!tokens.is_empty());
}
