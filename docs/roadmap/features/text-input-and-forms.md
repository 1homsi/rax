# Text Input, Keyboard & Forms

The hardest parity area (controlled input + IME). Goal: match RN `TextInput` and
Flutter `TextField`/`Form` completely. ⬜ planned unless noted.

## TextInput / TextField
- ✅ controlled value (value ↔ signal, two-way, race-free)
- ⬜ uncontrolled / defaultValue
- ✅ single-line + multi-line (`TextArea` via UITextView)
- ✅ placeholder, prefix/suffix, clear button (`.placeholder_color(Color)`, `.prefix(str)`, `.suffix(str)`, `.clear_button(bool)` → `Attribute::PlaceholderColor/InputPrefix/InputSuffix/ClearButton`; iOS: attributedPlaceholder, clearButtonMode; prefix/suffix TODO)
- ⬜ selection + caret control (programmatic get/set)
- ✅ keyboard types (`.keyboard_type(KeyboardType::Email/NumberPad/PhonePad/Url/DecimalPad/…)`)
- ✅ return key types (`return_key()`) + onSubmit (`on_submit()`)
- ⬜ autocapitalize, autocorrect, spellcheck, autocomplete/contentType
- ✅ secure entry (`secure()`, password)
- ✅ max length (`.max_length(n)` → `Attribute::MaxLength(usize)`; delegate enforcement TODO); ⬜ input masks / formatters
- ✅ editable/read-only/disabled (`.read_only(bool)` → `Attribute::ReadOnly(bool)` → `setEnabled:false`)
- ⬜ onFocus/onBlur/onChange/onKeyPress/onSelectionChange
- ⬜ focus management: focus()/blur(), focus order, focus traversal

## IME / composition (the hard part)
- ⬜ composition (marked text) for CJK/dictation without clobbering
- ⬜ autocorrect/suggestion bar integration
- ⬜ predictive text, inline completion
- ⬜ emoji & dictation input

## Keyboard
- ✅ keyboard avoidance (`keyboard_avoiding_view(content)` — scroll-backed, UIScrollView adjusts contentInset)
- ✅ keyboard show/hide events + frame (`use_keyboard_height() -> Signal<f32>`; `update_keyboard_height(h)` called from UIKeyboardWillShow/HideNotification)
- ⬜ input accessory view / toolbar (done button, custom)
- ⬜ hardware keyboard + shortcuts (desktop/tablet), key events, modifiers
- ⬜ custom in-app keyboards

## Forms
- ✅ form state management (values, touched, dirty) — `rax-form`: `FormField{value, error, dirty}` signals
- ✅ validation (sync), error messages — `Validator::{Required, MinLength, MaxLength, Email, Contains, Custom}`
- ✅ field-level + form-level validation — `FormField::validate()` + `Form::validate()` (all fields)
- ✅ submit handling, reset — `Form::validate() -> bool`, `Form::reset()`
- ⬜ async validation, schema validation
- ⬜ accessible labels/errors wiring
- ⬜ multi-step / wizard helpers
- ⬜ controlled groups (radio/checkbox/select)

## Customizability
- ⬜ headless input core (state + IME + a11y) with bring-your-own presentation
- ⬜ fully custom-rendered text editing on the GPU path (advanced)
