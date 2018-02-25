/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef __GCKIMG_NS_JPEG_DECODER_H__
#define __GCKIMG_NS_JPEG_DECODER_H__

#include "color_mgmt.h"
#include "image.h"
#include "qcms/qcms.h"

#include <setjmp.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

// NOTE: Include this last.
#include "jpeglib.h"

typedef enum {
  NS_JPEG_HEADER = 0,
  NS_JPEG_START_DECOMPRESS,
  NS_JPEG_DECOMPRESS_PROGRESSIVE,
  NS_JPEG_DECOMPRESS_SEQUENTIAL,
  NS_JPEG_DONE,
  NS_JPEG_SINK_NON_JPEG_TRAILER,
  NS_JPEG_ERROR
} NSJpegState;

struct NSJpegDecoderCtx {
  struct jpeg_decompress_struct info;
  struct jpeg_source_mgr source;
  struct jpeg_error_mgr err_pub;
  jmp_buf err_setjmp_buf;
  const JOCTET *segment;
  uint32_t segment_len;
  JOCTET *back_buffer;
  uint32_t back_buffer_len;
  uint32_t back_buffer_size;
  uint32_t back_buffer_unread_len;
  JOCTET *profile;
  uint32_t profile_len;
  // TODO
  uint8_t *imagebuf;
  uint32_t width;
  uint32_t height;
  NSJpegState state;
  qcms_profile *in_profile;
  qcms_transform *transform;
  int reading;
  int errorcode;
  int color_mgmt;
  struct ColorMgmtCtx *cm;
  void *writer;
  struct ImageWriterCallbacks callbacks;
};

void gckimg_ns_jpeg_init(struct NSJpegDecoderCtx *ctx, int color_mgmt);
void gckimg_ns_jpeg_cleanup(struct NSJpegDecoderCtx *ctx);
void gckimg_ns_jpeg_decode(struct NSJpegDecoderCtx *ctx, const uint8_t *buf, size_t buf_len, void *writer, struct ImageWriterCallbacks callbacks);

#endif
