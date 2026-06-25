# Animation & Transitions

Match RN Animated/Reanimated and Flutter's animation framework. Driven by the
scheduler's frame loop. ⬜ planned.

## Core
- ✅ animated values/signals driven on the frame loop
- ✅ timing animations (duration + easing curves: linear/ease/cubic-bezier/steps)
- ✅ spring animations (mass/stiffness/damping; presets: GENTLE/BOUNCY/SNAPPY/SLOW + default/stiff/wobbly)
- ✅ decay / fling (velocity-based coast to stop)
- ⬜ keyframes / sequences / staggers
- ✅ parallel + sequential composition (`parallel()` / `sequence()`)
- ✅ loop / oscillate / yoyo (`oscillate()`)
- ✅ delay (`delayed()`); ⬜ interpolation, clamping/extrapolation
- ⬜ interruptible & reversible animations

## Declarative transitions
- ⬜ implicit transitions (`animate`/`with_transition` on property change)
- ✅ fade enter/exit (`fade_transition(show, content)` — opacity tween on signal change); ⬜ slide/scale/mount unmount
- ⬜ layout animations (auto-animate position/size changes)
- ⬜ list item add/remove/reorder animations
- ⬜ shared-element / hero transitions
- ✅ `AnimatedSwitcher` / crossfade (`use_transition(key_fn, duration_ms) -> Signal<f32>` — fades out/in on key change via `sequence()`)

## Gesture-driven & advanced
- ⬜ gesture-linked animations (drag follows finger)
- ⬜ scroll-linked animations (parallax, collapsing headers)
- ⬜ worklet-style off-main-thread animation (evaluate Reanimated approach)
- ✅ physics-based interactions (`rubber_band(value, min, max)` — iOS overscroll resistance; `lerp`, `smooth_step`, `remap` interpolation helpers)
- ⬜ haptic-synced animation

## Performance & platform
- ⬜ run on the native compositor / layer-backed where possible (Core Animation,
  Android animators) to stay smooth under main-thread load
- ⬜ 120fps support, frame pacing
- ✅ reduced-motion accessibility setting respected (`use_reduced_motion()` signal + `animate_unless_reduced()` helper; platform hook `set_reduced_motion(bool)`)
- ⬜ animation on the GPU renderer path (custom-drawn)

## Customizability
- ✅ pluggable easing/curve functions (`Easing` enum extended: EaseInBack/OutBack/InOutBack, EaseInElastic/OutElastic, EaseOutBounce)
- ⬜ custom transition definitions (used by navigation, modals, etc.)
- ⬜ motion design tokens (durations/curves) in the theme
