use std::path::PathBuf;

use gpui::Entity;
use gpui_component::input::InputState;

use crate::{file_loader::ReadStrategy, preview::MarkdownPreview, workspace::WorkspaceFile};

mod logic;
mod render;

#[derive(Clone, Copy, PartialEq, Eq)]
enum LaunchContext {
    Scratch,
    SingleFile,
    Folder,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PaneMode {
    Preview,
    Edit,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppPage {
    Workspace,
    Settings,
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
    current_page: AppPage,
    sidebar_open: bool,
    workspace_root: Option<PathBuf>,
    workspace_files: Vec<WorkspaceFile>,
    current_path: Option<PathBuf>,
    document_bytes: u64,
    read_strategy: Option<ReadStrategy>,
    is_loading: bool,
    is_dirty: bool,
    load_error: Option<String>,
    // Transient footer feedback for actions like save/open.
    status_message: Option<String>,
    // Programmatic buffer updates should not mark the document as dirty.
    suppress_dirty_tracking: bool,
}
