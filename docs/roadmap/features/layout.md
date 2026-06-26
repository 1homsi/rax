# Layout

Flexbox-first (taffy), matching Yoga (RN) and Flutter's box model, plus grid.
✅ shipped · 🟡 wip · ⬜ planned.

## Flexbox
- ✅ flex-direction (row/column), gap, padding, align-items
- ✅ flex-grow
- ✅ align (start/center/end/stretch)
- ✅ justify-content (start/center/end/space-between/around/evenly)
- ✅ flex-wrap + align-content
- ✅ flex-shrink, flex-basis
- ✅ align-self (per-child override)
- ✅ row/column gap (independent)
- ✅ order (`.order(n: i32)` → `Attribute::FlexOrder(i32)` — flex child render order override)

## Sizing
- 🟡 width/height: auto, points, percent, `fr`
- ✅ min/max width & height
- ✅ aspect-ratio
- ⬜ intrinsic sizing from real text/content metrics
- ✅ `Expanded`/`Flexible` helpers (`expanded(child)` → flex_grow 1.0; `flexible(child, n)` → flex_grow n)
- ⬜ fit modes (contain/cover/fill) for media

## Positioning
- 🟡 position: relative / absolute / sticky
- 🟡 inset (top/right/bottom/left), z-index / z-order
- ⬜ `Stack`/overlay layout
- 🟡 transforms (translate/scale/rotate; reactive `transform_fn` for animation) — skew/origin pending

## Box model & spacing
- ✅ margin (incl. auto-margins for centering)
- ✅ padding
- ⬜ border width per-edge (paint side: color/style/radius)
- ✅ safe-area insets (auto root inset from the platform safe area; notch + home indicator)
- ✅ keyboard insets (avoidance) — runtime folds keyboard height into the bottom inset; iOS observes keyboard show/hide notifications

## Grid
- ⬜ CSS-grid: template rows/cols, areas, auto-flow, gaps
- ⬜ `LazyGrid` (virtualized)

## Direction & adaptivity
- ✅ RTL-aware layout (`LayoutDirection { Ltr, Rtl }` + `use_layout_direction() -> Signal<LayoutDirection>` + `update_layout_direction(dir)` — reactive signal, updateable from i18n)
- ⬜ writing modes
- ✅ responsive layout by size-class / orientation (`use_orientation()`, `use_size_class()`, `use_window_width()` — reactive signals; `update_window_size(w,h)` platform hook)
- ⬜ container queries
- ⬜ adaptive split-view (tablet/desktop)

## Custom layout
- ⬜ `Layout` trait — author bespoke layout algorithms
- ⬜ measure/arrange callbacks for custom widgets
- ✅ baseline alignment (`LayoutStyle::align_self` + `.align_self(AlignItems::Baseline)` — already in taffy)

## Performance
- ⬜ dirty-subtree relayout (don't relayout the world)
- ⬜ layout result caching + measure memoization
- ⬜ off-main-thread layout
