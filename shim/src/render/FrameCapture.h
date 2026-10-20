#pragma once

#include "levi_freecam.h"

#include <cstdint>

struct ID3D11Device;
struct ID3D11DeviceContext;
struct ID3D11Resource;

namespace levi_freecam {

// copies the presented back buffer for a screenshot and hands its pixels to the PNG writer.
//
// The game thread records the request fc_frame made and gives Rust the answers. The render thread, at Present,
// copies the back buffer into a staging texture, then maps it without waiting on a later Present and copies the rows
// out. Requests and "taken" answers cross threads through one atomic slot each; the written answer comes from the
// PngWriter slot. At most one staging texture exists, from the copy until its rows are read or the shot fails
class FrameCapture {
public:
    // game thread: fc_frame asked for the next presented frame; `presentCount` is the present count when it is posted
    static void request(uint32_t sequence, uint32_t presentCount);

    // render thread, each outermost Present after the count rises: a request no Present copied within 3 presents of
    // being posted is dropped and answered as not taken, so it is never copied later
    static void expire(uint32_t presentCount);

    // render thread: true while a request waits for a Present
    static bool wanted();

    // render thread, each Present before take(): reads a finished copy and hands it to the writer. A copy the GPU
    // has not finished is tried again next Present, up to a bound, then the shot fails
    static void poll(ID3D11DeviceContext* context);

    // render thread: answers the waiting request by copying `backBuffer`, before any mod drawing on this Present.
    // A wrapped D3D12 buffer must be acquired by the caller, which flushes the context afterwards
    static void take(ID3D11Device* device, ID3D11DeviceContext* context, ID3D11Resource* backBuffer);

    // render thread: answers the waiting request as not taken; this Present cannot be copied
    static void refuse(char const* reason);

    // render thread: drops the staging texture before a resize or a device change; a shot still being read fails,
    // and a request still waiting for a Present is dropped and answered as not taken
    static void release(char const* reason);

    // game thread, before fc_frame: gives Rust the taken answer, then the written answer, each once
    static void deliver(FcHandle* handle);

    // drops the request and an undelivered taken answer; at disable, after the overlay stopped
    static void reset();
};

} // namespace levi_freecam
