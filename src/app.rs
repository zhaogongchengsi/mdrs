use std::path::PathBuf;

use gpui::{div, prelude::*, Context, Entity, IntoElement, ParentElement, Render, Styled, Window};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState},
    v_flex, ActiveTheme, Selectable,
};

use crate::{
    file_loader::{
        format_bytes, load_markdown_file, LoadMarkdownError, LoadedMarkdown, ReadStrategy,
    },
    preview::{MarkdownPreview, PreviewStats},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Split,
    EditorOnly,
    PreviewOnly,
}

const DEFAULT_MARKDOWN: &str = r#"# Welcome to mdrs

A **lightweight** Markdown editor built with Rust and [gpui](https://gpui.rs).

## Features

- Live preview as you type
- Syntax highlighting
- Lightweight and fast

## Code Example

```rust
fn main() {
    println!("Hello, mdrs!");
}
```

## Blockquote

> This is a blockquote demonstrating the preview capabilities of mdrs.

---

Start editing on the left to see the preview update on the right!
"#;

pub struct MdrsApp {
    editor: Entity<InputState>,
    preview: Entity<MarkdownPreview>,
    _subscription: gpui::Subscription,
    view_mode: ViewMode,
    current_path: Option<PathBuf>,
    document_bytes: u64,
    read_strategy: Option<ReadStrategy>,
    is_loading: bool,
    is_dirty: bool,
    load_error: Option<String>,
    suppress_dirty_tracking: bool,
}

impl MdrsApp {
    pub fn new(window: &mut Window, initial_file: Option<PathBuf>, cx: &mut Context<Self>) -> Self {
        let initial_buffer = if initial_file.is_some() {
            ""
        } else {
            DEFAULT_MARKDOWN
        };

        let editor = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .soft_wrap(true)
                .default_value(initial_buffer)
        });

        let initial_text = editor.read(cx).value().to_string();
        let preview = cx.new(|_cx| MarkdownPreview::new(&initial_text));

        let subscription = cx.subscribe(
            &editor,
            move |this: &mut MdrsApp,
                  editor_entity: Entity<InputState>,
                  event: &InputEvent,
                  cx: &mut Context<MdrsApp>| {
                if let InputEvent::Change = event {
                    let new_text = editor_entity.read(cx).value().to_string();
                    this.handle_editor_change(&new_text, cx);
                }
            },
        );

        let mut app = Self {
            editor,
            preview,
            _subscription: subscription,
            view_mode: ViewMode::Split,
            current_path: initial_file.clone(),
            document_bytes: initial_text.len() as u64,
            read_strategy: None,
            is_loading: initial_file.is_some(),
            is_dirty: false,
            load_error: None,
            suppress_dirty_tracking: false,
        };

        if let Some(path) = initial_file {
            app.load_initial_file(path, window, cx);
        }

        app
    }

    fn handle_editor_change(&mut self, new_text: &str, cx: &mut Context<Self>) {
        self.document_bytes = new_text.len() as u64;

        if !self.suppress_dirty_tracking {
            self.load_error = None;
            if self.current_path.is_some() {
                self.is_dirty = true;
            }
        }

        self.preview.update(cx, |preview, cx| {
            preview.set_markdown(new_text);
            cx.notify();
        });
        cx.notify();
    }

    fn load_initial_file(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        self.current_path = Some(path.clone());
        self.is_loading = true;
        self.is_dirty = false;
        self.load_error = None;
        self.document_bytes = 0;
        cx.notify();

        let this = cx.weak_entity();
        window
            .spawn(cx, {
                async move |cx| {
                    let result = cx
                        .background_executor()
                        .spawn(async move { load_markdown_file(path) })
                        .await;
                    let _ = this.update_in(cx, move |app: &mut MdrsApp, window, cx| {
                        app.finish_file_load(result, window, cx);
                    });
                }
            })
            .detach();
    }

    fn finish_file_load(
        &mut self,
        result: Result<LoadedMarkdown, LoadMarkdownError>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.is_loading = false;

        match result {
            Ok(loaded) => {
                self.current_path = Some(loaded.path);
                self.document_bytes = loaded.bytes;
                self.read_strategy = Some(loaded.read_strategy);
                self.is_dirty = false;
                self.load_error = None;
                self.suppress_dirty_tracking = true;
                self.editor.update(cx, |editor, cx| {
                    editor.set_value(loaded.text, window, cx);
                });
                self.suppress_dirty_tracking = false;
            }
            Err(error) => {
                self.current_path = Some(error.path().to_path_buf());
                self.document_bytes = 0;
                self.read_strategy = None;
                self.is_dirty = false;
                self.load_error = Some(error.to_string());
                self.preview.update(cx, |preview, cx| {
                    preview.set_markdown("");
                    cx.notify();
                });
                cx.notify();
            }
        }
    }

    fn source_label(&self) -> String {
        self.current_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "Scratch buffer".to_string())
    }

    fn status_label(&self, preview_stats: PreviewStats) -> String {
        let mut parts = Vec::new();

        if self.is_loading {
            parts.push("Loading Markdown file".to_string());
        } else if let Some(error) = &self.load_error {
            parts.push(format!("Load failed: {error}"));
        } else if let Some(strategy) = self.read_strategy {
            parts.push(format!(
                "{} loaded with {}",
                format_bytes(self.document_bytes),
                strategy.label()
            ));
        } else {
            parts.push(format!("{} in buffer", format_bytes(self.document_bytes)));
        }

        if self.is_dirty {
            parts.push("modified".to_string());
        }

        if preview_stats.truncated {
            parts.push(format!(
                "preview limited to {}",
                format_bytes(preview_stats.rendered_bytes as u64)
            ));
        }

        parts.join(" · ")
    }
}

impl Render for MdrsApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg = cx.theme().colors.background;
        let border = cx.theme().colors.border;
        let muted = cx.theme().colors.muted_foreground;
        let entity = cx.entity();
        let view_mode = self.view_mode;
        let preview_stats = self.preview.read(cx).stats();

        let content = match view_mode {
            ViewMode::Split => h_flex()
                .flex_1()
                .w_full()
                .child(
                    div()
                        .w_1_2()
                        .h_full()
                        .border_r_1()
                        .border_color(border)
                        .child(Input::new(&self.editor).h_full()),
                )
                .child(
                    div()
                        .id("preview-pane")
                        .w_1_2()
                        .h_full()
                        .overflow_y_scroll()
                        .child(self.preview.clone()),
                ),
            ViewMode::EditorOnly => h_flex().flex_1().w_full().child(
                div()
                    .w_full()
                    .h_full()
                    .child(Input::new(&self.editor).h_full()),
            ),
            ViewMode::PreviewOnly => h_flex().flex_1().w_full().child(
                div()
                    .id("preview-pane")
                    .w_full()
                    .h_full()
                    .overflow_y_scroll()
                    .child(self.preview.clone()),
            ),
        };

        v_flex()
            .size_full()
            .bg(bg)
            .child(
                h_flex()
                    .w_full()
                    .flex_shrink_0()
                    .p_2()
                    .gap_1()
                    .border_b_1()
                    .border_color(border)
                    .justify_center()
                    .child(
                        Button::new("editor-mode")
                            .label("Editor")
                            .selected(view_mode == ViewMode::EditorOnly)
                            .ghost()
                            .on_click({
                                let e = entity.clone();
                                move |_, _, cx| {
                                    e.update(cx, |view, cx| {
                                        view.view_mode = ViewMode::EditorOnly;
                                        cx.notify();
                                    });
                                }
                            }),
                    )
                    .child(
                        Button::new("split-mode")
                            .label("Split")
                            .selected(view_mode == ViewMode::Split)
                            .ghost()
                            .on_click({
                                let e = entity.clone();
                                move |_, _, cx| {
                                    e.update(cx, |view, cx| {
                                        view.view_mode = ViewMode::Split;
                                        cx.notify();
                                    });
                                }
                            }),
                    )
                    .child(
                        Button::new("preview-mode")
                            .label("Preview")
                            .selected(view_mode == ViewMode::PreviewOnly)
                            .ghost()
                            .on_click({
                                let e = entity.clone();
                                move |_, _, cx| {
                                    e.update(cx, |view, cx| {
                                        view.view_mode = ViewMode::PreviewOnly;
                                        cx.notify();
                                    });
                                }
                            }),
                    ),
            )
            .child(
                h_flex()
                    .w_full()
                    .flex_shrink_0()
                    .px_4()
                    .py_2()
                    .justify_between()
                    .border_b_1()
                    .border_color(border)
                    .text_size(gpui::px(12.0))
                    .text_color(muted)
                    .child(div().child(self.source_label()))
                    .child(div().child(self.status_label(preview_stats))),
            )
            .child(content)
    }
}
