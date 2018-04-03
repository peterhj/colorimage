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

pub unsafe extern "C" fn generic_parse_exif(exif_buf: *const u8, exif_size: usize) -> i32 {
  assert!(!exif_buf.is_null());
  let raw_exif = from_raw_parts(exif_buf, exif_size);
  parse_exif(raw_exif).unwrap_or(0)
}

pub struct ColorImage {
  inner:    Option<PILImage>,
}

pub unsafe extern "C" fn color_image_init_size(img_p: *mut c_void, width: usize, height: usize) {
  assert!(!img_p.is_null());
  let mut img = &mut *(img_p as *mut ColorImage);
  img.inner = Some(PILImage::new(PILMode::RGBX, width as _, height as _));
}

pub unsafe extern "C" fn color_image_write_row(img_p: *mut c_void, row_idx: usize, row_buf: *const u8, row_width: usize) {
  assert!(!img_p.is_null());
  let mut img = &mut *(img_p as *mut ColorImage);

  assert!(!row_buf.is_null());
  // FIXME
  let row_size = 3 * row_width;
  let row = from_raw_parts(row_buf, row_size);

  assert!(img.inner.is_some());

  // TODO
  unimplemented!();
}

impl ImageWriter for ColorImage {
  fn callbacks() -> ImageWriterCallbacks {
    ImageWriterCallbacks{
      init_size:        Some(color_image_init_size),
      //write_row:        Some(color_image_write_row),
      // FIXME
      write_row_gray:   Some(color_image_write_row),
      write_row_grayx:  Some(color_image_write_row),
      write_row_rgb:    Some(color_image_write_row),
      write_row_rgbx:   Some(color_image_write_row),
      parse_exif:       Some(generic_parse_exif),
    }
  }
}

impl ColorImage {
  pub fn width(&self) -> usize {
    // TODO
    unimplemented!();
  }

  pub fn height(&self) -> usize {
    // TODO
    unimplemented!();
  }

  pub fn resize(&mut self, new_width: usize, new_height: usize) {
    assert!(self.inner.is_some());
    // TODO
    //self.img.resample(new_width as _, new_height as _, _);
    unimplemented!();
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
  let mut img = &mut *(img_p as *mut RasterImage);
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
  let mut img = &mut *(img_p as *mut RasterImage);

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
  let mut img = &mut *(img_p as *mut RasterImage);

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
      //write_row:        Some(raster_image_write_row),
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
