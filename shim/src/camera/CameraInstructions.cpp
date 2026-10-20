#include "camera/CameraInstructions.h"

#include "mod/FreecamMod.h"
#include "network/ClientPacketHandler.h"

#include <mc/deps/core/math/Vec3.h>
#include <mc/deps/minecraft_camera/CameraInstruction.h>
#include <mc/deps/shared_types/v1_26_50/camera/CameraPreset.h>
#include <mc/network/MinecraftPacketIds.h>
#include <mc/network/MinecraftPackets.h>
#include <mc/network/packet/CameraInstructionPacket.h>
#include <mc/world/level/Level.h>
#include <mc/world/level/camera/CameraPresets.h>

#include <memory>
#include <mutex>
#include <optional>

namespace levi_freecam {

namespace {

constexpr char FREE_PRESET[] = "minecraft:free";

struct Delivery {
    std::recursive_mutex                     mutex;
    std::optional<uint>                      freeIndex;
    std::shared_ptr<CameraInstructionPacket> packet;
};

Delivery& delivery() {
    static Delivery instance;
    return instance;
}

thread_local bool t_delivering = false;

// marks the mod's own delivery for the instruction hook; cleared on any exit, including an SEH unwind
class DeliveringScope {
public:
    DeliveringScope() { t_delivering = true; }
    ~DeliveringScope() { t_delivering = false; }
    DeliveringScope(DeliveringScope const&)            = delete;
    DeliveringScope& operator=(DeliveringScope const&) = delete;
};

// one packet for the whole session; each delivery overwrites its instruction
CameraInstructionPacket* reusablePacket(Delivery& state) {
    if (!state.packet) {
        state.packet = std::static_pointer_cast<CameraInstructionPacket>(
            MinecraftPackets::createPacket(MinecraftPacketIds::CameraInstruction)
        );
        if (!state.packet) {
            FreecamMod::getInstance().getSelf().getLogger().warn("could not create a camera instruction packet");
        }
    }
    return state.packet.get();
}

bool deliver(LocalPlayer const* player, CameraInstruction const& instruction) {
    if (!player) return false;
    auto&           state = delivery();
    std::lock_guard lock(state.mutex);
    if (!ClientPacketHandler::captured()) return false;
    CameraInstructionPacket* packet = reusablePacket(state);
    if (!packet) return false;
    packet->mCameraInstruction = instruction;
    DeliveringScope delivering;
    return ClientPacketHandler::deliver(*packet);
}

} // namespace

bool CameraInstructions::ready(Level& level) {
    std::optional<uint> found;
    uint                index = 0;
    for (auto& preset : *level.getCameraPresets().mPresets) {
        if (preset.mName->mValue == FREE_PRESET) {
            found = index;
            break;
        }
        ++index;
    }
    auto&           state = delivery();
    std::lock_guard lock(state.mutex);
    state.freeIndex = found;
    return ClientPacketHandler::captured() && found.has_value();
}

bool CameraInstructions::sendSet(LocalPlayer const* player, const FcPose& pose) {
    std::optional<uint> freeIndex;
    {
        auto&           state = delivery();
        std::lock_guard lock(state.mutex);
        freeIndex = state.freeIndex;
    }
    if (!freeIndex) return false;
    CameraInstructionOptions::SetInstruction set{};
    set.mPresetIndex = *freeIndex;
    set.mPos         = CameraInstructionOptions::SetInstruction::PosOption{
        Vec3(pose.position.x, pose.position.y, pose.position.z)
    };
    set.mRot = CameraInstructionOptions::SetInstruction::RotOption{pose.pitch, pose.yaw};
    CameraInstruction instruction{};
    instruction.mSet = set;
    return deliver(player, instruction);
}

bool CameraInstructions::sendClear(LocalPlayer const* player) {
    CameraInstruction instruction{};
    instruction.mClear = true;
    return deliver(player, instruction);
}

bool CameraInstructions::isDelivering() { return t_delivering; }

void CameraInstructions::reset() {
    auto&           state = delivery();
    std::lock_guard lock(state.mutex);
    state.freeIndex.reset();
    state.packet.reset();
}

} // namespace levi_freecam
