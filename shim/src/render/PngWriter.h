#pragma once

#include "render/PixelLayout.h"

#include <cstdint>
#include <string>

namespace levi_freecam {

// writes one screenshot at a time as a PNG with WIC on its own worker thread.
//
// One job slot moves through four owners: idle, reserved by the render thread (its copy waits on the GPU), queued
// for the worker, done (the result waits for the game thread). Only the owner of the current state touches the
// slot, so no thread ever waits on another. Memory: one pixel buffer of the last shot's size, kept for the next
// shot and freed at shutdown
class PngWriter {
public:
    // the expanded folder the files go to; call before start()
    static void configure(std::wstring folder);

    // starts the worker thread; false when it could not start, and every shot then fails
    static bool start();

    // asks the worker to stop after the job it holds, and joins it. Returns after a bounded wait; false when the
    // worker was still writing past it and was left to finish on its own
    static bool shutdown();

    // render thread: takes the idle slot for the shot `sequence`; false while a shot is still on its way or the
    // worker is not running
    static bool reserve(uint32_t sequence);

    // render thread, while reserved: a buffer of width * height * 4 bytes to fill with tightly packed rows; null
    // when it cannot be allocated
    static uint8_t* pixels(uint32_t width, uint32_t height);

    // render thread, after filling pixels(): hands the job to the worker
    static void submit(PixelLayout layout);

    // render thread, while reserved: ends the shot without a file; the game thread reports it as failed
    static void fail(char const* reason);

    // render thread, while reserved: gives the slot back before anything was copied; no answer follows
    static void cancel();

    // game thread: true when a finished shot waits for its answer; the fields stay valid until release()
    static bool result(uint32_t& sequence, bool& ok, std::string const*& fileName, std::string const*& detail);

    // game thread: frees the slot after the answer for result() was given
    static void release();
};

} // namespace levi_freecam
