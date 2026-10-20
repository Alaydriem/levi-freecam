#pragma once

#include "levi_freecam.h"

#include <optional>

class LocalPlayer;

namespace levi_freecam {

// casts one ray through the local player's dimension and returns the nearest block or actor hit; read-only
class LookRay {
public:
    // direction is a unit vector; max_distance is in blocks. No hit, or no usable input, gives nothing
    static std::optional<FcVec3> cast(LocalPlayer& player, FcVec3 origin, FcVec3 direction, float maxDistance);
};

} // namespace levi_freecam
