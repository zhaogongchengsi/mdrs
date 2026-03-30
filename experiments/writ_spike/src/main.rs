#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use gpui::{
    div, prelude::*, px, size, Application, Bounds, Context, Entity, IntoElement, ParentElement,
    Render, Styled, TitlebarOptions, Window, WindowBounds, WindowOptions,
};
use writ::Editor;

struct WritSpikeApp {
    editor: Entity<Editor>,
}

impl WritSpikeApp {
    fn new(initial_text: String, cx: &mut Context<Self>) -> Self {
        let editor = cx.new(|cx| Editor::new(&initial_text, cx));
        Self { editor }
    }
}

impl Render for WritSpikeApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.editor.clone())
    }
}

fn main() {
    let initial_text = std::env::args_os()
        .nth(1)
        .and_then(|path| std::fs::read_to_string(path).ok())
        .unwrap_or_else(|| "# Writ Spike\n\n直接编辑标题试试看".to_string());

    Application::new().run(move |cx| {
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
            |_, cx| cx.new(|cx| WritSpikeApp::new(initial_text.clone(), cx)),
        )
        .unwrap();
        cx.activate(true);
    });
}
