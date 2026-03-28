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
