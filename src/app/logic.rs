use std::{
    fs,
    path::{Path, PathBuf},
};

use gpui::{prelude::*, Context, Entity, PathPromptOptions, Window};
use gpui_component::input::{InputEvent, InputState};

use crate::{
    file_loader::{format_bytes, load_markdown_file, LoadMarkdownError, LoadedMarkdown},
    preview::{MarkdownPreview, PreviewStats},
    workspace::collect_markdown_files,
};

use super::{AppPage, LaunchContext, MdrsApp, PaneMode, DEFAULT_MARKDOWN};

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
            current_page: AppPage::Workspace,
            sidebar_open: launch_context == LaunchContext::Folder,
            workspace_root: None,
            workspace_files: Vec::new(),
            current_path: None,
            document_bytes: initial_text.len() as u64,
            read_strategy: None,
            is_loading: false,
            is_dirty: false,
            load_error: None,
            status_message: None,
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
            self.status_message = None;
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

    pub(super) fn open_file(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        self.current_path = Some(path.clone());
        self.is_loading = true;
        self.is_dirty = false;
        self.load_error = None;
        self.status_message = None;
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
                self.status_message = None;
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
                self.status_message = None;
                self.preview.update(cx, |preview, cx| {
                    preview.set_markdown("");
                    cx.notify();
                });
                cx.notify();
            }
        }
    }

    pub(crate) fn set_pane_mode(&mut self, pane_mode: PaneMode) {
        self.pane_mode = pane_mode;
    }

    pub(crate) fn toggle_sidebar(&mut self) {
        if self.launch_context == LaunchContext::Folder {
            self.sidebar_open = !self.sidebar_open;
        }
    }

    pub(crate) fn open_settings(&mut self) {
        self.current_page = AppPage::Settings;
    }

    pub(crate) fn open_workspace_page(&mut self) {
        self.current_page = AppPage::Workspace;
    }

    pub(crate) fn prompt_open_file(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Open Markdown file".into()),
        });
        let this = cx.weak_entity();
        window
            .spawn(cx, async move |cx| {
                if let Ok(Ok(Some(mut paths))) = receiver.await {
                    if let Some(path) = paths.pop() {
                        let _ = this.update_in(cx, move |app: &mut MdrsApp, window, cx| {
                            app.open_path(path, window, cx);
                        });
                    }
                }
            })
            .detach();
    }

    pub(crate) fn prompt_open_folder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Open folder".into()),
        });
        let this = cx.weak_entity();
        window
            .spawn(cx, async move |cx| {
                if let Ok(Ok(Some(mut paths))) = receiver.await {
                    if let Some(path) = paths.pop() {
                        let _ = this.update_in(cx, move |app: &mut MdrsApp, window, cx| {
                            app.open_workspace(path, window, cx);
                        });
                    }
                }
            })
            .detach();
    }

    pub(crate) fn save_document(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.status_message = None;

        if let Some(path) = self.current_path.clone() {
            self.save_to_path(path, window, cx);
            return;
        }

        let directory = self
            .workspace_root
            .clone()
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
        let receiver = cx.prompt_for_new_path(&directory, Some("Untitled.md"));
        let this = cx.weak_entity();
        window
            .spawn(cx, async move |cx| {
                if let Ok(Ok(Some(path))) = receiver.await {
                    let _ = this.update_in(cx, move |app: &mut MdrsApp, window, cx| {
                        app.save_to_path(path, window, cx);
                    });
                }
            })
            .detach();
    }

    fn open_path(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        self.current_page = AppPage::Workspace;
        match determine_launch_context(Some(&path)) {
            LaunchContext::Folder => self.open_workspace(path, window, cx),
            LaunchContext::SingleFile | LaunchContext::Scratch => {
                self.launch_context = LaunchContext::SingleFile;
                self.sidebar_open = false;
                self.workspace_root = None;
                self.workspace_files.clear();
                self.open_file(path, window, cx);
            }
        }
    }

    fn open_workspace(&mut self, root: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        self.launch_context = LaunchContext::Folder;
        self.current_page = AppPage::Workspace;
        self.sidebar_open = true;
        self.workspace_root = Some(root.clone());
        self.current_path = None;
        self.document_bytes = 0;
        self.read_strategy = None;
        self.is_loading = false;
        self.is_dirty = false;
        self.status_message = None;
        self.load_error = match collect_markdown_files(&root) {
            Ok(files) => {
                self.workspace_files = files;
                None
            }
            Err(error) => {
                self.workspace_files.clear();
                Some(format!("Failed to scan workspace: {error}"))
            }
        };
        self.set_document_contents("", window, cx);
        self.pane_mode = PaneMode::Preview;
        cx.notify();
    }

    // Keep file loads and workspace switches from being treated as user edits.
    fn set_document_contents(
        &mut self,
        contents: impl Into<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let contents = contents.into();
        self.suppress_dirty_tracking = true;
        self.editor.update(cx, |editor, cx| {
            editor.set_value(contents, window, cx);
        });
        self.suppress_dirty_tracking = false;
    }

    fn save_to_path(&mut self, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.editor.read(cx).value().to_string();
        let this = cx.weak_entity();
        window
            .spawn(cx, async move |cx| {
                let write_path = path.clone();
                let result = cx
                    .background_executor()
                    .spawn(async move {
                        if let Some(parent) = write_path.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        fs::write(&write_path, text)?;
                        Ok::<(), std::io::Error>(())
                    })
                    .await;
                let _ = this.update_in(cx, move |app: &mut MdrsApp, _window, cx| {
                    app.finish_save(path, result, cx);
                });
            })
            .detach();
    }

    fn finish_save(
        &mut self,
        path: PathBuf,
        result: Result<(), std::io::Error>,
        cx: &mut Context<Self>,
    ) {
        match result {
            Ok(()) => {
                let in_workspace = self
                    .workspace_root
                    .as_ref()
                    .is_some_and(|root| path.starts_with(root));

                if !in_workspace {
                    self.launch_context = LaunchContext::SingleFile;
                    self.sidebar_open = false;
                    self.workspace_root = None;
                    self.workspace_files.clear();
                } else if let Some(root) = self.workspace_root.clone() {
                    self.workspace_files = collect_markdown_files(&root).unwrap_or_default();
                }

                self.current_page = AppPage::Workspace;
                self.current_path = Some(path.clone());
                self.document_bytes = self.editor.read(cx).value().len() as u64;
                self.read_strategy = None;
                self.is_dirty = false;
                self.load_error = None;
                self.status_message = Some(format!("Saved {}", path.display()));
            }
            Err(error) => {
                self.status_message = Some(format!("Failed to save file: {error}"));
            }
        }

        cx.notify();
    }

    pub(super) fn status_label(&self, preview_stats: PreviewStats) -> String {
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

        if let Some(message) = &self.status_message {
            parts.push(message.clone());
        }

        parts.join(" · ")
    }

    pub(super) fn can_edit(&self) -> bool {
        matches!(self.launch_context, LaunchContext::Scratch) || self.current_path.is_some()
    }

    pub(super) fn has_sidebar(&self) -> bool {
        self.current_page == AppPage::Workspace
            && self.launch_context == LaunchContext::Folder
            && self.sidebar_open
    }

    pub(super) fn workspace_name(&self) -> String {
        self.workspace_root
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .map(str::to_string)
            .unwrap_or_else(|| "Workspace".to_string())
    }

    pub(super) fn document_name(&self) -> String {
        self.current_path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .map(str::to_string)
            .unwrap_or_else(|| "Untitled.md".to_string())
    }
}

pub(super) fn determine_launch_context(initial_target: Option<&Path>) -> LaunchContext {
    match initial_target.and_then(|path| fs::metadata(path).ok()) {
        Some(metadata) if metadata.is_dir() => LaunchContext::Folder,
        Some(_) => LaunchContext::SingleFile,
        None if initial_target.is_some() => LaunchContext::SingleFile,
        None => LaunchContext::Scratch,
    }
}
