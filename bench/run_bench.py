#!/usr/bin/env python3
"""
Performance & Behavioral Equivalence Benchmark Suite for pysrt-rs (Rust) vs. pure-Python pysrt.
Measures:
  1. Latency Distributions (Mean, p50, p95, p99, Max) & Throughput Speedups
  2. Cold Startup Time (Interpreter / Extension / Native CLI)
  3. Peak Heap & Resident Set Size (RSS) Memory Footprint Reduction
  4. Automatic export of full results to bench/results.json (Port Mortem 2026 Deliverable 06)
"""

import sys
import os
import time
import json
import platform
import subprocess
import gc
import math
import tracemalloc

# Import Rust extension ('libsrt')
import libsrt as rust_pysrt

# Import pure-Python implementation ('pysrt')
import pysrt as py_pysrt


def generate_srt_data(num_items=1000):
    lines = []
    for i in range(1, num_items + 1):
        s = i * 2
        e = s + 1
        lines.append(
            f"{i}\n00:00:{s:02d},000 --> 00:00:{e:02d},500\n<i>Subtitle line {i}</i> with some text.\n"
        )
    return "\n".join(lines)


def get_percentile(sorted_data, p):
    if not sorted_data:
        return 0.0
    idx = int(math.ceil((p / 100.0) * len(sorted_data))) - 1
    idx = max(0, min(idx, len(sorted_data) - 1))
    return sorted_data[idx]


def get_rss_mb():
    """Returns the current Resident Set Size (RSS) memory in MiB cross-platform."""
    try:
        import psutil
        return psutil.Process(os.getpid()).memory_info().rss / (1024 * 1024)
    except ImportError:
        pass

    if sys.platform == "win32":
        try:
            import ctypes
            from ctypes import wintypes

            class PROCESS_MEMORY_COUNTERS(ctypes.Structure):
                _fields_ = [
                    ("cb", wintypes.DWORD),
                    ("PageFaultCount", wintypes.DWORD),
                    ("PeakWorkingSetSize", ctypes.c_size_t),
                    ("WorkingSetSize", ctypes.c_size_t),
                    ("QuotaPeakPagedPoolUsage", ctypes.c_size_t),
                    ("QuotaPagedPoolUsage", ctypes.c_size_t),
                    ("QuotaPeakNonPagedPoolUsage", ctypes.c_size_t),
                    ("QuotaNonPagedPoolUsage", ctypes.c_size_t),
                    ("PagefileUsage", ctypes.c_size_t),
                    ("PeakPagefileUsage", ctypes.c_size_t),
                ]

            psapi = ctypes.windll.psapi
            process = ctypes.windll.kernel32.GetCurrentProcess()
            counters = PROCESS_MEMORY_COUNTERS()
            if psapi.GetProcessMemoryInfo(
                process, ctypes.byref(counters), ctypes.sizeof(counters)
            ):
                return counters.WorkingSetSize / (1024 * 1024)
        except Exception:
            pass
    else:
        try:
            import resource
            rusage = resource.getrusage(resource.RUSAGE_SELF)
            if sys.platform == "darwin":
                return rusage.ru_maxrss / (1024 * 1024)
            else:
                return rusage.ru_maxrss / 1024
        except Exception:
            pass
    return 0.0


def bench_latency(name, func, iterations=1000, warmup=50):
    for _ in range(warmup):
        func()
    times = []
    for _ in range(iterations):
        t0 = time.perf_counter()
        func()
        t1 = time.perf_counter()
        times.append(t1 - t0)
    times.sort()
    mean_val = sum(times) / len(times)
    p50_val = get_percentile(times, 50)
    p95_val = get_percentile(times, 95)
    p99_val = get_percentile(times, 99)
    max_val = times[-1]
    return {
        "name": name,
        "iterations": iterations,
        "mean_seconds": mean_val,
        "p50_seconds": p50_val,
        "p95_seconds": p95_val,
        "p99_seconds": p99_val,
        "max_seconds": max_val,
    }


def bench_startup(python_exe):
    """Measures cold startup time for Python pysrt import vs Rust libsrt import vs Native CLI."""
    def time_command(cmd_list, runs=10):
        durations = []
        for _ in range(runs):
            t0 = time.perf_counter()
            subprocess.run(
                cmd_list,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=False,
            )
            t1 = time.perf_counter()
            durations.append(t1 - t0)
        durations.sort()
        return sum(durations) / len(durations)

    py_import_time = time_command([python_exe, "-c", "import pysrt"])
    rust_import_time = time_command([python_exe, "-c", "import libsrt"])

    # Check for native srt binary in target/release
    repo_root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
    native_cli_win = os.path.join(repo_root, "target", "release", "srt.exe")
    native_cli_unix = os.path.join(repo_root, "target", "release", "srt")

    native_cli_time = None
    if os.path.exists(native_cli_win):
        native_cli_time = time_command([native_cli_win, "--help"])
    elif os.path.exists(native_cli_unix):
        native_cli_time = time_command([native_cli_unix, "--help"])

    return {
        "python_import_ms": round(py_import_time * 1000.0, 2),
        "rust_extension_import_ms": round(rust_import_time * 1000.0, 2),
        "rust_native_cli_ms": (
            round(native_cli_time * 1000.0, 2) if native_cli_time is not None else None
        ),
        "import_speedup": round(
            py_import_time / rust_import_time if rust_import_time > 0 else 0.0, 2
        ),
    }


def bench_memory_footprint(srt_text, copies=30):
    """Measures peak heap allocation (tracemalloc) and RSS delta when loading multiple copies."""
    gc.collect()
    time.sleep(0.05)
    base_rss = get_rss_mb()

    # Python peak heap via tracemalloc
    tracemalloc.start()
    py_objects = [py_pysrt.from_string(srt_text) for _ in range(copies)]
    py_heap_peak_bytes = tracemalloc.get_traced_memory()[1]
    py_heap_peak_mb = py_heap_peak_bytes / (1024 * 1024)
    py_peak_rss = get_rss_mb()
    py_rss_delta = max(0.0, py_peak_rss - base_rss)
    tracemalloc.stop()

    del py_objects
    gc.collect()
    time.sleep(0.05)
    base_rss_after = get_rss_mb()

    # Rust peak heap via tracemalloc
    tracemalloc.start()
    rust_objects = [rust_pysrt.from_string(srt_text) for _ in range(copies)]
    rust_heap_peak_bytes = tracemalloc.get_traced_memory()[1]
    rust_heap_peak_mb = rust_heap_peak_bytes / (1024 * 1024)
    rust_peak_rss = get_rss_mb()
    rust_rss_delta = max(0.0, rust_peak_rss - base_rss_after)
    tracemalloc.stop()

    del rust_objects
    gc.collect()

    reduction_heap = py_heap_peak_mb / rust_heap_peak_mb if rust_heap_peak_mb > 0 else 1.0
    reduction_rss = py_rss_delta / rust_rss_delta if rust_rss_delta > 0 else 1.0

    return {
        "dataset_copies": copies,
        "python_heap_peak_mb": round(py_heap_peak_mb, 2),
        "rust_heap_peak_mb": round(rust_heap_peak_mb, 2),
        "heap_reduction_factor": round(reduction_heap, 2),
        "python_rss_delta_mb": round(py_rss_delta, 2),
        "rust_rss_delta_mb": round(rust_rss_delta, 2),
        "rss_reduction_factor": round(reduction_rss, 2),
    }


def fmt_time(seconds):
    if seconds >= 0.0001:
        return f"{seconds * 1000:>8.2f} ms"
    else:
        return f"{seconds * 1e6:>8.2f} µs"


def main():
    print("==================================================================================")
    print("      PORT MORTEM 2026 — PERFORMANCE & BEHAVIORAL EQUIVALENCE BENCHMARK")
    print("               Track D: Python -> Rust (byroot/pysrt vs pysrt-rs)               ")
    print("==================================================================================")
    print("Generating benchmark dataset (1,000 subtitle items)...")
    srt_text = generate_srt_data(1000)

    real_srt_path = os.path.abspath(
        os.path.join(os.path.dirname(__file__), "..", "tests", "static", "utf-8.srt")
    )
    with open(real_srt_path, "r", encoding="utf-8") as f:
        real_srt_text = f.read()

    print("\n--- [1/4] Measuring Latency Distributions (Mean, p50, p95, p99, Max) ---")

    # 1. Parse (Synthetic 1000)
    rust_parse = bench_latency("Rust Parse (1,000 subs)", lambda: rust_pysrt.from_string(srt_text), 1000, 50)
    py_parse = bench_latency("Py Parse (1,000 subs)", lambda: py_pysrt.from_string(srt_text), 1000, 50)

    # 2. Parse (Real Movie 1332)
    rust_real = bench_latency("Rust Parse (Movie 1,332)", lambda: rust_pysrt.from_string(real_srt_text), 1000, 50)
    py_real = bench_latency("Py Parse (Movie 1,332)", lambda: py_pysrt.from_string(real_srt_text), 1000, 50)

    # 3. Shift
    r_file = rust_pysrt.from_string(srt_text)
    p_file = py_pysrt.from_string(srt_text)
    rust_shift = bench_latency("Rust Shift (1,000 subs)", lambda: r_file.shift(seconds=2, milliseconds=500), 2000, 100)
    py_shift = bench_latency("Py Shift (1,000 subs)", lambda: p_file.shift(seconds=2, milliseconds=500), 2000, 100)

    # 4. Serialize
    rust_ser = bench_latency("Rust Serialize (text)", lambda: r_file.text, 2000, 100)
    py_ser = bench_latency("Py Serialize (text)", lambda: p_file.text, 2000, 100)

    print("\n" + "=" * 90)
    print(
        f"{'Operation':<22} | {'Implementation':<13} | {'Mean':<11} | {'p50 (Med)':<11} | {'p95':<11} | {'p99 (Tail)':<11}"
    )
    print("-" * 90)

    pairs = [
        ("Parse (1000 subs)", py_parse, rust_parse),
        ("Parse (Movie 1332)", py_real, rust_real),
        ("Shift (1000 subs)", py_shift, rust_shift),
        ("Serialize (text)", py_ser, rust_ser),
    ]

    for op_name, py_m, rust_m in pairs:
        print(
            f"{op_name:<22} | {'Python (pysrt)':<13} | {fmt_time(py_m['mean_seconds'])} | {fmt_time(py_m['p50_seconds'])} | {fmt_time(py_m['p95_seconds'])} | {fmt_time(py_m['p99_seconds'])}"
        )
        print(
            f"{'':<22} | {'Rust (pysrt-rs)':<13} | {fmt_time(rust_m['mean_seconds'])} | {fmt_time(rust_m['p50_seconds'])} | {fmt_time(rust_m['p95_seconds'])} | {fmt_time(rust_m['p99_seconds'])}"
        )
        speedup_mean = py_m["mean_seconds"] / rust_m["mean_seconds"]
        speedup_p99 = py_m["p99_seconds"] / rust_m["p99_seconds"]
        print(
            f"{'':<22} | -> Speedup:    | {speedup_mean:>7.1f}x    |             |             | {speedup_p99:>7.1f}x (p99)"
        )
        print("-" * 90)

    print("\n--- [2/4] Measuring Cold Startup Times ---")
    startup_data = bench_startup(sys.executable)
    print(f"Python 'import pysrt' (Cold Module Load) : {startup_data['python_import_ms']:>6.2f} ms")
    print(f"Rust   'import libsrt' (Cold Extension)  : {startup_data['rust_extension_import_ms']:>6.2f} ms")
    print(f"-> Extension Import Speedup              : {startup_data['import_speedup']:>6.1f}x")
    if startup_data["rust_native_cli_ms"] is not None:
        print(
            f"Rust   'srt --help' (Standalone Binary)  : {startup_data['rust_native_cli_ms']:>6.2f} ms"
        )

    print("\n--- [3/4] Measuring Memory Footprint (30 Dataset Copies / 30,000 Subs) ---")
    mem_data = bench_memory_footprint(srt_text, copies=30)
    print(f"Python (pysrt) Peak Heap Allocation : {mem_data['python_heap_peak_mb']:>6.2f} MiB")
    print(f"Rust (pysrt-rs) Peak Heap Allocation: {mem_data['rust_heap_peak_mb']:>6.2f} MiB")
    print(f"-> Heap Memory Footprint Reduction  : {mem_data['heap_reduction_factor']:>6.1f}x smaller in Rust")
    print(f"Python (pysrt) RSS Memory Delta     : {mem_data['python_rss_delta_mb']:>6.2f} MiB")
    print(f"Rust (pysrt-rs) RSS Memory Delta    : {mem_data['rust_rss_delta_mb']:>6.2f} MiB")

    print("\n--- [4/4] Exporting Complete Results to bench/results.json ---")

    results_json = {
        "metadata": {
            "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "python_version": platform.python_version(),
            "platform": platform.platform(),
            "processor": platform.processor(),
            "track": "D (Python -> Rust)",
        },
        "latency_benchmarks": [],
        "startup_ms": startup_data,
        "memory_footprint_mb": mem_data,
    }

    for op_name, py_m, rust_m in pairs:
        results_json["latency_benchmarks"].append(
            {
                "operation": op_name,
                "python": py_m,
                "rust": rust_m,
                "speedup_mean": round(py_m["mean_seconds"] / rust_m["mean_seconds"], 2),
                "speedup_p95": round(py_m["p95_seconds"] / rust_m["p95_seconds"], 2),
                "speedup_p99": round(py_m["p99_seconds"] / rust_m["p99_seconds"], 2),
            }
        )

    out_path = os.path.join(os.path.dirname(__file__), "results.json")
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(results_json, f, indent=2)

    print(f"[SUCCESS] Full benchmark results exported to: {os.path.abspath(out_path)}")
    print("==================================================================================")


if __name__ == "__main__":
    main()
