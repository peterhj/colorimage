#ifndef __GCKIMG_EXIF_H__
#define __GCKIMG_EXIF_H__

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

#endif
