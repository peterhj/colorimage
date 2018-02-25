extern crate bindgen;
extern crate cc;

use std::env;
use std::path::{PathBuf};
use std::process::{Command};

fn main() {
  let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

  println!("cargo:rustc-link-search=native={}", out_dir.display());
  println!("cargo:rustc-link-lib=static=gckimg_native");
  // TODO: link to included libjpeg.
  //println!("cargo:rustc-link-lib=static=gckimg_native_jpeg");
  println!("cargo:rustc-link-lib=static=gckimg_native_png");
  println!("cargo:rustc-link-lib=z");

  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=wrapped.h");
  println!("cargo:rerun-if-changed=src/gckimg/ns_jpeg_decoder.c");
  println!("cargo:rerun-if-changed=src/gckimg/ns_jpeg_decoder.h");
  println!("cargo:rerun-if-changed=src/gckimg/ns_png_decoder.c");
  println!("cargo:rerun-if-changed=src/gckimg/ns_png_decoder.h");

  Command::new("rm")
    .current_dir(&out_dir)
    .arg("-f")
    .arg(out_dir.join("libgckimg_native.a").as_os_str().to_str().unwrap())
    .status().unwrap();

  cc::Build::new()
    .opt_level(2)
    .pic(true)
    .flag("-std=gnu99")
    .flag("-fno-strict-aliasing")
    .flag("-Wall")
    .flag("-Werror")
    .include("src/gckimg")
    //.include("src/gckimg/libjpeg")
    .include("src/gckimg/libpng")
    .file("src/gckimg/color_mgmt.c")
    .file("src/gckimg/ns_jpeg_decoder.c")
    .file("src/gckimg/ns_png_decoder.c")
    .file("src/gckimg/extras/iccjpeg.c")
    .file("src/gckimg/qcms/chain.c")
    .file("src/gckimg/qcms/iccread.c")
    .file("src/gckimg/qcms/matrix.c")
    .file("src/gckimg/qcms/transform.c")
    // TODO: target feature test?
    .file("src/gckimg/qcms/transform-sse1.c")
    .file("src/gckimg/qcms/transform-sse2.c")
    .file("src/gckimg/qcms/transform_util.c")
    // TODO: platform-dependent vectorized sources.
    .compile("libgckimg_native.a");

  // TODO: build libjpeg.

  Command::new("rm")
    .current_dir(&out_dir)
    .arg("-f")
    .arg(out_dir.join("libgckimg_native_png.a").as_os_str().to_str().unwrap())
    .status().unwrap();

  cc::Build::new()
    .opt_level(2)
    .pic(true)
    .flag("-std=c89")
    .flag("-fno-strict-aliasing")
    .flag("-Wall")
    .flag("-Werror")
    .include("src/gckimg/libpng")
    .file("src/gckimg/libpng/png.c")
    .file("src/gckimg/libpng/pngerror.c")
    .file("src/gckimg/libpng/pngget.c")
    .file("src/gckimg/libpng/pngmem.c")
    .file("src/gckimg/libpng/pngpread.c")
    .file("src/gckimg/libpng/pngread.c")
    .file("src/gckimg/libpng/pngrio.c")
    .file("src/gckimg/libpng/pngrtran.c")
    .file("src/gckimg/libpng/pngrutil.c")
    .file("src/gckimg/libpng/pngset.c")
    .file("src/gckimg/libpng/pngtrans.c")
    .file("src/gckimg/libpng/pngwio.c")
    .file("src/gckimg/libpng/pngwrite.c")
    .file("src/gckimg/libpng/pngwutil.c")
    .file("src/gckimg/libpng/intel/filter_sse2_intrinsics.c")
    .file("src/gckimg/libpng/intel/intel_init.c")
    // TODO: platform-dependent vectorized sources.
    .compile("libgckimg_native_png.a");

  Command::new("rm")
    .current_dir(&out_dir)
    .arg("-f")
    .arg(out_dir.join("gckimg_bind.rs").as_os_str().to_str().unwrap())
    .status().unwrap();

  bindgen::Builder::default()
    .header("wrapped.h")
    .clang_arg("-Isrc/gckimg")
    //.clang_arg("-Isrc/gckimg/libjpeg")
    .clang_arg("-Isrc/gckimg/libpng")
    .whitelist_type("ColorMgmtCtx")
    .whitelist_type("Angle")
    .whitelist_type("Flip")
    .whitelist_type("ExifData")
    .whitelist_type("ImageWriterCallbacks")
    .whitelist_type("NSJpegDecoderCtx")
    .whitelist_type("NSPngDecoderCtx")
    .whitelist_function("gckimg_color_mgmt_init_default")
    .whitelist_function("gckimg_color_mgmt_cleanup")
    .whitelist_function("gckimg_ns_jpeg_init")
    .whitelist_function("gckimg_ns_jpeg_cleanup")
    .whitelist_function("gckimg_ns_jpeg_decode")
    .whitelist_function("gckimg_ns_png_init")
    .whitelist_function("gckimg_ns_png_cleanup")
    .whitelist_function("gckimg_ns_png_decode")
    .generate()
    .unwrap()
    .write_to_file(out_dir.join("gckimg_bind.rs"))
    .unwrap();
}
