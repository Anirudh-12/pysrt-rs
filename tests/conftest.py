"""
Pytest configuration for running the original unmodified pysrt test suite against our Rust 'libsrt' extension.
Maps 'libsrt' to 'pysrt' in sys.modules so unmodified tests import our Rust implementation seamlessly.
"""
import sys

try:
    import libsrt
    sys.modules["pysrt"] = libsrt
    if hasattr(libsrt, "compat"):
        sys.modules["pysrt.compat"] = libsrt.compat
except ImportError:
    pass
