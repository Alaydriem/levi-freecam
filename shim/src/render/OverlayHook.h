#pragma once

namespace levi_freecam {

// hooks the swap chain's Present and ResizeBuffers to draw freecam's own ImGui overlay on the back buffer.
// Works in D3D11 and in D3D12 (through D3D11On12). Every hook chains to the function installed before it
class OverlayHook {
public:
    // allows the next init() to install the hooks; enable() calls it before the install trigger is hooked
    static void arm();

    // installs the hooks once; the device and the ImGui context are made on the first Present after it
    static bool init();

    // stops new frames, unhooks, waits for calls in flight, then releases the device and the ImGui context.
    // init() does nothing after it until arm() runs again. False when a thread stayed inside a hook past the
    // wait: the device is then left alive, and so must be everything that thread can still reach
    static bool shutdown();

    // true from the start of shutdown(); a draw that sees it calls nothing more
    static bool shuttingDown();

    // the bound game swap chain's back-buffer size in pixels, read on any thread; both 0 until a chain is bound
    static void backBufferSize(float& width, float& height);
};

} // namespace levi_freecam
