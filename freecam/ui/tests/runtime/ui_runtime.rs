use freecam_core::structs::UiFrameOutput;

use crate::support::ui_fixture::UiFixture;

fn max_x(out: &UiFrameOutput) -> f32 {
    out.vertices.iter().map(|vertex| vertex.x).fold(f32::MIN, f32::max)
}

#[test]
fn first_frame_uploads_font_texture_in_full() {
    let mut fixture = UiFixture::new();

    let out = fixture.frame(&[]);

    assert!(!out.textures.is_empty());
    let font = out.textures[0];
    assert!(font.full);
    assert_eq!(font.byte_len, font.width as usize * font.height as usize * 4);
    assert!(font.byte_offset + font.byte_len <= out.texture_bytes.len());
}

#[test]
fn identical_second_frame_has_no_texture_change() {
    let mut fixture = UiFixture::new();
    fixture.frame(&[]);

    let out = fixture.frame(&[]);

    assert!(out.textures.is_empty());
    assert!(out.texture_bytes.is_empty());
    assert!(out.freed.is_empty());
}

#[test]
fn draws_cover_every_index() {
    let mut fixture = UiFixture::new();

    let out = fixture.frame(&[]);

    assert!(!out.draws.is_empty());
    let covered: u32 = out.draws.iter().map(|draw| draw.index_count).sum();
    assert_eq!(covered as usize, out.indices.len());
    for draw in &out.draws {
        let range = draw.index_offset as usize..(draw.index_offset + draw.index_count) as usize;
        for index in &out.indices[range] {
            assert!(((index + draw.vertex_offset) as usize) < out.vertices.len());
        }
    }
}

#[test]
fn clip_rects_stay_inside_the_screen() {
    let mut fixture = UiFixture::new().cursor_mode();
    let (width, height) = (fixture.input.screen_width, fixture.input.screen_height);

    let out = fixture.frame(&[]);

    assert!(!out.draws.is_empty());
    for draw in &out.draws {
        let [x0, y0, x1, y1] = draw.clip;
        assert!(0.0 <= x0 && x0 <= x1 && x1 <= width, "clip {:?}", draw.clip);
        assert!(0.0 <= y0 && y0 <= y1 && y1 <= height, "clip {:?}", draw.clip);
    }
}

#[test]
fn vertices_scale_with_pixels_per_point() {
    let mut normal = UiFixture::new();
    let mut scaled = UiFixture::new().pixels_per_point(1.5);

    let normal_x = max_x(normal.frame(&[]));
    let scaled_x = max_x(scaled.frame(&[]));

    let expected = normal_x * 1.5;
    assert!((scaled_x - expected).abs() <= expected * 0.02, "{scaled_x} vs {expected}");
}

#[test]
fn over_max_vertices_is_skipped_but_uploads_fonts() {
    let mut fixture = UiFixture::with_max_vertices(16);

    let out = fixture.frame(&[]);

    assert!(out.skipped);
    assert!(out.draws.is_empty());
    assert!(out.vertices.is_empty());
    assert!(!out.textures.is_empty());
}

#[test]
fn hidden_snapshot_draws_nothing() {
    let mut fixture = UiFixture::new();
    fixture.snapshot.visible = false;

    let out = fixture.frame(&[]);

    assert!(out.vertices.is_empty());
    assert!(out.draws.is_empty());
    assert!(!out.skipped);
}

#[test]
fn reset_textures_sends_the_font_texture_in_full_again() {
    let mut fixture = UiFixture::new();
    let first = fixture.frame(&[]).textures[0];
    fixture.frame(&[]);

    fixture.runtime.reset_textures();
    let out = fixture.frame(&[]);

    let font = out.textures.iter().find(|delta| delta.texture == first.texture);
    assert!(font.is_some_and(|font| font.full && font.width == first.width && font.height == first.height));
    assert!(!out.draws.is_empty());
}
