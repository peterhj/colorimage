extern crate colorimage;

use colorimage::*;

use std::fs::{File};
use std::io::*;
use std::path::{PathBuf};

#[test]
fn test_png() {
  println!();
  let test_path = PathBuf::from("tests/test.png");
  let mut test_file = File::open(&test_path).unwrap();
  let mut test_buf = Vec::new();
  test_file.read_to_end(&mut test_buf).unwrap();
  let mut image = RasterImage::new();
  let _ = decode_png_image(&test_buf, &mut image);
}
