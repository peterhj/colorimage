extern crate bindgen;
extern crate cc;

use std::env;
use std::path::{PathBuf};
use std::process::{Command};

fn main() {
  let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

  println!("cargo:rustc-link-search=native={}", out_dir.display());
  println!("cargo:rustc-link-lib=static=image_io_wrapped");
  //println!("cargo:rustc-link-lib=static=png_native");
  println!("cargo:rustc-link-lib=png");
  println!("cargo:rustc-link-lib=z");

  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=wrapped.h");
  println!("cargo:rerun-if-changed=wrapped/ns_jpeg_decoder.c");
  println!("cargo:rerun-if-changed=wrapped/ns_jpeg_decoder.h");
  println!("cargo:rerun-if-changed=wrapped/ns_png_decoder.c");
  println!("cargo:rerun-if-changed=wrapped/ns_png_decoder.h");

  // TODO: build libpng.

  cc::Build::new()
    .opt_level(2)
    .pic(true)
    .flag("-std=gnu99")
    .flag("-fno-strict-aliasing")
    .flag("-Wall")
    .flag("-Werror")
    .include("wrapped")
    //.include("third_party/libjpeg-turbo")
    //.include("third_party/libpng")
    .file("wrapped/color_mgmt.c")
    .file("wrapped/ns_jpeg_decoder.c")
    .file("wrapped/ns_png_decoder.c")
    .file("wrapped/qcms/chain.c")
    .file("wrapped/qcms/iccread.c")
    .file("wrapped/qcms/matrix.c")
    .file("wrapped/qcms/transform.c")
    .file("wrapped/qcms/transform_util.c")
    // TODO: platform-dependent vectorized sources.
    .compile("libimage_io_wrapped.a");

  Command::new("rm")
    .current_dir(&out_dir)
    .arg("-f")
    .arg(out_dir.join("wrapped_bind.rs").as_os_str().to_str().unwrap())
    .status().unwrap();

  bindgen::Builder::default()
    .header("wrapped.h")
    .clang_arg("-Iwrapped")
    //.clang_arg("-Ithird_party/libjpeg-turbo")
    //.clang_arg("-Ithird_party/libpng")
    .whitelist_type("RasterWriterCallbacks")
    .whitelist_type("NSJpegDecoderCtx")
    .whitelist_type("NSPngDecoderCtx")
    .whitelist_function("wrapped_ns_jpeg_init")
    .whitelist_function("wrapped_ns_jpeg_cleanup")
    .whitelist_function("wrapped_ns_jpeg_decode")
    .whitelist_function("wrapped_ns_png_init")
    .whitelist_function("wrapped_ns_png_cleanup")
    .whitelist_function("wrapped_ns_png_decode")
    .generate()
    .unwrap()
    .write_to_file(out_dir.join("wrapped_bind.rs"))
    .unwrap();
}
