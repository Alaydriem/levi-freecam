#include "input/MouseGrab.h"

#include "frame/FrameDriver.h"

#include <ll/api/service/TargetedBedrock.h>
#include <mc/client/game/ClientInstance.h>
#include <mc/client/game/IMinecraftGame.h>

namespace levi_freecam {

namespace {

constexpr uint8_t MOUSE_RELEASE      = 1;
constexpr uint8_t MOUSE_GRAB         = 2;
constexpr uint8_t MOUSE_DRAG_GRAB    = 3;
constexpr uint8_t MOUSE_DRAG_RELEASE = 4;

// game thread only: the cursor position, in screen pixels, when the look drag started
POINT gDragCursor{};
bool  gDragCursorKept = false;

void dragGrab(ClientInstance& client) {
    gDragCursorKept = GetCursorPos(&gDragCursor) != FALSE;
    client.getMinecraftGame().grabMouse();
}

void dragRelease(ClientInstance& client) {
    client.getMinecraftGame().releaseMouse();
    if (gDragCursorKept) SetCursorPos(gDragCursor.x, gDragCursor.y);
    gDragCursorKept = false;
}

} // namespace

void MouseGrab::apply(ClientInstance& client, uint8_t command) {
    if (command == MOUSE_RELEASE) client.getMinecraftGame().releaseMouse();
    if (command == MOUSE_GRAB) client.getMinecraftGame().grabMouse();
    if (command == MOUSE_DRAG_GRAB) dragGrab(client);
    if (command == MOUSE_DRAG_RELEASE) dragRelease(client);
}

bool MouseGrab::blocksGrab() { return FrameDriver::cursorMode() && !FrameDriver::lookDrag(); }

void MouseGrab::restoreOnDisable(HWND gameWindow) {
    if (!gameWindow || GetForegroundWindow() != gameWindow) return;
    auto client = ll::service::getClientInstance();
    if (!client) return;
    if (!client->getLocalPlayer() || !client->isWorldActive() || client->isShowingMenu()) return;
    apply(*client, MOUSE_GRAB);
}

} // namespace levi_freecam
