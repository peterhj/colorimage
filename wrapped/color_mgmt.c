#include "color_mgmt.h"
#include "qcms/qcms.h"

void color_mgmt_init_default(struct ColorMgmtCtx *color_mgmt) {
  color_mgmt->out_profile = qcms_profile_sRGB();
}
