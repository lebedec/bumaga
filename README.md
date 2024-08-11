![.readme/header.png](.readme/header.png)

A Rust-native library for building user interfaces using web development experience.
You should use it if these features valuable to you:

+ HTML based UI definition and styling
+ CSS animations and transitions
+ Simple view bindings and interoperability based on JSON values
+ Graphics and windowing API agnostic
+ Fast prototyping first, hot reloading of view without compilation Rust app

The development of this library possible thanks to work of Rust enthusiasts:
[pest](https://github.com/pest-parser/pest),
[taffy](https://github.com/DioxusLabs/taffy), other 🔥 🚀

# TODO:

- check performance, use VTune (need < 1ms 100 elements)
    - use arena instead TaffyTree context ?
    - dont use recursion
    - share tree between calls (on static tree will be zero time to update)
    - separate to thread
    - measure text size skip ?!
    - use pools ?!
- hierarchy transforms (move location calculation to client side think about tree)
    - rework output tree traverse
- revise template language
    - use this as event in input callbacks
    - input.value separate state ?
    - text @= interpolation
    - rename component to view
    - templates ?!
- translation ?!
- book with concepts, architecture and tutorial https://github.com/rust-lang/mdBook
- share
    - https://arewegameyet.rs/
    - https://www.reddit.com/r/rust_gamedev/
    - habr ?

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
    let component = Component::compile("partial.html", "component.css");
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

Bumaga doesn't implement drawing. But here few examples with popular graphics solutions witch
should help to understand how to implement it:

* WIP: [Skia+Metal+Winit](examples/skia-metal-winit-app)
* TODO: Windows + DirectX
* [SDL2](examples/sdl2-app)
* TODO: Vulkan
* TODO: bevy
* [macroquad](examples/macroquad-app)




