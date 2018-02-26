/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "exif.h"
#include "image.h"
#include "qcms/qcms.h"

#include <assert.h>
#include <setjmp.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// NOTE: Include jpeg headers after others.
#include "jpeglib.h"
#include "iccjpeg.h"
#include "ns_jpeg_decoder.h"

#ifdef MOZ_BIG_ENDIAN
#define MOZ_JCS_EXT_NATIVE_ENDIAN_XRGB JCS_EXT_XRGB
#else
#define MOZ_JCS_EXT_NATIVE_ENDIAN_XRGB JCS_EXT_BGRX
#endif

static qcms_profile *_jpeg_get_icc_profile(struct jpeg_decompress_struct *info) {
  JOCTET *profilebuf;
  uint32_t profileLength;
  qcms_profile *profile = NULL;

  if (read_icc_profile(info, &profilebuf, &profileLength)) {
    profile = qcms_profile_from_memory(profilebuf, profileLength);
    free(profilebuf);
  }

  return profile;
}

///*************** Inverted CMYK -> RGB conversion *************************
/// Input is (Inverted) CMYK stored as 4 bytes per pixel.
/// Output is RGB stored as 3 bytes per pixel.
/// @param row Points to row buffer containing the CMYK bytes for each pixel
/// in the row.
/// @param width Number of pixels in the row.
static void _cmyk_convert_rgb(JSAMPROW row, JDIMENSION width) {
  // Work from end to front to shrink from 4 bytes per pixel to 3
  JSAMPROW in = row + width*4;
  JSAMPROW out = in;

  for (uint32_t i = width; i > 0; i--) {
    in -= 4;
    out -= 3;

    // Source is 'Inverted CMYK', output is RGB.
    // See: http://www.easyrgb.com/math.php?MATH=M12#text12
    // Or:  http://www.ilkeratalay.com/colorspacesfaq.php#rgb

    // From CMYK to CMY
    // C = ( C * ( 1 - K ) + K )
    // M = ( M * ( 1 - K ) + K )
    // Y = ( Y * ( 1 - K ) + K )

    // From Inverted CMYK to CMY is thus:
    // C = ( (1-iC) * (1 - (1-iK)) + (1-iK) ) => 1 - iC*iK
    // Same for M and Y

    // Convert from CMY (0..1) to RGB (0..1)
    // R = 1 - C => 1 - (1 - iC*iK) => iC*iK
    // G = 1 - M => 1 - (1 - iM*iK) => iM*iK
    // B = 1 - Y => 1 - (1 - iY*iK) => iY*iK

    // Convert from Inverted CMYK (0..255) to RGB (0..255)
    const uint32_t iC = in[0];
    const uint32_t iM = in[1];
    const uint32_t iY = in[2];
    const uint32_t iK = in[3];
    out[0] = iC*iK/255;   // Red
    out[1] = iM*iK/255;   // Green
    out[2] = iY*iK/255;   // Blue
  }
}

METHODDEF(void) my_error_exit(j_common_ptr cinfo) {
  // TODO
  (void)cinfo;

  // TODO
  //longjmp(err_setjmp_buf, (int)error_code);
}

METHODDEF(void) init_source(j_decompress_ptr jd) {
  // TODO
  (void)jd;
}

METHODDEF(boolean) fill_input_buffer(j_decompress_ptr jd) {
  // TODO
  (void)jd;
  return 0;
}

METHODDEF(void) skip_input_data(j_decompress_ptr jd, long num_bytes) {
  // TODO
  (void)jd;
  (void)num_bytes;
}

METHODDEF(void) term_source(j_decompress_ptr jd) {
  // TODO
  (void)jd;
}

static int _ns_jpeg_read_orientation_from_exif(struct NSJpegDecoderCtx *ctx, struct ImageExifData *exif) {
  jpeg_saved_marker_ptr marker;

  // Locate the APP1 marker, where EXIF data is stored, in the marker list.
  for (marker = ctx->info.marker_list; marker != NULL; marker = marker->next) {
    if (marker->marker == JPEG_APP0 + 1) {
      break;
    }
  }

  // If we're at the end of the list, there's no EXIF data.
  if (marker == NULL) {
    exif->orientation_rotation = D0;
    exif->orientation_flip = Unflipped;
    return 0;
  }

  // Extract the orientation information.
  if (ctx->callbacks.parse_exif == NULL) {
    assert(0 && "missing exif parser");
    return 0;
  }
  // TODO: error handling.
  (ctx->callbacks.parse_exif)(marker->data, marker->data_length, exif);
  return 1;
}

static void _ns_jpeg_output_scanlines(struct NSJpegDecoderCtx *ctx, int *suspend) {
  *suspend = 0;

  const uint32_t top = ctx->info.output_scanline;

  while (ctx->info.output_scanline < ctx->info.output_height) {
    // TODO: should use `imagebuf` or something else?
    uint32_t* imageRow = (uint32_t *)(ctx->imagebuf) +
        (ctx->info.output_scanline * ctx->info.output_width);

    //MOZ_ASSERT(imageRow, "Should have a row buffer here");
    assert(imageRow != NULL && "Should have a row buffer here");

    if (ctx->info.out_color_space == MOZ_JCS_EXT_NATIVE_ENDIAN_XRGB) {
      // Special case: scanline will be directly converted into packed ARGB
      if (jpeg_read_scanlines(&ctx->info, (JSAMPARRAY)&imageRow, 1) != 1) {
        *suspend = 1; // suspend
        break;
      }
      continue; // all done for this row!
    }

    JSAMPROW sampleRow = (JSAMPROW)imageRow;
    if (ctx->info.output_components == 3) {
      // Put the pixels at end of row to enable in-place expansion
      sampleRow += ctx->info.output_width;
    }

    // Request one scanline.  Returns 0 or 1 scanlines.
    if (jpeg_read_scanlines(&ctx->info, &sampleRow, 1) != 1) {
      *suspend = 1; // suspend
      break;
    }

    if (ctx->transform) {
      JSAMPROW source = sampleRow;
      if (ctx->info.out_color_space == JCS_GRAYSCALE) {
        // Convert from the 1byte grey pixels at begin of row
        // to the 3byte RGB byte pixels at 'end' of row
        sampleRow += ctx->info.output_width;
      }
      qcms_transform_data(ctx->transform, source, sampleRow, ctx->info.output_width);
      // Move 3byte RGB data to end of row
      if (ctx->info.out_color_space == JCS_CMYK) {
        memmove(sampleRow + ctx->info.output_width,
                sampleRow,
                3 * ctx->info.output_width);
        sampleRow += ctx->info.output_width;
      }
    } else {
      if (ctx->info.out_color_space == JCS_CMYK) {
        // Convert from CMYK to RGB
        // We cannot convert directly to Cairo, as the CMSRGBTransform
        // may wants to do a RGB transform...
        // Would be better to have platform CMSenabled transformation
        // from CMYK to (A)RGB...
        _cmyk_convert_rgb((JSAMPROW)imageRow, ctx->info.output_width);
        sampleRow += ctx->info.output_width;
      }
    }

    // counter for while() loops below
    uint32_t idx = ctx->info.output_width;

    // copy as bytes until source pointer is 32-bit-aligned
    // TODO
    //for (; (NS_PTR_TO_UINT32(sampleRow) & 0x3) && idx; --idx) {
    for (; ((size_t)(sampleRow) & 0x3) && idx; --idx) {
      // TODO: writing.
      //*imageRow = gfxPackedPixel(0xFF, sampleRow[0], sampleRow[1], sampleRow[2]);
      imageRow++;
      sampleRow += 3;
    }

    // copy pixels in blocks of 4
    while (idx >= 4) {
      // TODO: writing.
      //GFX_BLOCK_RGB_TO_FRGB(sampleRow, imageRow);
      idx       -=  4;
      sampleRow += 12;
      imageRow  +=  4;
    }

    // copy remaining pixel(s)
    while (idx--) {
      // 32-bit read of final pixel will exceed buffer, so read bytes
      // TODO: writing.
      //*imageRow = gfxPackedPixel(0xFF, sampleRow[0], sampleRow[1], sampleRow[2]);
      imageRow++;
      sampleRow += 3;
    }
  }

  if (top != ctx->info.output_scanline) {
    // TODO: post invalidation (?).
  }
}

void gckimg_ns_jpeg_init(struct NSJpegDecoderCtx *ctx, int color_mgmt) {
  (void)color_mgmt; // TODO
  ctx->info.client_data = ctx;

  // We set up the normal JPEG error routines, then override error_exit.
  ctx->info.err = jpeg_std_error(&ctx->err_pub);
  ctx->err_pub.error_exit = my_error_exit;
  // Establish the setjmp return context for my_error_exit to use.
  if (setjmp(ctx->err_setjmp_buf)) {
    // If we got here, the JPEG code has signaled an error, and initialization
    // has failed.
    // TODO
    return;
  }

  // Step 1: allocate and initialize JPEG decompression object.
  jpeg_create_decompress(&ctx->info);
  // Set the source manager.
  ctx->info.src = &ctx->source;

  // Step 2: specify data source (eg, a file).

  // Setup callback functions.
  ctx->source.init_source = init_source;
  ctx->source.fill_input_buffer = fill_input_buffer;
  ctx->source.skip_input_data = skip_input_data;
  ctx->source.resync_to_restart = jpeg_resync_to_restart;
  ctx->source.term_source = term_source;

  // Record app markers for ICC data.
  for (uint32_t m = 0; m < 16; m++) {
    jpeg_save_markers(&ctx->info, JPEG_APP0 + m, 0xffff);
  }
}

void gckimg_ns_jpeg_cleanup(struct NSJpegDecoderCtx *ctx) {
  // Step 8: release JPEG decompression object.
  ctx->info.src = NULL;
  jpeg_destroy_decompress(&ctx->info);

  free(ctx->back_buffer);
  ctx->back_buffer = NULL;
}

void gckimg_ns_jpeg_decode(
    struct NSJpegDecoderCtx *ctx,
    struct ColorMgmtCtx *cm,
    const uint8_t *buf, size_t buf_len,
    void *writer, struct ImageWriterCallbacks callbacks)
{
  ctx->cm = cm;
  ctx->writer = writer;
  ctx->callbacks = callbacks;

  ctx->segment = (const JOCTET *)(buf);
  ctx->segment_len = buf_len;

  int mismatch;

  // Return here if there is a fatal error within libjpeg.
  // TODO: setjmp here for error handling.

  ctx->state = NS_JPEG_HEADER;
  switch (ctx->state) {
    case NS_JPEG_HEADER: {
      // Step 3: read file parameters with jpeg_read_header().
      if (jpeg_read_header(&ctx->info, TRUE) == JPEG_SUSPENDED) {
        // TODO: error.
        return;
      }

      // Post our size to the superclass.
      // TODO: depends on orientation from exif.
      struct ImageExifData exif;
      _ns_jpeg_read_orientation_from_exif(ctx, &exif);
      ctx->width = ctx->info.image_width;
      ctx->height = ctx->info.image_height;

      // We're doing a full decode.
      ctx->in_profile = _jpeg_get_icc_profile(&ctx->info);
      if (ctx->in_profile != NULL && ctx->color_mgmt) {
        uint32_t profile_space = qcms_profile_get_color_space(ctx->in_profile);
        mismatch = 0;

        switch (ctx->info.jpeg_color_space) {
          case JCS_GRAYSCALE:
            if (profile_space == icSigRgbData) {
              ctx->info.out_color_space = JCS_RGB;
            } else if (profile_space != icSigGrayData) {
              mismatch = 1;
            }
            break;
          case JCS_RGB:
            if (profile_space != icSigRgbData) {
              mismatch = 1;
            }
            break;
          case JCS_YCbCr:
            if (profile_space == icSigRgbData) {
              ctx->info.out_color_space = JCS_RGB;
            } else {
              // qcms doesn't support ycbcr
              mismatch = 1;
            }
            break;
          case JCS_CMYK:
          case JCS_YCCK:
            // qcms doesn't support cmyk
            mismatch = 1;
          default:
            // TODO: error.
            return;
        }

        if (!mismatch) {
          qcms_data_type in_type;
          switch (ctx->info.out_color_space) {
            case JCS_GRAYSCALE:
              in_type = QCMS_DATA_GRAY_8;
              break;
            case JCS_RGB:
              in_type = QCMS_DATA_RGB_8;
              break;
            default:
              // TODO: error.
              return;
          }

#if 0
          // We don't currently support CMYK profiles. The following
          // code dealt with lcms types. Add something like this
          // back when we gain support for CMYK.

          // Adobe Photoshop writes YCCK/CMYK files with inverted data
          if (mInfo.out_color_space == JCS_CMYK) {
            type |= FLAVOR_SH(mInfo.saw_Adobe_marker ? 1 : 0);
          }
#endif

          if (ctx->cm->out_profile != NULL) {
            // Calculate rendering intent.
            int intent = qcms_profile_get_rendering_intent(ctx->in_profile);

            // Create the color management transform.
            ctx->transform = qcms_transform_create(
                ctx->in_profile,
                in_type,
                ctx->cm->out_profile,
                QCMS_DATA_RGB_8,
                (qcms_intent)(intent));
          }
        } else {
          // TODO: ICM profile colorspace mismatch.
        }
      }

      if (ctx->transform == NULL) {
        switch (ctx->info.jpeg_color_space) {
          case JCS_GRAYSCALE:
          case JCS_RGB:
          case JCS_YCbCr:
            // if we're not color managing we can decode directly to
            // MOZ_JCS_EXT_NATIVE_ENDIAN_XRGB
            ctx->info.out_color_space = MOZ_JCS_EXT_NATIVE_ENDIAN_XRGB;
            ctx->info.out_color_components = 4;
            break;
          case JCS_CMYK:
          case JCS_YCCK:
            // libjpeg can convert from YCCK to CMYK, but not to RGB
            ctx->info.out_color_space = JCS_CMYK;
            break;
          default:
            // TODO: error.
            return;
        }
      }

      // TODO: allocate `imagebuf` here.
      jpeg_calc_output_dimensions(&ctx->info);
      // TODO

      ctx->state = NS_JPEG_START_DECOMPRESS;
      // Fallthrough.
    }

    case NS_JPEG_START_DECOMPRESS: {
      // Step 4: set parameters for decompression.

      // FIXME -- Should reset dct_method and dither mode
      // for final pass of progressive JPEG

      ctx->info.dct_method = JDCT_ISLOW;
      ctx->info.dither_mode = JDITHER_FS;
      ctx->info.do_fancy_upsampling = TRUE;
      ctx->info.enable_2pass_quant = FALSE;
      ctx->info.do_block_smoothing = TRUE;

      // Step 5: start decompressor.
      if (jpeg_start_decompress(&ctx->info) == FALSE) {
        // TODO: error.
        return;
      }

      // If this is a progressive JPEG ...
      ctx->state = ctx->info.buffered_image ? NS_JPEG_DECOMPRESS_PROGRESSIVE : NS_JPEG_DECOMPRESS_SEQUENTIAL;
      // Fallthrough.
    }

    case NS_JPEG_DECOMPRESS_SEQUENTIAL: {
      if (ctx->state == NS_JPEG_DECOMPRESS_SEQUENTIAL) {
        int suspend;
        _ns_jpeg_output_scanlines(ctx, &suspend);

        if (suspend) {
          // TODO: I/O suspension.
          return; // I/O suspension
        }

        assert(ctx->info.output_scanline == ctx->info.output_height
            && "We didn't process all of the data!");
        ctx->state = NS_JPEG_DONE;
      }
      // Fallthrough.
    }

    case NS_JPEG_DECOMPRESS_PROGRESSIVE: {
      if (ctx->state == NS_JPEG_DECOMPRESS_PROGRESSIVE) {
        int status;
        do {
          status = jpeg_consume_input(&ctx->info);
        } while (status != JPEG_SUSPENDED &&
                 status != JPEG_REACHED_EOI);

        for (;;) {
          if (ctx->info.output_scanline == 0) {
            int scan = ctx->info.input_scan_number;

            // if we haven't displayed anything yet (output_scan_number==0)
            // and we have enough data for a complete scan, force output
            // of the last full scan
            if ((ctx->info.output_scan_number == 0) &&
                (scan > 1) &&
                (status != JPEG_REACHED_EOI)) {
              scan--;
            }

            if (!jpeg_start_output(&ctx->info, scan)) {
              // TODO: I/O suspension.
              return; // I/O suspension
            }
          }

          if (ctx->info.output_scanline == 0xffffff) {
            ctx->info.output_scanline = 0;
          }

          int suspend;
          _ns_jpeg_output_scanlines(ctx, &suspend);

          if (suspend) {
            if (ctx->info.output_scanline == 0) {
              // didn't manage to read any lines - flag so we don't call
              // jpeg_start_output() multiple times for the same scan
              ctx->info.output_scanline = 0xffffff;
            }
            // TODO: I/O suspension.
            return; // I/O suspension
          }

          if (ctx->info.output_scanline == ctx->info.output_height) {
            if (!jpeg_finish_output(&ctx->info)) {
              // TODO: I/O suspension.
              return; // I/O suspension
            }

            if (jpeg_input_complete(&ctx->info) &&
                (ctx->info.input_scan_number == ctx->info.output_scan_number)) {
              break;
            }

            ctx->info.output_scanline = 0;
          }
        }

        ctx->state = NS_JPEG_DONE;
      }
      // Fallthrough.
    }

    case NS_JPEG_DONE: {
      // Step 7: finish decompression.
      if (jpeg_finish_decompress(&ctx->info) == FALSE) {
        // TODO: I/O suspension; this shouldn't happen.
        return;
      }
      // Make sure we don't feed any more data to libjpeg-turbo.
      ctx->state = NS_JPEG_SINK_NON_JPEG_TRAILER;
      // We're done.
      return;
    }

    case NS_JPEG_SINK_NON_JPEG_TRAILER: {
      assert(0 && "unreachable");
      return;
    }

    case NS_JPEG_ERROR: {
      assert(0 && "unreachable");
      return;
    }

    default:
      assert(0 && "unreachable");
      return;
  }
}
