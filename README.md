# bumaga


```rust

fn update(state: serde::Value) -> bumaga::DrawInstruction {
    
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

## What it is not

+ Not a HTML/CSS/script engine (look [sciter](https://github.com/sciter-sdk/rust-sciter) if you need)
+ Not a drawing solution, just abstract drawing instructions in result
+ Not a text rendering engine (actually you should provide one for correct work)

Thanks for:

+ https://github.com/causal-agent/scraper
+ https://github.com/DioxusLabs/taffy
+ https://github.com/parcel-bundler/lightningcss