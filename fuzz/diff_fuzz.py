#!/usr/bin/env python3
"""
Differential fuzzing harness for pysrt-rs vs reference python pysrt.
Runs for a continuous duration (e.g., 60 seconds) generating randomized inputs
across both implementations and verifying identical output.
All output is written to both stdout and fuzz/log.txt.
"""

import sys
import os
import random
import logging
import time
from datetime import datetime

# ---------------------------------------------------------------------------
# Logging setup — write to log.txt (next to this script) AND to stdout
# ---------------------------------------------------------------------------
LOG_PATH = os.path.join(os.path.dirname(__file__), "log.txt")

logging.basicConfig(
    level=logging.DEBUG,
    format="%(asctime)s [%(levelname)s] %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S",
    handlers=[
        logging.FileHandler(LOG_PATH, mode="w", encoding="utf-8"),
        logging.StreamHandler(sys.stdout),
    ],
)
log = logging.getLogger("diff_fuzz")

# ---------------------------------------------------------------------------
# Import Rust extension (installed as 'libsrt' by maturin develop)
# ---------------------------------------------------------------------------
import libsrt as rust_pysrt

# Import normal pure-Python reference pysrt
import pysrt as py_pysrt


def fuzz_subrip_time():
    """Single differential test iteration for SubRipTime."""
    h = random.randint(0, 99)
    m = random.randint(0, 59)
    s = random.randint(0, 59)
    ms = random.randint(0, 999)

    rt = rust_pysrt.SubRipTime(h, m, s, ms)
    pt = py_pysrt.SubRipTime(h, m, s, ms)

    assert str(rt) == str(pt), f"str mismatch: {rt!r} vs {pt!r}"
    assert repr(rt) == repr(pt), f"repr mismatch: {repr(rt)} vs {repr(pt)}"
    assert rt.ordinal == pt.ordinal, f"ordinal mismatch: {rt.ordinal} vs {pt.ordinal}"

    h2 = random.randint(0, 10)
    m2 = random.randint(0, 59)
    s2 = random.randint(0, 59)
    ms2 = random.randint(0, 999)

    rt2 = rust_pysrt.SubRipTime(h2, m2, s2, ms2)
    pt2 = py_pysrt.SubRipTime(h2, m2, s2, ms2)

    assert str(rt + rt2) == str(pt + pt2), f"addition mismatch"
    assert str(rt - rt2) == str(pt - pt2), f"subtraction mismatch"
    assert (rt < rt2) == (pt < pt2), f"comparison mismatch"


def fuzz_subrip_item(i):
    """Single differential test iteration for SubRipItem."""
    tags = ["<i>", "</i>", "<b>", "</b>", "<font color='red'>", "</font>"]
    text_words = ["Hello", "world", "subtitle", "fuzzing", "test", "differential"]

    s_s = random.randint(0, 3599)
    dur_s = random.randint(1, 30)
    text = " ".join(random.choices(text_words, k=5))
    if i % 2 == 0:
        text = f"<i>{text}</i>"

    srt_text = f"{i}\n00:00:{s_s:02d},000 --> 00:00:{s_s + dur_s:02d},500\n{text}\n"

    ri = rust_pysrt.SubRipItem.from_string(srt_text)
    pi = py_pysrt.SubRipItem.from_string(srt_text)

    assert ri.index == pi.index, f"index mismatch"
    assert str(ri.start) == str(pi.start), f"start mismatch"
    assert str(ri.end) == str(pi.end), f"end mismatch"
    assert ri.text_without_tags == pi.text_without_tags, f"text_without_tags mismatch"
    assert abs(ri.characters_per_second - pi.characters_per_second) < 1e-6, f"cps mismatch"
    assert str(ri) == str(pi), f"str(item) mismatch"


def run_continuous_fuzz(duration_seconds: int = 60):
    """Runs differential fuzzing continuously for the specified duration."""
    log.info(f"Running differential fuzzer continuously for {duration_seconds} seconds...")
    passed = 0
    failed = 0
    start_time = time.time()
    
    # Run loop until exactly duration_seconds have passed
    while True:
        elapsed = time.time() - start_time
        if elapsed >= duration_seconds:
            break

        # Randomly choose between fuzzing time operations vs item parsing
        choice = random.choice(["time", "item"])
        try:
            if choice == "time":
                fuzz_subrip_time()
            else:
                fuzz_subrip_item(passed + 1)
            passed += 1
            
            # Periodically log progress every 10,000 iterations to avoid stdout flooding
            if passed % 10000 == 0:
                log.info(f"Elapsed: {elapsed:.1f}s / {duration_seconds}s | Passed: {passed}")
                
        except AssertionError as exc:
            failed += 1
            log.exception(f"FAIL at iteration {passed} ({choice}): {exc}")

    log.info(f"Finished continuous {duration_seconds}-second run.")
    log.info(f"Total passed: {passed}")
    log.info(f"Total failed: {failed}")
    return passed, failed


def main():
    log.info("=" * 60)
    log.info("STARTING CONTINUOUS DIFFERENTIAL FUZZING (pysrt-rs vs py-pysrt)")
    log.info(
        f"Rust  pysrt version : {getattr(rust_pysrt, 'VERSION_STRING', 'unknown')}"
    )
    log.info(f"Python pysrt path   : {py_pysrt.__file__}")
    log.info("=" * 60)

    # Hackathon requirement: 60 continuous seconds
    passed, failed = run_continuous_fuzz(duration_seconds=60)

    log.info("=" * 60)
    if failed == 0:
        log.info(f"ALL {passed} DIFFERENTIAL FUZZING TESTS PASSED IN 60 SECONDS — 100% PARITY")
    else:
        log.warning(f"SOME TESTS FAILED: {passed} passed, {failed} failed")
    log.info(f"Full log written to: {LOG_PATH}")
    log.info("=" * 60)


if __name__ == "__main__":
    main()
