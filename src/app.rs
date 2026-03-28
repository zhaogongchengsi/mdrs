use gpui::{Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div, prelude::*};
use gpui_component::{
    ActiveTheme, Selectable, h_flex, v_flex,
    button::{Button, ButtonVariants},
    input::{Input, InputEvent, InputState},
};

use crate::preview::MarkdownPreview;

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
}

impl MdrsApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(|cx| {
            InputState::new(window, cx)
                .multi_line(true)
                .soft_wrap(true)
                .default_value(DEFAULT_MARKDOWN)
        });

        let initial_text = editor.read(cx).value().to_string();
        let preview = cx.new(|_cx| MarkdownPreview::new(&initial_text));

        let preview_ref = preview.clone();
        let subscription = cx.subscribe(
            &editor,
            move |_this: &mut MdrsApp,
                  editor_entity: Entity<InputState>,
                  event: &InputEvent,
                  cx: &mut Context<MdrsApp>| {
                if let InputEvent::Change = event {
                    let new_text = editor_entity.read(cx).value().to_string();
                    preview_ref.update(cx, |p, cx| {
                        p.set_markdown(&new_text);
                        cx.notify();
                    });
                }
            },
        );

        Self {
            editor,
            preview,
            _subscription: subscription,
            view_mode: ViewMode::Split,
        }
    }
}

impl Render for MdrsApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg = cx.theme().colors.background;
        let border = cx.theme().colors.border;
        let entity = cx.entity();
        let view_mode = self.view_mode;

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
            ViewMode::EditorOnly => h_flex()
                .flex_1()
                .w_full()
                .child(
                    div()
                        .w_full()
                        .h_full()
                        .child(Input::new(&self.editor).h_full()),
                ),
            ViewMode::PreviewOnly => h_flex()
                .flex_1()
                .w_full()
                .child(
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
            // Toolbar
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
            // Content area
            .child(content)
    }
}
