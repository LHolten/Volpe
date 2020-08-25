from sys import version_info
import time

# use nanosecond perf_counter if available
if version_info >= (3, 7, 0):
    perf_counter = time.perf_counter_ns
    TICKS_IN_SEC = 1_000_000_000
else:
    perf_counter = time.perf_counter
    TICKS_IN_SEC = 1


def is_ascii(text):
    """Check that all characters are valid ascii characters."""
    if version_info >= (3, 7, 0):
        return text.isascii()
    else:
        return all(ord(char) < 128 for char in text)
