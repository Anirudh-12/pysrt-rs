# syntax=docker/dockerfile:1
# ---------------------------------------------------------------------------
# pysrt-rs — Port Mortem 2026 (Track D: Python → Rust)
#
# Stage 1 — builder
#   • Installs Rust toolchain + maturin via the official maturin base image
#   • cargo build --release         → native `srt` CLI binary
#   • cargo test --all-targets      → native Rust unit tests
#   • maturin build --release       → Python wheel (.whl)
#   • pip install <wheel>           → installed into /opt/venv
#
# Stage 2 — runtime
#   • Slim Python image
#   • Copies /opt/venv + CLI binary + tests + fuzz harness
#   • Default CMD: run the full original unmodified test suite
#
# Usage:
#   docker build -t pysrt-rs .
#   docker run --rm pysrt-rs                              # run test suite
#   docker run --rm pysrt-rs python fuzz/diff_fuzz.py    # differential fuzz
#   docker run --rm pysrt-rs python bench/run_bench.py   # benchmark
#   docker run --rm pysrt-rs srt --help                  # CLI
# ---------------------------------------------------------------------------

# ── Stage 1: builder ────────────────────────────────────────────────────────
# ghcr.io/pyo3/maturin ships Rust + maturin on a manylinux base.
FROM ghcr.io/pyo3/maturin:latest AS builder

WORKDIR /build

# ── Copy manifests first (maximises Docker layer-cache reuse) ───────────────
COPY Cargo.toml pyproject.toml ./
# Cargo.lock may not exist yet; the wildcard avoids an error if absent.
COPY Cargo.lock* ./

# Stub src/ so `cargo fetch` can resolve the graph without compiling real code.
RUN mkdir -p src/bin src/python && \
    printf 'fn main() {}' > src/bin/srt.rs && \
    printf 'pub fn placeholder() {}' > src/lib.rs

# Pre-fetch Rust dependencies (cached unless Cargo.toml changes).
RUN cargo fetch

# ── Copy the full project source ─────────────────────────────────────────────
COPY src/            ./src/
COPY tests/          ./tests/
COPY reference_pysrt/ ./reference_pysrt/
COPY fuzz/           ./fuzz/
COPY bench/          ./bench/
COPY DECISIONS.md .port-mortem.toml README.md ./

# ── 1a: Build native Rust library + CLI binary (`srt`) ──────────────────────
RUN cargo build --release
# Artifacts: target/release/srt  (CLI binary)
#            target/release/libpysrt.rlib  (Rust library)

# ── 1b: Native Rust unit tests ───────────────────────────────────────────────
RUN cargo test --all-targets

# ── 1c: Build Python wheel via maturin for Python 3.12 ───────────────────────
RUN maturin build --features python --release --out /wheels -i 3.12


# ── Stage 2: runtime ────────────────────────────────────────────────────────
FROM python:3.12-slim AS runtime

LABEL org.opencontainers.image.title="pysrt-rs" \
      org.opencontainers.image.description="Port Mortem 2026 Track D — memory-safe Rust port of byroot/pysrt" \
      org.opencontainers.image.licenses="MIT"

WORKDIR /app

# Install pytest and the compiled pysrt wheel into Python 3.12
COPY --from=builder /wheels /wheels
RUN pip install --no-cache-dir pytest /wheels/*.whl && rm -rf /wheels

# Copy the native CLI binary
COPY --from=builder /build/target/release/srt /usr/local/bin/srt

# Copy runtime artefacts
COPY --from=builder /build/tests/            ./tests/
COPY --from=builder /build/reference_pysrt/  ./reference_pysrt/
COPY --from=builder /build/fuzz/             ./fuzz/
COPY --from=builder /build/bench/            ./bench/
COPY --from=builder /build/DECISIONS.md      ./DECISIONS.md
COPY --from=builder /build/.port-mortem.toml ./.port-mortem.toml
COPY --from=builder /build/README.md         ./README.md

# Smoke-test: verify the CLI binary runs and the Python extension imports.
RUN srt --help > /dev/null && \
    python -c "import libsrt; print('libsrt', libsrt.VERSION_STRING, 'OK')"

# ── Default: run the original unmodified test suite ──────────────────────────
CMD ["pytest", \
     "tests/test_srttime.py", \
     "tests/test_srtitem.py", \
     "tests/test_srtfile.py", \
     "-v", "--tb=short"]
