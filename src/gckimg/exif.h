/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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
