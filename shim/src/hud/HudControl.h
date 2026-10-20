#pragma once

#include <cstdint>

class ClientInstance;

namespace levi_freecam {

// hides or shows the HUD, hand and crosshair when Rust asks
class HudControl {
public:
    // gui: 0 none, 1 hide, 2 show
    static void apply(ClientInstance& client, uint8_t gui);
};

} // namespace levi_freecam
