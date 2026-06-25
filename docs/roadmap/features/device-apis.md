# Device & Platform APIs (Native Modules)

The capability surface apps reach for — delivered as first-party **plugins** over
a stable plugin ABI (see [extensibility](extensibility-and-plugins.md)). Goal:
the union of RN community modules + Flutter plugins + Expo SDK. ⬜ planned.

## Sensors & hardware
- ⬜ accelerometer, gyroscope, magnetometer, barometer
- ⬜ device motion / orientation, pedometer
- ⬜ proximity, ambient light
- ✅ haptics (`haptic(HapticStyle)` — UIImpactFeedbackGenerator / UINotificationFeedbackGenerator / UISelectionFeedbackGenerator)
- ⬜ battery status, thermal state
- ⬜ flashlight / torch

## Location & maps
- ⬜ GPS location (one-shot + watch), permissions
- ⬜ geofencing, background location
- ⬜ geocoding / reverse geocoding
- ⬜ Map view (markers, polylines, regions, clustering)

## Connectivity
- ⬜ Bluetooth / BLE (central + peripheral)
- ⬜ NFC (read/write/HCE)
- ⬜ Wi-Fi info, network reachability/type
- ⬜ nearby / multipeer

## Camera, media & files
- ✅ camera capture + QR scanner (AVFoundation-backed); ⬜ image/video picker, media library
- ⬜ file system (read/write/stream), document picker
- ⬜ share sheet, open-with, clipboard
- ⬜ downloads / uploads (background)
- ⬜ printing, PDF generation

## Notifications & background
- ⬜ local notifications (schedule, categories, actions)
- ⬜ push notifications (APNs/FCM), rich/silent push
- ⬜ background tasks / background fetch / headless tasks
- ⬜ app badge, live activities / dynamic island, widgets (home-screen)

## Identity & security
- ⬜ biometrics (Face/Touch ID, fingerprint), passkeys/WebAuthn
- ⬜ secure storage / keychain / keystore
- ⬜ auth helpers (OAuth, sign-in-with-Apple/Google), deep-link auth
- ⬜ app attest / integrity, encryption primitives

## Commerce & platform services
- ⬜ in-app purchases / subscriptions (StoreKit / Play Billing)
- ⬜ contacts, calendar, reminders
- ⬜ health / fitness data (HealthKit / Google Fit)
- ⬜ speech-to-text / text-to-speech
- ⬜ on-device ML / vision hooks (Core ML / ML Kit)

## App & system integration
- ⬜ app lifecycle (foreground/background/terminate) — partially via event seam
- ⬜ permissions framework (request/check, rationale, settings deep-link)
- ⬜ deep links / universal links / app shortcuts / quick actions
- ⬜ system theme / appearance, locale, accessibility settings
- ⬜ device info, app info/version, environment
- ⬜ keyboard, status bar, orientation lock, screen brightness/keep-awake
- ⬜ App Clips / Instant Apps, handoff/continuity

> Each capability ships as a plugin with: a typed Rust API, generated
> platform glue, permission handling, graceful unsupported-platform fallbacks,
> and plugin-conformance tests.
