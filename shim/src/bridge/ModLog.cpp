#include "bridge/ModLog.h"

#include "levi_freecam.h"

namespace levi_freecam {

void ModLog::write(uint32_t level, std::string const& message) {
    fc_log(level, reinterpret_cast<const uint8_t*>(message.data()), message.size(), nullptr, 0, 0);
}

} // namespace levi_freecam
