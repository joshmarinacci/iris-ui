//! Extracts Rust snippets from docs/layout.md that are followed by a markdown image
//! reference, and generates examples/doc_screenshots.rs, which renders each snippet's
//! scene headlessly and writes the screenshot back to docs/.
//!
//! Run via scripts/gen_doc_screenshots.sh, which runs this and then the generated example.

use std::fs;
use std::path::Path;

struct Snippet {
    code: String,
    image_file: String,
}

fn extract_snippets(markdown: &str) -> Vec<Snippet> {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut snippets = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim() == "```rust" {
            let start = i + 1;
            let mut end = start;
            while end < lines.len() && lines[end].trim() != "```" {
                end += 1;
            }
            let code = lines[start..end].join("\n");

            // Look ahead for an image reference before the next heading or code fence.
            let mut j = end + 1;
            let mut image_file = None;
            while j < lines.len() {
                let line = lines[j].trim();
                if line.starts_with('#') || line == "```rust" {
                    break;
                }
                if let Some(file) = parse_image_line(line) {
                    image_file = Some(file);
                    break;
                }
                j += 1;
            }

            if let Some(image_file) = image_file {
                snippets.push(Snippet { code, image_file });
            }

            i = end + 1;
        } else {
            i += 1;
        }
    }
    snippets
}

/// Parses a markdown image line like `![alt text](file.png)` and returns the filename.
fn parse_image_line(line: &str) -> Option<String> {
    let rest = line.strip_prefix("![")?;
    let (_alt, rest) = rest.split_once(']')?;
    let rest = rest.strip_prefix('(')?;
    let (file, _) = rest.split_once(')')?;
    Some(file.to_string())
}

fn generate_source(snippets: &[Snippet]) -> String {
    let mut out = String::new();
    out.push_str(PREAMBLE);

    for (n, snippet) in snippets.iter().enumerate() {
        out.push_str(&format!("\nfn scene_{n}() -> Scene<Rgb565> {{\n"));
        out.push_str(&snippet.code);
        out.push_str("\n    scene\n}\n");
    }

    out.push_str("\nfn main() {\n");
    out.push_str("    let docs_dir = Path::new(env!(\"CARGO_MANIFEST_DIR\")).join(\"docs\");\n");
    out.push_str("    let mut theme = BW_THEME;\n");
    out.push_str("    copy_theme_colors(&mut theme, &LIGHT_THEME);\n");
    out.push_str("    let snapshots: Vec<(&str, fn() -> Scene<Rgb565>)> = vec![\n");
    for (n, snippet) in snippets.iter().enumerate() {
        out.push_str(&format!(
            "        ({:?}, scene_{n} as fn() -> Scene<Rgb565>),\n",
            snippet.image_file
        ));
    }
    out.push_str("    ];\n");
    out.push_str("    for (image_file, make_scene) in snapshots {\n");
    out.push_str("        let mut scene = make_scene();\n");
    out.push_str("        layout_scene(&mut scene, &theme);\n");
    out.push_str(
        "        let mut display: SimulatorDisplay<Rgb565> = SimulatorDisplay::new(Size::new(320, 240));\n",
    );
    out.push_str("        {\n");
    out.push_str("            let mut ctx = EmbeddedDrawingContext::new(&mut display);\n");
    out.push_str("            ctx.clip = scene.dirty_rect;\n");
    out.push_str("            draw_scene(&mut scene, &mut ctx, &theme);\n");
    out.push_str("        }\n");
    out.push_str("        let output_settings = OutputSettings::default();\n");
    out.push_str(
        "        let path = docs_dir.join(image_file);\n",
    );
    out.push_str(
        "        display.to_rgb_output_image(&output_settings).save_png(&path).expect(\"failed to save screenshot\");\n",
    );
    out.push_str("        println!(\"wrote {}\", path.display());\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    out
}

const PREAMBLE: &str = r##"// GENERATED FILE - do not edit by hand.
// Regenerate with scripts/gen_doc_screenshots.sh (see examples/gen_doc_screenshots.rs).
#![allow(unused_imports)]

use embedded_graphics::geometry::Size;
use embedded_graphics::mono_font::ascii::FONT_7X13_BOLD;
use embedded_graphics::mono_font::iso_8859_9::FONT_7X13;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::prelude::WebColors;
use embedded_graphics_simulator::{OutputSettings, SimulatorDisplay};
use iris_ui::button::make_button;
use iris_ui::device::EmbeddedDrawingContext;
use iris_ui::geom::{Bounds, Insets};
use iris_ui::grid::{GridLayoutState, LayoutConstraint, make_grid_panel};
use iris_ui::label::make_label;
use iris_ui::layouts::{layout_hbox, layout_std_panel, layout_vbox};
use iris_ui::panel::{PanelState, make_panel};
use iris_ui::scene::{Scene, draw_scene, layout_scene};
use iris_ui::tabbed_panel::{LayoutPanelState, make_tabbed_panel};
use iris_ui::text_input::make_text_input;
use iris_ui::util::hex_str_to_rgb565;
use iris_ui::view::{Align, Flex, View, ViewId};
use iris_ui::{BW_THEME, FontKind, Theme, ViewStyle};
use std::path::Path;

const LIGHT_THEME: Theme<Rgb565> = Theme {
    font: FontKind::Bitmap(FONT_7X13),
    bold_font: FontKind::Bitmap(FONT_7X13_BOLD),
    standard: ViewStyle {
        fill: Rgb565::WHITE,
        text: Rgb565::BLACK,
    },
    panel: ViewStyle {
        fill: Rgb565::CSS_LIGHT_GRAY,
        text: Rgb565::BLACK,
    },
    selected: ViewStyle {
        fill: hex_str_to_rgb565("#444444"),
        text: Rgb565::WHITE,
    },
    accented: ViewStyle {
        fill: hex_str_to_rgb565("#6688dd"),
        text: Rgb565::WHITE,
    },
};

fn copy_theme_colors(theme: &mut Theme<Rgb565>, new: &Theme<Rgb565>) {
    theme.standard = new.standard.clone();
    theme.panel = new.panel.clone();
    theme.selected = new.selected.clone();
    theme.accented = new.accented.clone();
}
"##;

fn main() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let doc_path = manifest_dir.join("docs/layout.md");
    let markdown = fs::read_to_string(&doc_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", doc_path.display()));

    let snippets = extract_snippets(&markdown);
    println!(
        "found {} snippet(s) with screenshots in {}",
        snippets.len(),
        doc_path.display()
    );
    for snippet in &snippets {
        println!("  -> {}", snippet.image_file);
    }

    let source = generate_source(&snippets);
    let out_path = manifest_dir.join("examples/doc_screenshots.rs");
    fs::write(&out_path, source)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", out_path.display()));
    println!("generated {}", out_path.display());
}
