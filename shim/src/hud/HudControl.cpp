#include "hud/HudControl.h"

#include <mc/client/game/ClientInstance.h>
#include <mc/client/options/IOptionRegistry.h>

namespace levi_freecam {

namespace {

constexpr uint8_t GUI_HIDE = 1;
constexpr uint8_t GUI_SHOW = 2;

} // namespace

void HudControl::apply(ClientInstance& client, uint8_t gui) {
    if (gui == GUI_HIDE) client.getOptions().setHideGUI(true);
    if (gui == GUI_SHOW) client.getOptions().setHideGUI(false);
}

} // namespace levi_freecam
