# Verification record — 2026-09-28

Validated on Windows x64 with Rust 1.97.1 and Node.js 24.19.0.

- `pnpm check`: no Svelte/TypeScript errors or warnings.
- `pnpm check:i18n`: 221 Simplified Chinese translations and interpolation placeholders verified.
- `pnpm build`: production assets generated successfully.
- `cargo test --manifest-path src-tauri/Cargo.toml --offline`: 35 fixture-based integration tests and native filesystem watcher, fixture-isolation, and language-migration tests passed (38 total).
- `cargo clippy --manifest-path src-tauri/Cargo.toml --offline --all-targets -- -D warnings`: passed.
- `pnpm tauri build`: produced the final Windows x64 executable (approximately 6 MB) and `src-tauri/target/release/bundle/nsis/Sunshine Library Sync_0.1.0_x64-setup.exe` (2.31 MiB). The initial NSIS download timed out; the retry completed with Tauri's hash verification. The installer was built but has not been installed as part of verification.
- Fixed the shared Switch component to use the installed Bits UI version's `data-state="checked"` / `"unchecked"` attributes. Both state colors and thumb movement now have matching generated CSS, with explicit pixel dimensions for the app's 13 px base font, visible On / Off text, dark theme styling, focus, disabled, RTL and reduced-motion variants. Svelte checks and the production build passed after the final change; visual inspection remains unverified as described below.
- An earlier Release executable was launched with an isolated `.test-data/desktop` configuration. Application logs confirmed 4 Steam and 2 Epic games from real-format local manifests. The fixture apps.json remained unchanged on first launch. The test script now launches only Debug; Release ignores `SUNSHINE_LIBRARY_SYNC_DATA` and always uses `%LOCALAPPDATA%\SunshineLibrarySync`.
- Rebuilt the Release executable and NSIS installer after this change (2026-09-28 16:19 local time). A search of the Release EXE and generated frontend assets found none of the test game names. SHA-256: EXE `65750E0BFA55CDD25995CFC7B4B55FCD8B4561AD296E1401449568B546C1F31F`; installer `F2C1A712E4E14E3E8F511EAF38A8E72C56C406A37B12146CA838C203E5FE37AD`.

The tests cover multi-library Steam scans, uninstall diffing, incomplete metadata, Epic filtering and URI encoding, provider failure isolation, identity collisions, manual applications, external changes, exclusions, ownership corruption, transaction recovery, atomic Windows replacement, failed backups, repeated-sync byte/mtime stability, proxy validation and failed artwork with offline sync.

Native UI inspection was attempted with the computer-use tool, but application access approval timed out. The browser-based frontend inspection also encountered an automatic approval timeout (including its one permitted retry). Consequently, visual layout, interactive Tauri command flows, tray/close behavior, and an actual Moonlight launch are **not recorded as verified**. The app was launched only against fixture data; no production Sunshine library was synchronized or restarted.
