#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod editor;
mod preview;

use gpui::{Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_component::Root;

fn main() {
    Application::new().run(|cx| {
        gpui_component::init(cx);

        let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);
        
        // 根据平台调整窗口配置
        let titlebar_options = if cfg!(target_os = "macos") {
            // macOS 使用系统标准标题栏
            Some(TitlebarOptions {
                title: Some("mdrs — Markdown Editor".into()),
                ..Default::default()
            })
        } else {
            // Windows 和其他平台
            Some(TitlebarOptions {
                title: Some("mdrs — Markdown Editor".into()),
                ..Default::default()
            })
        };
        
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: titlebar_options,
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| app::MdrsApp::new(window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
