use typst::layout::{Abs, Frame, FrameItem, PagedDocument, Point};
use typst::text::{Glyph, TextItem};

/// Returns an iterator over all words in a document, with their bounding boxes.
pub fn words_with_boxes(
    document: &PagedDocument,
    include_whitespace: bool,
    include_delimiters: bool
) -> impl Iterator<Item = (String, (f64, f64, f64, f64))> + '_ {
    document.pages.iter().flat_map(move |page| {
        words_in_frame(&page.frame, include_whitespace, include_delimiters)
    })
}

/// Returns an iterator over all words in a frame, with their bounding boxes.
fn words_in_frame(
    frame: &Frame,
    include_whitespace: bool,
    include_delimiters: bool
) -> impl Iterator<Item = (String, (f64, f64, f64, f64))> + '_ {
    // Recursively traverse frames and collect words
    fn traverse_frames(
        frame: &Frame,
        base_pos: Point,
        words: &mut Vec<(String, (f64, f64, f64, f64))>,
        include_whitespace: bool,
        include_delimiters: bool,
    ) {
        for (pos, item) in frame.items() {
            let absolute_pos = base_pos + *pos;
            match item {
                FrameItem::Text(text_item) => {
                    process_text_item(
                        &absolute_pos,
                        text_item,
                        words,
                        include_whitespace,
                        include_delimiters,
                    );
                }
                FrameItem::Group(group) => {
                    // Recurse into nested group
                    traverse_frames(
                        &group.frame,
                        absolute_pos,
                        words,
                        include_whitespace,
                        include_delimiters,
                    );
                }
                _ => {}
            }
        }
    }

    let mut words = Vec::new();
    traverse_frames(
        frame,
        Point::zero(),
        &mut words,
        include_whitespace,
        include_delimiters,
    );
    words.into_iter()
}

/// Processes a text item to extract words and their bounding boxes.
/// This function splits words based on whitespace and punctuation.
fn process_text_item(
    pos: &Point, 
    text_item: &TextItem, 
    words: &mut Vec<(String, (f64, f64, f64, f64))>,
    include_whitespace: bool,
    include_delimiters: bool
) {
    let text = &text_item.text;
    let glyphs = &text_item.glyphs;
    if glyphs.is_empty() {
        return;
    }

    let size = text_item.size;
    let ascender = text_item.font.metrics().ascender.at(size).to_pt();
    let descender = text_item.font.metrics().descender.at(size).to_pt();
    let height = ascender - descender;

    // Index of the first glyph of the current word.
    let mut word_start_glyph_index = 0;
    // Horizontal position where the current word starts, relative to the TextItem's origin.
    let mut word_start_x = Abs::zero();
    // The current horizontal position, advancing with each glyph.
    let mut current_x = Abs::zero();

    for (i, glyph) in glyphs.iter().enumerate() {
        let start_byte = glyph.range.start as usize;
        let end_byte = glyph.range.end as usize;
        let glyph_text = &text[start_byte..end_byte];
    
        // A glyph is a delimiter if all its characters are whitespace or punctuation.
        let is_delimiter = !glyph_text.is_empty() && glyph_text.chars().all(|c| c.is_whitespace() || c.is_ascii_punctuation());
        let is_whitespace = !glyph_text.is_empty() && glyph_text.chars().all(|c| c.is_whitespace());

        if is_delimiter {
            // If we have a pending word, finalize it.
            if word_start_glyph_index < i {
                let word_glyphs = &glyphs[word_start_glyph_index..i];
                finalize_word(pos, text, word_glyphs, word_start_x, ascender, height, size, words);
            }
            // Finalize the delimiter or whitespace itself.
            if (!is_whitespace || include_whitespace) && (is_whitespace || include_delimiters) {
                finalize_word(pos, text, &[glyph.clone()], current_x, ascender, height, size, words);
            }
            // The next word will start after this delimiter glyph.
            word_start_glyph_index = i + 1;
        }

        // Advance the cursor by the width of the current glyph.
        current_x += glyph.x_advance.at(size);

        if is_delimiter {
            // The next word will start at the new cursor position.
            word_start_x = current_x;
        }
    }

    // Finalize any trailing word at the end of the text item.
    if word_start_glyph_index < glyphs.len() {
        let word_glyphs = &glyphs[word_start_glyph_index..];
        finalize_word(pos, text, word_glyphs, word_start_x, ascender, height, size, words);
    }
}

/// Helper to construct the word string and bounding box and add it to the list.
fn finalize_word(
    pos: &Point,
    text: &str,
    word_glyphs: &[Glyph],
    word_start_x: Abs,
    ascender: f64,
    height: f64,
    font_size: Abs,
    words: &mut Vec<(String, (f64, f64, f64, f64))>,
) {
    if word_glyphs.is_empty() {
        return;
    }
    
    // Determine the text of the word from the glyph ranges.
    let start_byte = word_glyphs.first().unwrap().range.start as usize;
    let end_byte = word_glyphs.last().unwrap().range.end as usize;
    let word_text = &text[start_byte..end_byte];

    // The width of the word is the sum of the advances of its glyphs.
    let width: Abs = word_glyphs.iter().map(|g| g.x_advance.at(font_size)).sum();

    // The glyph's x_offset is a slight adjustment to its position.
    // We only need the one from the first glyph.
    let x_offset = word_glyphs.first().unwrap().x_offset.at(font_size);

    // Calculate the final bounding box coordinates.
    let x = pos.x.to_pt() + word_start_x.to_pt() + x_offset.to_pt();
    let y = pos.y.to_pt() - ascender;

    // The splitting logic is now precise, so no .trim() is needed.
    words.push((word_text.to_string(), (x, y, width.to_pt(), height)));
}