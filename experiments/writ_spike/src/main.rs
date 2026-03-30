#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    ops::Range,
    path::{Path, PathBuf},
};

use gpui::{
    canvas, div, prelude::*, px, rems, size, Application, Bounds, Context, ElementInputHandler,
    Entity, EntityInputHandler, FocusHandle, Focusable, IntoElement, ParentElement, Pixels,
    ReadGlobal, Render, Styled, TitlebarOptions, UTF16Selection, Window, WindowBounds,
    WindowOptions,
};
use writ::{
    config::Config,
    line::{CursorScreenPosition, HoveredRefScreenPosition},
    status_bar::StatusBarInfo,
    title_bar::FileInfo,
    Editor, EditorConfig, EditorTheme,
};

struct WritSpikeApp {
    editor: Entity<Editor>,
    focus_handle: FocusHandle,
    marked_range: Option<Range<usize>>,
    ime_selected_range: Option<Range<usize>>,
}

impl WritSpikeApp {
    fn new(editor: Entity<Editor>, focus_handle: FocusHandle, cx: &mut Context<Self>) -> Self {
        cx.observe_global::<FileInfo>(|_, cx| {
            cx.notify();
        })
        .detach();

        cx.observe_global::<StatusBarInfo>(|_, cx| {
            cx.notify();
        })
        .detach();

        Self {
            editor,
            focus_handle,
            marked_range: None,
            ime_selected_range: None,
        }
    }

    fn current_text(&self, cx: &Context<Self>) -> String {
        self.editor.read(cx).text()
    }

    fn selected_range(&self, cx: &Context<Self>) -> Range<usize> {
        if let Some(range) = self.ime_selected_range.clone() {
            return range;
        }

        let editor = self.editor.read(cx);
        editor
            .selection_range()
            .unwrap_or_else(|| editor.cursor_position()..editor.cursor_position())
    }

    fn offset_from_utf16(text: &str, offset_utf16: usize) -> usize {
        let mut utf16_count = 0;
        for (byte_offset, ch) in text.char_indices() {
            if utf16_count >= offset_utf16 {
                return byte_offset;
            }
            utf16_count += ch.len_utf16();
        }
        text.len()
    }

    fn offset_to_utf16(text: &str, offset: usize) -> usize {
        let mut utf16_offset = 0;
        for (byte_offset, ch) in text.char_indices() {
            if byte_offset >= offset {
                break;
            }
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }

    fn range_from_utf16(text: &str, range_utf16: &Range<usize>) -> Range<usize> {
        Self::offset_from_utf16(text, range_utf16.start)
            ..Self::offset_from_utf16(text, range_utf16.end)
    }

    fn range_to_utf16(text: &str, range: &Range<usize>) -> Range<usize> {
        Self::offset_to_utf16(text, range.start)..Self::offset_to_utf16(text, range.end)
    }

    fn replace_editor_text(
        &mut self,
        range: Range<usize>,
        new_text: &str,
        cursor: usize,
        marked_range: Option<Range<usize>>,
        ime_selected_range: Option<Range<usize>>,
        cx: &mut Context<Self>,
    ) {
        let current_text = self.current_text(cx);
        let next_text = format!(
            "{}{}{}",
            &current_text[..range.start],
            new_text,
            &current_text[range.end..]
        );

        self.editor.update(cx, |editor, cx| {
            editor.set_text(&next_text, cx);
            editor.set_cursor(cursor.min(next_text.len()), cx);
        });

        self.marked_range = marked_range;
        self.ime_selected_range = ime_selected_range;
    }
}

impl Render for WritSpikeApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app = cx.entity();
        let focus_handle = self.focus_handle.clone();

        div()
            .relative()
            .size_full()
            .child(self.editor.clone())
            .child(
                canvas(
                    move |bounds, _, _| bounds,
                    move |bounds, _, window, cx| {
                        window.handle_input(
                            &focus_handle,
                            ElementInputHandler::new(bounds, app.clone()),
                            cx,
                        );
                    },
                )
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .bottom_0(),
            )
    }
}

fn spike_editor_config(file_path: &Path) -> EditorConfig {
    EditorConfig {
        theme: EditorTheme::dracula(),
        base_path: file_path.parent().map(|path| path.to_path_buf()),
        padding_x: rems(2.0),
        padding_y: rems(1.6),
        line_height: rems(1.6),
        ..EditorConfig::default()
    }
}

impl Focusable for WritSpikeApp {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EntityInputHandler for WritSpikeApp {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<String> {
        let text = self.current_text(cx);
        let range = Self::range_from_utf16(&text, &range_utf16);
        actual_range.replace(Self::range_to_utf16(&text, &range));
        Some(text[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let text = self.current_text(cx);
        Some(UTF16Selection {
            range: Self::range_to_utf16(&text, &self.selected_range(cx)),
            reversed: false,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| Self::range_to_utf16(&self.current_text(cx), range))
    }

    fn unmark_text(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.marked_range = None;
        self.ime_selected_range = None;
        cx.notify();
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let should_handle =
            self.marked_range.is_some() || range_utf16.is_some() || !new_text.is_ascii();
        if !should_handle {
            return;
        }

        let current_text = self.current_text(cx);
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| Self::range_from_utf16(&current_text, range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range(cx));

        let cursor = range.start + new_text.len();
        self.replace_editor_text(range, new_text, cursor, None, None, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let current_text = self.current_text(cx);
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| Self::range_from_utf16(&current_text, range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range(cx));

        let selected_range = new_selected_range_utf16
            .as_ref()
            .map(|selection| Self::range_from_utf16(new_text, selection))
            .map(|selection| range.start + selection.start..range.start + selection.end)
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());

        let marked_range = if new_text.is_empty() {
            None
        } else {
            Some(range.start..range.start + new_text.len())
        };

        self.replace_editor_text(
            range,
            new_text,
            selected_range.end,
            marked_range,
            Some(selected_range),
            cx,
        );
    }

    fn bounds_for_range(
        &mut self,
        _range_utf16: Range<usize>,
        element_bounds: Bounds<Pixels>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let cursor_screen_pos = CursorScreenPosition::global(cx);
        cursor_screen_pos
            .position
            .map_or(Some(element_bounds), |position| {
                let size = size(px(1.0), px(20.0));
                Some(Bounds::new(position, size))
            })
    }

    fn character_index_for_point(
        &mut self,
        _point: gpui::Point<Pixels>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<usize> {
        let text = self.current_text(cx);
        Some(Self::offset_to_utf16(&text, self.selected_range(cx).end))
    }
}

fn main() {
    let file_path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("writ-spike.md"));

    let initial_text = std::fs::read_to_string(&file_path)
        .unwrap_or_else(|_| "# Writ Spike\n\n直接编辑标题试试看".to_string());

    let editor_config = spike_editor_config(&file_path);

    let config = Config {
        file: Some(file_path.clone()),
        demo: false,
        text_font: editor_config.text_font.clone(),
        code_font: editor_config.code_font.clone(),
        autosave: false,
        github_token: None,
        github_repo: None,
    };

    Application::new().run(move |cx| {
        cx.set_global(FileInfo {
            path: file_path.clone(),
            dirty: false,
        });
        cx.set_global(StatusBarInfo::default());
        cx.set_global(editor_config.theme.clone());
        cx.set_global(CursorScreenPosition::default());
        cx.set_global(HoveredRefScreenPosition::default());
        cx.set_global(config.clone());

        let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("mdrs writ spike".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                let editor = cx.new(|cx| {
                    Editor::with_config(&initial_text, spike_editor_config(&file_path), cx)
                });
                let focus_handle = editor.focus_handle(cx);
                focus_handle.focus(window);

                cx.new(|cx| WritSpikeApp::new(editor, focus_handle, cx))
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
