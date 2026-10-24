#pragma once

#include "levi_freecam.h"

#include <cstddef>
#include <cstdint>

namespace levi_freecam {

// turns Rust's marker entries into one PrimitiveShapesPacket and hands it to the captured client packet handler
class ShapeDelivery {
public:
    // the handler generation for this frame's status: 0 while no handler is captured, or when the caller is not on
    // the thread that captured the handler
    static uint32_t generation();

    // delivers the entries in order in one packet; false when nothing was delivered (no entries, no handler, another
    // thread, or no packet)
    static bool deliver(const FcMarkerEntry* entries, size_t count);

    static void reset();
};

} // namespace levi_freecam
