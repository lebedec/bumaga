/*use std::fs;
use std::time::Instant;

use cosmic_text::{
    Attrs, Buffer, CacheKeyFlags, Family, FontSystem, Metrics, Shaping, Stretch, Style, Weight,
    Wrap,
};
use lightningcss::printer::PrinterOptions;
use lightningcss::properties::font::{FontStretchKeyword, FontStyle};
use lightningcss::properties::text::OverflowWrap;
use lightningcss::traits::ToCss;
use serde_json::{json, Value};

use bumaga::{Component, Fonts, Input, TextStyle};

#[test]
fn test_something() {
    // awake
    let html = fs::read_to_string("./assets/index.html").expect("index.html");
    let css = fs::read_to_string("./assets/style.css").expect("style.css");
    let mut component = Component::compile(&html, &css);

    // update cycle
    let value: Value = json!({
        "name": "Alice",
        "nested": {
            "propertyA": 42,
            "propertyB": 43
        },
        "items": ["a", 32, "b", 33],
        "visible": true,
        "collection": [
            {"value": "v1", "name": "value 1"},
            {"value": "v2", "name": "value 2"},
        ]
    });

    let fonts = &mut CosmicFonts::new();

    let t = Instant::now();

    let input = Input::new()
        .fonts(fonts)
        .value(value.clone())
        .mouse([15.0, 15.0], true);
    let frame = component.update(input);

    for i in 0..60 {
        let input = Input::new()
            .fonts(fonts)
            .value(value.clone())
            .mouse([15.0, 15.0], true);
        let frame = component.update(input);
    }

    let input = Input::new()
        .fonts(fonts)
        .value(value.clone())
        .mouse([15.0, 15.0], false);
    let output = component.update(input);
    let t = t.elapsed().as_secs_f32();
    println!("elapsed: {t}");

    // drawing
    for call in output.calls {
        println!("CALL {:?} {:?}", call.function, call.arguments);
        println!("{:?}", call.arguments[0].as_f64());
        println!("{:?}", call.arguments[1].as_bool());
        println!("{:?}", call.arguments[2].as_str());
        println!("{:?}", call.arguments[3].as_bool());
    }
    let mut result = String::new();
    result += "<style>body { font-family: \"Courier New\"; font-size: 14px; }</style>\n";
    for view in output.elements {
        let layout = &view.layout;
        let k = &view
            .html_element
            .as_ref()
            .map(|el| el.name.local.to_string())
            .unwrap_or(String::from("text"));
        let x = layout.location.x;
        let y = layout.location.y;
        let w = layout.size.width;
        let h = layout.size.height;
        let empty = String::new();
        let t = view.text.as_ref().unwrap_or(&empty);
        let mut bg = view
            .background
            .color
            .to_css_string(PrinterOptions::default())
            .expect("css color");
        if let Some(img) = view.background.image.as_ref() {
            // println!("img {img}");
            bg = format!("url({img})");
        }
        let record = format!("<div key=\"{k}\" style=\"position: fixed; top: {y}px; left: {x}px; width: {w}px; height: {h}px; background: {bg};\">{t}</div>\n");
        result += &record;
    }
    fs::write("./assets/result.html", result).expect("result written");
}

pub struct CosmicFonts {
    pub(crate) font_system: FontSystem,
}

impl CosmicFonts {
    pub fn new() -> Self {
        let font_system = FontSystem::new();
        Self { font_system }
    }
}

impl Fonts for CosmicFonts {
    fn measure(&mut self, text: &str, style: &TextStyle, max_width: Option<f32>) -> [f32; 2] {
        let metrics = Metrics {
            font_size: style.font_size,
            line_height: style.line_height,
        };
        let mut buffer = Buffer::new_empty(metrics);
        let font_system = &mut self.font_system;
        buffer.set_size(font_system, max_width, None);
        let wrap = match style.wrap {
            OverflowWrap::Normal => Wrap::WordOrGlyph,
            OverflowWrap::Anywhere => Wrap::Glyph,
            OverflowWrap::BreakWord => Wrap::WordOrGlyph,
        };
        buffer.set_wrap(font_system, wrap);
        let attrs = Attrs {
            color_opt: None,
            family: Family::Name(&style.font_family),
            stretch: match style.font_stretch {
                FontStretchKeyword::Normal => Stretch::Normal,
                FontStretchKeyword::UltraCondensed => Stretch::UltraCondensed,
                FontStretchKeyword::ExtraCondensed => Stretch::ExtraCondensed,
                FontStretchKeyword::Condensed => Stretch::Condensed,
                FontStretchKeyword::SemiCondensed => Stretch::SemiCondensed,
                FontStretchKeyword::SemiExpanded => Stretch::SemiExpanded,
                FontStretchKeyword::Expanded => Stretch::Expanded,
                FontStretchKeyword::ExtraExpanded => Stretch::ExtraExpanded,
                FontStretchKeyword::UltraExpanded => Stretch::UltraExpanded,
            },
            style: match style.font_style {
                FontStyle::Normal => Style::Normal,
                FontStyle::Italic => Style::Italic,
                FontStyle::Oblique(_) => Style::Normal,
            },
            weight: Weight(style.font_weight),
            metadata: 0,
            cache_key_flags: CacheKeyFlags::empty(),
            metrics_opt: None,
        };
        buffer.set_text(font_system, text, attrs, Shaping::Advanced);

        // Compute layout
        buffer.shape_until_scroll(font_system, false);

        // Determine measured size of text
        let (width, total_lines) = buffer
            .layout_runs()
            .fold((0.0, 0usize), |(width, total_lines), run| {
                (run.line_w.max(width), total_lines + 1)
            });
        let height = total_lines as f32 * buffer.metrics().line_height;

        [width, height]
    }
}
*/
