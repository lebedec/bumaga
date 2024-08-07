![.readme/header.png](.readme/header.png)

A Rust-native library for building user interfaces using web development experience.
You should use it if these features valuable to you:

+ UI declaration and styling language similar to HTML and CSS
+ Hot reloading of view, prototyping without recompilation Rust app
+ Simple view bindings and interoperability based on JSON values
+ Graphics API agnostic
+ Windowing API agnostic
+ CSS animations

The development of this library possible thanks to work of Rust enthusiasts:
[pest](https://github.com/pest-parser/pest),
[taffy](https://github.com/DioxusLabs/taffy), other 🔥 🚀

# TODO:

- transform (move location calculation to client side think about tree)
- rework output tree traverse
- transition

## What it is not

+ Not a HTML/CSS/script engine or full spec implemenation (look [sciter](https://github.com/sciter-sdk/rust-sciter) if
  you need)
+ Not a drawing solution, just abstract drawing instructions in result
+ Not a text rendering engine (actually you should provide one for correct work)

## WARNING

Bumaga is still in the early stages of development. Important features are missing. Documentation is sparse.

## Example

TODO: short description of architecture and real code example

```rust
fn main() {
    let mut engine = MyEngine::startup();
    let mut name = String::from("Alice");
    let component = Component::compile("component.html", "component.css");
    loop {
        let value = json!({"name": name});
        let input = Input::from(engine.input)
            .fonts(engine.fonts)
            .value(value);
        let frame = component.update(input);
        if let Some(arg) = frame.calls.get("rename") {
            name = arg.to_string()
        }
        for element in frame.elements {
            engine.draw(element);
        }
    }
}
```

<table>
<td>

```html 

<body>
<div class="panel">
    <header>
        Bumaga Todo
        <span>Streamline Your Day, the Bumaga Way!</span>
    </header>
    <div *="todos" class="todo" onclick="remove(todos)">
        <span>{todos}</span>
        <div>×</div>
    </div>
    <input value="todo" oninput="edit" onchange="append"/>
</div>
```

</td>
<td>

![.readme/example.avif](.readme/example.avif)

</td>
</table>

# Using

Bumaga doesn't implement drawing, doesn't limit your application
with any details of concrete graphics API.
But here few examples with popular graphics solutions witch
should help to understand how it works:

* WIP: [Skia+Metal+Winit](examples/skia-metal-winit-app)
* [SDL2](examples/sdl2-app)
* TODO: Vulkan
* TODO: bevy
* [macroquad](examples/macroquad-app)

# Dev

TODO:

* hierarchy transforms
* transforms percent
* transitions
* input state





