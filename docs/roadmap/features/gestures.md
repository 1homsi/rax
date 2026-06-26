# Gestures & Input

Match RN Gesture Handler and Flutter's gesture system. A gesture arena resolves
competing recognizers. ✅ · 🟡 · ⬜.

## Pointer / touch
- ✅ tap (via event seam)
- 🟡 multi-touch pointers (down/move/up/cancel) with ids
- ✅ pressed/hover/focus states (`on_press_in(f)` / `on_press_out(f)` → `Attribute::OnPressIn/Out(Callback)` → UIControl touchDown/touchUpInside)
- ⬜ hit-testing control (pointer-events, hitSlop, z-order aware)

## Recognizers
- ✅ tap, double-tap, multi-tap
- ✅ long-press (with duration, movement tolerance)
- ✅ pan / drag (translation + velocity + phase); ⬜ thresholds/axis-lock
- ✅ pinch / zoom (`on_pinch()` — scale + velocity + phase via `UIPinchGestureRecognizer`)
- ✅ rotation (`on_rotate()` — RotateInfo{rotation, velocity, phase} via UIRotationGestureRecognizer)
- ✅ fling / swipe (`on_swipe(SwipeDirection::{Left|Right|Up|Down}, f)` → `Attribute::OnSwipe{direction, handler}` → UISwipeGestureRecognizer)
- ⬜ force/3D-touch / pressure
- ⬜ edge / screen-edge gestures

## Composition & resolution
- ⬜ gesture arena (declare relationships: simultaneous/exclusive/require-fail)
- ⬜ gesture priority & cancellation
- ⬜ native recognizer bridging (cooperate with platform scroll/back gestures)
- ⬜ nested/overlapping gesture coordination
- 🟡 gesture-driven animations (on_pan + reactive transform/signals; full arena pending)

## Desktop / hardware input
- 🟡 mouse (click/right-click/middle, wheel/trackpad scroll, hover, cursor styles — `cursor(CursorStyle::{Default|Pointer|Text|Grab})` added; full mouse events pending)
- ⬜ keyboard events, shortcuts, modifiers, focus traversal (tab/arrows)
- ⬜ drag-and-drop (in-app + OS-level)
- ⬜ stylus / Apple Pencil (pressure/tilt)
- ⬜ context menus / right-click menus

## Accessibility & feedback
- ⬜ accessible activation (works with screen readers / switch control)
- ✅ haptic feedback (`haptic(HapticStyle)` — Light/Medium/Heavy/Selection/Success/Warning/Error)
- ⬜ focus + keyboard equivalents for every gesture action
