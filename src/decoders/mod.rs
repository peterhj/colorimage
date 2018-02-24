use ffi::wrapped::{RasterWriterCallbacks};

pub mod jpeg;
pub mod png;

pub trait RasterWriter {
  fn callbacks() -> RasterWriterCallbacks;
}
