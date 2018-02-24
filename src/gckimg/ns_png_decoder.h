/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef __GCKIMG_NS_PNG_DECODER_H__
#define __GCKIMG_NS_PNG_DECODER_H__

#include "color_mgmt.h"
#include "image.h"
#include "qcms/qcms.h"

#include "png.h"

#include <stdint.h>
#include <stdlib.h>

struct NSPngDecoderCtx {
  png_structp png;
  png_infop info;
  uint32_t width;
  uint32_t height;
  uint32_t depth;
  uint32_t channels;
  qcms_profile *in_profile;
  qcms_transform *transform;
  int pass;
  uint8_t *cms_line;
  uint8_t *interlace_buf;
  int errorcode;
  int color_mgmt;
  struct ColorMgmtCtx *cm;
  void *writer;
  struct ImageWriterCallbacks callbacks;
};

void gckimg_ns_png_init(struct NSPngDecoderCtx *ctx, int color_mgmt);
void gckimg_ns_png_cleanup(struct NSPngDecoderCtx *ctx);
void gckimg_ns_png_decode(
    struct NSPngDecoderCtx *ctx,
    struct ColorMgmtCtx *cm,
    const uint8_t *buf, size_t buf_len,
    void *writer, struct ImageWriterCallbacks callbacks);

#endif
