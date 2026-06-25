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
| Flexbox layout | ✓ (Yoga) | ✓ | ✅ |
| Grid layout | community | ✓ | ✅ (`grid(columns, gap, items)` — composed row/column bucketing) |
| Image (cache/resize/placeholder) | ✓ | ✓ | ✅ (source+tint+ImageData+`network_image`; cache/resize modes later) |
| Icon (vector / SF Symbols) | community | ✓ | ✅ |
| ScrollView | ✓ | ✓ | ✅ |
| Virtualized list + recycling | ✓ | ✓ | ⬜ |
| SectionList / sticky headers | ✓ | ✓ (slivers) | ⬜ |
| TextInput (controlled + IME) | ✓ | ✓ | ✅ (controlled, secure, return key, onSubmit; IME later) |
| Switch / Slider | ✓ | ✓ | ✅ |
| SegmentedControl | ✓ | ✓ | ✅ |
| Stepper | ✓ | ✓ | ✅ |
| Checkbox / Radio | ✓ | ✓ | ✅ (composed in userland) |
| Picker | ✓ | ✓ | ✅ (inline, composed) |
| ActivityIndicator / Progress | ✓ | ✓ | ✅ |
| Modal/Sheet/Dialog/ActionSheet | ✓ | ✓ | ✅ (Modal + BottomSheet + Alert/Dialog + ActionSheet — all composed) |
| Tabs / BottomNav | community | ✓ | ✅ |
| Drawer | community | ✓ | ✅ (composed) |
| Divider / Spacer | ✓ | ✓ | ✅ |
| Pull-to-refresh / swipe actions | ✓ | ✓ | ✅ pull-to-refresh; ⬜ swipe actions |
| WebView (escape hatch) | community | plugin | ⬜ |

## Styling & theming
| Capability | RN | Flutter | rax |
|---|---|---|---|
| Inline styles | ✓ | ✓ | ✅ |
| Paint (border/shadow/opacity/radius) | ✓ | ✓ | ✅ |
| Design tokens / theme | community | ✓ (ThemeData) | ✅ |
| Runtime theme switching | ✓ | ✓ | ✅ |
| Dark mode | ✓ | ✓ | ✅ |
| High contrast | ✓ | ✓ | ⬜ |
| Reactive paint (theme/anim-driven) | partial | ✓ | ✅ |
| Variants + headless components | community | partial | ⬜ **(first-class)** |
| Replace any widget app-wide | — | partial | ⬜ **(registry)** |
| Per-platform look from one code | partial | ✓ | ⬜ |

## Navigation
| Capability | RN | Flutter | rax |
|---|---|---|---|
| Stack navigator (push/pop/replace/reset) | ✓ | ✓ | ✅ |
| Tab navigation | ✓ | ✓ | ✅ |
| Modal / Drawer | ✓ | ✓ | ✅ (Modal ✅; Drawer ✅ composed) |
| Typed routes | partial | partial | ✅ |
| Deep / universal links | ✓ | ✓ | ✅ (`on_deep_link` — openURL: bridged to `Event::DeepLink`) |
| Shared-element transitions | community | ✓ (Hero) | ⬜ |
| State restoration | ✓ | ✓ | ⬜ |
| Custom transitions | ✓ | ✓ | ⬜ |

## Animation & gestures
| Capability | RN | Flutter | rax |
|---|---|---|---|
| Timing + easing animations | ✓ | ✓ | ✅ |
| Spring / decay | ✓ | ✓ | ✅ |
| Tap / long-press / double-tap | ✓ | ✓ | ✅ |
| Gesture arena (pan/pinch/rotate) | ✓ | ✓ | 🟡 (pan ✅; pinch ✅; rotate ✅; arena ⬜) |
| Gesture-driven animation | ✓ (Reanimated) | ✓ | ⬜ |
| Layout / shared-element animation | community | ✓ | ⬜ |
| Off-main-thread animation | ✓ | ✓ | ⬜ |
| 120fps | ✓ | ✓ | ⬜ |

## Text & i18n & a11y
| Capability | RN | Flutter | rax |
|---|---|---|---|
| Font weight / italic / align | ✓ | ✓ | ✅ |
| Rich text / spans | ✓ | ✓ | ✅ (`rich_text().span(TextSpan)` — NSAttributedString font/color/underline) |
| Custom fonts / dynamic type | ✓ | ✓ | ✅ custom font family (`font_family()`); ⬜ dynamic type |
| RTL / bidi | ✓ | ✓ | ✅ (`.direction(LayoutDirection::Rtl)` — UISemanticContentAttribute) |
| i18n (catalog + interpolation) | community | ✓ (intl) | 🟡 (ICU/plurals later) |
| Screen-reader labels + roles | ✓ | ✓ | ✅ (`.accessibility_label/hint/role/hidden()` — UIAccessibilityTraits) |
| A11y as release gate | — | — | ⬜ **(policy)** |

## Data, async, storage
| Capability | RN | Flutter | rax |
|---|---|---|---|
| HTTP / fetch | ✓ | ✓ | ✅ (ureq-backed `HttpClient`) |
| WebSocket / SSE / GraphQL | community | community | 🟡 WebSocket ✅ + SSE ✅ (`connect_sse` — ureq streaming); GraphQL ⬜ |
| Resource (async data + loading state) | community | community | ✅ |
| Query cache (react-query-like) | community | community | 🟡 `use_query(url)` dedup/cache ✅; staleness/revalidation/mutations ⬜ |
| KV storage (+ persisted signals) | community | ✓ | ✅ |
| SQLite + secure storage | community | ✓ | 🟡 SQLite ✅ (`rax-sqlite::Database` — rusqlite bundled); secure storage ⬜ |
| Offline-first sync | community | community | ⬜ |
| Async runtime (no GC pauses) | JS event loop | Dart isolates | ✅ **(Rust async)** |

## Device & platform APIs
| Capability | RN | Flutter | rax |
|---|---|---|---|
| Camera / media picker | ✓ | ✓ | ✅ camera + QR scanner (AVFoundation); ⬜ media picker |
| Location / maps | ✓ | ✓ | ⬜ |
| Push + local notifications | ✓ | ✓ | 🟡 local ✅ (`schedule_notification`); push ⬜ |
| BLE / NFC | community | plugins | ⬜ |
| Biometrics / secure auth | ✓ | ✓ | 🟡 biometrics ✅ (`authenticate_biometric`); OAuth/passkeys ⬜ |
| In-app purchases | ✓ | ✓ | ⬜ |
| Sensors / haptics / background tasks | ✓ | ✓ | 🟡 haptics ✅ (`haptic(HapticStyle)`); sensors/background ⬜ |
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
| Error overlay | ✓ | ✓ | ✅ (`install_error_overlay()` panic hook + `error_overlay(signal)` composable) |
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
