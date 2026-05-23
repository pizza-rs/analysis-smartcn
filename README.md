<div align="center">

# 🇨🇳 pizza-analysis-smartcn

**Smart Chinese analysis plugin for [INFINI Pizza](https://pizza.rs)**

[![Crate](https://img.shields.io/badge/crate-pizza--analysis--smartcn-blue)](https://github.com/pizza-rs/analysis-smartcn)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)

</div>

---

## Overview

A probabilistic Chinese word segmenter using Hidden Markov Model (HMM) with
bigram probabilities. Provides high-quality Chinese text segmentation without
external dictionary dependencies — the model is compiled into the binary from
a pre-built word frequency table.

## Components

| Type | Name | Description |
|:-----|:-----|:------------|
| Tokenizer | `smartcn_tokenizer` | HMM-based Chinese word segmentation |
| TokenFilter | `smartcn_stop` | Chinese + English stop words |
| Analyzer | `smartcn` | Full pipeline: smartcn_tokenizer → stop words |

### How It Works

The SmartCN tokenizer uses a bigram language model:
1. Scans input character-by-character
2. For Chinese text blocks, finds optimal segmentation via Viterbi decoding
3. Non-Chinese text (ASCII, numbers) is segmented by whitespace/punctuation
4. Dictionary-free — works for any Chinese text including neologisms

## Example

```rust
use pizza_engine::analysis::Tokenizer;
use pizza_analysis_smartcn::SmartCnTokenizer;

let tk = SmartCnTokenizer::new();
let tokens = tk.tokenize("中华人民共和国");
// ["中华", "人民", "共和国"]
```

## Installation

```toml
[dependencies]
pizza-analysis-smartcn = "0.1"
```

Or via `pizza-analysis-all`:

```toml
[dependencies]
pizza-analysis-all = { version = "0.1", features = ["smartcn"] }
```

## License

MIT

---

<div align="center">
<sub>Part of the <a href="https://pizza.rs">INFINI Pizza</a> ecosystem</sub>
</div>
