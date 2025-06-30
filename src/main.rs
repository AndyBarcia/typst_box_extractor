mod render;
mod word_analysis;
mod world;

use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use serde::Serialize;
use typst::layout::PagedDocument;

use render::{render_to_png,render_to_png_with_boxes};
use word_analysis::words_with_boxes;
use world::TypstWrapperWorld;

#[derive(Serialize)]
struct WordBox {
    word: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The path to the input Typst file.
    input: PathBuf,

    /// The path for the output JSON file.
    output: PathBuf,

    /// Optional: The path for the rendered PNG file.
    #[arg(short, long, default_value = "output.png")]
    render: PathBuf,

    /// Optional: The path for the rendered PNG file.
    #[arg(long, default_value = "output_boxes.png")]
    render_boxes: PathBuf,

    // Whether to include boxes of whitespace.
    #[arg(long, action)]
    include_whitespace: bool,

    // Whether to include boxes of delimiters.
    #[arg(long, action)]
    include_delimiters: bool
}

fn main() {
    let cli = Cli::parse();
    
    let content = fs::read_to_string(&cli.input)
        .expect("Error: Could not read the input file.");
    let root_path = cli.input.parent().unwrap_or_else(|| Path::new(""));
    let world = TypstWrapperWorld::new(root_path.to_str().unwrap().to_owned(), content);

    // Layout document
    let document: PagedDocument = typst::compile(&world)
        .output
        .expect("Error compiling typst");

    // Collect word and box data into our `WordBox` struct.
    let mut word_boxes: Vec<WordBox> = vec![];
    for (word, (x, y, w, h)) in words_with_boxes(&document, cli.include_whitespace, cli.include_delimiters) {
        word_boxes.push(WordBox {
            word: word.to_string(),
            x,
            y,
            width: w,
            height: h,
        });
    }

    // Serialize the vector of WordBox structs into a pretty JSON string.
    let json_output = serde_json::to_string_pretty(&word_boxes)
        .expect("Failed to serialize data to JSON.");    
    fs::write(&cli.output, json_output)
        .expect("Failed to write JSON output file.");
    println!("✅ Successfully wrote word analysis to {}", cli.output.display());
    
    // Render a PNG as before, using the path from the CLI args.
    let pixmap = render_to_png(&document, 1.0);
    let data: Vec<u8> = pixmap.encode_png().unwrap();
    fs::write(&cli.render, data).unwrap();
    println!("✅ Rendered PNG to {}", cli.render.display());

    // Render a PNG, now passing the word_boxes to draw them.
    let pixmap_boxes = render_to_png_with_boxes(&document, 1.0, &word_boxes);
    let data: Vec<u8> = pixmap_boxes.encode_png().unwrap();
    fs::write(&cli.render_boxes, data).unwrap();
    println!("✅ Rendered PNG to {}", cli.render.display());
}