#!/usr/bin/env python3
"""
Performance benchmark comparing pysrt-rs (Rust) against pure-Python pysrt.
Measures parsing, shifting, and serialization speeds over large subtitle datasets.
"""
import sys
import os
import time

# Import Rust extension ('libsrt')
import libsrt as rust_pysrt

# Import pure-Python implementation ('pysrt')
import pysrt as py_pysrt

def generate_srt_data(num_items=1000):
    lines = []
    for i in range(1, num_items + 1):
        s = i * 2
        e = s + 1
        lines.append(f"{i}\n00:00:{s:02d},000 --> 00:00:{e:02d},500\n<i>Subtitle line {i}</i> with some text.\n")
    return "\n".join(lines)

def bench_func(name, func, iterations=1000, warmup=50):
    for _ in range(warmup):
        func()
    start = time.perf_counter()
    for _ in range(iterations):
        func()
    elapsed = time.perf_counter() - start
    return elapsed / iterations

def main():
    print("Generating benchmark dataset (1,000 subtitle items)...")
    srt_text = generate_srt_data(1000)
    
    print("\n=== RUNNING BENCHMARKS ===")
    
    # 1. Parsing (Synthetic 1000 items)
    rust_parse_t = bench_func("Rust Parse", lambda: rust_pysrt.from_string(srt_text), iterations=1000, warmup=50)
    py_parse_t = bench_func("Py Parse", lambda: py_pysrt.from_string(srt_text), iterations=1000, warmup=50)
    parse_speedup = py_parse_t / rust_parse_t

    # 2. Parsing (Real Movie File: tests/static/utf-8.srt - 1,332 items)
    real_srt_path = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "tests", "static", "utf-8.srt"))
    with open(real_srt_path, "r", encoding="utf-8") as f:
        real_srt_text = f.read()
    rust_real_t = bench_func("Rust Parse (Movie)", lambda: rust_pysrt.from_string(real_srt_text), iterations=1000, warmup=50)
    py_real_t = bench_func("Py Parse (Movie)", lambda: py_pysrt.from_string(real_srt_text), iterations=1000, warmup=50)
    real_speedup = py_real_t / rust_real_t
    
    # 3. Shifting
    r_file = rust_pysrt.from_string(srt_text)
    p_file = py_pysrt.from_string(srt_text)
    rust_shift_t = bench_func("Rust Shift", lambda: r_file.shift(seconds=2, milliseconds=500), iterations=2000, warmup=100)
    py_shift_t = bench_func("Py Shift", lambda: p_file.shift(seconds=2, milliseconds=500), iterations=2000, warmup=100)
    shift_speedup = py_shift_t / rust_shift_t
    
    # 4. Serialization
    rust_ser_t = bench_func("Rust Serialize", lambda: r_file.text, iterations=2000, warmup=100)
    py_ser_t = bench_func("Py Serialize", lambda: p_file.text, iterations=2000, warmup=100)
    ser_speedup = py_ser_t / rust_ser_t
    def fmt_time(seconds):
        if seconds >= 0.0001:
            return f"{seconds * 1000:>10.2f} ms"
        else:
            return f"{seconds * 1e6:>10.2f} µs"

    print("\n" + "="*68)
    print(f"{'Operation':<22} | {'Python (pysrt)':<15} | {'Rust (pysrt-rs)':<15} | {'Speedup':<10}")
    print("-" * 68)
    print(f"{'Parse (1000 subs)':<22} | {fmt_time(py_parse_t)} | {fmt_time(rust_parse_t)} | {parse_speedup:>7.1f}x")
    print(f"{'Parse (Movie - 1332)':<22} | {fmt_time(py_real_t)} | {fmt_time(rust_real_t)} | {real_speedup:>7.1f}x")
    print(f"{'Shift (1000 subs)':<22} | {fmt_time(py_shift_t)} | {fmt_time(rust_shift_t)} | {shift_speedup:>7.1f}x")
    print(f"{'Serialize (text)':<22} | {fmt_time(py_ser_t)} | {fmt_time(rust_ser_t)} | {ser_speedup:>7.1f}x")
    print("="*68)
    
if __name__ == "__main__":
    main()
