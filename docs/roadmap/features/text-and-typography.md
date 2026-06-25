# Text & Typography

Native-quality text. Goal: match Flutter's `RichText`/`TextSpan` power and RN's
`Text` with the platform's own shaping/a11y. ✅ · 🟡 · ⬜.

## Basic text
- ✅ string + reactive text content
- ✅ font size, color
- 🟡 font family, weight, style (italic)
- ✅ line height, letter spacing (`.line_height(f32)`, `.letter_spacing(f32)` builders; NSParagraphStyle/NSKern on iOS)
- 🟡 text alignment (start/center/end/justify)
- ⬜ truncation / ellipsis (head/middle/tail), max lines
- ⬜ text wrapping / break strategies
- ✅ text decoration (`.underline()`, `.strikethrough()` → `TextDecoration` attribute; NSUnderlineStyle on iOS)
- ⬜ text transform (uppercase/lowercase/capitalize)
- ✅ text shadow (`.text_shadow(color, offset_x, offset_y, blur)` → `NSShadow` on iOS)
- ⬜ selectable text (copy/select)

## Rich text
- ✅ spans / `TextSpan` (`rich_text().span(TextSpan::new(text).bold().color(…).underline().letter_spacing(f)`) — NSAttributedString)
- ⬜ inline images / widgets in text
- ⬜ tappable links / mentions / hashtags
- ⬜ markdown rendering helper

## Fonts
- ⬜ bundled custom fonts (asset pipeline)
- ⬜ runtime/downloadable fonts
- ⬜ variable fonts (weight/optical axes)
- ⬜ font fallback chains
- ⬜ system font + Dynamic Type / font scaling (a11y)
- ⬜ icon fonts

## Internationalized text
- ⬜ bidi (LTR/RTL mixed), correct mirroring
- ⬜ grapheme/emoji cluster correctness
- ⬜ CJK line breaking, Indic shaping, complex scripts
- ⬜ locale-aware casing & collation

## Measurement & rendering
- 🟡 real platform text metrics (replace heuristic)
- ⬜ text-on-GPU-renderer path (custom rendering)
- ⬜ accessibility: VoiceOver rotor, semantic text traits
