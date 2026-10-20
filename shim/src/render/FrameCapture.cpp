#include "render/FrameCapture.h"

#include "mod/FreecamMod.h"
#include "render/PixelLayout.h"
#include "render/PngWriter.h"

#include <d3d11.h>
#include <wrl/client.h>

#include "ll/api/io/Logger.h"

#include <atomic>
#include <cstring>
#include <string>

using Microsoft::WRL::ComPtr;

namespace levi_freecam {

namespace {

constexpr uint64_t SLOT_VALID      = 1ULL << 63;
constexpr uint64_t SLOT_OK         = 1ULL << 32;
constexpr uint64_t SEQUENCE_MASK   = 0xFFFFFFFFULL;
// a request keeps the low 31 bits of the present count it was posted at, between the sequence and SLOT_VALID
constexpr int      PRESENT_SHIFT   = 32;
constexpr uint32_t PRESENT_MASK    = 0x7FFFFFFFU;
// presents after posting that may still copy a request; Rust gives up after 10, so a dropped request is answered first
constexpr uint32_t REQUEST_PRESENT_LIMIT = 3;
constexpr uint32_t BYTES_PER_PIXEL = 4;
// presents a copy may stay unfinished on the GPU before the shot fails; a copy normally lands in 1 to 3
constexpr int MAP_WAIT_LIMIT = 60;

// the request fc_frame made: SLOT_VALID | present count at posting << PRESENT_SHIFT | sequence; 0 when none waits
std::atomic<uint64_t> gRequest{0};
// the taken answer: SLOT_VALID | SLOT_OK when copied | sequence; 0 when none waits
std::atomic<uint64_t> gTaken{0};

// render thread only: the copy waiting to be read, its pixel layout and the presents spent waiting
ComPtr<ID3D11Texture2D> gStaging;
uint32_t                gWidth    = 0;
uint32_t                gHeight   = 0;
PixelLayout             gLayout   = PixelLayout::Bgra8;
int                     gMapWaits = 0;

// one line per unsupported format, not one per shot
DXGI_FORMAT gLoggedFormat = DXGI_FORMAT_UNKNOWN;
bool        gLoggedTenBit = false;

ll::io::Logger& logger() { return FreecamMod::getInstance().getSelf().getLogger(); }

void answerTaken(uint32_t sequence, bool ok) {
    gTaken.store(SLOT_VALID | (ok ? SLOT_OK : 0) | sequence);
}

bool layoutOf(DXGI_FORMAT format, PixelLayout& layout) {
    switch (format) {
    case DXGI_FORMAT_B8G8R8A8_UNORM:
    case DXGI_FORMAT_B8G8R8A8_UNORM_SRGB:
    case DXGI_FORMAT_B8G8R8A8_TYPELESS:
    case DXGI_FORMAT_B8G8R8X8_UNORM:
    case DXGI_FORMAT_B8G8R8X8_UNORM_SRGB:
    case DXGI_FORMAT_B8G8R8X8_TYPELESS:
        layout = PixelLayout::Bgra8;
        return true;
    case DXGI_FORMAT_R8G8B8A8_UNORM:
    case DXGI_FORMAT_R8G8B8A8_UNORM_SRGB:
    case DXGI_FORMAT_R8G8B8A8_TYPELESS:
        layout = PixelLayout::Rgba8;
        return true;
    case DXGI_FORMAT_R10G10B10A2_UNORM:
    case DXGI_FORMAT_R10G10B10A2_TYPELESS:
        layout = PixelLayout::Rgb10A2;
        return true;
    default:
        return false;
    }
}

// ResolveSubresource needs a typed format; a typeless back buffer is read as UNORM
DXGI_FORMAT typedFormat(DXGI_FORMAT format) {
    switch (format) {
    case DXGI_FORMAT_B8G8R8A8_TYPELESS:
        return DXGI_FORMAT_B8G8R8A8_UNORM;
    case DXGI_FORMAT_B8G8R8X8_TYPELESS:
        return DXGI_FORMAT_B8G8R8X8_UNORM;
    case DXGI_FORMAT_R8G8B8A8_TYPELESS:
        return DXGI_FORMAT_R8G8B8A8_UNORM;
    case DXGI_FORMAT_R10G10B10A2_TYPELESS:
        return DXGI_FORMAT_R10G10B10A2_UNORM;
    default:
        return format;
    }
}

void noteFormat(DXGI_FORMAT format, bool supported, PixelLayout layout) {
    if (!supported && gLoggedFormat != format) {
        gLoggedFormat = format;
        logger().warn("screenshot: the back buffer format {} cannot be saved as a PNG", static_cast<int>(format));
    }
    if (supported && layout == PixelLayout::Rgb10A2 && !gLoggedTenBit) {
        gLoggedTenBit = true;
        logger().info("screenshot: the back buffer has 10 bits per colour; the PNG keeps the top 8");
    }
}

// a multisampled buffer is resolved into a plain texture first; the copy then reads that
bool resolveSource(
    ID3D11Device*            device,
    ID3D11DeviceContext*     context,
    ID3D11Texture2D*         source,
    D3D11_TEXTURE2D_DESC&    desc,
    ComPtr<ID3D11Texture2D>& plain
) {
    if (desc.SampleDesc.Count <= 1) {
        plain = source;
        return true;
    }
    desc.Format             = typedFormat(desc.Format);
    desc.SampleDesc.Count   = 1;
    desc.SampleDesc.Quality = 0;
    desc.Usage              = D3D11_USAGE_DEFAULT;
    desc.BindFlags          = 0;
    desc.CPUAccessFlags     = 0;
    desc.MiscFlags          = 0;
    if (FAILED(device->CreateTexture2D(&desc, nullptr, &plain))) return false;
    context->ResolveSubresource(plain.Get(), 0, source, 0, desc.Format);
    return true;
}

bool copyToStaging(ID3D11Device* device, ID3D11DeviceContext* context, ID3D11Resource* backBuffer, PixelLayout& layout) {
    ComPtr<ID3D11Texture2D> source;
    if (FAILED(backBuffer->QueryInterface(IID_PPV_ARGS(&source)))) return false;
    D3D11_TEXTURE2D_DESC desc{};
    source->GetDesc(&desc);
    bool supported = layoutOf(desc.Format, layout);
    noteFormat(desc.Format, supported, layout);
    if (!supported) return false;
    ComPtr<ID3D11Texture2D> plain;
    if (!resolveSource(device, context, source.Get(), desc, plain)) return false;
    D3D11_TEXTURE2D_DESC staging{};
    staging.Width            = desc.Width;
    staging.Height           = desc.Height;
    staging.MipLevels        = 1;
    staging.ArraySize        = 1;
    staging.Format           = desc.Format;
    staging.SampleDesc.Count = 1;
    staging.Usage            = D3D11_USAGE_STAGING;
    staging.CPUAccessFlags   = D3D11_CPU_ACCESS_READ;
    if (FAILED(device->CreateTexture2D(&staging, nullptr, &gStaging))) return false;
    context->CopySubresourceRegion(gStaging.Get(), 0, 0, 0, 0, plain.Get(), 0, nullptr);
    gWidth    = desc.Width;
    gHeight   = desc.Height;
    gMapWaits = 0;
    return true;
}

// the reason reaches the log once, when the game thread gives Rust the written answer
void failPending(char const* reason) {
    gStaging.Reset();
    PngWriter::fail(reason);
}

// the rows go out one by one: the mapped row pitch is wider than the pixels
void readRows(D3D11_MAPPED_SUBRESOURCE const& mapped, uint8_t* target) {
    size_t rowBytes = static_cast<size_t>(gWidth) * BYTES_PER_PIXEL;
    auto   source   = static_cast<uint8_t const*>(mapped.pData);
    for (uint32_t row = 0; row < gHeight; ++row) {
        std::memcpy(target + row * rowBytes, source + static_cast<size_t>(row) * mapped.RowPitch, rowBytes);
    }
}

double millisecondsSince(LARGE_INTEGER const& start) {
    static double const frequency = [] {
        LARGE_INTEGER value{};
        QueryPerformanceFrequency(&value);
        return static_cast<double>(value.QuadPart);
    }();
    LARGE_INTEGER now{};
    QueryPerformanceCounter(&now);
    return static_cast<double>(now.QuadPart - start.QuadPart) * 1000.0 / frequency;
}

// the readback runs inside Present on the render thread; its time is the cost a shot adds to that frame
void readRowsTimed(D3D11_MAPPED_SUBRESOURCE const& mapped, uint8_t* target) {
    LARGE_INTEGER start{};
    QueryPerformanceCounter(&start);
    readRows(mapped, target);
    logger().info("screenshot readback: {}x{}, {:.2f} ms", gWidth, gHeight, millisecondsSince(start));
}

uint32_t sequenceOf(uint64_t request) { return static_cast<uint32_t>(request & SEQUENCE_MASK); }

bool isStale(uint64_t request, uint32_t presentCount) {
    auto posted = static_cast<uint32_t>(request >> PRESENT_SHIFT) & PRESENT_MASK;
    return ((presentCount - posted) & PRESENT_MASK) > REQUEST_PRESENT_LIMIT;
}

// drops `request` only while it is still the waiting one, and answers it as not taken with one log line
void dropRequest(uint64_t request, char const* reason) {
    if ((request & SLOT_VALID) == 0 || !gRequest.compare_exchange_strong(request, 0)) return;
    logger().warn("screenshot: request {} dropped: {}", sequenceOf(request), reason);
    answerTaken(sequenceOf(request), false);
}

} // namespace

void FrameCapture::request(uint32_t sequence, uint32_t presentCount) {
    gRequest.store(SLOT_VALID | (static_cast<uint64_t>(presentCount & PRESENT_MASK) << PRESENT_SHIFT) | sequence);
}

void FrameCapture::expire(uint32_t presentCount) {
    uint64_t request = gRequest.load();
    if ((request & SLOT_VALID) == 0 || !isStale(request, presentCount)) return;
    dropRequest(request, "no Present could copy it in time");
}

bool FrameCapture::wanted() { return (gRequest.load() & SLOT_VALID) != 0; }

void FrameCapture::poll(ID3D11DeviceContext* context) {
    if (!gStaging || !context) return;
    D3D11_MAPPED_SUBRESOURCE mapped{};
    HRESULT hr = context->Map(gStaging.Get(), 0, D3D11_MAP_READ, D3D11_MAP_FLAG_DO_NOT_WAIT, &mapped);
    if (hr == DXGI_ERROR_WAS_STILL_DRAWING) {
        if (++gMapWaits > MAP_WAIT_LIMIT) failPending("the GPU did not finish the screenshot copy in time");
        return;
    }
    if (FAILED(hr)) {
        logger().warn("screenshot: mapping the copy failed with HRESULT 0x{:08X}", static_cast<unsigned>(hr));
        failPending("the screenshot copy could not be read");
        return;
    }
    uint8_t* target = PngWriter::pixels(gWidth, gHeight);
    if (target) readRowsTimed(mapped, target);
    context->Unmap(gStaging.Get(), 0);
    if (!target) {
        failPending("no memory for the screenshot pixels");
        return;
    }
    gStaging.Reset();
    PngWriter::submit(gLayout);
}

void FrameCapture::take(ID3D11Device* device, ID3D11DeviceContext* context, ID3D11Resource* backBuffer) {
    uint64_t request = gRequest.exchange(0);
    if ((request & SLOT_VALID) == 0) return;
    uint32_t sequence = sequenceOf(request);
    if (gStaging || !PngWriter::reserve(sequence)) {
        logger().warn("screenshot: request {} refused: the last screenshot is still being written", sequence);
        answerTaken(sequence, false);
        return;
    }
    PixelLayout layout = PixelLayout::Bgra8;
    if (!device || !context || !backBuffer || !copyToStaging(device, context, backBuffer, layout)) {
        gStaging.Reset();
        PngWriter::cancel();
        logger().warn("screenshot: request {} failed: the back buffer could not be copied", sequence);
        answerTaken(sequence, false);
        return;
    }
    gLayout = layout;
    answerTaken(sequence, true);
}

void FrameCapture::refuse(char const* reason) {
    uint64_t request = gRequest.exchange(0);
    if ((request & SLOT_VALID) == 0) return;
    uint32_t sequence = sequenceOf(request);
    logger().warn("screenshot: request {} failed: {}", sequence, reason);
    answerTaken(sequence, false);
}

void FrameCapture::release(char const* reason) {
    if (gStaging) failPending(reason);
    dropRequest(gRequest.load(), "the swap chain was resized or the device changed while it waited");
}

// the written answer is looked at first: it is published after its taken answer, so when it is ready the taken
// answer is visible too, and taken always reaches Rust before written
void FrameCapture::deliver(FcHandle* handle) {
    if (!handle) return;
    uint32_t           writtenSequence = 0;
    bool               writtenOk       = false;
    std::string const* fileName        = nullptr;
    std::string const* detail          = nullptr;
    bool               written         = PngWriter::result(writtenSequence, writtenOk, fileName, detail);
    uint64_t           taken           = gTaken.exchange(0);
    if ((taken & SLOT_VALID) != 0) {
        fc_capture_taken(handle, static_cast<uint32_t>(taken & SEQUENCE_MASK), (taken & SLOT_OK) != 0 ? 1 : 0);
    }
    if (!written) return;
    if (writtenOk) {
        logger().info("screenshot: saved {}", *detail);
        fc_capture_written(handle, writtenSequence, 1, reinterpret_cast<uint8_t const*>(fileName->data()), fileName->size());
    } else {
        logger().error("screenshot: request {} failed: {}", writtenSequence, *detail);
        fc_capture_written(handle, writtenSequence, 0, nullptr, 0);
    }
    PngWriter::release();
}

void FrameCapture::reset() {
    gRequest.store(0);
    gTaken.store(0);
}

} // namespace levi_freecam
