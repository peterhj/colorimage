#include "color_mgmt.h"
#include "qcms/qcms.h"

void gckimg_color_mgmt_init_default(struct ColorMgmtCtx *color_mgmt) {
  color_mgmt->out_profile = qcms_profile_sRGB();
}

void gckimg_color_mgmt_cleanup(struct ColorMgmtCtx *color_mgmt) {
  if (color_mgmt->out_profile != NULL) {
    qcms_profile_release(color_mgmt->out_profile);
  }
}
