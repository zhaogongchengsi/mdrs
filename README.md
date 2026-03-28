# mdrs

A **lightweight** Markdown editor and live-preview app built with Rust and [gpui](https://gpui.rs).

## Features

- **Split-pane layout** — editor on the left, live preview on the right
- **Live preview** — the preview updates as you type
- **Markdown rendering** — headings, paragraphs, bold/italic, inline code, code blocks (with language label), blockquotes, ordered/unordered lists, and horizontal rules
- **Lightweight** — no Electron, no browser engine; GPU-accelerated native UI via gpui

## Screenshot

```
┌──────────────────────────┬──────────────────────────┐
│  # Hello mdrs            │  Hello mdrs              │
│                          │                          │
│  A **lightweight** app   │  A lightweight app       │
│                          │                          │
│  ## Features             │  Features                │
│  - Live preview          │  • Live preview          │
│  - Fast                  │  • Fast                  │
└──────────────────────────┴──────────────────────────┘
       Editor (left)              Preview (right)
```

## Requirements

- Rust 1.80+
- Linux: `libxcb`, `libxkbcommon`, `libxkbcommon-x11`

  ```sh
  # Ubuntu / Debian
  sudo apt install libxcb1-dev libxkbcommon-dev libxkbcommon-x11-dev
  ```

- macOS: Xcode command-line tools

## Build & Run

```sh
git clone https://github.com/zhaogongchengsi/mdrs
cd mdrs
cargo run
```

## Architecture

| File | Purpose |
|---|---|
| `src/main.rs` | Application entry point, window setup |
| `src/app.rs` | Root view (`MdrsApp`) — wires editor ↔ preview |
| `src/editor.rs` | Editor extension point (gpui-component `InputState`) |
| `src/preview.rs` | Markdown parser + `MarkdownPreview` gpui view |

## Dependencies

| Crate | Purpose |
|---|---|
| [`gpui`](https://crates.io/crates/gpui) | GPU-accelerated UI framework by Zed |
| [`gpui-component`](https://crates.io/crates/gpui-component) | Rich UI components (text editor, theme, …) |
| [`pulldown-cmark`](https://crates.io/crates/pulldown-cmark) | CommonMark-compliant Markdown parser |

## License

MIT
