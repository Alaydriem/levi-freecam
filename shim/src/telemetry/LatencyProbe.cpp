#include "telemetry/LatencyProbe.h"

#include "bridge/RustBridge.h"

#include <ll/api/event/EventBus.h>
#include <ll/api/event/input/KeyInputEvent.h>

#include <windows.h>

#include <array>
#include <cstdint>

namespace levi_freecam {

namespace {

// W, A, S, D, Space, Shift: the default flight keys
constexpr std::array<uint32_t, 6> FLIGHT_KEYS{0x57, 0x41, 0x53, 0x44, 0x20, 0x10};

ll::event::ListenerPtr g_listener;
// whether each flight key is down, so an auto-repeat is not stamped as a new press
std::array<bool, FLIGHT_KEYS.size()> g_down{};

// the same QueryPerformanceCounter clock as FcFrameStatus::time_seconds
double nowSeconds() {
    static const LONGLONG frequency = [] {
        LARGE_INTEGER value;
        QueryPerformanceFrequency(&value);
        return value.QuadPart;
    }();
    LARGE_INTEGER now;
    QueryPerformanceCounter(&now);
    return static_cast<double>(now.QuadPart) / static_cast<double>(frequency);
}

void onKey(ll::event::KeyInputEvent& event) {
    auto code = static_cast<uint32_t>(event.keyCode());
    for (size_t i = 0; i < FLIGHT_KEYS.size(); ++i) {
        if (FLIGHT_KEYS[i] != code) continue;
        bool wasDown = g_down[i];
        g_down[i]    = event.isDown();
        const FcHandle* handle = RustBridge::handle();
        if (handle && event.isDown() && !wasDown) fc_probe_key(handle, nowSeconds());
        return;
    }
}

} // namespace

void LatencyProbe::install() {
    g_down     = {};
    // first of all listeners: the mod manager cancels the keys it routes, and a cancelled key reaches no later listener
    g_listener = ll::event::EventBus::getInstance().emplaceListener<ll::event::KeyInputEvent>(
        &onKey,
        ll::event::EventPriority::Highest
    );
}

void LatencyProbe::uninstall() {
    if (!g_listener) return;
    ll::event::EventBus::getInstance().removeListener<ll::event::KeyInputEvent>(g_listener);
    g_listener = nullptr;
}

} // namespace levi_freecam
