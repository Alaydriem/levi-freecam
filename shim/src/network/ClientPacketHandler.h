#pragma once

#include <cstdint>

class CameraInstructionPacket;
class ClientNetworkHandler;
class LocalPlayer;
class NetworkIdentifier;
class PrimitiveShapesPacket;

namespace levi_freecam {

// the one captured client packet handler, shared by every local delivery; nothing reaches the server
class ClientPacketHandler {
public:
    // a new handler pointer, or the first capture after a drop, raises the generation by one
    static void capture(ClientNetworkHandler* handler, NetworkIdentifier const& id);

    // when the local player goes away, drops the captured handler and id so a stale pointer is never used
    static void observeLocalPlayer(LocalPlayer const* player);

    static bool captured();

    // the number of captures so far while a handler is captured, 0 while none is
    static uint32_t generation();

    // the id of the thread that last captured the handler, 0 before the first capture
    static uint32_t captureThread();

    // hands the packet to the captured handler; false when none is captured
    static bool deliver(CameraInstructionPacket const& packet);

    // hands the packet to the captured handler; false when none is captured
    static bool deliver(PrimitiveShapesPacket const& packet);

    static void reset();
};

} // namespace levi_freecam
