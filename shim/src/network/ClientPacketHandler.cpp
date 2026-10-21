#include "network/ClientPacketHandler.h"
#include "bridge/ModLog.h"

#include "mod/FreecamMod.h"

#include <windows.h>

#include <mc/client/network/ClientNetworkHandler.h>
#include <mc/network/NetworkIdentifier.h>
#include <mc/network/packet/CameraInstructionPacket.h>
#include <mc/network/packet/PrimitiveShapeDataPayload.h>
#include <mc/network/packet/PrimitiveShapesPacket.h>

#include <mutex>
#include <optional>

namespace levi_freecam {

namespace {

struct Capture {
    std::recursive_mutex             mutex;
    ClientNetworkHandler*            handler = nullptr;
    std::optional<NetworkIdentifier> id;
    bool                             hadPlayer = false;
    uint32_t                         captures  = 0;
    uint32_t                         thread    = 0;
};

Capture& captureState() {
    static Capture instance;
    return instance;
}

void dropHandler(Capture& state) {
    state.handler = nullptr;
    state.id.reset();
}

// the lock is held while the game handles the packet, so a drop cannot free the handler mid-call; a re-entry from
// another handler hook on the same thread takes the recursive lock again
template <class PacketType>
bool deliverPacket(PacketType const& packet) {
    auto&           state = captureState();
    std::lock_guard lock(state.mutex);
    if (!state.handler || !state.id) return false;
    state.handler->handle(*state.id, packet);
    return true;
}

} // namespace

void ClientPacketHandler::capture(ClientNetworkHandler* handler, NetworkIdentifier const& id) {
    auto&           state = captureState();
    std::lock_guard lock(state.mutex);
    if (state.handler != handler) {
        ++state.captures;
        state.thread = static_cast<uint32_t>(GetCurrentThreadId());
        ModLog::info(
            "captured client packet handler: generation {} on thread {}",
            state.captures,
            state.thread
        );
    }
    state.handler = handler;
    state.id      = id;
}

void ClientPacketHandler::observeLocalPlayer(LocalPlayer const* player) {
    auto&           state = captureState();
    std::lock_guard lock(state.mutex);
    bool            present = player != nullptr;
    if (state.hadPlayer && !present) dropHandler(state);
    state.hadPlayer = present;
}

bool ClientPacketHandler::captured() {
    auto&           state = captureState();
    std::lock_guard lock(state.mutex);
    return state.handler != nullptr && state.id.has_value();
}

uint32_t ClientPacketHandler::generation() {
    auto&           state = captureState();
    std::lock_guard lock(state.mutex);
    return state.handler != nullptr && state.id.has_value() ? state.captures : 0;
}

uint32_t ClientPacketHandler::captureThread() {
    auto&           state = captureState();
    std::lock_guard lock(state.mutex);
    return state.thread;
}

bool ClientPacketHandler::deliver(CameraInstructionPacket const& packet) { return deliverPacket(packet); }

bool ClientPacketHandler::deliver(PrimitiveShapesPacket const& packet) { return deliverPacket(packet); }

void ClientPacketHandler::reset() {
    auto&           state = captureState();
    std::lock_guard lock(state.mutex);
    dropHandler(state);
    state.hadPlayer = false;
}

} // namespace levi_freecam
