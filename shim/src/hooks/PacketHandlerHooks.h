#pragma once

#include "bridge/RustBridge.h"
#include "camera/CameraInstructions.h"

#include <ll/api/memory/Hook.h>
#include <mc/client/network/ClientNetworkHandler.h>
#include <mc/network/NetworkIdentifier.h>
#include <mc/world/level/ChunkPos.h>
#include <mc/network/packet/CameraInstructionPacket.h>
#include <mc/network/packet/CameraPresetsPacket.h>
#include <mc/network/packet/NetworkChunkPublisherUpdatePacket.h>

namespace levi_freecam {

// a packet the server always sends: captures the client handler and its NetworkIdentifier
LL_TYPE_INSTANCE_HOOK(
    ChunkPublisherHook,
    ll::memory::HookPriority::Normal,
    ClientNetworkHandler,
    &ClientNetworkHandler::$handle,
    void,
    NetworkIdentifier const&                 id,
    NetworkChunkPublisherUpdatePacket const& packet
) {
    CameraInstructions::capture(this, id);
    origin(id, packet);
}

// a packet the server always sends: captures the client handler and its NetworkIdentifier
LL_TYPE_INSTANCE_HOOK(
    CameraPresetsHook,
    ll::memory::HookPriority::Normal,
    ClientNetworkHandler,
    &ClientNetworkHandler::$handle,
    void,
    NetworkIdentifier const&   id,
    CameraPresetsPacket const& packet
) {
    CameraInstructions::capture(this, id);
    origin(id, packet);
}

// every camera instruction the client handles; the server's are reported to Rust, the mod's own are not
LL_TYPE_INSTANCE_HOOK(
    CameraInstructionHook,
    ll::memory::HookPriority::Normal,
    ClientNetworkHandler,
    &ClientNetworkHandler::$handle,
    void,
    NetworkIdentifier const&       id,
    CameraInstructionPacket const& packet
) {
    const FcHandle* handle = RustBridge::handle();
    if (handle && !CameraInstructions::isDelivering()) {
        auto const& instruction = *packet.mCameraInstruction;
        bool        setOrClear  = instruction.mSet->has_value() || instruction.mClear->has_value();
        fc_server_instruction(handle, setOrClear ? 1 : 0);
    }
    origin(id, packet);
}

} // namespace levi_freecam
