# Components / Widget Library

The standard widget set. Goal: the **union** of React Native's core + essential
community components and Flutter's material/cupertino/widgets — every one
styleable, with a headless core, and replaceable via the theme registry
(see [customizability](../03-customizability.md)). ✅ shipped · 🟡 wip · ⬜ planned.

## Primitives & layout containers
- ✅ `View` (box / container)
- ✅ `Text`
- ✅ `Button`
- ✅ `Column` / `Row` (flex containers)
- ✅ `Spacer`
- ✅ `Dynamic` (reactive subtree)
- ✅ `Stack` / `ZStack` (overlapping, z-ordered children)
- ✅ `Wrap` (flow layout — `wrap(gap, items)` → row with FlexWrap::Wrap)
- ✅ `Grid` (`grid(columns, gap, items)` — composed); ⬜ `LazyGrid` (virtualized)
- ⬜ `Expanded` / `Flexible` helpers
- ⬜ `AspectRatio`, `Center`, `Align`, `Positioned` (absolute)
- ⬜ `SafeArea`
- 🟡 `ScrollView` (✅ basic), `LazyColumn`/`LazyRow`
- ⬜ `Fragment` / keyed `For` list helper

## Text & display
- ✅ `Text` with font family/weight/size/color/line-height/align/truncation/multi-line (`lines(n)`) — font_family shipped
- ✅ Rich text / spans (`rich_text().span(TextSpan::new(text).bold().color(…))` — NSAttributedString)
- ✅ `Icon` (vector icon set + custom)
- ✅ `Image` (source + tint + raw bytes/ImageData + `network_image(url, placeholder)` convenience component)
- ✅ `Avatar` (composed from public API)
- ✅ `Badge` (composed from public API)
- ✅ `Divider` / `Separator`
- ✅ `Card` primitive (composed from public API)
- ✅ `Chip` / `Tag` (composed from public API)
- ✅ `Tooltip` (`tooltip(content, message)` — tap-to-toggle bubble composed from column + dynamic)
- ✅ `Skeleton` / shimmer placeholder (`skeleton(width, height)` — animated opacity oscillation; `.color()` / `.radius()` builder)

## Input & controls
- ✅ `TextInput` / `TextField` (single + multi-line) — see [text-input](text-input-and-forms.md)
- ✅ `Switch` / `Toggle`
- ✅ `Checkbox` (composed from public API — no engine support needed)
- ✅ `Radio` / `RadioGroup` (composed from public API)
- ✅ `Slider` (single + range)
- ✅ `Stepper`
- ✅ `SegmentedControl`
- ✅ `Picker` / `Select` / `Dropdown` (inline, composed)
- ⬜ `DatePicker` / `TimePicker` / `DateTimePicker`
- ✅ `Pressable` / `Touchable` (`pressable(content, on_press)` — opacity 0.4 while pressed via pan-began/ended signal)
- ⬜ `RatingBar`
- ✅ `SearchBar` (`search_bar(query, on_change, placeholder)` — composed)
- ⬜ `ColorPicker`

## Feedback & status
- ✅ `ActivityIndicator` / `Spinner`
- ✅ `ProgressBar` (linear) / `ProgressRing` (circular)
- ✅ `Toast` / `Snackbar` (composed)
- ✅ `Alert` / `Dialog` (`alert(show, title, message, button_label)` — composed modal overlay)
- ✅ `ActionSheet` (`action_sheet(show, title, actions)` — composed from bottom_sheet)
- ✅ `Banner` / inline alert (`banner(visible, message, BannerKind::{Info|Success|Warning|Error})` — composed colored strip)
- ⬜ `RefreshControl` (pull-to-refresh)
- ✅ `StatusBar` control (`status_bar(StatusBarStyle::{Dark|Light|Auto})` — zero-sized view emitting UIStatusBarStyle attribute)

## Overlays & surfaces
- ✅ `Modal` (composed)
- ✅ `BottomSheet` (composed)
- ⬜ `Popover`
- ⬜ `Menu` / `ContextMenu`
- ✅ `Drawer` / `SideMenu` (`drawer(show, on_dismiss, width, content)` — composed)
- ✅ `Backdrop` / scrim (`backdrop(opacity, on_tap)` — semi-transparent overlay column)

## Navigation surfaces
- ✅ `AppBar` / `NavigationBar` / `Toolbar` (composed: title + optional back button + trailing actions)
- ✅ `TabBar` / `TabView` / `BottomNavigation` (`tab_bar(tabs, selected)` — content area + button row; opacity shows active pane)
- ✅ `Breadcrumbs` (`breadcrumbs(items, on_tap)` — row of tappable labels separated by " / ")
- ✅ `SegmentedTabs` (`segmented_control(options, selected)` — composed horizontal pill row)

## Containers & disclosure
- ✅ `Accordion` / `Disclosure` / `ExpansionPanel` (`accordion(sections)` — single-open, opacity-gated body panes)
- ✅ `Collapsible` (`collapsible(header, expanded, content)` — tappable header flips chevron, `show()` gates body)
- ✅ `Carousel` / `PageView` (`carousel(items_signal, gap, item_fn)` — dynamic horizontal scroll)
- ✅ `SwipeActions` (swipe-to-delete etc.) — see composite.rs `swipe_actions`
- ✅ `Pull-to-refresh` + `infinite_scroll(content, loading, on_load_more)` helpers
- ✅ `KeyboardAvoidingView` (`keyboard_avoiding_view(content)` — scroll-backed, platform adjusts insets)
- ⬜ `Resizable` / `SplitView` (desktop/tablet)

## Data display
- ⬜ `List` / `SectionList` / `VirtualizedList` (recycled) — see [lists](lists-and-scrolling.md)
- ⬜ `Table` / `DataGrid`
- ⬜ `Tree` view
- ⬜ Charts primitives (line/bar/pie) — custom-drawn on the GPU renderer

## Media
- ⬜ `Image`, `AnimatedImage` (GIF/WebP), `SVG`
- ⬜ `Video` player
- ✅ `Camera` preview view / QR scanner (AVFoundation-backed)
- ✅ `Map` view (`map_view(lat, lon).span(lat_d, lon_d).annotation(lat, lon, title)` — MKMapView backed)
- ✅ `WebView` (`web_view(url)` / `web_view_html(html)` — WKWebView escape hatch)

## Cross-cutting requirements for every component
- Styleable inline + via theme tokens + per-type variants.
- Headless core (state/a11y/gestures) separable from presentation.
- Replaceable app-wide via the component registry.
- Accessible by default (role/label/state) — see [accessibility](accessibility.md).
- RTL-correct and locale-aware where text is involved.
- Works under both the native-widget backends and the GPU renderer.
