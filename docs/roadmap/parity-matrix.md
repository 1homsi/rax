# Parity Matrix — rax vs React Native vs Flutter

Every major capability, side by side. The target is the **union** of what RN and
Flutter offer, plus Rust-only advantages. Columns: does the ecosystem support it
(✓ / partial / —) and `rax` status (✅ shipped · 🟡 wip · ⬜ planned).

> "RN" = React Native core + the de-facto community stack (React Navigation,
> Reanimated, Gesture Handler, FlatList, Expo modules). "Flutter" = framework +
> first-party plugins.

## Architecture
| Capability | RN | Flutter | rax |
|---|---|---|---|
| Language (app code) | JS/TS | Dart | **Rust** |
| Rendering | native widgets | own GPU canvas (Skia/Impeller) | **native widgets ✅ + opt-in GPU ⬜** |
| Update model | VDOM diff (Fabric) | element/widget diff | **fine-grained signals ✅** |
| Type-checked UI at compile time | partial | partial | **✅ (Rust types)** |
| No JS/extra runtime | — | ✓ | **✅** |
| Per-subtree native-vs-GPU choice | — | — | **⬜ (unique)** |

## Core UI
| Capability | RN | Flutter | rax |
|---|---|---|---|
| View / Text / Button | ✓ | ✓ | ✅ |
| Flexbox layout | ✓ (Yoga) | ✓ | 🟡 |
| Grid layout | community | ✓ | ⬜ |
| Image (cache/resize/placeholder) | ✓ | ✓ | ⬜ |
| ScrollView | ✓ | ✓ | 🟡 |
| Virtualized list + recycling | ✓ | ✓ | ⬜ |
| SectionList / sticky headers | ✓ | ✓ (slivers) | ⬜ |
| TextInput (controlled + IME) | ✓ | ✓ | ⬜ |
| Switch/Slider/Checkbox/Radio/Picker | ✓ | ✓ | ⬜ |
| Modal/Sheet/Dialog/ActionSheet | ✓ | ✓ | ⬜ |
| Tabs / BottomNav / Drawer | community | ✓ | ⬜ |
| Pull-to-refresh / swipe actions | ✓ | ✓ | ⬜ |
| WebView (escape hatch) | community | plugin | ⬜ |

## Styling & theming
| Capability | RN | Flutter | rax |
|---|---|---|---|
| Inline styles | ✓ | ✓ | ✅ |
| Design tokens / theme | community | ✓ (ThemeData) | ⬜ |
| Runtime theme switching | ✓ | ✓ | ⬜ |
| Dark mode / high contrast | ✓ | ✓ | ⬜ |
| Variants + headless components | community | partial | ⬜ **(first-class)** |
| Replace any widget app-wide | — | partial | ⬜ **(registry)** |
| Per-platform look from one code | partial | ✓ | ⬜ |

## Navigation
| Capability | RN | Flutter | rax |
|---|---|---|---|
| Stack / Tab / Modal / Drawer | ✓ | ✓ | ⬜ |
| Typed routes | partial | partial | ⬜ |
| Deep / universal links | ✓ | ✓ | ⬜ |
| Shared-element transitions | community | ✓ (Hero) | ⬜ |
| State restoration | ✓ | ✓ | ⬜ |
| Custom transitions | ✓ | ✓ | ⬜ |

## Animation & gestures
| Capability | RN | Flutter | rax |
|---|---|---|---|
| Timing / spring / decay | ✓ | ✓ | ⬜ |
| Gesture recognizers + arena | ✓ | ✓ | ⬜ |
| Gesture-driven animation | ✓ (Reanimated) | ✓ | ⬜ |
| Layout / shared-element animation | community | ✓ | ⬜ |
| Off-main-thread animation | ✓ | ✓ | ⬜ |
| 120fps | ✓ | ✓ | ⬜ |

## Text & i18n & a11y
| Capability | RN | Flutter | rax |
|---|---|---|---|
| Rich text / spans | ✓ | ✓ | ⬜ |
| Custom fonts / dynamic type | ✓ | ✓ | ⬜ |
| RTL / bidi | ✓ | ✓ | ⬜ |
| i18n (ICU, plurals) | community | ✓ (intl) | ⬜ |
| Screen-reader support | ✓ | ✓ | ⬜ |
| A11y as release gate | — | — | ⬜ **(policy)** |

## Data, async, storage
| Capability | RN | Flutter | rax |
|---|---|---|---|
| HTTP / fetch | ✓ | ✓ | ⬜ |
| WebSocket / SSE / GraphQL | community | community | ⬜ |
| Query cache (react-query-like) | community | community | ⬜ |
| KV + SQLite + secure storage | community | ✓ | ⬜ |
| Offline-first sync | community | community | ⬜ |
| Async runtime (no GC pauses) | JS event loop | Dart isolates | ⬜ **(Rust async)** |

## Device & platform APIs
| Capability | RN | Flutter | rax |
|---|---|---|---|
| Camera / media picker | ✓ | ✓ | ⬜ |
| Location / maps | ✓ | ✓ | ⬜ |
| Push + local notifications | ✓ | ✓ | ⬜ |
| BLE / NFC | community | plugins | ⬜ |
| Biometrics / secure auth | ✓ | ✓ | ⬜ |
| In-app purchases | ✓ | ✓ | ⬜ |
| Sensors / haptics / background tasks | ✓ | ✓ | ⬜ |
| Plugin system / native modules | ✓ | ✓ | ⬜ |

## Platforms
| Capability | RN | Flutter | rax |
|---|---|---|---|
| iOS | ✓ | ✓ | ✅ |
| Android | ✓ | ✓ | ⬜ (next) |
| macOS / Windows / Linux | partial | ✓ | ⬜ |
| Web | ✓ (RN-Web) | ✓ | ⬜ **(planned later)** |
| Brownfield / embed in native app | ✓ | ✓ | ⬜ |

## Tooling
| Capability | RN | Flutter | rax |
|---|---|---|---|
| CLI + project gen | ✓ | ✓ | ⬜ |
| Hot reload / fast refresh | ✓ | ✓ | ⬜ |
| DevTools / inspector | ✓ | ✓ | ⬜ |
| Error overlay | ✓ | ✓ | ⬜ |
| Testing framework | ✓ | ✓ | 🟡 |
| OTA / code-push | ✓ (community) | partial | ⬜ |

## Rust-only advantages (where we aim to *exceed* both)
- Compile-time-checked UIs and exhaustive state handling (no runtime "undefined").
- Fine-grained reactivity: surgical updates, no per-frame tree diff.
- No GC: predictable frame budgets, no collection pauses.
- Memory safety without a VM; small binaries; fast cold start.
- One language across UI, business logic, and native modules.
- Per-subtree choice of native widgets vs. custom GPU rendering.
- **Super-customizability** as a first-class, audited guarantee.

> This matrix is reviewed every release; an item flips to ✅ only when it passes
> the cross-platform conformance suite (and the a11y gate where applicable).
