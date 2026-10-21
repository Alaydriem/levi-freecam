#include "world/ShapeDelivery.h"
#include "bridge/ModLog.h"

#include "mod/FreecamMod.h"
#include "network/ClientPacketHandler.h"

#include <windows.h>

#include <mc/deps/core/math/Color.h>
#include <mc/deps/core/math/Vec3.h>
#include <mc/network/MinecraftPacketIds.h>
#include <mc/network/MinecraftPackets.h>
#include <mc/network/packet/ArrowDataPayload.h>
#include <mc/network/packet/BoxDataPayload.h>
#include <mc/network/packet/LineDataPayload.h>
#include <mc/network/packet/PrimitiveShapeDataPayload.h>
#include <mc/network/packet/PrimitiveShapesPacket.h>
#include <mc/world/level/dimension/DimensionType.h>

#include <atomic>
#include <memory>
#include <optional>
#include <vector>

namespace levi_freecam {

// layout check against the header: 8 id, 2 type, 16 loc, 16 rot, 8 scale, 20 color, 8 time, 8 dim,
// 16 attached (8-aligned), 8 max dist, 72 variant (8-aligned)
static_assert(sizeof(PrimitiveShapeDataPayload) == 184, "PrimitiveShapeDataPayload size differs from the header");

namespace {

using ShapeType = ScriptModuleMinecraft::ScriptPrimitiveShapeType;

constexpr uint8_t KIND_REMOVE = 0;
constexpr uint8_t KIND_LINE   = 1;
constexpr uint8_t KIND_BOX    = 2;
constexpr uint8_t KIND_ARROW  = 3;

// the arrow head that drew in the shape spike
constexpr float ARROW_HEAD_LENGTH   = 0.4f;
constexpr float ARROW_HEAD_RADIUS   = 0.2f;
constexpr uchar ARROW_HEAD_SEGMENTS = 8;

// touched only on the thread that delivers; the packet is created once and its shape vector keeps its capacity
std::shared_ptr<PrimitiveShapesPacket> g_packet;

std::atomic<bool> g_threadMismatchLogged{false};

PrimitiveShapesPacket* reusablePacket() {
    if (!g_packet) {
        g_packet = std::static_pointer_cast<PrimitiveShapesPacket>(
            MinecraftPackets::createPacket(MinecraftPacketIds::PrimitiveShapesPacket)
        );
        if (!g_packet) ModLog::warn("could not create a primitive shapes packet");
    }
    return g_packet.get();
}

// shapes go only from the thread that handles the server's packets; any other thread is logged once and refused
bool onHandlerThread() {
    uint32_t handlerThread = ClientPacketHandler::captureThread();
    uint32_t current       = static_cast<uint32_t>(GetCurrentThreadId());
    if (handlerThread == current) return true;
    if (!g_threadMismatchLogged.exchange(true)) {
        ModLog::warn(
            "markers: thread {} differs from the packet handler thread {}; no shape is sent",
            current,
            handlerThread
        );
    }
    return false;
}

Vec3 toVec3(FcVec3 const& value) { return Vec3(value.x, value.y, value.z); }

mce::Color argbColor(uint32_t argb) {
    return mce::Color(
        static_cast<int>((argb >> 16) & 0xFF),
        static_cast<int>((argb >> 8) & 0xFF),
        static_cast<int>(argb & 0xFF),
        static_cast<int>((argb >> 24) & 0xFF)
    );
}

void fillCommon(PrimitiveShapeDataPayload& shape, FcMarkerEntry const& entry, ShapeType type) {
    shape.mShapeType   = std::optional<ShapeType>(type);
    shape.mLocation    = std::optional<Vec3>(toVec3(entry.location));
    shape.mRotation    = std::optional<Vec3>(Vec3(0.0f, 0.0f, 0.0f));
    shape.mScale       = std::optional<float>(1.0f);
    shape.mColor       = std::optional<mce::Color>(argbColor(entry.color));
    shape.mDimensionId = std::optional<DimensionType>(DimensionType(entry.dimension));
}

void fillLine(PrimitiveShapeDataPayload& shape, FcMarkerEntry const& entry) {
    fillCommon(shape, entry, ShapeType::Line);
    LineDataPayload data;
    data.mEndLocation       = toVec3(entry.end);
    shape.mExtraDataPayload = data;
}

void fillBox(PrimitiveShapeDataPayload& shape, FcMarkerEntry const& entry) {
    fillCommon(shape, entry, ShapeType::Box);
    BoxDataPayload data;
    data.mBoxBound          = toVec3(entry.end);
    shape.mExtraDataPayload = data;
}

void fillArrow(PrimitiveShapeDataPayload& shape, FcMarkerEntry const& entry) {
    fillCommon(shape, entry, ShapeType::Arrow);
    ArrowDataPayload data;
    data.mEndLocation       = std::optional<Vec3>(toVec3(entry.end));
    data.mArrowHeadLength   = std::optional<float>(ARROW_HEAD_LENGTH);
    data.mArrowHeadRadius   = std::optional<float>(ARROW_HEAD_RADIUS);
    data.mNumSegments       = std::optional<uchar>(ARROW_HEAD_SEGMENTS);
    shape.mExtraDataPayload = data;
}

// a remove carries the network id only: every option empty and no extra data; an unknown kind adds nothing
void append(std::vector<PrimitiveShapeDataPayload>& shapes, FcMarkerEntry const& entry) {
    if (entry.kind > KIND_ARROW) return;
    PrimitiveShapeDataPayload& shape = shapes.emplace_back();
    shape.mNetworkId                 = entry.id;
    switch (entry.kind) {
    case KIND_LINE:
        fillLine(shape, entry);
        break;
    case KIND_BOX:
        fillBox(shape, entry);
        break;
    case KIND_ARROW:
        fillArrow(shape, entry);
        break;
    case KIND_REMOVE:
    default:
        break;
    }
}

} // namespace

uint32_t ShapeDelivery::generation() {
    uint32_t generation = ClientPacketHandler::generation();
    if (generation == 0) return 0;
    return onHandlerThread() ? generation : 0;
}

bool ShapeDelivery::deliver(const FcMarkerEntry* entries, size_t count) {
    if (!entries || count == 0) return false;
    if (!ClientPacketHandler::captured() || !onHandlerThread()) return false;
    PrimitiveShapesPacket* packet = reusablePacket();
    if (!packet) return false;
    std::vector<PrimitiveShapeDataPayload>& shapes = packet->mShapes.get();
    shapes.clear();
    for (size_t index = 0; index < count; ++index) append(shapes, entries[index]);
    if (shapes.empty()) return false;
    return ClientPacketHandler::deliver(*packet);
}

void ShapeDelivery::reset() {
    g_packet.reset();
    g_threadMismatchLogged.store(false);
}

} // namespace levi_freecam
