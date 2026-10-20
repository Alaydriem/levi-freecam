#include "render/PngWriter.h"

#include "render/ScreenshotPath.h"

#include <windows.h>

#include <wincodec.h>
#include <wrl/client.h>

#include <atomic>
#include <cstdio>
#include <cstring>
#include <memory>
#include <new>
#include <string>
#include <utility>

using Microsoft::WRL::ComPtr;

namespace levi_freecam {

namespace {

constexpr int      IDLE              = 0;
constexpr int      RESERVED          = 1;
constexpr int      QUEUED            = 2;
constexpr int      DONE              = 3;
constexpr DWORD    JOIN_TIMEOUT_MS   = 5000;
constexpr int      MAX_NAME_ATTEMPTS = 100;
constexpr size_t   BYTES_PER_PIXEL   = 4;
constexpr size_t   BGR_BYTES         = 3;
constexpr size_t   DETAIL_CAPACITY   = 512;
constexpr uint32_t TEN_BIT_MASK      = 0x3FF;
constexpr int      TEN_TO_EIGHT      = 2;
constexpr int      GREEN_SHIFT       = 10;
constexpr int      BLUE_SHIFT        = 20;

std::wstring      gFolder;
HANDLE            gThread = nullptr;
HANDLE            gWake   = nullptr;
std::atomic<bool> gStop{false};
std::atomic<bool> gRunning{false};
// a worker that outlived the join wait still owns the globals; no new worker starts beside it
bool gLeaked = false;

// the slot's owner; each hand-off stores the next owner with release and the next owner loads it with acquire, so
// the job and result fields written before the store are visible to the thread that sees the new state
std::atomic<int> gState{IDLE};

// the job: written by the render thread while RESERVED, read by the worker while QUEUED
uint32_t                   gSequence = 0;
uint32_t                   gWidth    = 0;
uint32_t                   gHeight   = 0;
PixelLayout                gLayout   = PixelLayout::Bgra8;
std::unique_ptr<uint8_t[]> gBuffer;
size_t                     gCapacity = 0;

// the result: written by the worker (or fail()) before DONE, read by the game thread while DONE. gDetail is the
// full path on success and the reason on failure
bool        gOk = false;
std::string gFileName;
std::string gDetail;

void finish(bool ok, std::string fileName, std::string detail) {
    gOk       = ok;
    gFileName = std::move(fileName);
    gDetail   = std::move(detail);
    gState.store(DONE, std::memory_order_release);
}

std::string describe(char const* step, HRESULT hr) {
    char text[DETAIL_CAPACITY]{};
    std::snprintf(text, sizeof(text), "%s (HRESULT 0x%08lX)", step, static_cast<unsigned long>(hr));
    return text;
}

void readPixel(uint8_t const* source, PixelLayout layout, uint8_t& blue, uint8_t& green, uint8_t& red) {
    switch (layout) {
    case PixelLayout::Bgra8:
        blue  = source[0];
        green = source[1];
        red   = source[2];
        return;
    case PixelLayout::Rgba8:
        red   = source[0];
        green = source[1];
        blue  = source[2];
        return;
    case PixelLayout::Rgb10A2: {
        uint32_t value = 0;
        std::memcpy(&value, source, sizeof(value));
        red   = static_cast<uint8_t>((value & TEN_BIT_MASK) >> TEN_TO_EIGHT);
        green = static_cast<uint8_t>(((value >> GREEN_SHIFT) & TEN_BIT_MASK) >> TEN_TO_EIGHT);
        blue  = static_cast<uint8_t>(((value >> BLUE_SHIFT) & TEN_BIT_MASK) >> TEN_TO_EIGHT);
        return;
    }
    }
}

// packs the 4-byte pixels into 3-byte BGR in place, dropping alpha: the back buffer's alpha is not opacity, and a
// PNG with it would show holes. Each pixel is read whole before its 3 bytes are written, and pixel i is written at
// 3i, never past 4i, so no pixel is overwritten before it is read
void packBgr(uint8_t* buffer, size_t count, PixelLayout layout) {
    for (size_t i = 0; i < count; ++i) {
        uint8_t blue  = 0;
        uint8_t green = 0;
        uint8_t red   = 0;
        readPixel(buffer + i * BYTES_PER_PIXEL, layout, blue, green, red);
        uint8_t* target = buffer + i * BGR_BYTES;
        target[0]       = blue;
        target[1]       = green;
        target[2]       = red;
    }
}

HRESULT encode(IWICImagingFactory* factory, std::wstring const& path, uint32_t width, uint32_t height, uint8_t* bgr) {
    ComPtr<IWICStream>            stream;
    ComPtr<IWICBitmapEncoder>     encoder;
    ComPtr<IWICBitmapFrameEncode> frame;
    ComPtr<IPropertyBag2>         properties;
    HRESULT                       hr = factory->CreateStream(&stream);
    if (SUCCEEDED(hr)) hr = stream->InitializeFromFilename(path.c_str(), GENERIC_WRITE);
    if (SUCCEEDED(hr)) hr = factory->CreateEncoder(GUID_ContainerFormatPng, nullptr, &encoder);
    if (SUCCEEDED(hr)) hr = encoder->Initialize(stream.Get(), WICBitmapEncoderNoCache);
    if (SUCCEEDED(hr)) hr = encoder->CreateNewFrame(&frame, &properties);
    if (SUCCEEDED(hr)) hr = frame->Initialize(properties.Get());
    if (SUCCEEDED(hr)) hr = frame->SetSize(width, height);
    WICPixelFormatGUID format = GUID_WICPixelFormat24bppBGR;
    if (SUCCEEDED(hr)) hr = frame->SetPixelFormat(&format);
    if (SUCCEEDED(hr) && format != GUID_WICPixelFormat24bppBGR) hr = WINCODEC_ERR_UNSUPPORTEDPIXELFORMAT;
    UINT stride = width * static_cast<UINT>(BGR_BYTES);
    if (SUCCEEDED(hr)) hr = frame->WritePixels(height, stride, stride * height, bgr);
    if (SUCCEEDED(hr)) hr = frame->Commit();
    if (SUCCEEDED(hr)) hr = encoder->Commit();
    return hr;
}

// claims the first free name for this second with CREATE_NEW, so two shots in one second never share a file
bool claimName(SYSTEMTIME const& time, std::wstring& name, std::wstring& path) {
    for (int attempt = 1; attempt <= MAX_NAME_ATTEMPTS; ++attempt) {
        name        = ScreenshotPath::fileName(time, attempt);
        path        = gFolder + L"\\" + name;
        HANDLE file = CreateFileW(path.c_str(), GENERIC_WRITE, 0, nullptr, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, nullptr);
        if (file != INVALID_HANDLE_VALUE) {
            CloseHandle(file);
            return true;
        }
        DWORD error = GetLastError();
        if (error != ERROR_FILE_EXISTS && error != ERROR_ALREADY_EXISTS) return false;
    }
    return false;
}

void writeJob(IWICImagingFactory* factory, HRESULT factoryResult) {
    if (!factory) {
        finish(false, {}, describe("the WIC imaging factory could not be created", factoryResult));
        return;
    }
    if (!ScreenshotPath::ensureFolder(gFolder)) {
        finish(false, {}, "the folder " + ScreenshotPath::utf8(gFolder) + " does not exist and could not be created");
        return;
    }
    SYSTEMTIME time{};
    GetLocalTime(&time);
    std::wstring name;
    std::wstring path;
    if (!claimName(time, name, path)) {
        finish(false, {}, describe("no free file name in the screenshot folder", HRESULT_FROM_WIN32(GetLastError())));
        return;
    }
    packBgr(gBuffer.get(), static_cast<size_t>(gWidth) * gHeight, gLayout);
    HRESULT hr = encode(factory, path, gWidth, gHeight, gBuffer.get());
    if (FAILED(hr)) {
        DeleteFileW(path.c_str());
        finish(false, {}, describe(("the PNG could not be written to " + ScreenshotPath::utf8(path)).c_str(), hr));
        return;
    }
    finish(true, ScreenshotPath::utf8(name), ScreenshotPath::utf8(path));
}

DWORD WINAPI runWorker(LPVOID) {
    HRESULT                    com = CoInitializeEx(nullptr, COINIT_MULTITHREADED);
    ComPtr<IWICImagingFactory> factory;
    HRESULT                    factoryResult = com;
    if (SUCCEEDED(com)) {
        factoryResult = CoCreateInstance(CLSID_WICImagingFactory, nullptr, CLSCTX_INPROC_SERVER, IID_PPV_ARGS(&factory));
    }
    for (;;) {
        WaitForSingleObject(gWake, INFINITE);
        // a queued job is finished before a stop is honoured, so a shot taken just before shutdown still saves
        if (gState.load(std::memory_order_acquire) == QUEUED) writeJob(factory.Get(), factoryResult);
        if (gStop.load()) break;
    }
    factory.Reset();
    if (SUCCEEDED(com)) CoUninitialize();
    return 0;
}

} // namespace

void PngWriter::configure(std::wstring folder) { gFolder = std::move(folder); }

bool PngWriter::start() {
    if (gThread) return true;
    if (gLeaked) return false;
    gStop.store(false);
    gState.store(IDLE, std::memory_order_release);
    gWake = CreateEventW(nullptr, FALSE, FALSE, nullptr);
    if (!gWake) return false;
    gThread = CreateThread(nullptr, 0, &runWorker, nullptr, 0, nullptr);
    if (!gThread) {
        CloseHandle(gWake);
        gWake = nullptr;
        return false;
    }
    gRunning.store(true);
    return true;
}

bool PngWriter::shutdown() {
    gRunning.store(false);
    if (!gThread) return true;
    gStop.store(true);
    SetEvent(gWake);
    if (WaitForSingleObject(gThread, JOIN_TIMEOUT_MS) != WAIT_OBJECT_0) {
        // the worker still owns its handles and the buffer; they go with the process
        gThread = nullptr;
        gWake   = nullptr;
        gLeaked = true;
        return false;
    }
    CloseHandle(gThread);
    CloseHandle(gWake);
    gThread = nullptr;
    gWake   = nullptr;
    gBuffer.reset();
    gCapacity = 0;
    gState.store(IDLE, std::memory_order_release);
    return true;
}

bool PngWriter::reserve(uint32_t sequence) {
    if (!gRunning.load()) return false;
    int expected = IDLE;
    if (!gState.compare_exchange_strong(expected, RESERVED, std::memory_order_acquire)) return false;
    gSequence = sequence;
    return true;
}

uint8_t* PngWriter::pixels(uint32_t width, uint32_t height) {
    size_t needed = static_cast<size_t>(width) * height * BYTES_PER_PIXEL;
    if (needed == 0) return nullptr;
    if (needed > gCapacity) {
        gBuffer.reset(new (std::nothrow) uint8_t[needed]);
        gCapacity = gBuffer ? needed : 0;
        if (!gBuffer) return nullptr;
    }
    gWidth  = width;
    gHeight = height;
    return gBuffer.get();
}

void PngWriter::submit(PixelLayout layout) {
    gLayout = layout;
    gState.store(QUEUED, std::memory_order_release);
    SetEvent(gWake);
}

void PngWriter::fail(char const* reason) { finish(false, {}, reason); }

void PngWriter::cancel() { gState.store(IDLE, std::memory_order_release); }

bool PngWriter::result(uint32_t& sequence, bool& ok, std::string const*& fileName, std::string const*& detail) {
    if (gState.load(std::memory_order_acquire) != DONE) return false;
    sequence = gSequence;
    ok       = gOk;
    fileName = &gFileName;
    detail   = &gDetail;
    return true;
}

void PngWriter::release() { gState.store(IDLE, std::memory_order_release); }

} // namespace levi_freecam
