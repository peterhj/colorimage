/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef __WRAPPED_NS_PNG_DECODER_H__
#define __WRAPPED_NS_PNG_DECODER_H__

#include "common.h"
#include "qcms/qcms.h"

#include "png.h"

#include <stdint.h>
#include <stdlib.h>

struct NSPngDecoderCtx {
  png_structp png;
  png_infop info;
  uint32_t width;
  uint32_t height;
  uint32_t channels;
  int color_mgmt;
  qcms_profile *profile;
  qcms_transform *transform;
  uint8_t *cms_line;
  uint8_t *interlace_buf;
  void *writer;
  struct RasterWriterCallbacks callbacks;
};

void wrapped_ns_png_init(struct NSPngDecoderCtx *ctx, int color_mgmt);
void wrapped_ns_png_cleanup(struct NSPngDecoderCtx *ctx);
void wrapped_ns_png_decode(struct NSPngDecoderCtx *ctx, const uint8_t *buf, size_t buf_len, void *writer, struct RasterWriterCallbacks callbacks);

#endif
