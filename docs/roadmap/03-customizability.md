# Pillar: Super-Customizability & Extensibility

> **Every layer is overridable. Nothing is a black box.** A `rax` app author can
> restyle, recompose, replace, or fully custom-draw any part of the UI — without
> forking the framework and without dropping to platform code. This is a
> headline differentiator, designed in from the start, not bolted on.

This pillar is **cross-cutting**: it constrains the API of every squad. A
feature is not "done" until it exposes the right extension points below.

## The 7 levels of customization (escape-hatch ladder)

Authors should reach for the *lowest* level that solves their problem, and every
level is always available:

1. **Props & inline style** — tweak any widget inline (`.font_size`, `.padding`,
   `.color`, `.background`, `.corner_radius`, …). Already real.
2. **Theme tokens** — change a token (color, spacing, radius, typography, motion)
   once and every widget that reads it updates. Runtime-switchable (light/dark,
   brand themes, per-user themes).
3. **Variants & style overrides** — override a widget's style per-instance or
   per-type via a theme (`Button` "primary"/"ghost"/your own), like a typed
   stylesheet. Cascading is explicit, never magic.
4. **Headless / slotted components** — use a widget's *behavior* (state, a11y,
   gestures, focus) while supplying your *own* presentation. Radix/HeadlessUI
   philosophy, in Rust: `Pressable`, `Disclosure`, `Listbox`, `Tabs` ship as
   logic + slots you render however you want.
5. **Composition & render-props** — compose primitives into your own components
   with the `View` trait; pass child-builders/slots; wrap and decorate freely.
6. **Custom widgets** — implement `View` to build a brand-new component from
   primitives, or register a **custom native widget kind** so a backend
   materializes a platform view you specify (and embed *arbitrary* existing
   native views via a host-view escape hatch).
7. **Custom rendering** — drop a subtree onto the **GPU renderer** (Vello/wgpu)
   to draw pixels yourself (custom controls, charts, canvases, fully bespoke
   visual languages) while the rest of the app stays native.

## What must be customizable (the checklist every squad honors)

- **Styling:** every visual property; inline + theme + variant; responsive by
  size-class/orientation/platform; dark mode; user-defined custom properties.
- **Theming:** swap the entire theme at runtime; nested/scoped themes; brand
  packages; tokens for color, spacing, radius, typography, shadows, motion,
  z-index, opacity, borders, gradients.
- **Components:** every built-in widget is (a) styleable, (b) has a headless
  core, (c) is replaceable app-wide via the theme/registry.
- **Layout:** any layout primitive is composable; custom layout via a
  `Layout` trait for authors who need bespoke positioning.
- **Behavior:** gestures, focus order, keyboard handling, and animations are
  overridable per-widget; transitions are pluggable.
- **Navigation:** custom transitions, custom navigators, custom route matching.
- **Platform look:** opt into native-default styling *or* a fully custom design
  system *per platform* — your choice, per widget.
- **Rendering:** choose native widgets or GPU-drawn per subtree.
- **Native interop:** embed any platform view; expose any native API via a plugin.

## Architectural mechanisms that deliver this

| Mechanism | Owning squad(s) |
|---|---|
| **Design tokens + `Theme` context** (scoped, runtime-switchable) | Layout/Styling, Design System |
| **Typed style system** (inline → variant → theme resolution, no magic cascade) | Layout/Styling |
| **Headless component cores** (state/a11y/gesture separated from presentation) | Components, A11y |
| **Slot/child-builder API** in the `View` layer | Core/Rendering, Components |
| **Component registry / override map** in the theme (swap any widget app-wide) | Components, Layout/Styling |
| **Custom `WidgetKind` registration** at the render seam | Rendering, Platforms |
| **Host-view embedding** (drop any `UIView`/`android.view.View` into the tree) | Platforms |
| **`Layout` trait** for custom layout algorithms | Layout |
| **Pluggable transitions / navigators** | Navigation, Animation |
| **GPU renderer path** (`rax-vello`) for custom-drawn subtrees | Research, Rendering |
| **Custom backends** via the public, versioned `Backend`/`Event` seam | Rendering |
| **Plugin ABI** for arbitrary native capability extension | Plugin Platform |

## Design rules so customization doesn't become chaos

- **Explicit over implicit.** Resolution order (inline > variant > theme >
  default) is documented and predictable; no CSS-style global cascade surprises.
- **Headless + styled, never one or the other.** Every component ships a headless
  core *and* a default styled skin built on it. You can keep, restyle, or replace
  the skin.
- **Typed tokens.** Themes are typed Rust values, not stringly-typed maps —
  autocomplete and compile-time checking for your design system.
- **No customization requires a fork.** If an author has to patch `rax` to
  achieve a look or behavior, that's a roadmap bug for the owning squad.
- **Performance-safe by construction.** Theme reads are signals (fine-grained);
  switching a theme updates only the affected properties, not the whole tree.

## "Definition of customizable" gate

Before any widget reaches stable, it must demonstrate, in the conformance suite:
1. fully restyled via tokens **and** via per-instance overrides,
2. recomposed from its headless core with custom presentation,
3. replaced app-wide via the registry,
4. (where visual) re-themeable at runtime with one token change.
