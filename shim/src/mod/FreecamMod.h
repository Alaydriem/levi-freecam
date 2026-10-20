#pragma once

#include "ll/api/mod/NativeMod.h"

namespace levi_freecam {

// the LeviLamina entry point: starts Rust, installs hooks and input listeners
class FreecamMod {
public:
    static FreecamMod& getInstance();

    FreecamMod() : mSelf(*ll::mod::NativeMod::current()) {}

    [[nodiscard]] ll::mod::NativeMod& getSelf() const { return mSelf; }

    bool load();

    bool enable();

    bool disable();

private:
    // asks Rust for a remove of every marker still in the world and delivers it while the handler is captured
    void removeMarkers();

    // reads screenshot.folder from Rust, expands it and gives it to the PNG writer
    void configureScreenshots();

    ll::mod::NativeMod& mSelf;
};

} // namespace levi_freecam
