use typst::layout::{Abs, Frame, FrameItem, PagedDocument, Point};
use typst::text::{Glyph, TextItem};

/// Returns an iterator over all words in a document, with their bounding boxes.
pub fn words_with_boxes(document: &PagedDocument) -> impl Iterator<Item = (String, (f64, f64, f64, f64))> + '_ {
    document.pages.iter().flat_map(|page| {
        words_in_frame(&page.frame)
    })
}

/// Returns an iterator over all words in a frame, with their bounding boxes.
fn words_in_frame(frame: &Frame) -> impl Iterator<Item = (String, (f64, f64, f64, f64))> + '_ {
    frame.items().flat_map(|(pos, item)| {
        let mut words = Vec::new();
        match item {
            FrameItem::Text(text_item) => {
                process_text_item(pos, text_item, &mut words);
            }
            FrameItem::Group(group) => {
                // Recursively process groups
                for (sub_pos, sub_item) in group.frame.items() {
                    if let FrameItem::Text(text_item) = sub_item {
                        let point = *pos + *sub_pos;
                        process_text_item(&point, text_item, &mut words);
                    }
                }
            }
            _ => {}
        }
        words.into_iter()
    })
}

/// Processes a text item to extract words and their bounding boxes.
fn process_text_item(pos: &Point, text_item: &TextItem, words: &mut Vec<(String, (f64, f64, f64, f64))>) {
    let text = &text_item.text.to_string();
    let glyphs = &text_item.glyphs;
    if glyphs.is_empty() {
        return;
    }

    let ascender = text_item.font.metrics().ascender.at(text_item.size).to_pt();
    let descender = text_item.font.metrics().descender.at(text_item.size).to_pt();
    let height = ascender - descender;

    let mut current_word_glyphs = Vec::new();
    let mut current_word_start_x = 0.0;
    let mut current_x = 0.0;
    let mut last_text_idx: usize = 0;

    for (i, glyph) in glyphs.iter().enumerate() {
        let glyph_text_start = glyph.range.start as usize;

        if last_text_idx < glyph_text_start {
            let text_chunk = &text[last_text_idx..glyph_text_start];
            if text_chunk.chars().any(|c| c.is_whitespace() || c.is_ascii_punctuation()) && !current_word_glyphs.is_empty() {
                finalize_word(pos, text, &current_word_glyphs, current_word_start_x, ascender, height, text_item.size, words);
                current_word_glyphs.clear();
            }
        }

        if current_word_glyphs.is_empty() {
            current_word_start_x = current_x;
        }

        current_word_glyphs.push(glyph);
        current_x += glyph.x_advance.at(text_item.size).to_pt();
        last_text_idx = glyph.range.end as usize;

        // Also check for trailing whitespace on the last glyph.
        let next_glyph_text_start = if i < glyphs.len() - 1 {
            glyphs[i + 1].range.start as usize
        } else {
            text.len()
        };
        let glyph_text_chunk = &text[last_text_idx..next_glyph_text_start];
        if glyph_text_chunk.chars().any(|c| c.is_whitespace() || c.is_ascii_punctuation()) && !current_word_glyphs.is_empty() {
            finalize_word(pos, text, &current_word_glyphs, current_word_start_x, ascender, height, text_item.size, words);
            current_word_glyphs.clear();
        }
    }

    if !current_word_glyphs.is_empty() {
        finalize_word(pos, text, &current_word_glyphs, current_word_start_x, ascender, height, text_item.size, words);
    }
}

/// Helper to construct the word string and bounding box and add it to the list.
fn finalize_word<'a>(
    pos: &Point,
    text: &str,
    word_glyphs: &[&'a Glyph],
    word_start_x: f64,
    ascender: f64,
    height: f64,
    font_size: Abs,
    words: &mut Vec<(String, (f64, f64, f64, f64))>,
) {
    if word_glyphs.is_empty() {
        return;
    }
    let start_byte = word_glyphs.first().unwrap().range.start as usize;
    let end_byte = word_glyphs.last().unwrap().range.end as usize;

    let word_text = &text[start_byte..end_byte];

    let width: f64 = word_glyphs.iter().map(|g| g.x_advance.at(font_size).to_pt()).sum();
    let x_offset = word_glyphs.first().unwrap().x_offset.at(font_size).to_pt();

    let x = pos.x.to_pt() + word_start_x + x_offset;
    let y = pos.y.to_pt() - ascender;

    words.push((word_text.trim().to_string(), (x, y, width, height)));
}