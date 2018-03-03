use ::*;
use color::*;
use ffi::gckimg::*;

use std::mem::{transmute, zeroed};

pub struct NSPngDecoder {
  ctx:  NSPngDecoderCtx,
}

impl NSPngDecoder {
  pub fn new(color_mgmt: bool) -> NSPngDecoder {
    let mut ctx: NSPngDecoderCtx = unsafe { zeroed() };
    NSPngDecoder{ctx: ctx}
  }

  pub fn decode<W>(&mut self, buf: &[u8], writer: &mut W) -> Result<(), ()>
  where W: ImageWriter + 'static {
    // TODO: zero out the context memory.
    COLOR_MGMT.with(|cm| {
      let mut cm = cm.borrow_mut();
      unsafe { gckimg_ns_png_init(
          &mut self.ctx as *mut _,
          // TODO: color mgmt option.
          1) };
      unsafe { gckimg_ns_png_decode(
          &mut self.ctx as *mut _,
          &mut cm.ctx as *mut _,
          buf.as_ptr(), buf.len(),
          writer as *mut W as *mut _,
          <W as ImageWriter>::callbacks(),
      ) };
      unsafe { gckimg_ns_png_cleanup(
          &mut self.ctx as *mut _) };
    });
    // TODO
    Err(())
  }
}
