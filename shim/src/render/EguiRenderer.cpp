#include "render/EguiRenderer.h"

#include "mod/FreecamMod.h"

#include <d3d11.h>
#include <d3dcompiler.h>
#include <wrl/client.h>

#include "ll/api/io/Logger.h"

#include <algorithm>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <iterator>
#include <unordered_map>

using Microsoft::WRL::ComPtr;

namespace levi_freecam {

namespace {

// positions arrive in physical pixels; the constant buffer holds the target size to map them to clip space.
// Colours are premultiplied and in gamma space, blended into the UNORM target with no sRGB conversion
constexpr char SHADER_SOURCE[] = R"hlsl(
cbuffer Screen : register(b0) {
    float2 screenSize;
    float2 padding;
};

struct VertexIn {
    float2 pos : POSITION;
    float2 uv  : TEXCOORD0;
    float4 col : COLOR0;
};

struct PixelIn {
    float4 pos : SV_POSITION;
    float2 uv  : TEXCOORD0;
    float4 col : COLOR0;
};

PixelIn vs_main(VertexIn input) {
    PixelIn output;
    output.pos = float4(input.pos.x * 2.0 / screenSize.x - 1.0, 1.0 - input.pos.y * 2.0 / screenSize.y, 0.0, 1.0);
    output.uv  = input.uv;
    output.col = input.col;
    return output;
}

Texture2D    uiTexture : register(t0);
SamplerState uiSampler : register(s0);

float4 ps_main(PixelIn input) : SV_Target {
    return input.col * uiTexture.Sample(uiSampler, input.uv);
}
)hlsl";

constexpr UINT VERTEX_STRIDE    = sizeof(FcUiVertex);
constexpr UINT INDEX_STRIDE     = sizeof(uint32_t);
constexpr UINT BYTES_PER_PIXEL  = 4;
constexpr UINT INITIAL_VERTICES = 8192;
constexpr UINT INITIAL_INDICES  = 16384;
constexpr UINT CONSTANT_FLOATS  = 4;
// a frame past this many vertices or indices is not drawn; Rust's ui.maxVertices stops it far below
constexpr UINT MAX_BUFFER_ELEMENTS = 4u * 1024u * 1024u;
// egui holds the font atlas and very few others; a texture past this count is not created
constexpr size_t MAX_TEXTURES = 64;

static_assert(sizeof(FcUiVertex) == 20, "FcUiVertex must match the input layout");

// compiled once per process: bytecode does not depend on the device
ComPtr<ID3DBlob>                gVertexCode;
ComPtr<ID3DBlob>                gPixelCode;
bool                            gCompileFailed = false;
ComPtr<ID3D11VertexShader>      gVertexShader;
ComPtr<ID3D11PixelShader>       gPixelShader;
ComPtr<ID3D11InputLayout>       gLayout;
ComPtr<ID3D11Buffer>            gConstants;
ComPtr<ID3D11BlendState>        gBlend;
ComPtr<ID3D11RasterizerState>   gRaster;
ComPtr<ID3D11DepthStencilState> gDepth;
ComPtr<ID3D11SamplerState>      gSampler;
ComPtr<ID3D11Buffer>            gVertices;
ComPtr<ID3D11Buffer>            gIndices;
UINT                            gVertexCapacity     = 0;
UINT                            gIndexCapacity      = 0;
bool                            gReady              = false;
bool                            gTextureLimitLogged = false;
// texture id from Rust to its view, which holds the texture. Bounded by MAX_TEXTURES; entries leave on `freed`
std::unordered_map<uint64_t, ComPtr<ID3D11ShaderResourceView>> gTextures;

ll::io::Logger& logger() { return FreecamMod::getInstance().getSelf().getLogger(); }

bool compile(char const* entry, char const* target, ComPtr<ID3DBlob>& code) {
    ComPtr<ID3DBlob> errors;
    HRESULT          result = D3DCompile(
        SHADER_SOURCE,
        sizeof(SHADER_SOURCE) - 1,
        "levi_freecam_ui",
        nullptr,
        nullptr,
        entry,
        target,
        D3DCOMPILE_OPTIMIZATION_LEVEL3,
        0,
        &code,
        &errors
    );
    if (SUCCEEDED(result)) return true;
    char const* text = errors ? static_cast<char const*>(errors->GetBufferPointer()) : "no compiler output";
    logger().error("ui renderer: shader {} did not compile: {}", entry, text);
    return false;
}

// a compile error does not go away on a retry, so it is not tried again
bool ensureBytecode() {
    if (gVertexCode && gPixelCode) return true;
    if (gCompileFailed) return false;
    ComPtr<ID3DBlob> vertexCode;
    ComPtr<ID3DBlob> pixelCode;
    if (!compile("vs_main", "vs_4_0", vertexCode) || !compile("ps_main", "ps_4_0", pixelCode)) {
        gCompileFailed = true;
        return false;
    }
    gVertexCode = vertexCode;
    gPixelCode  = pixelCode;
    return true;
}

bool createShaders(ID3D11Device* device) {
    if (!ensureBytecode()) return false;
    ID3DBlob* vertexCode = gVertexCode.Get();
    ID3DBlob* pixelCode  = gPixelCode.Get();
    if (FAILED(device->CreateVertexShader(vertexCode->GetBufferPointer(), vertexCode->GetBufferSize(), nullptr, &gVertexShader))
        || FAILED(device->CreatePixelShader(pixelCode->GetBufferPointer(), pixelCode->GetBufferSize(), nullptr, &gPixelShader))) {
        return false;
    }
    D3D11_INPUT_ELEMENT_DESC layout[] = {
        {"POSITION", 0, DXGI_FORMAT_R32G32_FLOAT,   0, offsetof(FcUiVertex, x),    D3D11_INPUT_PER_VERTEX_DATA, 0},
        {"TEXCOORD", 0, DXGI_FORMAT_R32G32_FLOAT,   0, offsetof(FcUiVertex, u),    D3D11_INPUT_PER_VERTEX_DATA, 0},
        {"COLOR",    0, DXGI_FORMAT_R8G8B8A8_UNORM, 0, offsetof(FcUiVertex, rgba), D3D11_INPUT_PER_VERTEX_DATA, 0},
    };
    return SUCCEEDED(device->CreateInputLayout(
        layout,
        static_cast<UINT>(std::size(layout)),
        vertexCode->GetBufferPointer(),
        vertexCode->GetBufferSize(),
        &gLayout
    ));
}

bool createStates(ID3D11Device* device) {
    D3D11_BUFFER_DESC constants{};
    constants.ByteWidth      = CONSTANT_FLOATS * sizeof(float);
    constants.Usage          = D3D11_USAGE_DYNAMIC;
    constants.BindFlags      = D3D11_BIND_CONSTANT_BUFFER;
    constants.CPUAccessFlags = D3D11_CPU_ACCESS_WRITE;

    D3D11_BLEND_DESC blend{};
    blend.RenderTarget[0].BlendEnable           = TRUE;
    blend.RenderTarget[0].SrcBlend              = D3D11_BLEND_ONE;
    blend.RenderTarget[0].DestBlend             = D3D11_BLEND_INV_SRC_ALPHA;
    blend.RenderTarget[0].BlendOp               = D3D11_BLEND_OP_ADD;
    blend.RenderTarget[0].SrcBlendAlpha         = D3D11_BLEND_INV_DEST_ALPHA;
    blend.RenderTarget[0].DestBlendAlpha        = D3D11_BLEND_ONE;
    blend.RenderTarget[0].BlendOpAlpha          = D3D11_BLEND_OP_ADD;
    blend.RenderTarget[0].RenderTargetWriteMask = D3D11_COLOR_WRITE_ENABLE_ALL;

    D3D11_RASTERIZER_DESC raster{};
    raster.FillMode        = D3D11_FILL_SOLID;
    raster.CullMode        = D3D11_CULL_NONE;
    raster.ScissorEnable   = TRUE;
    raster.DepthClipEnable = TRUE;

    D3D11_DEPTH_STENCIL_DESC depth{};
    depth.DepthEnable    = FALSE;
    depth.DepthWriteMask = D3D11_DEPTH_WRITE_MASK_ZERO;
    depth.DepthFunc      = D3D11_COMPARISON_ALWAYS;
    depth.StencilEnable  = FALSE;

    D3D11_SAMPLER_DESC sampler{};
    sampler.Filter         = D3D11_FILTER_MIN_MAG_MIP_LINEAR;
    sampler.AddressU       = D3D11_TEXTURE_ADDRESS_CLAMP;
    sampler.AddressV       = D3D11_TEXTURE_ADDRESS_CLAMP;
    sampler.AddressW       = D3D11_TEXTURE_ADDRESS_CLAMP;
    sampler.ComparisonFunc = D3D11_COMPARISON_ALWAYS;
    sampler.MaxLOD         = D3D11_FLOAT32_MAX;

    return SUCCEEDED(device->CreateBuffer(&constants, nullptr, &gConstants))
        && SUCCEEDED(device->CreateBlendState(&blend, &gBlend))
        && SUCCEEDED(device->CreateRasterizerState(&raster, &gRaster))
        && SUCCEEDED(device->CreateDepthStencilState(&depth, &gDepth))
        && SUCCEEDED(device->CreateSamplerState(&sampler, &gSampler));
}

// grows only: a buffer is made again, at least twice as large, when a frame needs more than it holds
bool ensureBuffer(
    ID3D11Device*         device,
    ComPtr<ID3D11Buffer>& buffer,
    UINT&                 capacity,
    UINT                  needed,
    UINT                  stride,
    UINT                  bind,
    UINT                  initial
) {
    if (buffer && needed <= capacity) return true;
    if (needed > MAX_BUFFER_ELEMENTS) return false;
    UINT grown = std::max({needed, initial, std::min(capacity * 2, MAX_BUFFER_ELEMENTS)});
    D3D11_BUFFER_DESC desc{};
    desc.ByteWidth      = grown * stride;
    desc.Usage          = D3D11_USAGE_DYNAMIC;
    desc.BindFlags      = bind;
    desc.CPUAccessFlags = D3D11_CPU_ACCESS_WRITE;
    buffer.Reset();
    capacity = 0;
    if (FAILED(device->CreateBuffer(&desc, nullptr, &buffer))) return false;
    capacity = grown;
    return true;
}

bool upload(ID3D11DeviceContext* ctx, ID3D11Buffer* buffer, void const* data, size_t bytes) {
    D3D11_MAPPED_SUBRESOURCE mapped{};
    if (FAILED(ctx->Map(buffer, 0, D3D11_MAP_WRITE_DISCARD, 0, &mapped))) return false;
    std::memcpy(mapped.pData, data, bytes);
    ctx->Unmap(buffer, 0);
    return true;
}

bool createTexture(ID3D11Device* device, FcUiTextureDelta const& delta, ComPtr<ID3D11ShaderResourceView>& view) {
    D3D11_TEXTURE2D_DESC desc{};
    desc.Width            = delta.width;
    desc.Height           = delta.height;
    desc.MipLevels        = 1;
    desc.ArraySize        = 1;
    desc.Format           = DXGI_FORMAT_R8G8B8A8_UNORM;
    desc.SampleDesc.Count = 1;
    desc.Usage            = D3D11_USAGE_DEFAULT;
    desc.BindFlags        = D3D11_BIND_SHADER_RESOURCE;
    D3D11_SUBRESOURCE_DATA data{};
    data.pSysMem     = delta.rgba;
    data.SysMemPitch = delta.width * BYTES_PER_PIXEL;
    ComPtr<ID3D11Texture2D> texture;
    if (FAILED(device->CreateTexture2D(&desc, &data, &texture))) return false;
    return SUCCEEDED(device->CreateShaderResourceView(texture.Get(), nullptr, &view));
}

// a full delta is create-or-replace: after fc_ui_reset_textures the font comes again with the same id
void applyFull(ID3D11Device* device, FcUiTextureDelta const& delta) {
    auto found = gTextures.find(delta.texture);
    if (found == gTextures.end() && gTextures.size() >= MAX_TEXTURES) {
        if (!gTextureLimitLogged) {
            logger().warn("ui renderer: more than {} UI textures; the extra textures are not drawn", MAX_TEXTURES);
        }
        gTextureLimitLogged = true;
        return;
    }
    ComPtr<ID3D11ShaderResourceView> view;
    if (!createTexture(device, delta, view)) {
        logger().warn("ui renderer: could not create UI texture {} ({}x{})", delta.texture, delta.width, delta.height);
        return;
    }
    if (found != gTextures.end()) {
        found->second = view;
    } else {
        gTextures.emplace(delta.texture, view);
    }
}

void applyPartial(ID3D11DeviceContext* ctx, FcUiTextureDelta const& delta) {
    auto found = gTextures.find(delta.texture);
    if (found == gTextures.end()) return;
    ComPtr<ID3D11Resource> resource;
    found->second->GetResource(&resource);
    ComPtr<ID3D11Texture2D> texture;
    if (FAILED(resource.As(&texture))) return;
    D3D11_TEXTURE2D_DESC desc{};
    texture->GetDesc(&desc);
    if (uint64_t{delta.x} + delta.width > desc.Width || uint64_t{delta.y} + delta.height > desc.Height) return;
    D3D11_BOX box{delta.x, delta.y, 0, delta.x + delta.width, delta.y + delta.height, 1};
    ctx->UpdateSubresource(texture.Get(), 0, &box, delta.rgba, delta.width * BYTES_PER_PIXEL, 0);
}

void applyTextures(ID3D11DeviceContext* ctx, ID3D11Device* device, FcUiFrame const& frame) {
    for (uint32_t i = 0; i < frame.texture_count; ++i) {
        FcUiTextureDelta const& delta  = frame.textures[i];
        size_t                  needed = static_cast<size_t>(delta.width) * delta.height * BYTES_PER_PIXEL;
        if (!delta.rgba || delta.width == 0 || delta.height == 0 || delta.byte_len < needed) continue;
        if (delta.full) {
            applyFull(device, delta);
        } else {
            applyPartial(ctx, delta);
        }
    }
}

void freeTextures(FcUiFrame const& frame) {
    for (uint32_t i = 0; i < frame.freed_count; ++i) gTextures.erase(frame.freed[i]);
}

bool uploadGeometry(ID3D11DeviceContext* ctx, ID3D11Device* device, FcUiFrame const& frame, float width, float height) {
    if (!ensureBuffer(device, gVertices, gVertexCapacity, frame.vertex_count, VERTEX_STRIDE, D3D11_BIND_VERTEX_BUFFER, INITIAL_VERTICES)
        || !ensureBuffer(device, gIndices, gIndexCapacity, frame.index_count, INDEX_STRIDE, D3D11_BIND_INDEX_BUFFER, INITIAL_INDICES)) {
        return false;
    }
    float constants[CONSTANT_FLOATS] = {width, height, 0.0f, 0.0f};
    return upload(ctx, gVertices.Get(), frame.vertices, static_cast<size_t>(frame.vertex_count) * VERTEX_STRIDE)
        && upload(ctx, gIndices.Get(), frame.indices, static_cast<size_t>(frame.index_count) * INDEX_STRIDE)
        && upload(ctx, gConstants.Get(), constants, sizeof(constants));
}

void bindPipeline(ID3D11DeviceContext* ctx, float width, float height) {
    D3D11_VIEWPORT viewport{0.0f, 0.0f, width, height, 0.0f, 1.0f};
    ctx->RSSetViewports(1, &viewport);
    ctx->RSSetState(gRaster.Get());
    UINT          stride   = VERTEX_STRIDE;
    UINT          offset   = 0;
    ID3D11Buffer* vertices = gVertices.Get();
    ctx->IASetInputLayout(gLayout.Get());
    ctx->IASetVertexBuffers(0, 1, &vertices, &stride, &offset);
    ctx->IASetIndexBuffer(gIndices.Get(), DXGI_FORMAT_R32_UINT, 0);
    ctx->IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
    ID3D11Buffer* constants = gConstants.Get();
    ctx->VSSetShader(gVertexShader.Get(), nullptr, 0);
    ctx->VSSetConstantBuffers(0, 1, &constants);
    ctx->GSSetShader(nullptr, nullptr, 0);
    ctx->HSSetShader(nullptr, nullptr, 0);
    ctx->DSSetShader(nullptr, nullptr, 0);
    ID3D11SamplerState* sampler = gSampler.Get();
    ctx->PSSetShader(gPixelShader.Get(), nullptr, 0);
    ctx->PSSetSamplers(0, 1, &sampler);
    float const blendFactor[4] = {0.0f, 0.0f, 0.0f, 0.0f};
    ctx->OMSetBlendState(gBlend.Get(), blendFactor, 0xffffffffu);
    ctx->OMSetDepthStencilState(gDepth.Get(), 0);
}

// the clip rectangle in pixels, cut to the target; false when nothing of it is left
bool scissorFor(FcUiDraw const& draw, float width, float height, D3D11_RECT& rect) {
    rect.left   = static_cast<LONG>(std::floor(std::clamp(draw.clip[0], 0.0f, width)));
    rect.top    = static_cast<LONG>(std::floor(std::clamp(draw.clip[1], 0.0f, height)));
    rect.right  = static_cast<LONG>(std::ceil(std::clamp(draw.clip[2], 0.0f, width)));
    rect.bottom = static_cast<LONG>(std::ceil(std::clamp(draw.clip[3], 0.0f, height)));
    return rect.right > rect.left && rect.bottom > rect.top;
}

bool drawInRange(FcUiDraw const& draw, FcUiFrame const& frame) {
    return draw.index_count != 0 && draw.index_offset <= frame.index_count
        && draw.index_count <= frame.index_count - draw.index_offset && draw.vertex_offset < frame.vertex_count;
}

void drawCalls(ID3D11DeviceContext* ctx, FcUiFrame const& frame, float width, float height) {
    for (uint32_t i = 0; i < frame.draw_count; ++i) {
        FcUiDraw const& draw = frame.draws[i];
        if (!drawInRange(draw, frame)) continue;
        auto found = gTextures.find(draw.texture);
        if (found == gTextures.end()) continue;
        D3D11_RECT rect{};
        if (!scissorFor(draw, width, height, rect)) continue;
        ID3D11ShaderResourceView* view = found->second.Get();
        ctx->RSSetScissorRects(1, &rect);
        ctx->PSSetShaderResources(0, 1, &view);
        ctx->DrawIndexed(draw.index_count, draw.index_offset, static_cast<INT>(draw.vertex_offset));
    }
}

void drawGeometry(ID3D11DeviceContext* ctx, ID3D11Device* device, FcUiFrame const& frame, float width, float height) {
    if (frame.skipped || frame.draw_count == 0 || frame.vertex_count == 0 || frame.index_count == 0) return;
    if (width <= 0.0f || height <= 0.0f) return;
    if (!uploadGeometry(ctx, device, frame, width, height)) return;
    bindPipeline(ctx, width, height);
    drawCalls(ctx, frame, width, height);
    // the texture view must not stay bound past the draw: a freed texture would live on through the binding
    ID3D11ShaderResourceView* none = nullptr;
    ctx->PSSetShaderResources(0, 1, &none);
}

} // namespace

bool EguiRenderer::init(ID3D11Device* device) {
    release();
    if (!device || !createShaders(device) || !createStates(device)) {
        release();
        return false;
    }
    gReady = true;
    return true;
}

bool EguiRenderer::ready() { return gReady; }

void EguiRenderer::draw(ID3D11DeviceContext* ctx, ID3D11Device* device, FcUiFrame const& frame, float width, float height) {
    if (!gReady || !ctx || !device) return;
    applyTextures(ctx, device, frame);
    drawGeometry(ctx, device, frame, width, height);
    freeTextures(frame);
}

void EguiRenderer::releaseTextures() {
    gTextures.clear();
    gTextureLimitLogged = false;
}

void EguiRenderer::release() {
    releaseTextures();
    gVertices.Reset();
    gIndices.Reset();
    gVertexCapacity = 0;
    gIndexCapacity  = 0;
    gSampler.Reset();
    gDepth.Reset();
    gRaster.Reset();
    gBlend.Reset();
    gConstants.Reset();
    gLayout.Reset();
    gPixelShader.Reset();
    gVertexShader.Reset();
    gReady = false;
}

} // namespace levi_freecam
