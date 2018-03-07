#ifndef __GCKIMG_IMAGE_H__
#define __GCKIMG_IMAGE_H__

#include <stdint.h>
#include <stdlib.h>

struct ImageExifData;

struct ImageWriterCallbacks {
  void (*init_size)(void *, size_t, size_t, size_t);
  void (*write_row)(void *, size_t, const uint8_t *, size_t);
  //void (*write_row_rgbx)(void *, size_t, const uint8_t *, size_t);
  int (*parse_exif)(const uint8_t *, size_t);
};

#endif
