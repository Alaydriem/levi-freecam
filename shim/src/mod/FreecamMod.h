#pragma once

#include "ll/api/mod/NativeMod.h"

#include <filesystem>

namespace levi_freecam {

// the LeviLamina entry point: registers with the mod manager through Rust and installs the camera, render and packet
// hooks
class FreecamMod {
public:
    static FreecamMod& getInstance();

    FreecamMod() : mSelf(*ll::mod::NativeMod::current()) {}

    [[nodiscard]] ll::mod::NativeMod& getSelf() const { return mSelf; }

    // runs under the OS loader lock, so it only keeps the config folder
    bool load();

    bool enable();

    bool disable();

private:
    // asks Rust for a remove of every marker still in the world and delivers it while the handler is captured
    void removeMarkers();

    ll::mod::NativeMod&   mSelf;
    std::filesystem::path mConfigDir;
    // true while enable installed the hooks
    bool mHooked = false;
};

} // namespace levi_freecam
