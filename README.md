# pizza-analysis-smartcn

Chinese word segmentation for the [Pizza](https://pizza.rs) search engine. Implements the SmartCN algorithm — a Viterbi/dynamic-programming word segmenter using word frequency statistics, equivalent to Apache Lucene's SmartChineseAnalyzer.

## Components

| Name | Type | Description |
|------|------|-------------|
| `smartcn_tokenizer` | Tokenizer | DP/Viterbi Chinese word segmentation |
| `smartcn_stop` | Token Filter | Chinese stop words removal |
| `smartcn` | Analyzer | Full pipeline: tokenizer + stop filter |

## Usage

### Full Analyzer

```json
{
  "analyzer": {
    "type": "smartcn"
  }
}
```

### Custom Pipeline

```json
{
  "analyzer": {
    "type": "custom",
    "tokenizer": "smartcn_tokenizer",
    "filter": ["smartcn_stop", "lowercase"]
  }
}
```

### Example

**Input:** `我是中国人民的一员`

**Output tokens:** `我`, `是`, `中国`, `人民`, `的`, `一`, `员`

## Algorithm

SmartCN uses a Viterbi/dynamic-programming approach:

1. **Sentence detection** — Splits input into CJK sentence segments and ASCII/digit sequences
2. **DAG construction** — For each position in a CJK segment, uses prefix matching on the word dictionary (via a double-array trie) to find all possible words starting at that position
3. **Viterbi DP** — Finds the optimal segmentation path by maximizing the total word log-probability across the entire sentence
4. **Backtracking** — Traces back through the DP table to produce the final token sequence

This produces linguistically superior segmentation compared to greedy approaches, as it considers the global context of the entire sentence.

## Data Sources

- **Word frequency dictionary**: 85,607 entries extracted from Apache Lucene's `coredict.mem` (the SmartCN unigram dictionary)
- **DARTS trie**: Built at load time using the [cedarwood](https://github.com/nickel-org/cedarwood) double-array trie for fast prefix matching
- **Stop words**: Standard Chinese stop words list

## Technical Details

- Uses `include_str!` to embed the word frequency data at compile time
- Dictionary entries are sorted by bytes for cedarwood trie construction
- Word probabilities are computed as `log(freq / total)` with a smoothed floor for unknown words
- CJK detection handles the full BMP CJK Unified Ideographs range (U+4E00–U+9FFF)
- Non-CJK text (ASCII, digits) is emitted as separate tokens without segmentation

## Comparison with Other Chinese Tokenizers

| Feature | SmartCN | IK | Jieba |
|---------|---------|-----|-------|
| Algorithm | Viterbi DP | Dictionary + ambiguity resolution | HMM + Dictionary |
| Dictionary size | 85K words | 275K words | 350K words |
| Speed | Fast | Very fast | Fast |
| New words | Limited | Limited | HMM for unknown words |
| Memory | Low | Medium | Medium |

## License

Apache-2.0
