# Styling & Theming

Typed, predictable styling with a runtime-switchable theme system. The core of
[super-customizability](../03-customizability.md). ✅ · 🟡 · ⬜.

## Paint properties
- ✅ background color, corner radius
- ✅ text color, font size
- 🟡 borders (per-edge width, color, style, per-corner radius)
- 🟡 shadows (box + text), elevation
- ✅ linear gradient (vertical/horizontal/custom points); 🟡 radial/sweep + multiple backgrounds pending
- ✅ custom font family (`font_family()`)
- 🟡 opacity, blend modes
- ⬜ background images / nine-patch
- ✅ blur / backdrop-filter (`.blur(radius)` → `Attribute::BlurRadius(f32)` → UIVisualEffectView stub; real blur pending)
- ✅ clip / overflow (`.clip(bool)` → `Attribute::ClipToBounds(bool)` → `setClipsToBounds:`)
- 🟡 filters (brightness/contrast/saturate), tint (`.tint(color)` → `Attribute::TintColor` → `setTintColor:` ✅; brightness/contrast pending)

## Style application model (resolution order, explicit)
- ✅ inline style (per instance)
- ⬜ per-type variants (e.g. Button "primary"/"ghost"/custom)
- ✅ theme defaults (tokens)
- ⬜ documented precedence: inline > variant > theme > default (no magic cascade)
- ✅ conditional styles (disabled/visible/hidden — `.disabled_opacity()`, `.visible_when()`, `.hidden_when()` reactive opacity helpers)
- ✅ responsive styles (`responsive(|size_class, orientation| ...)` — reactive builder re-runs on size/orientation change using `use_size_class` + `use_orientation` memos); also `.style_if(cond, apply)` and `.dark_mode_style(apply)`
- ⬜ style composition / merge / extend

## Design tokens (typed)
- ✅ color palette + semantic roles (`ColorTokens` — primary/surface/onSurface/error/success/warning/info/outline + light/dark Material-3 palettes)
- ✅ spacing scale, radius scale (`SpacingTokens{xs/sm/md/lg/xl/xxl}`, `RadiusTokens{xs/sm/md/lg/xl/full}`)
- ✅ typography scale (`TypographyTokens` — display/headline/title/body/label at all sizes)
- ✅ shadow/elevation tokens (`ShadowTokens{sm/md/lg/xl}` in `Theme`; `ShadowToken{color,offset_x,offset_y,blur}`)
- ✅ motion tokens (`MotionTokens` — duration_short/medium/long + easing names)
- ✅ z-index (`.z_index(n)` → `Attribute::ZIndex(i32)` → CALayer `setZPosition:`); ⬜ opacity/breakpoints tokens
- ✅ custom/user-defined tokens (`CustomTokens{values: HashMap<String,String>}` in Theme; `.set(key, value)` + `.get(key)`)

## Theming
- ✅ `Theme` context (scoped/nested themes)
- ✅ runtime theme switching (no rebuild) via signals — only affected props update
- 🟡 light / dark / high-contrast modes; system-driven + manual override
  - ✅ reactive system color-scheme signal (`use_color_scheme`) — content auto-adapts to OS light/dark
  - ✅ safe-area backdrop: fixed color or `System { light, dark }` auto-following appearance
  - ⬜ high-contrast; manual app-level override of the system scheme
- ✅ brand theme packages (`ThemeBuilder::from(base).primary(color).surface(color).custom_token(k,v).build()` — composable theme derivation)
- ✅ component registry (`register_component(name, factory)` → thread-local `HashMap<String, Factory>`; `resolve_component`, `unregister_component`, `ComponentProps` builder)
- ⬜ per-platform theme overrides (native-feel iOS vs Android vs your own)
- ⬜ dynamic color (Material You / system accent) integration

## Tooling
- ⬜ theme editor / preview in devtools
- ⬜ contrast & a11y linting of token combos
- ⬜ export/import design tokens (Style Dictionary / Figma interop)
