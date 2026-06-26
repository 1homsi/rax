# Org & Teams (~100 engineers)

`rax` is built by ~20 squads whose boundaries mirror the crate/seam boundaries,
so the architecture stays clean as the org scales (Conway's Law, on purpose).
Squads own **crates and seams**; cross-cutting features are delivered by
time-boxed **strike teams** drawn from squads, each with one DRI.

## Squads (~100 engineers, ~5 each)

Squad sizes are indicative steady-state, summing to ~100. They scale with the
surface area each owns; this is about *parallelism and ownership*, not a hiring
schedule.

| # | Squad | Charter | Owns | Size |
|---|---|---|---|---:|
| 1 | **Core & Reactivity** | Signals, ownership, scheduler hooks, arena, geometry | `raxon-core`, `raxon-reactive` | 5 |
| 2 | **Layout & Styling** | Flexbox/grid engine, style resolution, density | `rax-layout`, `rax-style` | 5 |
| 3 | **Rendering, Scheduler & Compositor** | Render seam, command buffer, frame pipeline, off-main-thread layout | `rax-scheduler`, `raxon-dom` seam, `rax-render` | 5 |
| 4 | **iOS Platform** | UIKit backend, bootstrap, packaging | `raxon-ios` | 5 |
| 5 | **Android Platform** | JNI backend, View interop, packaging | `rax-android` | 6 |
| 6 | **Desktop Platform** | macOS/Windows/Linux backends | `rax-macos`, `rax-windows`, `rax-linux` | 5 |
| 7 | **Web / WASM Platform** | WASM target, DOM/canvas backend | `rax-web` | 5 |
| 8 | **Text, Fonts & IME** | Text layout, shaping, input, composition, keyboard | `rax-text` | 5 |
| 9 | **Components (widget library)** | The standard widget set + their semantics | `rax-widgets` | 6 |
| 10 | **Navigation & Routing** | Stack/tab/modal nav, deep links, state restoration | `raxon-navigation` | 4 |
| 11 | **Animation, Gestures & Input** | Animation engine, transitions, gesture arena | `raxon-animation`, `rax-gesture` | 5 |
| 12 | **Accessibility & i18n** | Platform a11y mapping, l10n, RTL, dynamic type | `rax-a11y`, `rax-intl` | 5 |
| 13 | **Async, Networking, Data & Storage** | Executor, HTTP, persistence, secure storage | `raxon-async`, `raxon-net`, `rax-store` | 5 |
| 14 | **Native Modules & Plugin Platform** | Plugin ABI, platform-channel codegen, first-party modules | `raxon-plugin`, `rax-modules/*` | 5 |
| 15 | **CLI, Build & Toolchain** | `rax` CLI, generators, cross-compile, packaging | `raxon-cli` | 5 |
| 16 | **DevTools, Inspector & Hot Reload** | Inspector, tree/state viewer, fast refresh, error overlay | `rax-devtools` | 5 |
| 17 | **Testing, QA & Conformance** | Headless host, finder/query API, the conformance suite | `rax-test` | 5 |
| 18 | **Performance & Security** | Benchmarks, profiling, supply-chain, fuzzing | tooling, CI gates | 4 |
| 19 | **Docs & Developer Relations** | The book, API docs, examples, flagship dogfood app | `docs/`, examples | 6 |
| 20 | **Release Eng, Infra & Research** | CI/CD, release trains, semver gates; GPU-renderer R&D | infra, `rax-vello` (R&D) | 4 |

> The ~20 squads run in **parallel** — that's the point of staffing at this
> scale: platform backends, the widget library, devtools, and the plugin
> ecosystem all advance simultaneously rather than in sequence.

## Leadership & roles (within the 100)

- **1 Head of Engineering**, **3–4 Directors** (Platforms, Core/Rendering, DX/Ecosystem, QA/Infra).
- **~8 Engineering Managers** (1 per 2–3 squads).
- **~5 Product Managers** (Platforms, DX, Ecosystem, Enterprise, Community).
- **~3 Designers** (design system, devtools UX, docs/marketing) embedded in DX.
- **Staff/Principal engineers** float across squads to guard the render seam and
  the public API; they are the RFC approvers.

## How cross-cutting work ships

A feature like **TextInput with controlled value + IME** touches Text, iOS,
Android, Components, A11y, and Testing. We run it as a **strike team**:

1. A DRI (usually from the most-affected squad) writes the RFC.
2. 1–2 engineers from each touched squad join for the duration.
3. The conformance suite gains tests for the feature **before** it merges.
4. The strike team dissolves; ongoing maintenance returns to owning squads.

This keeps squads stable (they own durable surface area) while features move fast
across them.

## Standing programs (not squads, but funded)

- **Conformance & release gating** — owned by Testing/QA, blocks every release.
- **Performance budget** — owned by Perf, enforced in CI on every PR.
- **RFC process** — owned by Staff eng; required for public API changes.
- **Dogfooding** — DevRel maintains a real flagship app built in `rax`.
- **Security response** — Security squad runs disclosure + dependency audits.
