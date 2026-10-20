#pragma once

#include "levi_freecam.h"

struct ID3D11Device;
struct ID3D11DeviceContext;

namespace levi_freecam {

// draws one UI frame from Rust with D3D11. Render thread only. Works on the game's own D3D11 device and on the
// D3D11On12 device of the D3D12 path; the caller binds the render target and keeps the context state it needs
class EguiRenderer {
public:
    // makes the fixed pipeline objects on `device`; false leaves nothing behind. The shaders compile once per
    // process, on the first call; later calls build only device objects from the kept bytecode
    static bool init(ID3D11Device* device);

    // true between a successful init() and release()
    static bool ready();

    // applies the frame's texture changes, draws its geometry into the bound render target of `width` x `height`
    // pixels, then frees the textures the frame lists. A skipped frame applies and frees textures only
    static void draw(ID3D11DeviceContext* ctx, ID3D11Device* device, FcUiFrame const& frame, float width, float height);

    // drops the UI textures only. Called before ResizeBuffers and on a new swap chain; the caller must then ask Rust
    // to send the textures again with fc_ui_reset_textures
    static void releaseTextures();

    // drops every device object, the UI textures included; the compiled bytecode stays. Called on a new device and
    // on shutdown; the caller must then ask Rust to send the textures again with fc_ui_reset_textures
    static void release();
};

} // namespace levi_freecam
