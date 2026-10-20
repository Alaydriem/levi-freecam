#pragma once

#include "frame/FrameDriver.h"

#include <ll/api/memory/Hook.h>
#include <mc/client/game/ClientInstance.h>

namespace levi_freecam {

// once per client tick: keeps the ClientInstance the frame reads from, and runs the out-of-world frame
LL_TYPE_INSTANCE_HOOK(
    ClientUpdateHook,
    ll::memory::HookPriority::Normal,
    ClientInstance,
    &ClientInstance::$update,
    bool,
    bool a1
) {
    FrameDriver::setClient(this);
    bool result = origin(a1);
    FrameDriver::tick(*this);
    return result;
}

} // namespace levi_freecam
