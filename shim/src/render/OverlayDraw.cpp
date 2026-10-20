#include "render/OverlayDraw.h"


#include <imgui.h>

#include <algorithm>
#include <cmath>
#include <cstdio>

namespace levi_freecam {

namespace {

constexpr ImU32 PATH_YELLOW   = IM_COL32(0xFF, 0xD2, 0x3F, 0xFF);
constexpr ImU32 PATH_SHADOW   = IM_COL32(0x00, 0x00, 0x00, 0x59);
constexpr ImU32 WEDGE_COLOUR  = IM_COL32(0xFF, 0xD2, 0x3F, 0x73);
constexpr ImU32 LAST_MARKER   = IM_COL32(0xFF, 0xFF, 0xFF, 0xFF);
constexpr ImU32 OUTLINE       = IM_COL32(0x00, 0x00, 0x00, 0xFF);
constexpr ImU32 MARKER_TEXT   = IM_COL32(0x00, 0x00, 0x00, 0xFF);
constexpr ImU32 PANEL_FILL    = IM_COL32(0x0B, 0x0B, 0x0F, 0xB8);
constexpr ImU32 PANEL_BORDER  = IM_COL32(0xFF, 0xD2, 0x3F, 0x66);
constexpr ImU32 PANEL_TEXT    = IM_COL32(0xFF, 0xFF, 0xFF, 0xE6);
constexpr float PATH_WIDTH    = 3.0f;
constexpr float SHADOW_WIDTH  = 6.0f;
constexpr float DASH_ON       = 10.0f;
constexpr float DASH_PERIOD   = 16.0f;
// a longer segment, from a point near the camera plane, is drawn solid rather than walked dash by dash
constexpr float MAX_DASHED_LENGTH = 4096.0f;
constexpr float WEDGE_WIDTH       = 4.0f;
constexpr float MARKER_RADIUS     = 14.0f;
constexpr float MARKER_OUTLINE    = 2.0f;
constexpr int   MARKER_SEGMENTS   = 24;
constexpr float PANEL_MARGIN      = 12.0f;
constexpr float PANEL_PADDING     = 8.0f;
constexpr float PANEL_ROUNDING    = 6.0f;
constexpr float LEGEND_COLUMN_GAP = 12.0f;
constexpr float LEGEND_ROW_GAP    = 3.0f;
constexpr char  STRIP_TITLE[]     = "FREECAM";

constexpr char const* LEGEND_KEYS[]   = {"Ctrl+B", "Ctrl+Z", "Ctrl+Enter", "Ctrl+P"};
constexpr char const* LEGEND_LABELS[] = {"add point", "undo", "play / stop", "exit"};
constexpr int         LEGEND_ROWS     = 4;

ImVec2 lerp(ImVec2 a, ImVec2 b, float t) { return ImVec2{a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t}; }

// phase is the distance into the dash pattern at a, in [0, DASH_PERIOD); it carries over from the segment before so
// dashes run on. The loop counts whole periods, so it ends after at most length / DASH_PERIOD + 2 passes whatever the
// float rounding
void drawDashes(ImDrawList& list, ImVec2 a, ImVec2 b, float& phase) {
    float length = std::hypot(b.x - a.x, b.y - a.y);
    if (!(length > 0.0f)) return;
    if (length > MAX_DASHED_LENGTH) {
        list.AddLine(a, b, PATH_YELLOW, PATH_WIDTH);
        return;
    }
    int periods = static_cast<int>(length / DASH_PERIOD) + 2;
    for (int period = 0; period < periods; ++period) {
        float dashStart = static_cast<float>(period) * DASH_PERIOD - phase;
        if (dashStart >= length) break;
        float from = std::max(0.0f, dashStart);
        float to   = std::min(length, dashStart + DASH_ON);
        if (to > from) list.AddLine(lerp(a, b, from / length), lerp(a, b, to / length), PATH_YELLOW, PATH_WIDTH);
    }
    phase = std::fmod(phase + length, DASH_PERIOD);
}

void drawPath(ImDrawList& list, FcOverlaySegment const* segments, uint32_t count) {
    for (uint32_t i = 0; i < count; ++i) {
        list.AddLine(ImVec2{segments[i].x0, segments[i].y0}, ImVec2{segments[i].x1, segments[i].y1}, PATH_SHADOW, SHADOW_WIDTH);
    }
    float phase = 0.0f;
    for (uint32_t i = 0; i < count; ++i) {
        FcOverlaySegment const& segment = segments[i];
        bool joined = i > 0 && segments[i - 1].x1 == segment.x0 && segments[i - 1].y1 == segment.y0;
        if (!joined) phase = 0.0f;
        drawDashes(list, ImVec2{segment.x0, segment.y0}, ImVec2{segment.x1, segment.y1}, phase);
    }
}

void drawMarker(ImDrawList& list, FcOverlayMarker const& marker) {
    ImVec2 centre{marker.x, marker.y};
    list.AddCircleFilled(centre, MARKER_RADIUS, marker.is_last ? LAST_MARKER : PATH_YELLOW, MARKER_SEGMENTS);
    list.AddCircle(centre, MARKER_RADIUS, OUTLINE, MARKER_SEGMENTS, MARKER_OUTLINE);
    char number[16];
    std::snprintf(number, sizeof(number), "%u", marker.number);
    ImVec2 size = ImGui::CalcTextSize(number);
    list.AddText(ImVec2{centre.x - size.x * 0.5f, centre.y - size.y * 0.5f}, MARKER_TEXT, number);
}

// every wedge goes under every marker
void drawMarkers(ImDrawList& list, FcOverlayMarker const* markers, uint32_t count) {
    for (uint32_t i = 0; i < count; ++i) {
        list.AddLine(ImVec2{markers[i].x, markers[i].y}, ImVec2{markers[i].wedge_x, markers[i].wedge_y}, WEDGE_COLOUR, WEDGE_WIDTH);
    }
    for (uint32_t i = 0; i < count; ++i) drawMarker(list, markers[i]);
}

void drawPanel(ImDrawList& list, ImVec2 min, ImVec2 max) {
    list.AddRectFilled(min, max, PANEL_FILL, PANEL_ROUNDING);
    list.AddRect(min, max, PANEL_BORDER, PANEL_ROUNDING);
}

// "FREECAM, PATH, {point_count} points, {shot_seconds} s" joined by middle dots, at the top left; returns the strip's bottom edge
float drawStrip(ImDrawList& list, uint32_t pointCount, float shotSeconds) {
    char detail[96];
    std::snprintf(
        detail,
        sizeof(detail),
        " \xC2\xB7 PATH \xC2\xB7 %u %s \xC2\xB7 %.1f s",
        pointCount,
        pointCount == 1 ? "point" : "points",
        shotSeconds
    );
    ImVec2 titleSize  = ImGui::CalcTextSize(STRIP_TITLE);
    ImVec2 detailSize = ImGui::CalcTextSize(detail);
    ImVec2 min{PANEL_MARGIN, PANEL_MARGIN};
    ImVec2 max{min.x + titleSize.x + detailSize.x + 2.0f * PANEL_PADDING, min.y + titleSize.y + 2.0f * PANEL_PADDING};
    drawPanel(list, min, max);
    ImVec2 text{min.x + PANEL_PADDING, min.y + PANEL_PADDING};
    list.AddText(text, PATH_YELLOW, STRIP_TITLE);
    list.AddText(ImVec2{text.x + titleSize.x, text.y}, PANEL_TEXT, detail);
    return max.y;
}

// the key column in yellow and the action column beside it, at the left, directly below the strip
void drawLegend(ImDrawList& list, float top) {
    float keyWidth   = 0.0f;
    float labelWidth = 0.0f;
    for (int i = 0; i < LEGEND_ROWS; ++i) {
        keyWidth   = std::max(keyWidth, ImGui::CalcTextSize(LEGEND_KEYS[i]).x);
        labelWidth = std::max(labelWidth, ImGui::CalcTextSize(LEGEND_LABELS[i]).x);
    }
    float  rowHeight = ImGui::GetTextLineHeight();
    float  width     = keyWidth + LEGEND_COLUMN_GAP + labelWidth + 2.0f * PANEL_PADDING;
    float  height    = LEGEND_ROWS * rowHeight + (LEGEND_ROWS - 1) * LEGEND_ROW_GAP + 2.0f * PANEL_PADDING;
    ImVec2 min{PANEL_MARGIN, top};
    ImVec2 max{min.x + width, top + height};
    drawPanel(list, min, max);
    for (int i = 0; i < LEGEND_ROWS; ++i) {
        float y = min.y + PANEL_PADDING + i * (rowHeight + LEGEND_ROW_GAP);
        list.AddText(ImVec2{min.x + PANEL_PADDING, y}, PATH_YELLOW, LEGEND_KEYS[i]);
        list.AddText(ImVec2{min.x + PANEL_PADDING + keyWidth + LEGEND_COLUMN_GAP, y}, PANEL_TEXT, LEGEND_LABELS[i]);
    }
}

} // namespace

void OverlayDraw::draw(ImDrawList& list, const FcOverlayView& view) {
    if (view.segments) drawPath(list, view.segments, view.segment_count);
    if (view.markers) drawMarkers(list, view.markers, view.marker_count);
    float stripBottom = drawStrip(list, view.point_count, view.shot_seconds);
    drawLegend(list, stripBottom + PANEL_MARGIN);
}

} // namespace levi_freecam
