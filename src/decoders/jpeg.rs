use ::*;
use color::*;
use ffi::gckimg::*;

use std::mem::{zeroed};

pub struct NSJpegDecoder {
  ctx:  NSJpegDecoderCtx,
}

impl NSJpegDecoder {
  pub fn new(color_mgmt: bool) -> NSJpegDecoder {
    let mut ctx: NSJpegDecoderCtx = unsafe { zeroed() };
    NSJpegDecoder{ctx: ctx}
  }

  pub fn decode<W>(&mut self, buf: &[u8], writer: &mut W) -> Result<(), ()>
  where W: ImageWriter {
    COLOR_MGMT.with(|cm| {
      let mut cm = cm.borrow_mut();
      unsafe { gckimg_ns_jpeg_init(
          &mut self.ctx as *mut _,
          // TODO: color mgmt option.
          1) };
      unsafe { gckimg_ns_jpeg_decode(
          &mut self.ctx as *mut _,
          &mut cm.ctx as *mut _,
          buf.as_ptr(), buf.len(),
          (writer as *mut W) as *mut _,
          <W as ImageWriter>::callbacks(),
      ) };
      unsafe { gckimg_ns_jpeg_cleanup(
          &mut self.ctx as *mut _) };
    });
    // TODO
    Err(())
  }
}
