# QStarem Universal

<p align="center">
  <img src="docs/assets/logo.svg" alt="QStarem logo" width="160">
</p>

<p align="center">
  Open-source Rust desktop browser shell for <a href="https://zstream.mov">Z-Stream</a> with bundled P-Stream userscript injection.
</p>

<p align="center">
  <a href="https://github.com/perlytiara/QStarem-Universal/actions/workflows/ci.yml"><img src="https://github.com/perlytiara/QStarem-Universal/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/perlytiara/QStarem-Universal/releases/latest"><img src="https://img.shields.io/github/v/release/perlytiara/QStarem-Universal?label=release" alt="Latest release"></a>
  <img src="https://img.shields.io/badge/Rust-Tauri%202-orange" alt="Tauri 2">
</p>

<p align="center">
  <a href="https://github.com/perlytiara/QStarem-Android">Android app</a>
  ·
  <a href="https://github.com/perlytiara/QStarem-Universal/releases/latest">Download</a>
</p>

## Why QStarem Universal

Inspired by community desktop wrappers like [P-Stream Desktop](https://github.com/p-stream/p-stream-desktop), QStarem Universal wraps Z-Stream in a native desktop app with:

- **P-Stream userscript** injected automatically (toggle in settings)
- **Configurable home URL** (default `https://zstream.mov`)
- **Native menu** — Back, Forward, Reload, Home, Settings (`Cmd+,`)
- **Cross-platform builds** — macOS (primary), Linux and Windows via separate CI jobs

## Platform support

| Platform | Status | Notes |
|----------|--------|-------|
| macOS (Apple Silicon + Intel) | Tested primary target | WebKit-based; P-Stream via userscript (~90% extension parity) |
| Linux | Community / CI | WebKitGTK; may need distro-specific fixes |
| Windows | Community / CI | WebView2; ad blocking is best-effort |

**Android note:** [QStarem Android](https://github.com/perlytiara/QStarem-Android) uses GeckoView with full Firefox WebExtension support (strongest P-Stream fidelity).

## Install (macOS)

1. Download the latest `.dmg` from [Releases](https://github.com/perlytiara/QStarem-Universal/releases/latest).
2. Open the DMG and drag **QStarem** to Applications.
3. On first launch, if macOS blocks the app: **System Settings → Privacy & Security → Open Anyway**.

Release builds are unsigned in v1.0.0 (no Apple notarization yet).

## Usage

- The main window loads your Z-Stream instance.
- **Navigation → Settings…** or press `Cmd+,` / `Ctrl+,`.
- Toggle P-Stream, change home URL, or clear browsing data from settings.

## Build from source

### Requirements

- Rust stable
- macOS: Xcode Command Line Tools
- Linux: WebKitGTK and build essentials (see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/))
- Windows: WebView2 runtime + Visual Studio Build Tools

### Commands

```bash
git clone https://github.com/perlytiara/QStarem-Universal.git
cd QStarem-Universal
./scripts/fetch-assets.sh
cd src-tauri
cargo tauri dev
```

Release build:

```bash
cd src-tauri
cargo tauri build
```

## Project structure

```text
src-tauri/src/       Rust shell, settings, P-Stream injection
ui/                  Settings panel (HTML)
scripts/             Fetch P-Stream userscript asset
```

## Disclaimer

QStarem is a browser shell for user-configured streaming frontends. It does not host, index, or distribute content. P-Stream userscript and Z-Stream are third-party projects with their own licenses and terms.

## Related projects

- **[QStarem Android](https://github.com/perlytiara/QStarem-Android)** — GeckoView mobile shell with bundled P-Stream extension and ad blockers.

## License

[MIT](LICENSE)
