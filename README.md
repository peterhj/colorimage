# colorimage

`colorimage` is a Rust library for loading RGB image formats (JPEG, PNG) for
in-memory usage while paying particular attention to colorspace metadata.
Specifically, `colorimage` modifies and wraps the image decoder implementations
from gecko (https://github.com/mozilla/gecko-dev) which are colorspace-aware.
Unlike the original gecko image decoders, `colorspace` mainly tries to ensure
conversion to sRGB for in-memory processing.

Because `colorimage` is in large part derived from gecko, `colorimage` is
licensed according to the MPL.

Currently, the gecko-derived parts originate from gecko-dev commit
0b6ee32aeedd70891d890dd252fd31b769237e2b.
