#include "bridge/RustBridge.h"

#include <atomic>

namespace levi_freecam {

namespace {

std::atomic<FcHandle*> g_handle{nullptr};

} // namespace

bool RustBridge::create(const std::filesystem::path& configDir) {
    std::string folder = utf8(configDir);
    FcStr       path{reinterpret_cast<const uint8_t*>(folder.data()), folder.size()};
    FcHandle*   handle = fc_create(&path);
    g_handle.store(handle, std::memory_order_release);
    return handle != nullptr;
}

void RustBridge::destroy() {
    FcHandle* handle = g_handle.exchange(nullptr, std::memory_order_acq_rel);
    if (handle) fc_destroy(handle);
}

const FcHandle* RustBridge::handle() { return g_handle.load(std::memory_order_acquire); }

FcHandle* RustBridge::mutableHandle() { return g_handle.load(std::memory_order_acquire); }

std::string RustBridge::utf8(const std::filesystem::path& path) {
    auto bytes = path.u8string();
    return std::string(reinterpret_cast<const char*>(bytes.data()), bytes.size());
}

} // namespace levi_freecam
