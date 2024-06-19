use std::fs;
use std::time::Instant;
use lightningcss::printer::PrinterOptions;
use lightningcss::traits::ToCss;
use serde_json::{json, Value};
use bumaga::{Component, Input};

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
    let t = Instant::now();

    let input = Input::new().value(value.clone()).mouse([15.0, 15.0], true);
    let frame = component.update(input);

    let input = Input::new().value(value.clone()).mouse([15.0, 15.0], false);
    let frame = component.update(input);
    let t = t.elapsed().as_secs_f32();
    println!("elapsed: {t}");

    // drawing
    for call in frame.calls {
        println!("CALL {:?} {:?}", call.function, call.arguments);
        println!("{:?}", call.arguments[0].as_f64());
        println!("{:?}", call.arguments[1].as_bool());
        println!("{:?}", call.arguments[2].as_str());
        println!("{:?}", call.arguments[3].as_bool());
    }
    let mut result = String::new();
    result += "<style>body { font-family: \"Courier New\"; font-size: 14px; }</style>\n";
    for element in frame.elements {
        let rectangle = &element.rectangle;
        let layout = &element.layout;
        let k = &rectangle.key;
        let x = layout.location.x;
        let y = layout.location.y;
        let w = layout.size.width;
        let h = layout.size.height;
        let empty = String::new();
        let t = rectangle.text.as_ref().unwrap_or(&empty);
        // println!(
        //     "{k} bg {:?} cs{:?} sc{:?} s{:?}",
        //     rectangle.background.color, layout.content_size, layout.scrollbar_size, layout.size
        // );
        let mut bg = rectangle
            .background
            .color
            .to_css_string(PrinterOptions::default())
            .expect("css color");
        if let Some(img) = rectangle.background.image.as_ref() {
            // println!("img {img}");
            bg = format!("url({img})");
        }
        let record = format!("<div key=\"{k}\" style=\"position: fixed; top: {y}px; left: {x}px; width: {w}px; height: {h}px; background: {bg};\">{t}</div>\n");
        result += &record;
    }
    fs::write("./assets/result.html", result).expect("result written");
}