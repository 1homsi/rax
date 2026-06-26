# raxon

A reactive, signal-driven native UI framework for Rust.

```toml
[dependencies]
raxon = "0.0.2"
```

```rust
use raxon::prelude::*;

fn counter(count: Signal<i32>) -> impl View {
    column((
        text(move || format!("Count: {}", count.get())).font_size(48.0),
        row((
            button("−", move || count.update(|c| *c -= 1)),
            button("+", move || count.update(|c| *c += 1)),
        ))
        .gap(12.0),
    ))
    .padding(24.0)
    .gap(16.0)
}
```

## License

MIT OR Apache-2.0
