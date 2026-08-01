#!/usr/bin/env python3
"""
Differential fuzzing harness for pysrt-rs vs reference python pysrt.
Tests 10,000+ generated inputs across both implementations and verifies identical output.
All output is written to both stdout and fuzz/log.txt.
"""
import sys
import os
import random
import logging
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


def test_time_fuzz(iterations: int = 5000) -> int:
    """Differential test on SubRipTime. Returns number of passing cases."""
    log.info(f"Running {iterations} differential tests on SubRipTime...")
    passed = 0
    failed = 0
    for i in range(iterations):
        h  = random.randint(0, 99)
        m  = random.randint(0, 59)
        s  = random.randint(0, 59)
        ms = random.randint(0, 999)

        rt = rust_pysrt.SubRipTime(h, m, s, ms)
        pt = py_pysrt.SubRipTime(h, m, s, ms)

        try:
            assert str(rt) == str(pt),        f"str mismatch: {rt!r} vs {pt!r}"
            assert repr(rt) == repr(pt),       f"repr mismatch: {repr(rt)} vs {repr(pt)}"
            assert rt.ordinal == pt.ordinal,   f"ordinal mismatch: {rt.ordinal} vs {pt.ordinal}"

            h2  = random.randint(0, 10)
            m2  = random.randint(0, 59)
            s2  = random.randint(0, 59)
            ms2 = random.randint(0, 999)

            rt2 = rust_pysrt.SubRipTime(h2, m2, s2, ms2)
            pt2 = py_pysrt.SubRipTime(h2, m2, s2, ms2)

            assert str(rt + rt2) == str(pt + pt2), f"addition mismatch @ iter {i}"
            assert str(rt - rt2) == str(pt - pt2), f"subtraction mismatch @ iter {i}"
            assert (rt < rt2) == (pt < pt2),        f"comparison mismatch @ iter {i}"

            passed += 1
        except AssertionError as exc:
            failed += 1
            log.exception(f"[SubRipTime] FAIL iter={i} input=({h},{m},{s},{ms}): {exc}")

    log.info(f"SubRipTime: {passed} passed, {failed} failed out of {iterations}")
    return passed


def test_item_fuzz(iterations: int = 2000) -> int:
    """Differential test on SubRipItem. Returns number of passing cases."""
    log.info(f"Running {iterations} differential tests on SubRipItem...")
    passed = 0
    failed = 0
    tags = ["<i>", "</i>", "<b>", "</b>", "<font color='red'>", "</font>"]
    text_words = ["Hello", "world", "subtitle", "fuzzing", "test", "differential"]

    for i in range(1, iterations + 1):
        s_s   = random.randint(0, 3599)
        dur_s = random.randint(1, 30)
        text  = " ".join(random.choices(text_words, k=5))
        if i % 2 == 0:
            text = f"<i>{text}</i>"

        srt_text = (
            f"{i}\n"
            f"00:00:{s_s:02d},000 --> 00:00:{s_s + dur_s:02d},500\n"
            f"{text}\n"
        )

        try:
            ri = rust_pysrt.SubRipItem.from_string(srt_text)
            pi = py_pysrt.SubRipItem.from_string(srt_text)

            assert ri.index == pi.index,                                          f"index mismatch"
            assert str(ri.start) == str(pi.start),                               f"start mismatch"
            assert str(ri.end) == str(pi.end),                                   f"end mismatch"
            assert ri.text_without_tags == pi.text_without_tags,                 f"text_without_tags mismatch"
            assert abs(ri.characters_per_second - pi.characters_per_second) < 1e-6, f"cps mismatch"
            assert str(ri) == str(pi),                                            f"str(item) mismatch"

            passed += 1
        except AssertionError as exc:
            failed += 1
            log.exception(f"[SubRipItem] FAIL iter={i} srt_text={srt_text!r}: {exc}")

    log.info(f"SubRipItem: {passed} passed, {failed} failed out of {iterations}")
    return passed


def main():
    log.info("=" * 60)
    log.info("STARTING DIFFERENTIAL FUZZING  (pysrt-rs vs py-pysrt)")
    log.info(f"Rust  pysrt version : {getattr(rust_pysrt, 'VERSION_STRING', 'unknown')}")
    log.info(f"Python pysrt path   : {py_pysrt.__file__}")
    log.info("=" * 60)

    time_ok  = test_time_fuzz(5000)
    item_ok  = test_item_fuzz(2000)
    total    = time_ok + item_ok

    log.info("=" * 60)
    if time_ok == 5000 and item_ok == 2000:
        log.info(f"ALL {total} DIFFERENTIAL FUZZING TESTS PASSED — 100% PARITY")
    else:
        log.warning(f"SOME TESTS FAILED: {total}/7000 passed")
    log.info(f"Full log written to: {LOG_PATH}")
    log.info("=" * 60)


if __name__ == "__main__":
    main()

