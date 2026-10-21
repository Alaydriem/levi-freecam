#pragma once

#include <cstdint>
#include <format>
#include <string>
#include <string_view>
#include <utility>

namespace levi_freecam {

// every C++ log line goes to Rust with fc_log and from there to the mod manager's log for freecam. Lines written
// before fc_create succeeded are dropped; the one line about a failed registration uses the LeviLamina logger
class ModLog {
public:
    template <class... Args>
    static void error(std::format_string<Args...> format, Args&&... args) {
        write(LEVEL_ERROR, std::format(format, std::forward<Args>(args)...));
    }

    template <class... Args>
    static void warn(std::format_string<Args...> format, Args&&... args) {
        write(LEVEL_WARN, std::format(format, std::forward<Args>(args)...));
    }

    template <class... Args>
    static void info(std::format_string<Args...> format, Args&&... args) {
        write(LEVEL_INFO, std::format(format, std::forward<Args>(args)...));
    }

    template <class... Args>
    static void debug(std::format_string<Args...> format, Args&&... args) {
        write(LEVEL_DEBUG, std::format(format, std::forward<Args>(args)...));
    }

private:
    static constexpr uint32_t LEVEL_ERROR = 0;
    static constexpr uint32_t LEVEL_WARN  = 1;
    static constexpr uint32_t LEVEL_INFO  = 2;
    static constexpr uint32_t LEVEL_DEBUG = 3;

    static void write(uint32_t level, std::string const& message);
};

} // namespace levi_freecam
