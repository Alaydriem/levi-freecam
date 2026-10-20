#pragma once

#include "levi_freecam.h"

#include <filesystem>
#include <string>

namespace levi_freecam {

// owns the Rust handle and the log callback; every call into Rust goes through handle()
class RustBridge {
public:
    static bool create(const std::filesystem::path& configFile);

    static void destroy();

    // null before create and after destroy
    static const FcHandle* handle();

    // the same handle for the exports that take a mutable one (fc_ray_result, fc_markers_shutdown); null before
    // create and after destroy
    static FcHandle* mutableHandle();

private:
    static std::string utf8(const std::filesystem::path& path);
};

} // namespace levi_freecam
