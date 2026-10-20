#pragma once

#include "camera/RenderOverride.h"
#include "frame/FrameDriver.h"
#include "frame/FrameStatusReader.h"

#include <ll/api/memory/Hook.h>
#include <mc/client/renderer/game/LevelRendererCamera.h>
#include <mc/client/renderer/game/LevelRendererPlayer.h>
#include <mc/deps/minecraft_renderer/objects/ViewRenderData.h>
#include <mc/deps/minecraft_renderer/objects/ViewRenderObject.h>
#include <mc/deps/renderer/Camera.h>

namespace levi_freecam {

// once per rendered frame: runs the frame through Rust before the camera is set up, then rewrites
// the view rotation when Rust asks for an override and keeps the camera's field of view and aspect
LL_TYPE_INSTANCE_HOOK(
    SetupCameraHook,
    ll::memory::HookPriority::Normal,
    LevelRendererPlayer,
    &LevelRendererPlayer::setupCamera,
    void,
    mce::Camera& camera,
    float        alpha
) {
    FrameDriver::beforeSetupCamera();
    origin(camera, alpha);
    RenderOverride::applyRotation(camera);
    FrameStatusReader::observeCamera(camera);
}

// the world position the frame is drawn from
LL_TYPE_INSTANCE_HOOK(
    ViewRenderObjectHook,
    ll::memory::HookPriority::Normal,
    LevelRendererPlayer,
    &LevelRendererPlayer::createViewRenderObject,
    ViewRenderObject,
    ScreenContext& screenContext,
    SubClientId    clientSubId
) {
    ViewRenderObject result = origin(screenContext, clientSubId);
    RenderOverride::applyPosition(result.mViewData);
    return result;
}

// the position chunk sorting uses
LL_TYPE_INSTANCE_HOOK(
    LevelCameraHook,
    ll::memory::HookPriority::Normal,
    LevelRendererCamera,
    &LevelRendererCamera::$preRenderUpdate,
    void,
    ScreenContext&                        screenContext,
    LevelRenderPreRenderUpdateParameters& params
) {
    origin(screenContext, params);
    RenderOverride::applyPosition(*this);
}

} // namespace levi_freecam
