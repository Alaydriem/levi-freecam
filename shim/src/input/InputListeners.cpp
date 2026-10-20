#include "input/InputListeners.h"

#include "bridge/RustBridge.h"

#include <ll/api/event/EventBus.h>
#include <ll/api/event/input/KeyInputEvent.h>
#include <ll/api/event/input/MouseInputEvent.h>

namespace levi_freecam {

namespace {

constexpr int MOUSE_MOVEMENT = 0;

ll::event::ListenerPtr g_keyListener;
ll::event::ListenerPtr g_mouseListener;

void onKey(ll::event::KeyInputEvent& event) {
    const FcHandle* handle = RustBridge::handle();
    if (!handle) return;
    uint8_t consume = 0;
    auto    code    = static_cast<uint32_t>(event.keyCode());
    if (fc_key(handle, code, event.isDown() ? 1 : 0, &consume) != FC_STATUS_OK) return;
    if (consume) event.cancel();
}

void onMouse(ll::event::MouseInputEvent& event) {
    const FcHandle* handle = RustBridge::handle();
    if (!handle) return;
    int button = static_cast<int>(event.actionButtonId());
    // movement always reaches the game, so it still produces the turn delta the hook blocks
    if (button == MOUSE_MOVEMENT) {
        fc_mouse_move(handle, event.dx(), event.dy());
        return;
    }
    uint8_t consume = 0;
    auto    data    = static_cast<int8_t>(event.buttonData());
    if (fc_mouse_button(handle, static_cast<uint8_t>(button), data, &consume) != FC_STATUS_OK) return;
    if (consume) event.cancel();
}

} // namespace

void InputListeners::registerListeners() {
    auto& bus       = ll::event::EventBus::getInstance();
    g_keyListener   = bus.emplaceListener<ll::event::KeyInputEvent>(&onKey);
    g_mouseListener = bus.emplaceListener<ll::event::MouseInputEvent>(&onMouse);
}

void InputListeners::unregisterListeners() {
    auto& bus = ll::event::EventBus::getInstance();
    if (g_keyListener) {
        bus.removeListener<ll::event::KeyInputEvent>(g_keyListener);
        g_keyListener = nullptr;
    }
    if (g_mouseListener) {
        bus.removeListener<ll::event::MouseInputEvent>(g_mouseListener);
        g_mouseListener = nullptr;
    }
}

} // namespace levi_freecam
