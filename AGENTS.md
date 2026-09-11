# AGENTS.md

macOS app (Swift 5, macOS 15+, SwiftUI/AppKit) that pins a black notch to a
screen edge showing how much of each coding assistant's usage limit is left.
XcodeGen-managed, no SPM package. A Rust/Tauri 2 Windows port lives in
`windows/` as a separate app — no code is shared with `Sources/`.

## Build & test

- `brew install xcodegen` is required once: `*.xcodeproj` is **gitignored** and
  regenerated from `project.yml` by `make gen`. Every `make` target implies
  `gen`. Never hand-edit the `.xcodeproj`.
- `make build` / `make test` / `make run` / `make clean`. No Apple account or
  signing identity needed — Debug builds ad-hoc sign automatically.
- `make test-ci` = unsigned tests; this is what CI runs (`macos-26` only — an
  older Xcode cannot read the project at all).
- `make release` (archive → notarize → Sparkle appcast) needs the maintainer's
  Developer ID + notary credentials. `make dmg-ci` is the unsigned-CI disk
  image path. Release ritual: bump `CURRENT_PROJECT_VERSION` (and
  `MARKETING_VERSION` in project.yml) — `testTheCurrentVersionHasANote` fails
  the build if a version is bumped without a `ReleaseNotes` entry.

## Generated-from-config trap

`Sources/Info.plist` is **generated** from the `project.yml` `info:` block and
is overwritten by every `make gen`. Edit project.yml, never the plist. The
version number also lives only in project.yml; hardcoding it anywhere pins
every build to one version and Sparkle never sees an update.

## Testing conventions

- Unit tests are hosted **by the app itself**, so a test run is a real app
  launch. Any host-app side effect (network I/O, window creation, polling)
  must be guarded with `Runtime.isUnderTest` — `Sources/App/Runtime.swift.
  Never let tests hit a provider endpoint.
- Tests read fixtures and recorded response bodies, never keychains or live
  accounts. Provider parser tests are pinned to recorded JSON — when a vendor
  changes a response shape, those tests fail first. Add the same pin for a new
  provider parser.
- `Tests/` mirrors `Sources/` **by concern, not by file**. Find the existing
  test class closest to what you change before adding a new file.
- `make test` runs the whole suite; there is no focused per-file target
  (xcodebuild needs scheme `Codenotch` / bundle `com.vinz.codenotch.tests` if
  you want to filter with `-only-testing`).

## Architecture landmarks

- `Sources/Providers/` — one adapter per provider, each implementing
  `UsageProvider` and declaring a `Fidelity` (`.official` / `.derived` /
  `.manual`). Never present a guessed number as official.
- `Sources/Model/UsageStore.swift` — a `@MainActor ObservableObject` that polls
  providers on a timer (60s while a session runs, 5 min otherwise) and persists
  last-good readings via `UsageArchive`. Restored readings must come back
  `.stale` and dimmed — never present an archived number as live.
- `Sources/Notch/NotchLayout.swift` — every layout constant is quoted from
  `docs/design/frame-124-hover-tooltip.png` via `Design.px(_:)`. If you change
  layout math, check it against that frame.

## Provider rules (high-traffic footguns)

- Every failure path must map to a `ProviderStatus` / `UsageProviderError`
  (`stale`, `needsAuth`, `accessDenied`, `credentialExpired`,
  `signedOutByOwner`, `nothingMetered`…), never throw something the UI cannot
  render, never invent a number or default a missing window to zero.
- `account()` and `signInRoute`/`signOut()`/`forgetCachedCredential()` must be
  **protocol requirements, not extension members** — extension members dispatch
  statically through `any UsageProvider` and silently no-op.
- The app deliberately runs **no OAuth flow of its own**; it borrows the
  owning tool's credential. Don't add a sign-in flow except via
  `WebSessionProvider`.
- Keychain-held credentials go through `CredentialCache` + `KeychainItem
  .modifiedAt` (read only when the owning app changed the item). Ad-hoc builds
  get a fresh identity every rebuild, so keychain "Always Allow" grants don't
  survive — `Scripts/sign-local.sh` signs with a stable self-signed identity
  for local development after `make run`.
- 429 handling: the endpoint can answer `Retry-After: 0`; treat the hint as a
  floor-raiser only (60s, doubling, capped at 15 min) and persist the deadline.

## Localization

User-visible strings go through `L10n.t("English source")` — the English
string **is** the key. Put optional translations in
`Sources/Localizable.xcstrings`; a missing translation falls back to English
and must never fail tests. Don't freeze `L10n.t` in a `static let` (lookup must
see the current language). `windows/codenotch/src/i18n.rs` is a separate
system — don't merge the two.

## Style

Comments explain **why** (a hidden constraint, a bug worked around), not what.
No premature abstraction. See `CONTRIBUTING.md`.

## Logging

The app has no window; diagnostics go to the unified log:
`/usr/bin/log stream --predicate 'subsystem == "com.vinz.codenotch"'`.
Note `/usr/bin/log`, not `log` — zsh has a builtin that swallows it.
`CODENOTCH_DEMO=1` runs the app on fixed sample data instead of live readings.