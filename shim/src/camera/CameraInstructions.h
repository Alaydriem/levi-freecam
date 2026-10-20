#pragma once

#include "levi_freecam.h"

class ClientNetworkHandler;
class Level;
class LocalPlayer;
class NetworkIdentifier;

namespace levi_freecam {

// builds camera instructions and hands them to the captured client packet handler; nothing reaches the server
class CameraInstructions {
public:
    static void capture(ClientNetworkHandler* handler, NetworkIdentifier const& id);

    // when the local player goes away, drops the captured handler and id so a stale pointer is never used
    static void observeLocalPlayer(LocalPlayer const* player);

    // true when a handler is captured and the level's preset list has minecraft:free; keeps that preset's index
    static bool ready(Level& level);

    // refused without a local player or a captured handler
    static bool sendSet(LocalPlayer const* player, const FcPose& pose);

    // refused without a local player or a captured handler
    static bool sendClear(LocalPlayer const* player);

    // true on the thread that is delivering one of the mod's own instructions
    static bool isDelivering();

    static void reset();
};

} // namespace levi_freecam
