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


def pytest_addoption(parser):
    parser.addoption(
        "--original",
        action="store_true",
        default=False,
        help="Run original unmodified upstream test suite (tests/original)",
    )
    parser.addoption(
        "--all-tests",
        action="store_true",
        default=False,
        help="Run both fixed and original test suites",
    )


def pytest_configure(config):
    if config.getoption("--original"):
        config.args = ["tests/original"]
    elif config.getoption("--all-tests"):
        config.args = ["tests/fixed", "tests/original"]
