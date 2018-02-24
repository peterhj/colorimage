#ifndef __WRAPPED_COLOR_MGMT_H__
#define __WRAPPED_COLOR_MGMT_H__

#include "qcms/qcms.h"

struct ColorMgmtCtx {
  qcms_profile *out_profile;
};

void color_mgmt_init_default(struct ColorMgmtCtx *color_mgmt);

#endif
