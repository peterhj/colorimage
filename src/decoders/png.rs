use decoders::*;
use ffi::gckimg::*;

use std::mem::{transmute, zeroed};

pub struct NSPngDecoder {
  ctx:  NSPngDecoderCtx,
}

impl Drop for NSPngDecoder {
  fn drop(&mut self) {
    unsafe { gckimg_ns_png_cleanup(&mut self.ctx as *mut _) };
  }
}

impl NSPngDecoder {
  pub fn new(color_mgmt: bool) -> NSPngDecoder {
    let mut ctx: NSPngDecoderCtx = unsafe { zeroed() };
    unsafe { gckimg_ns_png_init(&mut ctx as *mut _, if color_mgmt { 1 } else { 0 }) };
    NSPngDecoder{ctx: ctx}
  }

  pub fn decode<W>(&mut self, buf: &[u8], writer: &mut W) -> Result<(), (Option<()>, ())>
  where W: ImageWriter {
    COLOR_MGMT.with(|cm| {
      let mut cm = cm.borrow_mut();
      unsafe { gckimg_ns_png_decode(
          &mut self.ctx as *mut _,
          &mut cm.ctx as *mut _,
          buf.as_ptr(), buf.len(),
          transmute(writer as *mut _), <W as ImageWriter>::callbacks(),
      ) };
    });
    // TODO
    Err((None, ()))
  }
}
