//! Register SmartCN analysis components into [`AnalysisFactory`].

use alloc::boxed::Box;
use alloc::vec;

use pizza_engine::analysis::AnalysisFactory;
use pizza_engine::analysis::Analyzer;

use crate::SmartCnStopFilter;
use crate::SmartCnTokenizer;

/// Register SmartCN tokenizer, filter, and analyzer.
///
/// Matches Elasticsearch's analysis-smartcn plugin registration:
/// - Tokenizer: `smartcn_tokenizer`
/// - Token Filter: `smartcn_stop`
/// - Analyzer: `smartcn` (SmartChineseAnalyzer pipeline: tokenizer → stop)
pub fn register_all(factory: &mut AnalysisFactory) {
    factory.register_tokenizer_with("smartcn_tokenizer", || Box::new(SmartCnTokenizer::new()));
    factory.register_token_filter_with("smartcn_stop", || Box::new(SmartCnStopFilter::new()));

    factory.register_analyzer_with("smartcn", || {
        Analyzer::new(
            vec![],
            Box::new(SmartCnTokenizer::new()),
            vec![Box::new(SmartCnStopFilter::new())],
        )
    });
}
