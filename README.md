# bumaga

```html
<div>
    <h1>Hello, {name}!</h1>
    <input value="name" onchange="rename" />
</div>
```

```css
h1 {
    font-size: 20px;
    color: #2c2c2c;
}

input {

}
```

```rust
fn main() {
    let mut fonts = MyFontsSystem::new();
    let mut name = String::from("Alice");
    let component = Component::compile("index.html", "style.css");
    loop {
        let value = json!({"name": name});
        let input = Input::new(fonts).value(value);
        let frame = component.update(input);
        for call in frame.calls {
            if call.function == "rename" {
                name = call.args[0].to_string()
            }
        }
        for element in frame.elements {
            render_element(element);
        }
    }
}
```

A Rust library for building user interfaces inspired by HTML/CSS web development experience.
You should use it if these features important to you:

+ UI declaration and styling language similar to HTML/CSS
+ Ability to use any handy IDE to edit HTML
+ Hot reloading of view, prototyping without recompilation Rust app
+ CSS animations
+ Simple view bindings and data management based on `serde_json`
+ Graphics API agnostic
+ Windowing API agnostic

The development of this library possible thanks to work of Rust enthusiasts: 
[scraper](https://github.com/causal-agent/scraper), 
[taffy](https://github.com/DioxusLabs/taffy), 
[lightningcss](https://github.com/parcel-bundler/lightningcss)

## What it is not

+ Not a HTML/CSS/script engine (look [sciter](https://github.com/sciter-sdk/rust-sciter) if you need)
+ Not a drawing solution, just abstract drawing instructions in result
+ Not a text rendering engine (actually you should provide one for correct work)
