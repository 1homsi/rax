# Internationalization (i18n) & Localization (l10n)

Match Flutter `intl` / RN i18n ecosystems. ⬜ planned.

## Messages & translation
- ✅ message catalogs (`I18n::add_locale(locale, &[(key, template)])` — runtime hashmap catalog)
- ✅ interpolation (`{var}` replacement in templates via `i18n.t(key, &[("var","val")])`)
- ✅ pluralization (`singular|plural` split; `i18n.t_plural(key, count, args)`)
- ✅ runtime locale switching (`i18n.set_locale(code)` — `Signal<String>`, no rebuild; reactive reads re-derive)
- ✅ fallback locale chain (falls back to "en" if key missing in active locale)
- ⬜ ICU MessageFormat (select, gender, complex plurals)
- ⬜ extraction tooling (scan source → catalog)
- ⬜ translation file formats (ARB / PO / JSON / XLIFF) import/export

## Formatting
- ✅ numbers, currency (basic — `format_number(f64, decimals)`, `format_currency(amount, symbol)`)
- ⬜ locale-aware number formatting (decimal separator, grouping)
- ⬜ dates, times, relative time, durations
- ⬜ lists, units, measurements
- ⬜ collation / locale-aware sorting & search
- ⬜ calendars (Gregorian + non-Gregorian)

## Layout & text direction
- ⬜ RTL layout mirroring (logical start/end)
- ⬜ bidi text handling
- ⬜ per-locale typography / font selection
- ⬜ locale-aware casing

## Integration
- ✅ RTL detection (`i18n.is_rtl()` — checks locale against known RTL language tags)
- ✅ system locale detection (`system_locale()` — parses `LANG` env var, falls back to "en")
- ⬜ system locale detection (iOS/Android CFLocale / Resources.getConfiguration)
- ⬜ override UI
- ⬜ pseudolocalization for testing
- ⬜ i18n lints (hard-coded strings, missing translations)
- ⬜ region-specific assets (images/audio per locale)
