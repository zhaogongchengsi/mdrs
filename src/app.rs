use std::fs;
use std::path::{Path, PathBuf};

use gpui::{div, prelude::*, Context, Entity, IntoElement, ParentElement, Render, Styled, Window};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    v_flex, ActiveTheme, Selectable,
};

use crate::{
    file_loader::{
        format_bytes, load_markdown_file, LoadMarkdownError, LoadedMarkdown, ReadStrategy,
    },
    preview::{MarkdownPreview, PreviewStats},
    workspace::{collect_markdown_files, WorkspaceFile},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum LaunchContext {
    Scratch,
    SingleFile,
    Folder,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PaneMode {
    Preview,
    Edit,
}

const DEFAULT_MARKDOWN: &str = r#"# Welcome to mdrs

A **lightweight** Markdown editor built with Rust and [gpui](https://gpui.rs).

## Start here

- Create a new document from scratch
- Write Markdown in the editor
- See the preview update beside it
"#;

pub struct MdrsApp {
    editor: Entity<InputState>,
    preview: Entity<MarkdownPreview>,
    _subscription: gpui::Subscription,
    launch_context: LaunchContext,
    pane_mode: PaneMode,
    workspace_root: Option<PathBuf>,
    workspace_files: Vec<WorkspaceFile>,
    current_path: Option<PathBuf>,
    document_bytes: u64,
    read_strategy: Option<ReadStrategy>,
    is_loading: bool,
    is_dirty: bool,
    load_error: Option<String>,
    suppress_dirty_tracking: bool,
}

impl MdrsApp {
    pub fn new(
        window: &mut Window,
        initial_target: Option<PathBuf>,
        cx: &mut Context<Self>,
    ) -> Self {
        let launch_context = determine_launch_context(initial_target.as_deref());
        let initial_buffer = match launch_context {
            LaunchContext::Scratch => DEFAULT_MARKDOWN,
            LaunchContext::SingleFile | LaunchContext::Folder => "",
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
            launch_context,
            pane_mode: if launch_context == LaunchContext::Scratch {
                PaneMode::Edit
            } else {
                PaneMode::Preview
            },
            workspace_root: None,
            workspace_files: Vec::new(),
            current_path: None,
            document_bytes: initial_text.len() as u64,
            read_strategy: None,
            is_loading: false,
            is_dirty: false,
            load_error: None,
            suppress_dirty_tracking: false,
        };

        match launch_context {
            LaunchContext::Scratch => {}
            LaunchContext::SingleFile => {
                if let Some(path) = initial_target {
                    app.open_file(path, window, cx);
                }
            }
            LaunchContext::Folder => {
                if let Some(root) = initial_target {
                    app.workspace_root = Some(root.clone());
                    match collect_markdown_files(&root) {
                        Ok(files) => app.workspace_files = files,
                        Err(error) => {
                            app.load_error = Some(format!("Failed to scan workspace: {error}"))
                        }
                    }
                }
            }
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

    fn open_file(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        self.current_path = Some(path.clone());
        self.is_loading = true;
        self.is_dirty = false;
        self.load_error = None;
        self.document_bytes = 0;
        self.pane_mode = PaneMode::Preview;
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
        if let Some(path) = &self.current_path {
            return path.display().to_string();
        }

        match self.launch_context {
            LaunchContext::Scratch => "Untitled document".to_string(),
            LaunchContext::SingleFile => "Markdown document".to_string(),
            LaunchContext::Folder => self
                .workspace_root
                .as_ref()
                .map(|path| format!("Workspace: {}", path.display()))
                .unwrap_or_else(|| "Workspace".to_string()),
        }
    }

    fn status_label(&self, preview_stats: PreviewStats) -> String {
        let mut parts = Vec::new();

        if self.is_loading {
            parts.push("Loading Markdown file".to_string());
        } else if let Some(error) = &self.load_error {
            parts.push(error.clone());
        } else if let Some(strategy) = self.read_strategy {
            parts.push(format!(
                "{} loaded with {}",
                format_bytes(self.document_bytes),
                strategy.label()
            ));
        } else {
            parts.push(format!("{} in buffer", format_bytes(self.document_bytes)));
        }

        if self.launch_context == LaunchContext::Folder {
            parts.push(format!("{} files", self.workspace_files.len()));
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

    fn can_edit(&self) -> bool {
        matches!(self.launch_context, LaunchContext::Scratch) || self.current_path.is_some()
    }

    fn has_sidebar(&self) -> bool {
        self.launch_context == LaunchContext::Folder
    }

    fn workspace_name(&self) -> String {
        self.workspace_root
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .map(str::to_string)
            .unwrap_or_else(|| "Workspace".to_string())
    }

    fn document_name(&self) -> String {
        self.current_path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .map(str::to_string)
            .unwrap_or_else(|| "Untitled.md".to_string())
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let border = cx.theme().colors.border;
        let fg = cx.theme().colors.foreground;
        let muted = cx.theme().colors.muted_foreground;
        let entity = cx.entity();
        let current_path = self.current_path.clone();

        let mut file_list = v_flex().w_full().gap_1();
        if self.workspace_files.is_empty() {
            file_list = file_list.child(
                div()
                    .px_3()
                    .py_2()
                    .text_color(muted)
                    .text_size(gpui::px(12.0))
                    .child("No Markdown files found"),
            );
        } else {
            for (index, file) in self.workspace_files.iter().enumerate() {
                let file_path = file.path.clone();
                let is_selected = current_path.as_ref() == Some(&file.path);
                file_list = file_list.child(
                    Button::new(("workspace-file", index))
                        .label(file.label())
                        .selected(is_selected)
                        .ghost()
                        .on_click({
                            let entity = entity.clone();
                            move |_, window, cx| {
                                let file_path = file_path.clone();
                                entity.update(cx, |app, cx| {
                                    app.open_file(file_path, window, cx);
                                });
                            }
                        }),
                );
            }
        }

        div()
            .w(gpui::px(248.0))
            .h_full()
            .border_r_1()
            .border_color(border)
            .child(
                v_flex()
                    .size_full()
                    .child(
                        div().px_4().py_3().border_b_1().border_color(border).child(
                            v_flex()
                                .gap_1()
                                .child(
                                    div()
                                        .text_color(muted)
                                        .text_size(gpui::px(11.0))
                                        .child("WORKSPACE"),
                                )
                                .child(
                                    div()
                                        .text_color(fg)
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .child(self.workspace_name()),
                                ),
                        ),
                    )
                    .child(
                        v_flex()
                            .w_full()
                            .px_3()
                            .py_3()
                            .gap_2()
                            .child(sidebar_entry("Switch Workspace", muted, border))
                            .child(sidebar_entry("Settings", muted, border)),
                    )
                    .child(
                        div().flex_1().overflow_y_scrollbar().child(
                            v_flex()
                                .w_full()
                                .px_3()
                                .pb_4()
                                .gap_2()
                                .child(
                                    div()
                                        .px_1()
                                        .text_color(muted)
                                        .text_size(gpui::px(11.0))
                                        .child("FILES"),
                                )
                                .child(file_list),
                        ),
                    ),
            )
    }

    fn render_preview_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().colors.muted_foreground;
        if self.is_loading {
            return div()
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .text_color(muted)
                .child("Loading preview...");
        }

        if let Some(error) = &self.load_error {
            return div()
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .text_color(muted)
                .child(error.clone());
        }

        if self.current_path.is_none() && self.launch_context == LaunchContext::Folder {
            return div()
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .text_color(muted)
                .child("Select a Markdown file to preview it.");
        }

        div().flex_1().child(self.preview.clone())
    }

    fn render_editor_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let border = cx.theme().colors.border;
        let fg = cx.theme().colors.foreground;
        let muted = cx.theme().colors.muted_foreground;

        div()
            .flex_1()
            .h_full()
            .border_r_1()
            .border_color(border)
            .child(
                v_flex()
                    .size_full()
                    .child(panel_header(
                        "Editor",
                        &self.document_name(),
                        fg,
                        muted,
                        border,
                    ))
                    .child(
                        div()
                            .flex_1()
                            .px_4()
                            .py_3()
                            .child(Input::new(&self.editor).h_full()),
                    ),
            )
    }

    fn render_preview_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let border = cx.theme().colors.border;
        let fg = cx.theme().colors.foreground;
        let muted = cx.theme().colors.muted_foreground;

        div().flex_1().h_full().child(
            v_flex()
                .size_full()
                .child(panel_header(
                    "Preview",
                    &self.document_name(),
                    fg,
                    muted,
                    border,
                ))
                .child(
                    div()
                        .flex_1()
                        .px_4()
                        .py_3()
                        .child(self.render_preview_content(cx)),
                ),
        )
    }
}

impl Render for MdrsApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg = cx.theme().colors.background;
        let border = cx.theme().colors.border;
        let fg = cx.theme().colors.foreground;
        let muted = cx.theme().colors.muted_foreground;
        let entity = cx.entity();
        let preview_stats = self.preview.read(cx).stats();
        let show_edit = self.can_edit();

        let mut body = h_flex().flex_1().w_full();
        if self.has_sidebar() {
            body = body.child(self.render_sidebar(cx));
        }

        let content = match self.pane_mode {
            PaneMode::Preview => h_flex()
                .flex_1()
                .w_full()
                .child(self.render_preview_panel(cx)),
            PaneMode::Edit => h_flex()
                .flex_1()
                .w_full()
                .child(self.render_editor_panel(cx))
                .child(self.render_preview_panel(cx)),
        };
        body = body.child(content);

        let mut controls = h_flex().gap_1();
        controls = controls.child(
            Button::new("preview-mode")
                .label("Preview")
                .selected(self.pane_mode == PaneMode::Preview)
                .ghost()
                .on_click({
                    let entity = entity.clone();
                    move |_, _, cx| {
                        entity.update(cx, |app, cx| {
                            app.pane_mode = PaneMode::Preview;
                            cx.notify();
                        });
                    }
                }),
        );
        if show_edit {
            controls = controls.child(
                Button::new("edit-mode")
                    .label("Edit")
                    .selected(self.pane_mode == PaneMode::Edit)
                    .ghost()
                    .on_click({
                        let entity = entity.clone();
                        move |_, _, cx| {
                            entity.update(cx, |app, cx| {
                                app.pane_mode = PaneMode::Edit;
                                cx.notify();
                            });
                        }
                    }),
            );
        }

        v_flex()
            .size_full()
            .bg(bg)
            .child(
                h_flex()
                    .w_full()
                    .flex_shrink_0()
                    .px_4()
                    .py_3()
                    .justify_between()
                    .items_center()
                    .border_b_1()
                    .border_color(border)
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_color(fg)
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child("mdrs"),
                            )
                            .child(
                                div()
                                    .text_color(muted)
                                    .text_size(gpui::px(12.0))
                                    .child(self.source_label()),
                            ),
                    )
                    .child(controls),
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
                    .child(div().child(match self.launch_context {
                        LaunchContext::Scratch => "New document",
                        LaunchContext::SingleFile => "Single file",
                        LaunchContext::Folder => "Workspace preview",
                    }))
                    .child(div().child(self.status_label(preview_stats))),
            )
            .child(body)
    }
}

fn determine_launch_context(initial_target: Option<&Path>) -> LaunchContext {
    match initial_target.and_then(|path| fs::metadata(path).ok()) {
        Some(metadata) if metadata.is_dir() => LaunchContext::Folder,
        Some(_) => LaunchContext::SingleFile,
        None if initial_target.is_some() => LaunchContext::SingleFile,
        None => LaunchContext::Scratch,
    }
}

fn panel_header(
    title: &str,
    subtitle: &str,
    fg: gpui::Hsla,
    muted: gpui::Hsla,
    border: gpui::Hsla,
) -> gpui::Div {
    div().border_b_1().border_color(border).child(
        h_flex()
            .w_full()
            .justify_between()
            .items_center()
            .px_4()
            .py_3()
            .child(
                div()
                    .text_color(fg)
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(title.to_string()),
            )
            .child(
                div()
                    .text_color(muted)
                    .text_size(gpui::px(12.0))
                    .child(subtitle.to_string()),
            ),
    )
}

fn sidebar_entry(label: &str, muted: gpui::Hsla, border: gpui::Hsla) -> gpui::Div {
    div()
        .w_full()
        .px_3()
        .py_2()
        .border_1()
        .border_color(border)
        .text_color(muted)
        .text_size(gpui::px(12.0))
        .child(label.to_string())
}
