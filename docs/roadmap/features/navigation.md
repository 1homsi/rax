# Navigation & Routing

Match React Navigation / Expo Router and Flutter Navigator 2.0 / go_router.
Rust-owned navigation state with native transition primitives. ⬜ planned.

## Navigators
- ✅ stack navigator (push/pop/replace/popToTop/popToRoot)
- ✅ tab navigator (bottom tabs + top tabs)
- ✅ modal presentation stack (`present_modal(route)`, `dismiss_modal()`, `current_modal()`, `modal_stack()` — separate stack, does not affect main nav)
- ✅ drawer / side-menu navigator (`drawer(show, on_dismiss, width, content)` — composed)
- ✅ deep link parsing (`parse_deep_link(url) -> (path, HashMap<key,val>)` — strips scheme, splits path/query); ✅ `try_navigate(route) -> bool` (guard-aware, returns false if redirected); ⬜ nested navigators (tabs containing stacks)
- ⬜ split-view / master-detail (tablet/desktop adaptive)

## Routing
- ✅ typed routes (compile-checked params)
- ⬜ declarative URL routing (path patterns, params, query)
- ✅ deep links (`on_deep_link(handler)` — `application:openURL:options:` → `Event::DeepLink`)
- ⬜ web-history integration (for the web target)
- ✅ redirects / guards / auth gating (`add_route_guard(condition, redirect)` — checked on every `navigate()` call; `check_guards(route) -> Option<String>`)
- ✅ not-found / fallback routes (`set_not_found(fn -> BoxedView)`, `get_not_found()`)
- ✅ programmatic navigation API (`navigate(route)`, `go_back()`, `can_go_back()`, `current_route() -> Signal<String>`, `match_route(pattern, route)` params)

## Transitions & gestures
- ⬜ default platform transitions (iOS push/Android shared-axis)
- ⬜ custom transitions (pluggable, fully overridable)
- ⬜ interactive pop / swipe-back gesture
- ⬜ predictive back (Android), interruptible transitions
- ⬜ shared-element / hero transitions
- ⬜ transition lifecycle hooks

## State & lifecycle
- ⬜ navigation state restoration (kill/restore)
- ✅ screen focus/blur lifecycle events (`on_appear(route, fn)`, `on_disappear(route, fn)`, `use_focus_effect(route, fn)`, `fire_appear/disappear` hooks)
- ⬜ params passing + result return (e.g. pick-and-return)
- ✅ back-handling (`on_back(fn -> bool)` + `handle_back()` — chains handlers, falls back to `go_back()`)
- ✅ navigation events / listeners / analytics hooks (`on_navigate(fn(from, to))`, `fire_navigate_event`)
- ⬜ preserve/lazy screen mounting; keep-alive tabs

## Advanced
- ⬜ server-driven navigation
- ⬜ deep-link preview / handoff / quick actions
- ⬜ nav devtools (current stack inspector)
