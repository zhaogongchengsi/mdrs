# mdrs

A lightweight Markdown editor and live-preview app built with Rust and [gpui](https://gpui.rs).

## Features

- Minimal desktop Markdown workflow with preview-first and edit modes
- Single file launch opens preview first, then expands the editor on demand
- Folder launch shows a workspace sidebar and file list, then previews a file before editing
- No-argument launch starts directly in editing mode for new documents
- Native desktop UI powered by `gpui`
- Large-file-aware reads and preview protection for oversized Markdown buffers

## Large File Handling

- Files up to 1 MB use a direct read path for lower overhead
- Files above 1 MB switch to buffered streaming reads
- Preview parsing is limited to the first 512 KB so the UI stays responsive

## Build and Run

```sh
git clone https://github.com/zhaogongchengsi/mdrs
cd mdrs
cargo run
```

Open a file directly:

```sh
cargo run -- path/to/file.md
```

Open a workspace folder:

```sh
cargo run -- path/to/folder
```

## Package for macOS

Use the project script to build a macOS installer package:

```sh
./scripts/package-macos.sh
```

Package for Windows on a Windows machine:

```powershell
.\scripts\package-windows.ps1
```

Notes:

- The packaged icons live at `src/assets/icon_1024.icns` and `src/assets/icon_1024.ico`.
- If you refresh the AppIcon asset catalog later, rebuild the Windows `.ico` with:

```sh
cargo run --offline --manifest-path tools/ico-builder/Cargo.toml -- src/assets/AppIcon.appiconset src/assets/icon_1024.ico
```

- Runtime assets are bundled into the app package from `src/assets/`.
- Output artifacts are written to `dist/packager/`.
- Windows packaging is configured for `nsis` now, and the script also accepts `wix` as an optional format.

## Architecture

| File | Purpose |
| --- | --- |
| `src/main.rs` | Application entry point and startup wiring |
| `src/file_loader.rs` | Markdown file loading and large-file read strategy |
| `src/app.rs` | Root UI state, async loading, editor and preview coordination |
| `src/editor.rs` | Editor extension point |
| `src/preview.rs` | Markdown parsing and preview rendering |

## License

MIT
