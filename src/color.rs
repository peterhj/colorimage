use ffi::gckimg::*;

use std::cell::{RefCell};
use std::mem::{zeroed};

thread_local! {
  pub static COLOR_MGMT: RefCell<ColorMgmt> = RefCell::new(ColorMgmt::default());
}

pub struct ColorMgmt {
  pub ctx:  ColorMgmtCtx,
}

impl Drop for ColorMgmt {
  fn drop(&mut self) {
    unsafe { gckimg_color_mgmt_cleanup(&mut self.ctx as *mut _) };
  }
}

impl Default for ColorMgmt {
  fn default() -> Self {
    let mut ctx: ColorMgmtCtx = unsafe { zeroed() };
    unsafe { gckimg_color_mgmt_init_default(&mut ctx as *mut _) };
    ColorMgmt{ctx:  ctx}
  }
}
