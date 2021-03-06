/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "ns_png_decoder.h"
#include "color_mgmt.h"
#include "image.h"
#include "qcms/qcms.h"

#include "png.h"

#include <assert.h>
#include <stdint.h>
#include <stdlib.h>

// limit image dimensions (bug #251381, #591822, #967656, and #1283961)
#ifndef MOZ_PNG_MAX_WIDTH
#  define MOZ_PNG_MAX_WIDTH 0x7fffffff // Unlimited
#endif
#ifndef MOZ_PNG_MAX_HEIGHT
#  define MOZ_PNG_MAX_HEIGHT 0x7fffffff // Unlimited
#endif

#ifdef PNG_HANDLE_AS_UNKNOWN_SUPPORTED
static const int num_color_chunks = 2;
static png_byte color_chunks[] = {
     99,  72,  82,  77, '\0',   // cHRM
    105,  67,  67,  80, '\0'};  // iCCP
static const int num_unused_chunks = 13;
static png_byte unused_chunks[] = {
     98,  75,  71,  68, '\0',   // bKGD
    101,  88,  73, 102, '\0',   // eXIf
    104,  73,  83,  84, '\0',   // hIST
    105,  84,  88, 116, '\0',   // iTXt
    111,  70,  70, 115, '\0',   // oFFs
    112,  67,  65,  76, '\0',   // pCAL
    115,  67,  65,  76, '\0',   // sCAL
    112,  72,  89, 115, '\0',   // pHYs
    115,  66,  73,  84, '\0',   // sBIT
    115,  80,  76,  84, '\0',   // sPLT
    116,  69,  88, 116, '\0',   // tEXt
    116,  73,  77,  69, '\0',   // tIME
    122,  84,  88, 116, '\0'};  // zTXt
#endif

static void PNGAPI error_callback(png_structp png, png_const_charp msg) {
  // TODO
  (void)png;
  (void)msg;
}

static void PNGAPI warning_callback(png_structp png, png_const_charp msg) {
  // TODO
  (void)png;
  (void)msg;
}

// Adapted from http://www.littlecms.com/pngchrm.c example code
static qcms_profile *_png_get_color_profile(png_structp png, png_infop info, int color_type, qcms_data_type *in_type, uint32_t *intent) {
  qcms_profile* profile = NULL;
  *intent = QCMS_INTENT_PERCEPTUAL; // Our default

  // First try to see if iCCP chunk is present
  if (png_get_valid(png, info, PNG_INFO_iCCP)) {
    png_uint_32 profileLen;
    png_bytep profileData;
    png_charp profileName;
    int compression;

    png_get_iCCP(png, info, &profileName, &compression,
                 &profileData, &profileLen);

    profile = qcms_profile_from_memory((char*)profileData, profileLen);
    if (profile) {
      uint32_t profileSpace = qcms_profile_get_color_space(profile);

      int mismatch = 0;
      if (color_type & PNG_COLOR_MASK_COLOR) {
        if (profileSpace != icSigRgbData) {
          mismatch = 1;
        }
      } else {
        if (profileSpace == icSigRgbData) {
          png_set_gray_to_rgb(png);
        } else if (profileSpace != icSigGrayData) {
          mismatch = 1;
        }
      }

      if (mismatch) {
        qcms_profile_release(profile);
        profile = NULL;
      } else {
        *intent = qcms_profile_get_rendering_intent(profile);
      }
    }
  }

  // Check sRGB chunk
  if (!profile && png_get_valid(png, info, PNG_INFO_sRGB)) {
    profile = qcms_profile_sRGB();

    if (profile) {
      int fileIntent;
      png_set_gray_to_rgb(png);
      png_get_sRGB(png, info, &fileIntent);
      uint32_t map[] = { QCMS_INTENT_PERCEPTUAL,
                         QCMS_INTENT_RELATIVE_COLORIMETRIC,
                         QCMS_INTENT_SATURATION,
                         QCMS_INTENT_ABSOLUTE_COLORIMETRIC };
      assert(fileIntent >= 0 && fileIntent < 4);
      *intent = map[fileIntent];
    }
  }

  // Check gAMA/cHRM chunks
  if (!profile &&
       png_get_valid(png, info, PNG_INFO_gAMA) &&
       png_get_valid(png, info, PNG_INFO_cHRM)) {
    qcms_CIE_xyYTRIPLE primaries;
    qcms_CIE_xyY whitePoint;

    png_get_cHRM(png, info,
                 &whitePoint.x, &whitePoint.y,
                 &primaries.red.x,   &primaries.red.y,
                 &primaries.green.x, &primaries.green.y,
                 &primaries.blue.x,  &primaries.blue.y);
    whitePoint.Y =
      primaries.red.Y = primaries.green.Y = primaries.blue.Y = 1.0;

    double gammaOfFile;

    png_get_gAMA(png, info, &gammaOfFile);

    profile = qcms_profile_create_rgb_with_gamma(whitePoint, primaries,
                                                 1.0/gammaOfFile);

    if (profile) {
      png_set_gray_to_rgb(png);
    }
  }

  if (profile) {
    uint32_t profileSpace = qcms_profile_get_color_space(profile);
    if (profileSpace == icSigGrayData) {
      if (color_type & PNG_COLOR_MASK_ALPHA) {
        *in_type = QCMS_DATA_GRAYA_8;
      } else {
        *in_type = QCMS_DATA_GRAY_8;
      }
    } else {
      if (color_type & PNG_COLOR_MASK_ALPHA ||
          png_get_valid(png, info, PNG_INFO_tRNS)) {
        *in_type = QCMS_DATA_RGBA_8;
      } else {
        *in_type = QCMS_DATA_RGB_8;
      }
    }
  }

  return profile;
}

static void _png_do_gamma_correction(png_structp png, png_infop info) {
  // Sets up gamma pre-correction in libpng before our callback gets called.
  // We need to do this if we don't end up with a CMS profile.
  double gamma;
  if (png_get_gAMA(png, info, &gamma)) {
    if (gamma <= 0.0 || gamma > 21474.83) {
      gamma = 0.45455;
      png_set_gAMA(png, info, gamma);
    }
    png_set_gamma(png, 2.2, gamma);
  } else {
    png_set_gamma(png, 2.2, 0.45455);
  }
}

static void PNGAPI info_callback(png_structp _png, png_infop _info) {
  (void)_info;

  struct NSPngDecoderCtx *ctx = (struct NSPngDecoderCtx *)(png_get_progressive_ptr(_png));

  png_uint_32 width, height;
  int bit_depth, color_type, interlace_type, compression_type, filter_type;
  unsigned int channels;

  png_bytep trans = NULL;
  int num_trans = 0;

  // Always decode to 24-bit RGB or 32-bit RGBA.
  png_get_IHDR(
      ctx->png, ctx->info,
      &width, &height, &bit_depth, &color_type,
      &interlace_type, &compression_type, &filter_type);

  ctx->width = width;
  ctx->height = height;

  // Post our size to the superclass.
  ctx->callbacks.init_size(ctx->writer, width, height);

  // TODO: check size limits.

  if (PNG_COLOR_TYPE_PALETTE == color_type) {
    png_set_expand(ctx->png);
  }

  if (PNG_COLOR_TYPE_GRAY == color_type && bit_depth < 8) {
    png_set_expand(ctx->png);
  }

  if (png_get_valid(ctx->png, ctx->info, PNG_INFO_tRNS)) {
    png_color_16p trans_values;
    png_get_tRNS(ctx->png, ctx->info, &trans, &num_trans, &trans_values);
    // libpng doesn't reject a tRNS chunk with out-of-range samples
    // so we check it here to avoid setting up a useless opacity
    // channel or producing unexpected transparent pixels (bug #428045)
    if (bit_depth < 16) {
      png_uint_16 sample_max = (1 << bit_depth) - 1;
      if ((PNG_COLOR_TYPE_GRAY == color_type && trans_values->gray > sample_max) ||
          (PNG_COLOR_TYPE_RGB == color_type && (trans_values->red > sample_max ||
                                                trans_values->green > sample_max ||
                                                trans_values->blue > sample_max)))
      {
        // clear the tRNS valid flag and release tRNS memory
        png_free_data(ctx->png, ctx->info, PNG_FREE_TRNS, 0);
        num_trans = 0;
      }
    }
    if (0 != num_trans) {
      png_set_expand(ctx->png);
    }
  }

  if (16 == bit_depth) {
    png_set_scale_16(ctx->png);
  }

  qcms_data_type in_type = QCMS_DATA_RGBA_8;
  uint32_t intent = (uint32_t)(-1);
  if (ctx->color_mgmt) {
    uint32_t p_intent;
    ctx->in_profile = _png_get_color_profile(
        ctx->png,
        ctx->info,
        color_type,
        &in_type,
        &p_intent);
    // If we're not mandating an intent, use the one from the image.
    if (intent == (uint32_t)(-1)) {
      intent = p_intent;
    }
  }
  if (ctx->in_profile != NULL && ctx->color_mgmt) {
    qcms_data_type out_type;

    if ((color_type & PNG_COLOR_MASK_ALPHA) || num_trans) {
      out_type = QCMS_DATA_RGBA_8;
      ctx->out_channels = 4;
    } else {
      out_type = QCMS_DATA_RGB_8;
      ctx->out_channels = 3;
    }

    ctx->transform = qcms_transform_create(
        ctx->in_profile,
        in_type,
        ctx->cm->out_profile,
        out_type,
        (qcms_intent)(intent));
  } else {
    ctx->out_channels = 0;

    png_set_gray_to_rgb(ctx->png);

    // only do gamma correction if CMS isn't entirely disabled
    if (ctx->color_mgmt) {
      _png_do_gamma_correction(ctx->png, ctx->info);
    }

    // NOTE: `color_mgmt` on is currently equal to eCMSMode=2,
    // so do not force a color space transform.
  }

  // Let libpng expand interlaced images.
  const int is_interlaced = interlace_type == PNG_INTERLACE_ADAM7;
  if (is_interlaced) {
    png_set_interlace_handling(ctx->png);
  }

  // now all of those things we set above are used to update various struct
  // members and whatnot, after which we can get channels, rowbytes, etc.
  png_read_update_info(ctx->png, ctx->info);
  channels = png_get_channels(ctx->png, ctx->info);
  ctx->channels = channels;
  if (0 == ctx->out_channels) {
    ctx->out_channels = channels;
  }

  //---------------------------------------------------------------//
  // copy PNG info into imagelib structs (formerly png_set_dims()) //
  //---------------------------------------------------------------//

  if (channels < 1 || channels > 4) {
    png_error(ctx->png, "Invalid number of channels");
  }

  // NOTE: The original code called "CreateFrame" here but we do not need to,
  // as `ctx->callbacks.init_size()` is sufficient.
  ctx->pass = 0;

  if (ctx->transform && (channels <= 2 || is_interlaced)) {
    const uint32_t bpp[] = { 0, 3, 4, 3, 4 };
    assert(channels <= 4);
    const uint32_t cms_channels = bpp[channels];
    assert(cms_channels == ctx->out_channels);
    ctx->cms_line = (uint8_t *)(malloc(sizeof(uint8_t) * cms_channels * width));
    if (ctx->cms_line == NULL) {
      png_error(ctx->png, "malloc of mCMSLine failed");
    }
  }

  if (interlace_type == PNG_INTERLACE_ADAM7) {
    const size_t buffer_size = (size_t)(channels) * (size_t)(width) * (size_t)(height);
    ctx->interlace_buf = (uint8_t *)(malloc(buffer_size));
    if (ctx->interlace_buf == NULL) {
      png_error(ctx->png, "malloc of interlacebuf failed");
    }
  }
}

static void _write_row(struct NSPngDecoderCtx *ctx, uint32_t row_num, uint8_t *row) {
  assert(row != NULL);

  uint8_t *row_to_write = row;
  const uint32_t width = ctx->width;
  const uint32_t out_channels = ctx->out_channels;

  // Apply color management to the row, if necessary, before writing it out.
  if (ctx->transform) {
    if (ctx->cms_line != NULL) {
      qcms_transform_data(ctx->transform, row_to_write, ctx->cms_line, width);

      /*// Copy alpha over.
      if (ctx->channels == 2 || ctx->channels == 4) {
        for (uint32_t i = 0; i < width; ++i) {
          ctx->cms_line[4 * i + 3] = row_to_write[channels * i + channels - 1];
        }
      }*/

      row_to_write = ctx->cms_line;
    } else {
      qcms_transform_data(ctx->transform, row_to_write, row_to_write, width);
    }
  }

  // Write this row to the SurfacePipe.
  // TODO: packing to the correct pixel format.
  /*ctx->callbacks.write_row(ctx->writer, row_num, row_to_write, out_channels * width);*/
  switch (out_channels) {
    case 1: {
      ctx->callbacks.write_row_gray(ctx->writer, row_num, row_to_write, width);
    } break;
    case 2: {
      ctx->callbacks.write_row_grayx(ctx->writer, row_num, row_to_write, width);
    } break;
    case 3: {
      ctx->callbacks.write_row_rgb(ctx->writer, row_num, row_to_write, width);
    } break;
    case 4: {
      ctx->callbacks.write_row_rgbx(ctx->writer, row_num, row_to_write, width);
    } break;
    default:
      assert(0 && "unimplemented");
  }
}

static void PNGAPI row_callback(png_structp _png, png_bytep new_row, png_uint_32 row_num, int pass) {
  /* libpng comments:
   *
   * This function is called for every row in the image.  If the
   * image is interlacing, and you turned on the interlace handler,
   * this function will be called for every row in every pass.
   * Some of these rows will not be changed from the previous pass.
   * When the row is not changed, the new_row variable will be
   * nullptr. The rows and passes are called in order, so you don't
   * really need the row_num and pass, but I'm supplying them
   * because it may make your life easier.
   *
   * For the non-nullptr rows of interlaced images, you must call
   * png_progressive_combine_row() passing in the row and the
   * old row.  You can call this function for nullptr rows (it will
   * just return) and for non-interlaced images (it just does the
   * memcpy for you) if it will make the code easier.  Thus, you
   * can just do this for all cases:
   *
   *    png_progressive_combine_row(png_ptr, old_row, new_row);
   *
   * where old_row is what was displayed for previous rows.  Note
   * that the first pass (pass == 0 really) will completely cover
   * the old row, so the rows do not have to be initialized.  After
   * the first pass (and only for interlaced images), you will have
   * to pass the current row, and the function will combine the
   * old row and the new row.
   */

  struct NSPngDecoderCtx *ctx = (struct NSPngDecoderCtx *)(png_get_progressive_ptr(_png));

  while (pass > ctx->pass) {
    // Advance to the next pass. We may have to do this multiple times because
    // libpng will skip passes if the image is so small that no pixels have
    // changed on a given pass, but ADAM7InterpolatingFilter needs to be reset
    // once for every pass to perform interpolation properly.

    // NOTE: No need to force the writer to reset to the first row,
    // since `write_row` passes the row index.
    ctx->pass++;
  }

  const png_uint_32 height = ctx->height;
  if (row_num >= height) {
    // Bail if we receive extra rows. This is especially important because if we
    // didn't, we might overflow the deinterlacing buffer.
    assert(0 && "unreachable");
    return;
  }

  // Note that |new_row| may be null here, indicating that this is an interlaced
  // image and |row_callback| is being called for a row that hasn't changed.
  if (new_row == NULL) {
    assert(ctx->interlace_buf != NULL);
  }
  uint8_t *row_to_write = new_row;

  if (ctx->interlace_buf != NULL) {
    const uint32_t width = ctx->width;

    // We'll output the deinterlaced version of the row.
    row_to_write = ctx->interlace_buf + (row_num * ctx->channels * width);

    // Update the deinterlaced version of this row with the new data.
    png_progressive_combine_row(ctx->png, row_to_write, new_row);
  }

  _write_row(ctx, row_num, row_to_write);
}

static void PNGAPI end_callback(png_structp _png, png_infop _info) {
  // TODO: this is for error checking.
  (void)_png;
  (void)_info;
}

size_t gckimg_ns_png_sizeof(void) {
  return sizeof(struct NSPngDecoderCtx);
}

void gckimg_ns_png_init(struct NSPngDecoderCtx *ctx, int color_mgmt) {
  // Initialize the container's source image header
  // Always decode to 24 bit pixdepth
  ctx->png = png_create_read_struct(
      PNG_LIBPNG_VER_STRING,
      NULL,
      error_callback,
      warning_callback);
  assert(ctx->png != NULL);

  ctx->info = png_create_info_struct(ctx->png);
  assert(ctx->info != NULL);

  ctx->color_mgmt = color_mgmt;

#ifdef PNG_HANDLE_AS_UNKNOWN_SUPPORTED
  // Ignore unused chunks
  if (!color_mgmt) {
    png_set_keep_unknown_chunks(ctx->png, 1, color_chunks, num_color_chunks);
  }
  png_set_keep_unknown_chunks(ctx->png, 1, unused_chunks, num_unused_chunks);
#endif

#ifdef PNG_SET_USER_LIMITS_SUPPORTED
  png_set_user_limits(ctx->png, MOZ_PNG_MAX_WIDTH, MOZ_PNG_MAX_HEIGHT);
  /*if (color_mgmt) {
    png_set_chunk_malloc_max(ctx->png, 4000000L);
  }*/
#endif

#ifdef PNG_CHECK_FOR_INVALID_INDEX_SUPPORTED
  // Disallow palette-index checking, for speed; we would ignore the warning
  // anyhow.  This feature was added at libpng version 1.5.10 and is disabled
  // in the embedded libpng but enabled by default in the system libpng.  This
  // call also disables it in the system libpng, for decoding speed.
  // Bug #745202.
  png_set_check_for_invalid_index(ctx->png, 0);
#endif

#ifdef PNG_SET_OPTION_SUPPORTED
#if defined(PNG_sRGB_PROFILE_CHECKS) && PNG_sRGB_PROFILE_CHECKS >= 0
  // Skip checking of sRGB ICC profiles
  png_set_option(ctx->png, PNG_SKIP_sRGB_CHECK_PROFILE, PNG_OPTION_ON);
#endif
#ifdef PNG_MAXIMUM_INFLATE_WINDOW
  // Force a larger zlib inflate window as some images in the wild have
  // incorrectly set metadata (specifically CMF bits) which prevent us from
  // decoding them otherwise.
  png_set_option(ctx->png, PNG_MAXIMUM_INFLATE_WINDOW, PNG_OPTION_ON);
#endif
#endif
}

void gckimg_ns_png_cleanup(struct NSPngDecoderCtx *ctx) {
  if (ctx->png != NULL) {
    png_destroy_read_struct(&ctx->png, &ctx->info, NULL);
  }
  if (ctx->cms_line != NULL) {
    free(ctx->cms_line);
  }
  if (ctx->interlace_buf != NULL) {
    free(ctx->interlace_buf);
  }
  if (ctx->in_profile != NULL) {
    qcms_profile_release(ctx->in_profile);
    // mTransform belongs to us only if mInProfile is non-null
    if (ctx->transform != NULL) {
      qcms_transform_release(ctx->transform);
    }
  }
}

void gckimg_ns_png_decode(
    struct NSPngDecoderCtx *ctx,
    struct ColorMgmtCtx *cm,
    const uint8_t *buf, size_t buf_len,
    void *writer, struct ImageWriterCallbacks callbacks)
{
  // libpng uses setjmp/longjmp for error handling.
  if (setjmp(png_jmpbuf(ctx->png))) {
    // TODO
    return;
  }

  ctx->cm = cm;
  ctx->writer = writer;
  ctx->callbacks = callbacks;

  // use this as libpng "progressive pointer" (retrieve in callbacks)
  png_set_progressive_read_fn(
      ctx->png,
      ctx,
      info_callback,
      row_callback,
      end_callback);

  // Pass the data off to libpng.
  png_process_data(ctx->png, ctx->info, (png_bytep)buf, buf_len);
}
