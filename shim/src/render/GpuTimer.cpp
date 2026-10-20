#include "render/GpuTimer.h"

#include <d3d11.h>
#include <wrl/client.h>

#include <array>
#include <cstddef>
#include <cstdint>

using Microsoft::WRL::ComPtr;

namespace levi_freecam {

namespace {

constexpr size_t RING_SIZE     = 4;
constexpr double MS_PER_SECOND = 1000.0;
constexpr float  NO_READING    = -1.0f;

// one measurement: the disjoint query brackets two timestamps. A slot is pending from end() until its data is read
std::array<ComPtr<ID3D11Query>, RING_SIZE> gDisjoint;
std::array<ComPtr<ID3D11Query>, RING_SIZE> gStart;
std::array<ComPtr<ID3D11Query>, RING_SIZE> gEnd;
std::array<bool, RING_SIZE>                gPending{};
size_t                                     gNext    = 0;
bool                                       gActive  = false;
bool                                       gCreated = false;
bool                                       gFailed  = false;
float                                      gReading = NO_READING;

bool createQueries(ID3D11Device* device) {
    D3D11_QUERY_DESC disjoint{D3D11_QUERY_TIMESTAMP_DISJOINT, 0};
    D3D11_QUERY_DESC stamp{D3D11_QUERY_TIMESTAMP, 0};
    for (size_t i = 0; i < RING_SIZE; ++i) {
        if (FAILED(device->CreateQuery(&disjoint, &gDisjoint[i])) || FAILED(device->CreateQuery(&stamp, &gStart[i]))
            || FAILED(device->CreateQuery(&stamp, &gEnd[i]))) {
            return false;
        }
    }
    return true;
}

void resetQueries() {
    for (size_t i = 0; i < RING_SIZE; ++i) {
        gDisjoint[i].Reset();
        gStart[i].Reset();
        gEnd[i].Reset();
        gPending[i] = false;
    }
    gNext    = 0;
    gActive  = false;
    gCreated = false;
    gReading = NO_READING;
}

// reads one slot without waiting; false while the GPU has not finished it
bool readSlot(ID3D11DeviceContext* ctx, size_t slot) {
    D3D11_QUERY_DATA_TIMESTAMP_DISJOINT disjoint{};
    uint64_t                            start = 0;
    uint64_t                            end   = 0;
    if (ctx->GetData(gDisjoint[slot].Get(), &disjoint, sizeof(disjoint), D3D11_ASYNC_GETDATA_DONOTFLUSH) != S_OK
        || ctx->GetData(gStart[slot].Get(), &start, sizeof(start), D3D11_ASYNC_GETDATA_DONOTFLUSH) != S_OK
        || ctx->GetData(gEnd[slot].Get(), &end, sizeof(end), D3D11_ASYNC_GETDATA_DONOTFLUSH) != S_OK) {
        return false;
    }
    gPending[slot] = false;
    if (!disjoint.Disjoint && disjoint.Frequency != 0 && end >= start) {
        double ms = static_cast<double>(end - start) / static_cast<double>(disjoint.Frequency) * MS_PER_SECOND;
        gReading  = static_cast<float>(ms);
    }
    return true;
}

// oldest first, so the newest finished measurement is the one kept
void collect(ID3D11DeviceContext* ctx) {
    for (size_t step = 0; step < RING_SIZE; ++step) {
        size_t slot = (gNext + step) % RING_SIZE;
        if (gPending[slot]) readSlot(ctx, slot);
    }
}

} // namespace

void GpuTimer::begin(ID3D11DeviceContext* ctx, ID3D11Device* device) {
    gActive = false;
    if (gFailed || !ctx || !device) return;
    if (!gCreated) {
        gCreated = createQueries(device);
        if (!gCreated) {
            resetQueries();
            gFailed = true;
            return;
        }
    }
    collect(ctx);
    if (gPending[gNext]) return;
    ctx->Begin(gDisjoint[gNext].Get());
    ctx->End(gStart[gNext].Get());
    gActive = true;
}

void GpuTimer::end(ID3D11DeviceContext* ctx) {
    if (!gActive || !ctx) return;
    ctx->End(gEnd[gNext].Get());
    ctx->End(gDisjoint[gNext].Get());
    gPending[gNext] = true;
    gNext           = (gNext + 1) % RING_SIZE;
    gActive         = false;
}

float GpuTimer::lastMs() {
    float reading = gReading;
    gReading      = NO_READING;
    return reading;
}

// a later device may support the queries, so a failure is forgotten here
void GpuTimer::release() {
    resetQueries();
    gFailed = false;
}

} // namespace levi_freecam
