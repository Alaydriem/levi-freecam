#pragma once

// exported for other mods: true while freecam controls the camera, from turning on until the controller is fully off;
// reads one atomic flag, safe to call from any thread every frame
extern "C" __declspec(dllexport) bool alaydriem_freecam_is_active();
