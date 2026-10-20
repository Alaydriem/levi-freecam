#pragma once

struct ID3D11Device;
struct ID3D11DeviceContext;

namespace levi_freecam {

// measures the GPU time of the UI draw with D3D11 timestamp and disjoint queries. A ring of 4 query sets is read
// back a few frames later without waiting, so the render thread never stalls. Render thread only
class GpuTimer {
public:
    // starts a measurement; does nothing when every query set still waits for the GPU
    static void begin(ID3D11DeviceContext* ctx, ID3D11Device* device);

    // ends the measurement begin() started, if it started one
    static void end(ID3D11DeviceContext* ctx);

    // the newest GPU time in milliseconds not yet returned; < 0 when no new reading is ready. Each reading is
    // returned once, so Rust never counts one draw twice
    static float lastMs();

    // drops the queries; called with the renderer's release
    static void release();
};

} // namespace levi_freecam
