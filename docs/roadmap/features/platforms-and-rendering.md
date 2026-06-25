# Platforms & Rendering

One codebase, every surface — all behind the same versioned `Backend`/`Event`
seam, validated by the shared conformance suite. ✅ · 🟡 · ⬜.

## Platform targets
- ✅ **iOS** — UIKit backend (pure Rust via objc2), running today
- ⬜ **Android** — Android Views via JNI (`cargo-ndk`, Choreographer) — next
- ⬜ **macOS** — AppKit backend
- ⬜ **Windows** — Win32 / WinUI backend
- ⬜ **Linux** — GTK backend
- ⬜ **Web** — *planned later*: WASM target with a **DOM backend** (+ optional
  canvas/GPU backend for pixel parity); powers in-browser apps and the docs
  playground from the same Rust code
- ⬜ **tvOS / watchOS / visionOS** — exploratory
- ⬜ **Embedded / kiosk** — via the GPU renderer on bare wgpu

## Rendering models (per-subtree choice)
- ✅ native-widget rendering (map mutations → platform views)
- ⬜ **GPU renderer** (`rax-vello`, Vello on wgpu/Metal/Vulkan) — opt-in,
  per-subtree, for fully custom-drawn UI / pixel parity / custom controls
- ⬜ mix native + GPU in one app

## Rendering pipeline
- ✅ command-buffer (mutation) seam; deferred dynamic rebuilds
- 🟡 layout → frames → native positioning
- ⬜ batched/encoded command buffer across FFI (one call per frame)
- ⬜ scheduler-driven commit on vsync; priority lanes
- ⬜ background "shadow" diffing (off-main-thread)
- ⬜ view recycling/pooling; partial subtree commits
- ⬜ layer-backed / compositor animations
- ⬜ 120fps pipeline, frame pacing, jank instrumentation

## Native interop
- ⬜ embed arbitrary native views (host-view escape hatch)
- ⬜ register custom native `WidgetKind`s (ship native-backed widgets as packages)
- ⬜ expose Rust views *to* native screens (incremental adoption in existing apps)
- ⬜ brownfield integration (add a `rax` screen to an existing iOS/Android app)

## Platform integration
- ⬜ app lifecycle, multi-window, window management (desktop)
- ⬜ system theme/appearance, locale, accessibility settings
- ⬜ orientation, size classes, foldables, safe areas / cutouts
- ⬜ menus (desktop), system tray, dock/taskbar
- ⬜ per-platform packaging, signing, store submission
