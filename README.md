# rax

A **100% Rust**, React-like framework for building **native** mobile apps. No
JavaScript runtime, no WebView — your declarative Rust UI is rendered with real
platform widgets (UIKit today; Android Views next).

```rust
fn counter(count: Signal<i32>) -> impl View {
    column((
        text(move || format!("Count: {}", count.get())).font_size(48.0),
        button("Increment", move || count.update(|c| *c += 1)),
    ))
    .padding(16.0)
    .gap(8.0)
}
```

It runs today: the demo in [`rust-native-examples`](../dummy/rust-native-examples)
builds for the iOS Simulator and renders native `UILabel`/`UIButton` laid out by
the flexbox engine, driven entirely by Rust.

## Why

- **Declarative + fine-grained reactive.** Signals update only the views that
  read them — no virtual-DOM diff. Structure builds once; values bind in place.
- **Native rendering.** Real platform views via `objc2` (iOS) — native text,
  accessibility, and scrolling, not a canvas.
- **Pure Rust public API.** No Swift/Kotlin/JS in app code. (A few hundred lines
  of unavoidable platform entry-point glue live inside the backend crates, in
  Rust via `objc2`/JNI — never in your app.)
- **Stable Rust, minimal macros, testable.** The whole pipeline is verified
  host-side through a recording backend with zero platform code.

## Workspace layout

```
rax-core       geometry, generational arena, color, layout style   (no deps, no_std)
rax-reactive   signals / memos / effects, Runtime, ownership scopes
rax-scheduler  frame phases, priority tasks, cross-thread marshaling
rax-dom        retained element tree, mutation + event seam, dynamic structure
rax-layout     flexbox via taffy, behind a neutral LayoutStyle
rax-view       declarative, macro-free view builder (column/row/text/button/dynamic)
rax-runtime    the App driver: mount + layout + events + frames
rax-ios        UIKit backend (pure Rust via objc2)
```

The dependency graph is a strict DAG; the render seam (`Backend` trait +
`Mutation`/`Event`) is the single, identical boundary every platform implements.

## Status

Early but real. A reactive, multi-screen app with a tab bar, a dynamic
add/remove list, and styled cards runs on the iOS Simulator. See
[`docs/ARCHITECTURE_AUDIT.md`](docs/ARCHITECTURE_AUDIT.md) for the full
subsystem audit, the load-bearing decisions, known debt, and the roadmap
(Android backend, text input/IME, navigation, animation, accessibility next).

## Running the demo

```sh
rustup target add aarch64-apple-ios-sim
cd ../dummy/rust-native-examples
./run-ios.sh "iPhone 17 Pro"
```

## Building & testing the framework

```sh
cargo test --workspace      # host-side, no platform needed
cargo clippy --workspace --all-targets
```

## License

MIT OR Apache-2.0.
