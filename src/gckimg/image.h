#ifndef __WRAPPED_COMMON_H__
#define __WRAPPED_COMMON_H__

#include <stdint.h>
#include <stdlib.h>

struct ImageWriterCallbacks {
  void (*init_size)(void *, size_t, size_t, size_t);
  void (*write_row)(void *, size_t, const uint8_t *, size_t);
};

#endif
