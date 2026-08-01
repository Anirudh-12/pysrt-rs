#!/usr/bin/env python3
"""
Performance benchmark comparing pysrt-rs (Rust) against pure-Python pysrt.
Measures parsing, shifting, and serialization speeds over large subtitle datasets.
"""
import sys
import os
import time

# Import Rust extension
import pysrt as rust_pysrt

# Clear sys.modules so Python imports the reference pure-Python implementation
for key in list(sys.modules.keys()):
    if key.startswith("pysrt"):
        sys.modules.pop(key)

# Add reference folder and import pure-Python implementation
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "reference_pysrt")))
import pysrt as py_pysrt

def generate_srt_data(num_items=1000):
    lines = []
    for i in range(1, num_items + 1):
        s = i * 2
        e = s + 1
        lines.append(f"{i}\n00:00:{s:02d},000 --> 00:00:{e:02d},500\n<i>Subtitle line {i}</i> with some text.\n")
    return "\n".join(lines)

def bench_func(name, func, iterations=50):
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
    rust_parse_t = bench_func("Rust Parse", lambda: rust_pysrt.from_string(srt_text), iterations=30)
    py_parse_t = bench_func("Py Parse", lambda: py_pysrt.from_string(srt_text), iterations=30)
    parse_speedup = py_parse_t / rust_parse_t

    # 2. Parsing (Real Movie File: tests/static/utf-8.srt - 1,332 items)
    real_srt_path = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "tests", "static", "utf-8.srt"))
    with open(real_srt_path, "r", encoding="utf-8") as f:
        real_srt_text = f.read()
    rust_real_t = bench_func("Rust Parse (Movie)", lambda: rust_pysrt.from_string(real_srt_text), iterations=30)
    py_real_t = bench_func("Py Parse (Movie)", lambda: py_pysrt.from_string(real_srt_text), iterations=30)
    real_speedup = py_real_t / rust_real_t
    
    # 3. Shifting
    r_file = rust_pysrt.from_string(srt_text)
    p_file = py_pysrt.from_string(srt_text)
    rust_shift_t = bench_func("Rust Shift", lambda: r_file.shift(seconds=2, milliseconds=500), iterations=100)
    py_shift_t = bench_func("Py Shift", lambda: p_file.shift(seconds=2, milliseconds=500), iterations=100)
    shift_speedup = py_shift_t / rust_shift_t
    
    # 4. Serialization
    rust_ser_t = bench_func("Rust Serialize", lambda: r_file.text, iterations=50)
    py_ser_t = bench_func("Py Serialize", lambda: p_file.text, iterations=50)
    ser_speedup = py_ser_t / rust_ser_t
    
    print("\n" + "="*68)
    print(f"{'Operation':<22} | {'Python (pysrt)':<15} | {'Rust (pysrt-rs)':<15} | {'Speedup':<10}")
    print("-" * 68)
    print(f"{'Parse (1000 subs)':<22} | {py_parse_t*1000:>10.2f} ms | {rust_parse_t*1000:>10.2f} ms | {parse_speedup:>7.1f}x")
    print(f"{'Parse (Movie - 1332)':<22} | {py_real_t*1000:>10.2f} ms | {rust_real_t*1000:>10.2f} ms | {real_speedup:>7.1f}x")
    print(f"{'Shift (1000 subs)':<22} | {py_shift_t*1000:>10.2f} ms | {rust_shift_t*1000:>10.2f} ms | {shift_speedup:>7.1f}x")
    print(f"{'Serialize (text)':<22} | {py_ser_t*1000:>10.2f} ms | {rust_ser_t*1000:>10.2f} ms | {ser_speedup:>7.1f}x")
    print("="*68)
    
if __name__ == "__main__":
    main()
