use egui::epaint::textures::TexturesDelta;
use egui::epaint::{ClippedPrimitive, ImageData, Primitive, TextureId};
use freecam_core::structs::{UiDraw, UiFrameOutput, UiTextureDelta, UiVertex};

/// Flattens egui's tessellated meshes and texture changes into the reused buffers of a [`UiFrameOutput`].
pub(crate) struct MeshWriter;

impl MeshWriter {
    /// Managed egui texture `n` becomes texture `n + TEXTURE_BASE`; lower ids stay free for the shim.
    pub(crate) const TEXTURE_BASE: u64 = 16;

    /// Writes every texture upload and every freed texture.
    pub(crate) fn write_textures(delta: &TexturesDelta, out: &mut UiFrameOutput) {
        for (id, images) in &delta.set {
            let Some(texture) = Self::texture(*id) else {
                continue;
            };
            for image in images {
                let ImageData::Color(color) = &image.image;
                let [width, height] = color.size;
                let [x, y] = image.pos.unwrap_or([0, 0]);
                let byte_offset = out.texture_bytes.len();
                out.texture_bytes.reserve(color.pixels.len() * 4);
                for pixel in &color.pixels {
                    out.texture_bytes.extend_from_slice(&pixel.to_array());
                }
                out.textures.push(UiTextureDelta {
                    texture,
                    full: image.pos.is_none(),
                    x: x as u32,
                    y: y as u32,
                    width: width as u32,
                    height: height as u32,
                    byte_offset,
                    byte_len: out.texture_bytes.len() - byte_offset,
                });
            }
        }
        out.freed.extend(delta.free.iter().filter_map(|id| Self::texture(*id)));
    }

    /// Writes vertices, indices and draws in physical pixels. More than `max_vertices` vertices writes nothing and
    /// marks the frame skipped.
    pub(crate) fn write_meshes(
        primitives: &[ClippedPrimitive],
        pixels_per_point: f32,
        screen: [f32; 2],
        max_vertices: usize,
        out: &mut UiFrameOutput,
    ) {
        let total: usize = primitives
            .iter()
            .map(|clipped| match &clipped.primitive {
                Primitive::Mesh(mesh) => mesh.vertices.len(),
                Primitive::Callback(_) => 0,
            })
            .sum();
        if total > max_vertices {
            out.skipped = true;
            return;
        }
        for clipped in primitives {
            let Primitive::Mesh(mesh) = &clipped.primitive else {
                continue;
            };
            let Some(texture) = Self::texture(mesh.texture_id) else {
                continue;
            };
            let clip = Self::clip(clipped.clip_rect, pixels_per_point, screen);
            if mesh.indices.is_empty() || clip[2] <= clip[0] || clip[3] <= clip[1] {
                continue;
            }
            out.draws.push(UiDraw {
                clip,
                texture,
                index_offset: out.indices.len() as u32,
                index_count: mesh.indices.len() as u32,
                vertex_offset: out.vertices.len() as u32,
            });
            out.indices.extend_from_slice(&mesh.indices);
            out.vertices.extend(mesh.vertices.iter().map(|vertex| UiVertex {
                x: vertex.pos.x * pixels_per_point,
                y: vertex.pos.y * pixels_per_point,
                u: vertex.uv.x,
                v: vertex.uv.y,
                rgba: vertex.color.to_array(),
            }));
        }
    }

    // the clip rect in whole physical pixels, inside the screen; max and min, unlike clamp, cannot panic on NaN
    fn clip(rect: egui::Rect, pixels_per_point: f32, screen: [f32; 2]) -> [f32; 4] {
        let x0 = (rect.min.x * pixels_per_point).round().max(0.0).min(screen[0]);
        let y0 = (rect.min.y * pixels_per_point).round().max(0.0).min(screen[1]);
        let x1 = (rect.max.x * pixels_per_point).round().max(x0).min(screen[0]);
        let y1 = (rect.max.y * pixels_per_point).round().max(y0).min(screen[1]);
        [x0, y0, x1, y1]
    }

    // user textures do not exist in this UI, so only managed ones map to an id
    fn texture(id: TextureId) -> Option<u64> {
        match id {
            TextureId::Managed(n) => Some(n + Self::TEXTURE_BASE),
            TextureId::User(_) => None,
        }
    }
}
