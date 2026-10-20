#pragma once

#include "levi_freecam.h"

struct ImDrawList;

namespace levi_freecam {

// fills the overlay's draw list once per Present, inside freecam's ImGui frame, from a view the caller got
// from fc_overlay and checked visible; the view is valid only within that Present
class OverlayDraw {
public:
    static void draw(ImDrawList& list, const FcOverlayView& view);
};

} // namespace levi_freecam
