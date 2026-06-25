# Vision & Principles

## North star

> **Build apps once, in Rust, that feel genuinely native on every device — with
> the developer experience of React and the performance and safety of Rust.**

A developer should write a single declarative Rust codebase and ship a polished
app to iOS, Android, desktop, and web, where each platform renders with its own
native widgets (or a tuned GPU renderer where that's the better trade), with
fine-grained reactivity keeping updates surgical and the borrow checker keeping
whole classes of UI bugs impossible.

## What "winning" looks like

- A team can build a top-100-App-Store-quality app in `rax` and not hit a wall
  that forces them to drop to platform code for core functionality.
- Cold-start, frame-time, memory, and binary-size are competitive with or better
  than React Native's new architecture and within striking distance of Flutter.
- A healthy third-party ecosystem of native modules exists, governed by a stable
  plugin ABI.
- `rax` is maintained by a foundation with multiple corporate contributors, not a
  single vendor.

## Non-negotiables (the things we will not trade away)

1. **100% Rust public API.** App code and the entire view/state layer are Rust.
   The only non-Rust artifacts are the unavoidable platform entry points
   (`JNI_OnLoad`, app delegate, packaging), and those are generated/owned by the
   backend crates — never written by app authors.
2. **Native rendering by default.** No WebView, no bundled JS engine. Platform
   widgets first; a GPU renderer is an *opt-in* second path, not the default.
3. **Fine-grained reactivity.** No per-frame virtual-DOM diff of the whole tree.
4. **Stable Rust.** Nightly features are never required to build an app.
5. **Type safety over reflection.** Traits and generics over runtime `Any`,
   except the one contained pocket inside the reactive runtime.
6. **Accessibility is not optional.** Semantics are first-class from the widget
   API up; a screen-reader-broken release does not ship.
7. **The render seam stays small and versioned.** New platforms must be "just an
   implementation," provable by the shared conformance suite.
8. **Super-customizable by default.** Every layer — style, theme, component,
   layout, behavior, navigation, rendering, native interop — is overridable
   without forking the framework. If an author must patch `rax` to get the look
   or behavior they want, that's a bug. See [03-customizability.md](03-customizability.md).

## Principles for a 100-person org

- **Conway's Law on purpose.** Squad boundaries match crate/seam boundaries so
  the architecture stays clean as the org grows.
- **Decisions are documents.** RFCs for public API; ADRs for internal calls.
  A 100-person team cannot align by hallway conversation.
- **One DRI per cross-cutting feature.** Features that span squads get a single
  Directly-Responsible Individual and a time-boxed strike team.
- **Quality gates are automated, not heroic.** Conformance suite, benchmarks,
  semver-checks, and a11y audits run in CI and block merges/releases.
- **Dogfood relentlessly.** An internal flagship app (see DevRel) is built in
  `rax` and is the canary for every release.

## What we deliberately deprioritize

- A custom design language. We provide unstyled primitives + a theming system;
  we do not ship a Material/Cupertino clone as the core (a styled kit can be a
  community/first-party *package*).
- Game-engine ambitions. `rax` is an app framework; the GPU renderer targets app
  UI, not 3D scenes.
- "Write once, look identical everywhere." We target *native-feeling* per
  platform, not pixel-identical — except on the opt-in GPU renderer.
