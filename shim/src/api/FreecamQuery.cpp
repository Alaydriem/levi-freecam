#include "api/FreecamQuery.h"

#include "frame/FrameDriver.h"

extern "C" bool alaydriem_freecam_is_active() { return levi_freecam::FrameDriver::cameraOwned(); }
