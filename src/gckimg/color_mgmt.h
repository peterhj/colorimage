#ifndef __GCKIMG_COLOR_MGMT_H__
#define __GCKIMG_COLOR_MGMT_H__

#include "qcms/qcms.h"

struct ColorMgmtCtx {
  qcms_profile *out_profile;
};

void gckimg_color_mgmt_init_default(struct ColorMgmtCtx *color_mgmt);
void gckimg_color_mgmt_cleanup(struct ColorMgmtCtx *color_mgmt);

#endif
