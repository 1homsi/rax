# rax — Feature Roadmap

> The complete catalog of everything `rax` must support to match and exceed
> **React Native** and **Flutter**, organized by domain. This is a feature plan,
> not a schedule — no dates, no headcount. The goal is parity-or-better on every
> capability a serious app needs, with **super-customizability** ([pillar](03-customizability.md))
> threaded through all of it.
>
> Status legend used throughout: ✅ shipped · 🟡 in progress · ⬜ planned.

## Implemented so far

Shipped and tested (host-side + running on the iOS Simulator), by crate:

- **raxon-core** — geometry, generational arena, color, full `LayoutStyle`.
- **raxon-reactive** — signals/memos/effects, `Runtime`/ownership, **context/providers**.
- **rax-scheduler** — frame phases, priority tasks, cross-thread marshaling.
- **raxon-dom** — element tree, mutation **+ event** seam, **dynamic structure**.
- **rax-layout** — full flexbox via taffy (justify/wrap/shrink/min-max/absolute/aspect/%).
- **rax-style** — typed design tokens + runtime-switchable light/dark **theme**.
- **raxon-view** — declarative builder: `column/row/text/button/spacer/dynamic`,
  **image/switch/slider/divider/text_input**, and universal `ViewExt` layout +
  paint modifiers (size/grow/margin/align/absolute · background/border/shadow/
  opacity/corner-radius).
- **raxon-anim** — tweened animated signals + easing, advanced by the frame loop.
- **raxon-async** — UI-thread executor + async-aware `Resource`.
- **raxon-nav** — stack navigator + `routes` view (typed routes, context-provided).
- **raxon-runtime** — the `App` driver (mount + layout + events + async + anim + frames).
- **raxon-ios** — UIKit backend (pure Rust via objc2): View/Label/Button/Image/
  Switch/Slider/TextField, taps + value/text changes, layout frames, paint.
- **rax-test** — headless harness + finder/query API for host-side UI tests.

~106 tests green; the showcase app (counter · dynamic to-do list · native
controls · about) runs on the iOS Simulator. Remaining roadmap items (scroll/
recycled lists, gestures, accessibility, i18n, native modules, desktop/web,
CLI, devtools) are tracked below.

## Principles & pillars

| File | What it covers |
|---|---|
| [00-vision-and-principles.md](00-vision-and-principles.md) | North star and non-negotiables |
| [03-customizability.md](03-customizability.md) | **Pillar:** every layer overridable — the escape-hatch ladder |
| [01-org-and-teams.md](01-org-and-teams.md) | How a ~100-engineer org is structured around the crates/seams |

## Feature catalog (by domain)

| Domain | File |
|---|---|
| Components / widget library | [features/components.md](features/components.md) |
| Layout | [features/layout.md](features/layout.md) |
| Styling & theming | [features/styling-and-theming.md](features/styling-and-theming.md) |
| Text & typography | [features/text-and-typography.md](features/text-and-typography.md) |
| Text input, keyboard & forms | [features/text-input-and-forms.md](features/text-input-and-forms.md) |
| Lists & scrolling | [features/lists-and-scrolling.md](features/lists-and-scrolling.md) |
| Navigation & routing | [features/navigation.md](features/navigation.md) |
| Animation & transitions | [features/animation.md](features/animation.md) |
| Gestures & input | [features/gestures.md](features/gestures.md) |
| Images, media & graphics | [features/media-and-graphics.md](features/media-and-graphics.md) |
| Device & platform APIs (native modules) | [features/device-apis.md](features/device-apis.md) |
| Accessibility | [features/accessibility.md](features/accessibility.md) |
| Internationalization | [features/internationalization.md](features/internationalization.md) |
| State & reactivity | [features/state-and-reactivity.md](features/state-and-reactivity.md) |
| Async, networking & data | [features/async-networking-data.md](features/async-networking-data.md) |
| Storage & persistence | [features/storage.md](features/storage.md) |
| Platforms & rendering | [features/platforms-and-rendering.md](features/platforms-and-rendering.md) |
| Developer experience & tooling | [features/developer-experience.md](features/developer-experience.md) |
| Extensibility & plugins | [features/extensibility-and-plugins.md](features/extensibility-and-plugins.md) |

## Parity tracking

| File | What it covers |
|---|---|
| [parity-matrix.md](parity-matrix.md) | Side-by-side: `rax` vs React Native vs Flutter, every capability |

## Platform targets

Mobile first, everywhere eventually — all behind the **same** `Backend`/`Event`
seam, so a new platform is "just an implementation":

- ✅ **iOS** (UIKit) — running today
- ⬜ **Android** (Android Views via JNI) — next
- ⬜ **Desktop** — macOS, Windows, Linux
- ⬜ **Web** — **planned later**: compile the same Rust app to **WebAssembly**
  with a DOM backend (and an optional canvas/GPU backend for pixel parity).
  Enables in-browser apps *and* the docs playground from one codebase.

## The bar

For each domain we aim to support **the union** of what React Native (core +
the essential community ecosystem) and Flutter offer — and then go further on
the things only Rust can do well: compile-time-checked UIs, fine-grained
reactivity, zero-GC frame loops, and per-subtree choice of native widgets vs. a
custom GPU renderer. If RN or Flutter can do it, `rax` should too; where they
force you into platform code or a fork, `rax` should expose a first-class
extension point instead.
