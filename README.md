# mdrs

A lightweight Markdown editor and live-preview app built with Rust and [gpui](https://gpui.rs).

## Features

- Split-pane editing with live preview
- Native desktop UI powered by `gpui`
- Command-line opening for Markdown files: `cargo run -- path/to/file.md`
- Large-file-aware reads
- Preview protection for oversized Markdown buffers

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
