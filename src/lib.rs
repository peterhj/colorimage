extern crate byteorder;
extern crate pillowimage;

use decoders::*;
use decoders::jpeg::*;
use decoders::png::*;
use exif::*;
use ffi::gckimg::*;

use pillowimage::*;

use std::collections::{HashSet};
use std::os::raw::{c_void};
use std::slice::{from_raw_parts};
use std::str::{from_utf8};

pub mod color;
pub mod decoders;
pub mod exif;
pub mod ffi;

pub const BMP_MAGICNUM:     [u8; 2] = [b'B', b'M'];
pub const GIF87A_MAGICNUM:  [u8; 6] = [b'G', b'I', b'F', b'8', b'7', b'a'];
pub const GIF89A_MAGICNUM:  [u8; 6] = [b'G', b'I', b'F', b'8', b'9', b'a'];
pub const JPEG_MAGICNUM:    [u8; 3] = [0xff, 0xd8, 0xff];
pub const PNG_MAGICNUM:     [u8; 8] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
pub const TIFF_MAGICNUM:    [u8; 4] = [b'I', b'I', b'*',    0];
pub const TIFFB_MAGICNUM:   [u8; 4] = [b'M', b'M',    0, b'*'];

pub trait ImageWriter {
  fn callbacks() -> ImageWriterCallbacks;
}

pub unsafe extern "C" fn generic_parse_exif(img_p: *mut c_void, exif_buf: *const u8, exif_size: usize) -> i32 {
  assert!(!exif_buf.is_null());
  let raw_exif = from_raw_parts(exif_buf, exif_size);
  parse_exif(raw_exif).unwrap_or(0)
}

pub struct ColorImage {
  inner:    Option<PILImage>,
  // TODO: exif metadata.
  exif_rot: Option<i32>,
}

pub unsafe extern "C" fn color_image_init_size(img_p: *mut c_void, width: usize, height: usize) {
  //println!("DEBUG: colorimage: init size: width: {} height: {}", width, height);
  assert!(!img_p.is_null());
  let img = &mut *(img_p as *mut ColorImage);
  img.inner = Some(PILImage::new(PILMode::RGB, width as _, height as _));
}

pub unsafe extern "C" fn color_image_write_row_gray(img_p: *mut c_void, row_idx: usize, row_buf: *const u8, row_width: usize) {
  // TODO
  unimplemented!();
}

pub unsafe extern "C" fn color_image_write_row_grayx(img_p: *mut c_void, row_idx: usize, row_buf: *const u8, row_width: usize) {
  // TODO
  unimplemented!();
}

pub unsafe extern "C" fn color_image_write_row_rgb(img_p: *mut c_void, row_idx: usize, row_buf: *const u8, row_width: usize) {
  //println!("DEBUG: colorimage: write row rgb: row idx: {} row width: {}", row_idx, row_width);
  assert!(!img_p.is_null());
  let img = &mut *(img_p as *mut ColorImage);

  assert!(!row_buf.is_null());
  let row_size = 3 * row_width;
  let row = from_raw_parts(row_buf, row_size);

  assert!(img.inner.is_some());
  assert_eq!(row_width, img.inner.as_ref().unwrap().width() as _);
  let mut dst_line = img.inner.as_mut().unwrap().raster_line_mut(row_idx as _);
  for i in 0 .. row_width {
    dst_line[4 * i]     = row[3 * i];
    dst_line[4 * i + 1] = row[3 * i + 1];
    dst_line[4 * i + 2] = row[3 * i + 2];
    /*dst_line[4 * i + 3] = 0xff;*/
  }
}

pub unsafe extern "C" fn color_image_write_row_rgbx(img_p: *mut c_void, row_idx: usize, row_buf: *const u8, row_width: usize) {
  assert!(!img_p.is_null());
  let img = &mut *(img_p as *mut ColorImage);

  assert!(!row_buf.is_null());
  let row_size = 4 * row_width;
  let row = from_raw_parts(row_buf, row_size);

  assert!(img.inner.is_some());
  img.inner.as_mut().unwrap().raster_line_mut(row_idx as _).copy_from_slice(row);
}

pub unsafe extern "C" fn color_image_parse_exif(img_p: *mut c_void, exif_buf: *const u8, exif_size: usize) -> i32 {
  assert!(!img_p.is_null());
  let img = &mut *(img_p as *mut ColorImage);
  assert!(!exif_buf.is_null());
  let raw_exif = from_raw_parts(exif_buf, exif_size);
  let exif_rot = parse_exif(raw_exif).unwrap_or(0);
  if exif_rot >= 1 && exif_rot <= 8 {
    img.exif_rot = Some(exif_rot);
  } else {
    img.exif_rot = None;
  }
  exif_rot
}

impl ImageWriter for ColorImage {
  fn callbacks() -> ImageWriterCallbacks {
    ImageWriterCallbacks{
      init_size:        Some(color_image_init_size),
      write_row_gray:   Some(color_image_write_row_gray),
      write_row_grayx:  Some(color_image_write_row_grayx),
      write_row_rgb:    Some(color_image_write_row_rgb),
      write_row_rgbx:   Some(color_image_write_row_rgbx),
      parse_exif:       Some(color_image_parse_exif),
    }
  }
}

impl ColorImage {
  pub fn new() -> Self {
    ColorImage{
      inner: None,
      exif_rot: None,
    }
  }

  pub fn decode(buf: &[u8]) -> Result<Self, ()> {
    let mut image = ColorImage::new();
    decode_image(buf, &mut image)
      .and_then(|_| match image.exif_rot {
        None => Ok(image),
        Some(exif_rot) => {
          // TODO: rotate or flip the image.
          if exif_rot != 1 {
            println!("WARNING: ColorImage: not handling exif orientation code: {}", exif_rot);
          }
          Ok(image)
        }
      })
  }

  /*pub fn to_vec(&self) -> Vec<u8> {
    assert!(self.inner.is_some());
    self.inner.as_ref().unwrap().to_vec()
  }*/

  pub fn dump_pixels(&self, buf: &mut [u8]) {
    assert!(self.inner.is_some());
    self.inner.as_ref().unwrap().dump_pixels(buf);
  }

  pub fn dump_planes(&self, buf: &mut [u8]) {
    assert!(self.inner.is_some());
    self.inner.as_ref().unwrap().dump_planes(buf);
  }

  pub fn width(&self) -> usize {
    assert!(self.inner.is_some());
    self.inner.as_ref().unwrap().width() as _
  }

  pub fn height(&self) -> usize {
    assert!(self.inner.is_some());
    self.inner.as_ref().unwrap().height() as _
  }

  pub fn channels(&self) -> usize {
    assert!(self.inner.is_some());
    self.inner.as_ref().unwrap().pixel_channels() as _
  }

  pub fn crop(&mut self, x: usize, y: usize, new_width: usize, new_height: usize) {
    // TODO
    unimplemented!();
  }

  pub fn resize(&mut self, new_width: usize, new_height: usize) {
    assert!(self.inner.is_some());
    if new_width == self.width() && new_height == self.height() {
      // Do nothing.
    } else if new_width <= self.width() && new_height <= self.height() {
      self.inner = Some(self.inner.as_ref().unwrap().resample(new_width as _, new_height as _, PILFilter::Box_));
    } else {
      self.inner = Some(self.inner.as_ref().unwrap().resample(new_width as _, new_height as _, PILFilter::Bicubic));
    }
  }
}

pub struct RasterImage {
  width:    usize,
  height:   usize,
  channels: usize,
  data:     Vec<Vec<u8>>,
}

pub unsafe extern "C" fn raster_image_init_size(img_p: *mut c_void, width: usize, height: usize) {
  //println!("DEBUG: RasterImage: init size: {} {} {}", width, height, channels);
  assert!(!img_p.is_null());
  let img = &mut *(img_p as *mut RasterImage);
  let channels = 3;
  img.width = width;
  img.height = height;
  img.channels = channels;
  img.data.clear();
  for _ in 0 .. height {
    let mut row = Vec::with_capacity(width * channels);
    for _ in 0 .. width * channels {
      row.push(0);
    }
    img.data.push(row);
  }
}

pub unsafe extern "C" fn raster_image_write_row_gray(img_p: *mut c_void, row_idx: usize, row_buf: *const u8, row_width: usize) {
  unimplemented!();
}

pub unsafe extern "C" fn raster_image_write_row_grayx(img_p: *mut c_void, row_idx: usize, row_buf: *const u8, row_width: usize) {
  unimplemented!();
}

pub unsafe extern "C" fn raster_image_write_row_rgb(img_p: *mut c_void, row_idx: usize, row_buf: *const u8, row_width: usize) {
  //println!("DEBUG: RasterImage: write row: {} {}", row_idx, row_size);
  assert!(!img_p.is_null());
  let img = &mut *(img_p as *mut RasterImage);

  assert!(!row_buf.is_null());
  let row_size = 3 * row_width;
  let row = from_raw_parts(row_buf, row_size);

  assert_eq!(row_width, img.width);
  for i in 0 .. row_size {
    img.data[row_idx][i] = row[i];
  }
}

pub unsafe extern "C" fn raster_image_write_row_rgbx(img_p: *mut c_void, row_idx: usize, row_buf: *const u8, row_width: usize) {
  //println!("DEBUG: RasterImage: write row: {} {}", row_idx, row_size);
  assert!(!img_p.is_null());
  let img = &mut *(img_p as *mut RasterImage);

  assert!(!row_buf.is_null());
  let row_size = 4 * row_width;
  let row = from_raw_parts(row_buf, row_size);

  assert_eq!(row_width, img.width);
  for x in 0 .. row_width {
    img.data[row_idx][3 * x]     = row[4 * x];
    img.data[row_idx][3 * x + 1] = row[4 * x + 1];
    img.data[row_idx][3 * x + 2] = row[4 * x + 2];
  }
}

impl ImageWriter for RasterImage {
  fn callbacks() -> ImageWriterCallbacks {
    ImageWriterCallbacks{
      init_size:        Some(raster_image_init_size),
      write_row_gray:   Some(raster_image_write_row_gray),
      write_row_grayx:  Some(raster_image_write_row_grayx),
      write_row_rgb:    Some(raster_image_write_row_rgb),
      write_row_rgbx:   Some(raster_image_write_row_rgbx),
      parse_exif:       Some(generic_parse_exif),
    }
  }
}

impl RasterImage {
  pub fn new() -> Self {
    RasterImage{
      width:    0,
      height:   0,
      channels: 0,
      data:     vec![],
    }
  }

  pub fn width(&self) -> usize {
    self.width
  }

  pub fn height(&self) -> usize {
    self.height
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ImageFormat {
  Bmp,
  Gif,
  Jpeg,
  Png,
  Tiff,
}

pub fn guess_image_format_from_magicnum(buf: &[u8]) -> Option<ImageFormat> {
  if &buf[ .. 3] == &JPEG_MAGICNUM {
    return Some(ImageFormat::Jpeg);
  } else if &buf[ .. 8] == &PNG_MAGICNUM {
    return Some(ImageFormat::Png);
  } else if &buf[ .. 6] == &GIF89A_MAGICNUM ||
            &buf[ .. 6] == &GIF87A_MAGICNUM {
    return Some(ImageFormat::Gif);
  } else if &buf[ .. 2] == &BMP_MAGICNUM {
    return Some(ImageFormat::Bmp);
  } else if &buf[ .. 4] == &TIFF_MAGICNUM ||
            &buf[ .. 4] == &TIFFB_MAGICNUM {
    return Some(ImageFormat::Tiff);
  } else {
    println!("DEBUG: colorimage: unknown magicnum: {:?}", &buf[ .. 10.min(buf.len())]);
  }
  None
}

pub fn decode_jpeg_image<W>(buf: &[u8], writer: &mut W) -> Result<(), ()> where W: ImageWriter + 'static {
  match NSJpegDecoder::new(true).decode(buf, writer) {
    Ok(_) => Ok(()),
    Err(_) => Err(()),
  }
}

pub fn decode_png_image<W>(buf: &[u8], writer: &mut W) -> Result<(), ()> where W: ImageWriter + 'static {
  match NSPngDecoder::new(true).decode(buf, writer) {
    Ok(_) => Ok(()),
    Err(_) => Err(()),
  }
}

pub fn decode_image<W>(buf: &[u8], writer: &mut W) -> Result<(), ()> where W: ImageWriter + 'static {
  let maybe_format = guess_image_format_from_magicnum(buf);
  let mut tried_formats = None;
  if let Some(format) = maybe_format {
    let res = match format {
      ImageFormat::Jpeg => decode_jpeg_image(buf, writer),
      ImageFormat::Png  => decode_png_image(buf, writer),
      // TODO
      _ => unimplemented!(),
    };
    if res.is_ok() {
      return Ok(());
    }
    tried_formats = Some(HashSet::new());
    tried_formats.as_mut().unwrap().insert(format);
  }
  if !tried_formats.as_ref().unwrap().contains(&ImageFormat::Jpeg) {
    if decode_jpeg_image(buf, writer).is_ok() {
      return Ok(());
    }
    tried_formats.as_mut().unwrap().insert(ImageFormat::Jpeg);
  }
  if !tried_formats.as_ref().unwrap().contains(&ImageFormat::Png) {
    if decode_png_image(buf, writer).is_ok() {
      return Ok(());
    }
    tried_formats.as_mut().unwrap().insert(ImageFormat::Png);
  }
  // TODO
  Err(())
}
