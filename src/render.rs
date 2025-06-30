use typst::layout::{Abs, PagedDocument};
use typst::visualize::Color;
use tiny_skia;

/// Draw all frames into one image with padding in between.
pub fn render_to_png(document: &PagedDocument, pixel_per_pt: f32) -> tiny_skia::Pixmap {
    for page in &document.pages {
        let limit = Abs::cm(100.0);
        if page.frame.width() > limit || page.frame.height() > limit {
            panic!("overlarge frame: {:?}", page.frame.size());
        }
    }

    let gap = Abs::pt(1.0);
    typst_render::render_merged(document, pixel_per_pt, gap, Some(Color::BLACK))
}