#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod actions;
mod app;
mod app_icon;
mod app_title_bar;
mod assets;
mod editor;
mod file_loader;
mod preview;
mod style_config;
mod workspace;

use std::path::PathBuf;

use gpui::{
    prelude::*, px, size, Application, Bounds, Menu, MenuItem, OsAction, SystemMenuType,
    TitlebarOptions, WindowBounds, WindowOptions,
};
use gpui_component::{Root, TitleBar};

fn main() {
    let initial_file = std::env::args_os().nth(1).map(PathBuf::from);

    Application::new()
        .with_assets(assets::AppAssets::new())
        .run(move |cx| {
            gpui_component::init(cx);
            actions::register(cx);

            #[cfg(target_os = "macos")]
            cx.set_menus(vec![
                // Application menu (macOS puts the app name here automatically)
                Menu {
                    name: "MDRS".into(),
                    items: vec![
                        MenuItem::os_submenu("Services", SystemMenuType::Services),
                        MenuItem::separator(),
                        MenuItem::action("Quit mdrs", actions::Quit),
                    ],
                },
                // File menu
                Menu {
                    name: "File".into(),
                    items: vec![
                        MenuItem::action("Open File…", actions::OpenFile),
                        MenuItem::action("Open Folder…", actions::OpenFolder),
                        MenuItem::separator(),
                        MenuItem::action("Save", actions::SaveFile),
                        MenuItem::separator(),
                        MenuItem::action("Settings", actions::OpenSettings),
                    ],
                },
                // Edit menu — Cut/Copy/Paste/SelectAll use OsAction so macOS
                // routes them through the responder chain to the focused text view.
                Menu {
                    name: "Edit".into(),
                    items: vec![
                        MenuItem::os_action("Undo", actions::NoOp, OsAction::Undo),
                        MenuItem::os_action("Redo", actions::NoOp, OsAction::Redo),
                        MenuItem::separator(),
                        MenuItem::os_action("Cut", actions::NoOp, OsAction::Cut),
                        MenuItem::os_action("Copy", actions::NoOp, OsAction::Copy),
                        MenuItem::os_action("Paste", actions::NoOp, OsAction::Paste),
                        MenuItem::separator(),
                        MenuItem::os_action("Select All", actions::NoOp, OsAction::SelectAll),
                    ],
                },
                // View menu
                Menu {
                    name: "View".into(),
                    items: vec![MenuItem::action("Toggle Sidebar", actions::ToggleSidebar)],
                },
            ]);

            let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);
            let titlebar = if cfg!(target_os = "windows") || cfg!(target_os = "macos") {
                Some(TitleBar::title_bar_options())
            } else {
                Some(TitlebarOptions {
                    title: Some("mdrs - Markdown Editor".into()),
                    ..Default::default()
                })
            };

            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar,
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
