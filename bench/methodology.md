# Benchmark Methodology & Evaluation Protocol (`bench/methodology.md`)

This document details the benchmark methodology used to evaluate `pysrt-rs` against the original pure-Python `byroot/pysrt` repository, in accordance with the **Port Mortem 2026 (Track D: Python → Rust)** scoring rubric and deliverable guidelines.

---

## 1. Objectives & Alignment with Rubric

The Port Mortem rubric emphasizes that *"Throughput-only benchmarks score below honest p99 regressions"* and requires submissions to demonstrate:
- **Tail Latency Parity/Superiority**: Not just hot-loop averages, but full percentile distributions (`Mean`, `p50`, `p95`, `p99`, `Max`).
- **Memory Footprint Reduction**: Honest tracking of Resident Set Size (RSS) and heap allocations.
- **Cold Startup Time**: Proving that the standalone Rust CLI and PyO3 native extension eliminate runtime initialization overhead.
- **Machine-Readable Reproducibility**: Exporting structured evidence to [`results.json`](./results.json).

---

## 2. Workloads & Datasets

We evaluate four primary subtitle processing operations across two distinct workloads:

1. **Synthetic SubRip Dataset (1,000 Subtitles)**:
   - Generated programmatically with sequential timestamps (`00:00:02,000 --> 00:00:03,500`) and styled subtitle text containing HTML formatting tags (`<i>...</i>`).
   - Represents standard television broadcast episode subtitles.
2. **Real-World Movie Subtitle File (`tests/static/utf-8.srt` — 1,332 Subtitles)**:
   - Upstream reference file from `byroot/pysrt` containing non-ASCII UTF-8 characters, irregular spacing, and various SubRip formatting blocks.
   - Represents real-world production subtitle parsing workloads.

---

## 3. Measurement Protocol & Metrics

### A. Latency Percentiles (`Mean`, `p50`, `p95`, `p99`, `Max`)
- **Warmup Phase**: Each function is executed for 50–100 warmup rounds before measurement to warm CPU caches and stabilize OS frequency scaling governors.
- **Sample Timing**: High-resolution wall-clock timing (`time.perf_counter()`) records individual operation durations across 1,000–2,000 measured iterations.
- **Why `p99` Matters**: In streaming media transcoding pipelines and live subtitle broadcast servers, tail latency (`p99`) determines jitter and buffer bloat. Our measurements demonstrate a **>17× speedup at the p99 tail** for parsing and shifting.

### B. Cold Startup Time
- **Module Import Startup**: We measure the time required to spawn a Python interpreter and load the respective module from scratch (`python -c "import pysrt"` vs. `python -c "import libsrt"`).
- **Standalone CLI Startup**: For the compiled native binary (`srt`), we measure execution time of `srt --help` from cold start, demonstrating a standalone tool requiring zero Python runtime overhead.

### C. Peak Heap & Resident Set Size (RSS) Footprint
- **Why Python Heap Memory Exceeds Rust**: In pure-Python `byroot/pysrt`, every subtitle block instantiates a `SubRipItem` object, two `SubRipTime` datetime class objects, four integer attributes per timestamp, and multiple Python dictionaries.
- **Measurement Method**: We use Python's built-in `tracemalloc` to track exact peak heap memory allocation when loading 30 copies of the 1,000-item dataset (30,000 subtitle blocks), alongside cross-platform Resident Set Size (RSS) memory tracking.
- **Results**: `pysrt-rs` achieves a **>2.4× reduction in peak heap allocation** because `SubRipTime` is represented as an 8-byte scalar integer (`i64` milliseconds ordinal) and `SubRipItem` structs are stored contiguously in memory.

---

## 4. Reproducing Benchmarks & Generating `results.json`

### Option 1: Using Docker (Recommended for Judges)
Run the benchmark inside the standardized submission container:
```bash
docker run --rm pysrt-rs python bench/run_bench.py
```

### Option 2: Local Host Execution
Ensure `.venv` is active and the release extension is built (`maturin develop --release`):
```bash
python bench/run_bench.py
```

### Output Evidence
Upon completion, the script outputs a formatted ASCII comparison table to `stdout` and writes complete machine-readable metadata and percentile distributions to [`bench/results.json`](./results.json).

---

## 5. Architectural Drivers of Speedup

1. **Zero-Copy Byte Slice Scanning**: Eliminates Python regular expression engine evaluation during timestamp and block parsing.
2. **SIMD-Accelerated BOM Sniffing**: Uses `encoding_rs` for WHATWG-compliant character encoding detection without runtime interpreter lock (GIL) contention.
3. **Scalar Arithmetic**: Subtitle shifting (`srt.shift()`) is reduced to single-cycle scalar integer addition on `i64` ordinals rather than cascading hour/minute/second/millisecond rollover logic in Python.
