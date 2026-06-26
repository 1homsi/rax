# Lists & Scrolling

Match RN `FlatList`/`SectionList`/`VirtualizedList` and Flutter
`ListView`/`CustomScrollView`/slivers. Performance-critical. ✅ · 🟡 · ⬜.

## Scrolling
- ✅ `ScrollView` (vertical, basic)
- ✅ horizontal `ScrollView`; ⬜ bidirectional
- ⬜ momentum / deceleration matching platform physics
- ⬜ overscroll / bounce (iOS) / stretch (Android)
- ✅ paging (`scroll.paging()` → `Attribute::PagingEnabled`; `carousel` now snap-pages)
- ✅ scroll events (`on_scroll(ScrollInfo{offset_x,offset_y,velocity_x,velocity_y})`, `on_scroll_begin`, `on_scroll_end` builders; `Attribute::OnScrollChange/Begin/End`)
- ✅ programmatic scroll (`tree.scroll_to(id, x, y, animated)`, `tree.scroll_to_top(id, animated)` → `Mutation::ScrollTo/ScrollToTop`)
- ✅ scroll indicators (`shows_indicator(bool)` — `UIScrollView.showsVertical/HorizontalScrollIndicator`)
- ⬜ nested scrolling + scroll coordination
- ✅ keyboard-dismiss-on-drag (`scroll.keyboard_dismiss_mode(KeyboardDismissMode::{None|OnDrag|Interactive})` → `setKeyboardDismissMode:` on UIScrollView)
- ⬜ zoomable scroll (pinch)
- ⬜ scroll-to-on-focus (forms)

## Virtualized lists (recycling)
- ⬜ `LazyColumn`/`LazyRow` (windowed rendering)
- ⬜ view **recycling / pooling** (bounded memory for huge lists)
- 🟡 `List` with keyed items + minimal reconciliation
- ⬜ `SectionList` (sticky section headers/footers)
- ⬜ variable / dynamic item heights, measured & cached
- ⬜ horizontal + grid virtualization (`LazyGrid`)
- ✅ separators (`item_separator(color, inset)` — 1pt horizontal rule with leading inset)
- ✅ header / footer / empty-state components (`list_with_header`, `empty_state(msg)`, `sticky_header(content)`)
- ⬜ initialScrollIndex, maintainVisibleContentPosition
- ⬜ estimated item size hints
- ⬜ windowing tuning (overscan)

## Interactions & affordances
- ✅ pull-to-refresh (`RefreshControl`)
- ⬜ infinite scroll / onEndReached / pagination
- ⬜ swipe actions (swipe-to-delete, leading/trailing)
- ⬜ drag-to-reorder
- ⬜ multi-select
- ⬜ sticky headers, floating headers, collapsing toolbars

## Advanced (slivers-equivalent)
- ⬜ composable scroll effects (parallax, collapsing, pinned)
- ⬜ scroll-linked animations
- ⬜ `CustomScrollView`-style composition of scrollable regions
- ⬜ virtualized 2D grids / tables

## Performance
- ⬜ off-main-thread layout for list items
- ⬜ recycle + diff so only changed items mutate
- ⬜ jank instrumentation for scroll
