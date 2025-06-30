use crate::WordBox;
use typst::layout::{Abs, PagedDocument};
use typst::visualize::Color;
use tiny_skia;
use tiny_skia::{Transform,Paint,Stroke,Rect,PathBuilder};

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

/// Draw all frames into one image with padding in between and overlay word boxes.
pub fn render_to_png_with_boxes(
    document: &PagedDocument,
    pixel_per_pt: f32,
    word_boxes: &[WordBox],
) -> tiny_skia::Pixmap {
    for page in &document.pages {
        let limit = Abs::cm(100.0);
        if page.frame.width() > limit || page.frame.height() > limit {
            panic!("overlarge frame: {:?}", page.frame.size());
        }
    }

    let gap = Abs::pt(1.0);
    let mut pixmap = typst_render::render_merged(document, pixel_per_pt, gap, Some(Color::BLACK));

    // Define the paint for the stroke
    let mut stroke_paint = Paint::default();
    stroke_paint.set_color_rgba8(255, 0, 0, 180); // Red with some transparency
    stroke_paint.anti_alias = true;

    // Define the stroke properties
    let stroke = Stroke {
        width: 1.0,
        ..Default::default()
    };

    // Iterate over the word boxes and draw a rectangle for each
    for word_box in word_boxes {
        // Create a rectangle from the word box coordinates, scaling by pixel_per_pt
        let rect = Rect::from_xywh(
            word_box.x as f32 * pixel_per_pt,
            word_box.y as f32 * pixel_per_pt,
            word_box.width as f32 * pixel_per_pt,
            word_box.height as f32 * pixel_per_pt,
        );

        if let Some(rect) = rect {
            // Create a path from the rectangle
            let path = PathBuilder::from_rect(rect);
            // Stroke the path on the pixmap
            pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
        }
    }

    pixmap
}