#pragma once

namespace levi_freecam {

// stamps each flight key-down for the key-to-move telemetry, as the first listener of the key event; it never cancels
// an event, and Rust ignores the stamps while telemetry is off
class LatencyProbe {
public:
    static void install();
    static void uninstall();
};

} // namespace levi_freecam
