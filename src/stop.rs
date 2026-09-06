//! SmartCN stop words for Chinese.

use hashbrown::HashSet;

use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// SmartCN Chinese stop words (common particles, measure words, etc.).
pub const SMARTCN_STOP_WORDS: &[&str] = &[
    "的", "了", "在", "是", "我", "有", "和", "就", "不", "人", "都", "一", "一个", "上", "也",
    "很", "到", "说", "要", "去", "你", "会", "着", "没有", "看", "好", "自己", "这", "他", "她",
    "它", "地", "得", "这个", "那个", "那", "吗", "什么", "怎么", "哪", "谁", "几", "多", "啊",
    "吧", "呢", "呀", "嗯", "哦", "哈", "把", "被", "让", "给", "从", "向", "对", "于", "以",
    "因为", "所以", "但是", "而且", "如果", "虽然", "但", "只", "已经", "还是", "或者", "比较",
    "非常", "可以", "可能", "应该", "这样", "那样", "一样", "不同", "然后", "之后", "以后",
];

/// SmartCN Chinese stop word filter.
#[derive(Clone)]
pub struct SmartCnStopFilter {
    stop_words: HashSet<String>,
}

impl SmartCnStopFilter {
    pub fn new() -> Self {
        Self {
            stop_words: SMARTCN_STOP_WORDS.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn with_words(words: Vec<String>) -> Self {
        Self {
            stop_words: words.into_iter().collect(),
        }
    }
}

impl Default for SmartCnStopFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for SmartCnStopFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let deleted = self.stop_words.contains(token.term.as_ref());
        (deleted, None)
    }
}
