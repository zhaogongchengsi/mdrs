mod app;
mod editor;
mod file_loader;
mod preview;

use std::path::PathBuf;

use gpui::{
    prelude::*, px, size, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions,
};
use gpui_component::Root;

fn main() {
    let initial_file = std::env::args_os().nth(1).map(PathBuf::from);

    Application::new().run(move |cx| {
        gpui_component::init(cx);

        let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("mdrs — Markdown Editor".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                let initial_file = initial_file.clone();
                let view = cx.new(|cx| app::MdrsApp::new(window, initial_file, cx));
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
