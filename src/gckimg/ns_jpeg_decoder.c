/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "ns_jpeg_decoder.h"
#include "image.h"

#include <assert.h>
#include <setjmp.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#include "jpeglib.h"

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
    const uint8_t *buf, size_t buf_len,
    void *writer, struct ImageWriterCallbacks callbacks)
{
  // TODO
  (void)writer;
  (void)callbacks;

  ctx->segment = (const JOCTET *)(buf);
  ctx->segment_len = buf_len;

  jpeg_saved_marker_ptr marker;
  int mismatch;

  // Return here if there is a fatal error within libjpeg.
  // TODO: setjmp here for error handling.

  ctx->state = NS_JPEG_HEADER;
  switch (ctx->state) {
    case NS_JPEG_HEADER: {
      // Step 3: read file parameters with jpeg_read_header().
      if (jpeg_read_header(&ctx->info, TRUE) == JPEG_SUSPENDED) {
      }

      // TODO: read orientation from exif.
      for (marker = ctx->info.marker_list; marker != NULL; marker = marker->next) {
        if (marker->marker == JPEG_APP0 + 1) {
          break;
        }
      }
      if (!marker) {
        // TODO: no orientation.
      } else {
        // TODO: read orientation from exif parser.
      }

      // TODO: We're doing a full decode.
      mismatch = 0;
      switch (ctx->info.jpeg_color_space) {
        case JCS_GRAYSCALE:
          break;
        case JCS_RGB:
          break;
        case JCS_YCbCr:
          break;
        case JCS_CMYK:
        case JCS_YCCK:
          break;
        default:
          break;
      }
      if (!mismatch) {
        // TODO
      }

      // TODO

      ctx->state = NS_JPEG_START_DECOMPRESS;
      // Fallthrough.
    }

    case NS_JPEG_START_DECOMPRESS: {
      // Step 4: set parameters for decompression.

      // Step 5: start decompressor.

      // TODO

      // If this is a progressive JPEG ...
      ctx->state = ctx->info.buffered_image ? NS_JPEG_DECOMPRESS_PROGRESSIVE : NS_JPEG_DECOMPRESS_SEQUENTIAL;
      // Fallthrough.
    }

    case NS_JPEG_DECOMPRESS_SEQUENTIAL: {
      if (ctx->state == NS_JPEG_DECOMPRESS_SEQUENTIAL) {
        // TODO

        ctx->state = NS_JPEG_DONE;
      }
      // Fallthrough.
    }

    case NS_JPEG_DECOMPRESS_PROGRESSIVE: {
      if (ctx->state == NS_JPEG_DECOMPRESS_PROGRESSIVE) {
        // TODO

        ctx->state = NS_JPEG_DONE;
      }
      // Fallthrough.
    }

    case NS_JPEG_DONE: {
      // Step 7: finish decompression.
      if (jpeg_finish_decompress(&ctx->info) == FALSE) {
        // TODO: this shouldn't happen.
        return;
      }
      // Make sure we don't feed any more data to libjpeg-turbo.
      ctx->state = NS_JPEG_SINK_NON_JPEG_TRAILER;
      // We're done.
      return;
    }

    case NS_JPEG_SINK_NON_JPEG_TRAILER: {
      assert(0 && "unreachable");
    }

    case NS_JPEG_ERROR: {
      assert(0 && "unreachable");
    }

    default:
      assert(0 && "unreachable");
      return;
  }
}
