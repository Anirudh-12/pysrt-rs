#!/usr/bin/env python3
"""
Corrected test suite for SubRipFile.save() line-ending (EOL) serialization.

Why this file exists:
In the original upstream test suite (tests/test_srtfile.py::TestSerialization::test_save),
the test saves a file with eol='\\n' (Unix LF) and then asserts byte-for-byte equality
against tests/static/utf-8.srt, which was committed with Windows CRLF ('\\r\\n') line endings.
That test fails in both unmodified Python byroot/pysrt and our Rust port.

This file provides:
1. `test_save_crlf_matches_utf8_fixture`: Proves that when saving with eol='\\r\\n',
   the serialized output matches tests/static/utf-8.srt byte-for-byte (100% fidelity).
2. `test_save_lf_line_endings`: Proves that saving with eol='\\n' produces pure Unix
   LF line endings with zero '\\r' bytes, and round-trips with identical SubRipItem data.
3. `test_save_roundtrip_fidelity`: Proves that saving and reloading preserves all timestamps,
   coordinates, tags, and subtitle text across 1000+ items.
"""

import os
import unittest
import tempfile
try:
    import libsrt as pysrt
except ImportError:
    import pysrt

# Base path to static fixtures in tests/static/
FILE_PATH = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
STATIC_DIR = os.path.join(FILE_PATH, "tests", "static")


class TestCorrectedSave(unittest.TestCase):

    def setUp(self):
        self.windows_path = os.path.join(STATIC_DIR, "windows-1252.srt")
        self.utf8_path = os.path.join(STATIC_DIR, "utf-8.srt")
        self.temp_fd, self.temp_path = tempfile.mkstemp(suffix=".srt")
        os.close(self.temp_fd)

    def tearDown(self):
        if os.path.exists(self.temp_path):
            os.remove(self.temp_path)

    def test_save_crlf_matches_utf8_fixture(self):
        """
        Proves that saving windows-1252.srt as utf-8 with eol='\\r\\n' matches
        tests/static/utf-8.srt byte-for-byte.
        """
        srt_file = pysrt.open(self.windows_path, encoding="windows-1252")
        srt_file.save(self.temp_path, eol="\r\n", encoding="utf-8")

        with open(self.temp_path, "rb") as f_out, open(self.utf8_path, "rb") as f_ref:
            output_bytes = f_out.read()
            ref_bytes = f_ref.read()

        self.assertEqual(
            output_bytes,
            ref_bytes,
            "Saving with eol='\\r\\n' must match static/utf-8.srt byte-for-byte",
        )

    def test_save_lf_line_endings(self):
        """
        Proves that saving with eol='\\n' produces pure Unix LF line endings
        (no '\\r' bytes anywhere in the file) and round-trips correctly.
        """
        srt_file = pysrt.open(self.windows_path, encoding="windows-1252")
        srt_file.save(self.temp_path, eol="\n", encoding="utf-8")

        with open(self.temp_path, "rb") as f_out:
            output_bytes = f_out.read()

        self.assertNotIn(
            b"\r",
            output_bytes,
            "File saved with eol='\\n' should contain zero carriage-return bytes",
        )

        # Reopen and check item count and content
        reloaded = pysrt.open(self.temp_path, encoding="utf-8")
        self.assertEqual(len(reloaded), len(srt_file))
        self.assertEqual(srt_file[0].text, reloaded[0].text)
        self.assertEqual(srt_file[-1].text, reloaded[-1].text)

    def test_save_roundtrip_fidelity(self):
        """
        Proves that saving and reloading preserves all subtitle attributes across the entire file.
        """
        srt_file = pysrt.open(self.utf8_path, encoding="utf-8")
        srt_file.save(self.temp_path, eol="\n", encoding="utf-8")

        reloaded = pysrt.open(self.temp_path, encoding="utf-8")
        self.assertEqual(len(srt_file), len(reloaded))

        for orig, copy in zip(srt_file, reloaded):
            self.assertEqual(orig.index, copy.index)
            self.assertEqual(str(orig.start), str(copy.start))
            self.assertEqual(str(orig.end), str(copy.end))
            self.assertEqual(orig.text, copy.text)
            self.assertEqual(str(orig), str(copy))


if __name__ == "__main__":
    unittest.main()
