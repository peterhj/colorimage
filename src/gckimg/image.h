#ifndef __GCKIMG_IMAGE_H__
#define __GCKIMG_IMAGE_H__

#include <stdint.h>
#include <stdlib.h>

enum Angle {
  D0,
  D90,
  D180,
  D270
};

enum Flip {
  Unflipped,
  Horizontal
};

struct ImageExifData {
  enum Angle  orientation_rotation;
  enum Flip   orientation_flip;
};

struct ImageWriterCallbacks {
  void (*init_size)(void *, size_t, size_t, size_t);
  void (*write_row)(void *, size_t, const uint8_t *, size_t);
  void (*parse_exif)(const uint8_t *, size_t, struct ImageExifData *);
};

#endif
