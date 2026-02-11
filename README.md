# High-Cut CLI

![Rust](https://img.shields.io/badge/Language-Rust-orange?logo=rust)
![License](https://img.shields.io/badge/License-ZCI%20Commercial-blue)
![Platform](https://img.shields.io/badge/Platform-macOS%20Silicon-lightgrey?logo=apple)
![Performance](https://img.shields.io/badge/Performance-M4%20Pro%20Optimized-red)

**Deterministic Highlight Extraction for Elite Content Creators.**
Optimized for Apple M4 Pro Silicon. Standalone. Private. High-Velocity.

## Core USP

- **Architecture-Aware**: Leverages macOS-native FFmpeg optimization paths.
- **Physics-Based Heuristics**: Deterministic silence and peak detection instead of probabilistic AI hallucinations.
- **SLA-Free Extraction**: No cloud dependency. Zero latency to first pixel.
- **Privacy First**: No telemetry. Your data stays on your machine.

## Hardware Baseline (Reproducibility)

Benchmarked on:

- **Model**: A3401 (M4 Pro, 14-inch)
- **CPU**: 12-Core Apple Silicon
- **RAM**: 24 GB Unified Memory
- **OS**: macOS Sequoia

## Build & Test

```bash
# Clean build
cargo build --release

# Run unit tests
cargo test
```

## Usage

```bash
./target/release/high-cut run input.mp4 --output highlights_dir
```

### Advanced Configuration (`config.yaml`)

```yaml
silence_threshold_db: -35.0   # Adjust for background noise
min_silence_duration: 1.2     # Min gap between highlights
min_clip_length: 3.0          # Avoid tiny snippets
max_clip_length: 60.0         # Auto-split long segments
margin_s: 0.5                 # Breathable padding around highlights
```

## Performance Benchmarks (Actual M4 Pro Data)

| Operation | Scale | Latency / Speed |
| :--- | :--- | :--- |
| **Heuristic Analysis** | 10,000 events | **32.6 µs** (Criterion) |
| **Highlight Extraction** | H.264 1080p60 | **25x - 30x Speed** |
| **Cold Start** | CLI Ready | **< 10ms** |

### Competitive Edge

Compared to probabilistic AI tools, High-Cut is **~1000x faster** at decision mapping. While others "think" about highlights, we've already finished the extraction.

## Licensing

Licensed under the **ZCI Commercial License v1.0**.

- **Free for evaluation, personal, and academic use.**
- **Commercial license required for professional/business use.**
- **Enforcement**: No DRM. No watermarks. We trust our elite users.
See [LICENSE](LICENSE) for full details.

---
© 2026 ZCI Tech. All Rights Reserved.
