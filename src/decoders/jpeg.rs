use decoders::*;
use ffi::gckimg::*;

use std::mem::{transmute, zeroed};

pub struct NSJpegDecoder {
  ctx:  NSJpegDecoderCtx,
}

impl Drop for NSJpegDecoder {
  fn drop(&mut self) {
    unsafe { gckimg_ns_jpeg_cleanup(&mut self.ctx as *mut _) };
  }
}

impl NSJpegDecoder {
  pub fn new(color_mgmt: bool) -> NSJpegDecoder {
    let mut ctx: NSJpegDecoderCtx = unsafe { zeroed() };
    unsafe { gckimg_ns_jpeg_init(&mut ctx as *mut _, if color_mgmt { 1 } else { 0 }) };
    NSJpegDecoder{ctx: ctx}
  }

  pub fn decode<W>(&mut self, buf: &[u8], writer: &mut W) -> Result<(), (Option<()>, ())>
  where W: ImageWriter {
    unsafe { gckimg_ns_jpeg_decode(
        &mut self.ctx as *mut _,
        buf.as_ptr(), buf.len(),
        transmute(writer as *mut _), <W as ImageWriter>::callbacks(),
    ) };
    // TODO
    Err((None, ()))
  }
}
