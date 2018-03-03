extern crate bindgen;
extern crate cc;

use std::env;
use std::fs;
use std::path::{PathBuf};
use std::process::{Command};

fn main() {
  let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

  println!("cargo:rustc-link-search=native={}", out_dir.display());
  println!("cargo:rustc-link-lib=static=gckimg_native");
  println!("cargo:rustc-link-lib=static=gckimg_native_jpeg");
  println!("cargo:rustc-link-lib=static=gckimg_native_jpeg_simd");
  println!("cargo:rustc-link-lib=static=gckimg_native_png");
  println!("cargo:rustc-link-lib=z");

  /*println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=wrapped.h");
  println!("cargo:rerun-if-changed=src/gckimg/ns_jpeg_decoder.c");
  println!("cargo:rerun-if-changed=src/gckimg/ns_jpeg_decoder.h");
  println!("cargo:rerun-if-changed=src/gckimg/ns_png_decoder.c");
  println!("cargo:rerun-if-changed=src/gckimg/ns_png_decoder.h");*/

  fs::remove_file(out_dir.join("libgckimg_native.a")).ok();

  cc::Build::new()
    .opt_level(2)
    .pic(true)
    .flag("-std=gnu99")
    .flag("-fno-strict-aliasing")
    .flag("-Wall")
    .flag("-Werror")
    .include("src/gckimg/libjpeg")
    .include("src/gckimg/libpng")
    .include("src/gckimg")
    .file("src/gckimg/color_mgmt.c")
    .file("src/gckimg/ns_jpeg_decoder.c")
    .file("src/gckimg/ns_png_decoder.c")
    .file("src/gckimg/iccjpeg.c")
    .file("src/gckimg/qcms/chain.c")
    .file("src/gckimg/qcms/iccread.c")
    .file("src/gckimg/qcms/matrix.c")
    .file("src/gckimg/qcms/transform.c")
    // TODO: target feature test?
    // TODO: platform-dependent vectorized sources.
    .file("src/gckimg/qcms/transform-sse1.c")
    .file("src/gckimg/qcms/transform-sse2.c")
    .file("src/gckimg/qcms/transform_util.c")
    .compile("libgckimg_native.a");

  fs::remove_file(out_dir.join("libgckimg_native_jpeg.a")).ok();

  cc::Build::new()
    .opt_level(2)
    .pic(true)
    .flag("-std=gnu99")
    .flag("-fno-strict-aliasing")
    .flag("-Wall")
    .flag("-Werror")
    .flag("-Wno-sign-compare")
    .flag("-Wno-unused-parameter")
    .include("src/gckimg/libjpeg")
    .include("src/gckimg")
    .file("src/gckimg/libjpeg/jcomapi.c")
    .file("src/gckimg/libjpeg/jdapimin.c")
    .file("src/gckimg/libjpeg/jdapistd.c")
    .file("src/gckimg/libjpeg/jdatadst.c")
    .file("src/gckimg/libjpeg/jdatasrc.c")
    .file("src/gckimg/libjpeg/jdcoefct.c")
    .file("src/gckimg/libjpeg/jdcolor.c")
    .file("src/gckimg/libjpeg/jddctmgr.c")
    .file("src/gckimg/libjpeg/jdhuff.c")
    .file("src/gckimg/libjpeg/jdinput.c")
    .file("src/gckimg/libjpeg/jdmainct.c")
    .file("src/gckimg/libjpeg/jdmarker.c")
    .file("src/gckimg/libjpeg/jdmaster.c")
    .file("src/gckimg/libjpeg/jdmerge.c")
    .file("src/gckimg/libjpeg/jdphuff.c")
    .file("src/gckimg/libjpeg/jdpostct.c")
    .file("src/gckimg/libjpeg/jdsample.c")
    .file("src/gckimg/libjpeg/jdtrans.c")
    .file("src/gckimg/libjpeg/jerror.c")
    .file("src/gckimg/libjpeg/jfdctflt.c")
    .file("src/gckimg/libjpeg/jfdctfst.c")
    .file("src/gckimg/libjpeg/jfdctint.c")
    .file("src/gckimg/libjpeg/jidctflt.c")
    .file("src/gckimg/libjpeg/jidctfst.c")
    .file("src/gckimg/libjpeg/jidctint.c")
    .file("src/gckimg/libjpeg/jidctred.c")
    .file("src/gckimg/libjpeg/jmemmgr.c")
    .file("src/gckimg/libjpeg/jmemnobs.c")
    .file("src/gckimg/libjpeg/jquant1.c")
    .file("src/gckimg/libjpeg/jquant2.c")
    .file("src/gckimg/libjpeg/jutils.c")
    // TODO: target feature test?
    // TODO: platform-dependent vectorized sources.
    .file("src/gckimg/libjpeg/simd/jsimd_x86_64.c")
    .compile("libgckimg_native_jpeg.a");

  fs::remove_file(out_dir.join("libgckimg_native_jpeg_simd.a")).ok();

  cc::Build::new()
    .compiler("/opt/yasm/bin/yasm")
    .explicit_flags_only(true)
    .flag("-felf64")
    //.flag("-rnasm")
    //.flag("-pnasm")
    .flag("-DELF")
    .flag("-D__x86_64__")
    .flag("-DPIC")
    .flag("-Isrc/gckimg/libjpeg/simd/")
    .file("src/gckimg/libjpeg/simd/jccolor-sse2-64.asm")
    .file("src/gckimg/libjpeg/simd/jcgray-sse2-64.asm")
    .file("src/gckimg/libjpeg/simd/jchuff-sse2-64.asm")
    .file("src/gckimg/libjpeg/simd/jcsample-sse2-64.asm")
    .file("src/gckimg/libjpeg/simd/jdcolor-sse2-64.asm")
    .file("src/gckimg/libjpeg/simd/jdmerge-sse2-64.asm")
    .file("src/gckimg/libjpeg/simd/jdsample-sse2-64.asm")
    .file("src/gckimg/libjpeg/simd/jfdctflt-sse-64.asm")
    .file("src/gckimg/libjpeg/simd/jfdctfst-sse2-64.asm")
    .file("src/gckimg/libjpeg/simd/jfdctint-sse2-64.asm")
    .file("src/gckimg/libjpeg/simd/jidctflt-sse2-64.asm")
    .file("src/gckimg/libjpeg/simd/jidctfst-sse2-64.asm")
    .file("src/gckimg/libjpeg/simd/jidctint-sse2-64.asm")
    .file("src/gckimg/libjpeg/simd/jidctred-sse2-64.asm")
    .file("src/gckimg/libjpeg/simd/jquantf-sse2-64.asm")
    .file("src/gckimg/libjpeg/simd/jquanti-sse2-64.asm")
    .compile("libgckimg_native_jpeg_simd.a");

  fs::remove_file(out_dir.join("libgckimg_native_png.a")).ok();

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
    // TODO: platform-dependent vectorized sources.
    .file("src/gckimg/libpng/intel/filter_sse2_intrinsics.c")
    .file("src/gckimg/libpng/intel/intel_init.c")
    .compile("libgckimg_native_png.a");

  fs::remove_file(out_dir.join("gckimg_bind.rs")).ok();

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
