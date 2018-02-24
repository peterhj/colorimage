use decoders::*;
use decoders::jpeg::*;
use decoders::png::*;
use ffi::gckimg::{ImageWriterCallbacks};

use std::collections::{HashSet};
use std::os::raw::{c_void};
use std::str::{from_utf8};

pub mod decoders;
pub mod ffi;

pub const BMP_SIGNATURE:    [u8; 2] = [b'B', b'M'];
pub const GIF87A_SIGNATURE: [u8; 6] = [b'G', b'I', b'F', b'8', b'7', b'a'];
pub const GIF89A_SIGNATURE: [u8; 6] = [b'G', b'I', b'F', b'8', b'9', b'a'];
pub const JPEG_SIGNATURE:   [u8; 3] = [0xff, 0xd8, 0xff];
pub const PNG_SIGNATURE:    [u8; 8] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
pub const TIFF_SIGNATURE:   [u8; 4] = [b'I', b'I', 42, 0];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ImageFormat {
  Bmp,
  Gif,
  Jpeg,
  Png,
  Tiff,
}

pub fn guess_image_format_from_signature(buf: &[u8]) -> Option<ImageFormat> {
  if &buf[ .. 3] == &JPEG_SIGNATURE {
    return Some(ImageFormat::Jpeg);
  } else if &buf[ .. 8] == &PNG_SIGNATURE {
    return Some(ImageFormat::Png);
  } else if &buf[ .. 6] == &GIF89A_SIGNATURE ||
            &buf[ .. 6] == &GIF87A_SIGNATURE {
    return Some(ImageFormat::Gif);
  } else if &buf[ .. 2] == &BMP_SIGNATURE {
    return Some(ImageFormat::Bmp);
  } else if &buf[ .. 4] == &TIFF_SIGNATURE {
    return Some(ImageFormat::Tiff);
  } else {
    println!("DEBUG: colorimage: unknown signature: {:?}", &buf[ .. 10.min(buf.len())]);
  }
  None
}

pub struct RasterImage {
  width:    usize,
  height:   usize,
  channels: usize,
  data:     Vec<Vec<u8>>,
}

pub unsafe extern "C" fn raster_image_init_size(img_p: *mut c_void, width: usize, height: usize, channels: usize) {
  println!("DEBUG: RasterImage: init size: {} {} {}",
      width, height, channels);
  assert!(!img_p.is_null());
  let mut img = unsafe { &mut *(img_p as *mut RasterImage) };
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

pub unsafe extern "C" fn raster_image_write_row(img_p: *mut c_void, row_idx: usize, row_data: *const u8, row_size: usize) {
  // TODO
  println!("DEBUG: RasterImage: write row: {} {}",
      row_idx, row_size);
  assert!(!img_p.is_null());
}

impl ImageWriter for RasterImage {
  fn callbacks() -> ImageWriterCallbacks {
    ImageWriterCallbacks{
      init_size:    Some(raster_image_init_size),
      write_row:    Some(raster_image_write_row),
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
}

pub fn decode_png_image(buf: &[u8], writer: &mut RasterImage) -> Result<(), ()> {
  match NSPngDecoder::new(true).decode(buf, writer) {
    Ok(_) => Ok(()),
    Err(_) => Err(()),
  }
}

pub fn decode_image(buf: &[u8], writer: &mut RasterImage) -> Result<(), (Option<()>, ())> {
  let maybe_format = guess_image_format_from_signature(buf);
  let mut tried_formats = None;
  if let Some(format) = maybe_format {
    match format {
      ImageFormat::Jpeg => {
        match NSJpegDecoder::new(true).decode(buf, writer) {
          Ok(img) => return Ok(img),
          Err(_) => {}
        }
      }
      ImageFormat::Png => {
        match NSPngDecoder::new(true).decode(buf, writer) {
          Ok(img) => return Ok(img),
          Err(_) => {}
        }
      }
      /*ImageFormat::Gif => {
      }*/
      _ => {}
    }
    tried_formats = Some(HashSet::new());
    tried_formats.as_mut().unwrap().insert(format);
  }
  // TODO
  unimplemented!();
}
