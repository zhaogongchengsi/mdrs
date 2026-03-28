use std::path::PathBuf;

use gpui::{div, img, prelude::*, Corner, Entity, IntoElement, ParentElement, RenderOnce, Window};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    menu::{DropdownMenu, PopupMenu, PopupMenuItem},
    ActiveTheme, Selectable, TitleBar,
};

use crate::{
    app::{AppPage, MdrsApp, PaneMode},
    app_icon::AppIcon,
};

#[derive(IntoElement)]
pub struct MdrsTitleBar {
    pub app: Entity<MdrsApp>,
    pub pane_mode: PaneMode,
    pub current_page: AppPage,
    pub show_edit: bool,
    pub sidebar_toggleable: bool,
    pub source_label: String,
    pub page_title: &'static str,
}

impl RenderOnce for MdrsTitleBar {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let muted = cx.theme().colors.muted_foreground;
        let fg = cx.theme().colors.foreground;
        let logo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/assets/logo.png");

        let mut left = h_flex().items_center().gap_2();
        if cfg!(target_os = "windows") {
            left = left.child(
                Button::new("titlebar-menu")
                    .icon(AppIcon::Menu)
                    .ghost()
                    .on_click({
                        let app = self.app.clone();
                        let can_toggle = self.sidebar_toggleable;
                        move |_, _, cx| {
                            if !can_toggle {
                                return;
                            }
                            app.update(cx, |app, cx| {
                                app.toggle_sidebar();
                                cx.notify();
                            });
                        }
                    }),
            );
            left = left.child(
                Button::new("titlebar-file-menu")
                    .label("File")
                    .ghost()
                    .dropdown_menu_with_anchor(Corner::BottomLeft, {
                        let app = self.app.clone();
                        move |menu: PopupMenu, _window, _cx| {
                            menu.item(
                                PopupMenuItem::new("Open File")
                                    .icon(AppIcon::File)
                                    .on_click({
                                        let app = app.clone();
                                        move |_, window, cx| {
                                            app.update(cx, |app, cx| {
                                                app.prompt_open_file(window, cx);
                                            });
                                        }
                                    }),
                            )
                            .item(
                                PopupMenuItem::new("Open Folder")
                                    .icon(AppIcon::Folder)
                                    .on_click({
                                        let app = app.clone();
                                        move |_, window, cx| {
                                            app.update(cx, |app, cx| {
                                                app.prompt_open_folder(window, cx);
                                            });
                                        }
                                    }),
                            )
                            .separator()
                            .item(
                                PopupMenuItem::new("Save File").on_click({
                                    let app = app.clone();
                                    move |_, window, cx| {
                                        app.update(cx, |app, cx| {
                                            app.save_document(window, cx);
                                        });
                                    }
                                }),
                            )
                        }
                    }),
            );
            left = left.child(
                Button::new("titlebar-edit-menu")
                    .label("Edit")
                    .ghost()
                    .dropdown_menu_with_anchor(
                        Corner::BottomLeft,
                        |menu: PopupMenu, _window, _cx| {
                            menu.item(PopupMenuItem::new("Undo").disabled(true))
                                .item(PopupMenuItem::new("Redo").disabled(true))
                                .separator()
                                .item(PopupMenuItem::new("Cut").disabled(true))
                                .item(PopupMenuItem::new("Copy").disabled(true))
                                .item(PopupMenuItem::new("Paste").disabled(true))
                        },
                    ),
            );
        }

        left = left.child(
            h_flex()
                .items_center()
                .gap_2()
                .child(img(logo_path).w(gpui::px(18.0)).h(gpui::px(18.0)))
                .child(
                    div()
                        .text_color(fg)
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .child("mdrs"),
                ),
        );
        left = left.child(
            div()
                .text_color(muted)
                .text_size(gpui::px(12.0))
                .child(self.source_label),
        );

        let mut right = h_flex().items_center().gap_1();
        match self.current_page {
            AppPage::Workspace => {
                right = right.child(
                    Button::new("titlebar-settings")
                        .icon(AppIcon::Settings)
                        .label("Settings")
                        .ghost()
                        .on_click({
                            let app = self.app.clone();
                            move |_, _, cx| {
                                app.update(cx, |app, cx| {
                                    app.open_settings();
                                    cx.notify();
                                });
                            }
                        }),
                );
                right = right.child(
                    Button::new("titlebar-preview")
                        .icon(AppIcon::Preview)
                        .label("Preview")
                        .selected(self.pane_mode == PaneMode::Preview)
                        .ghost()
                        .on_click({
                            let app = self.app.clone();
                            move |_, _, cx| {
                                app.update(cx, |app, cx| {
                                    app.set_pane_mode(PaneMode::Preview);
                                    cx.notify();
                                });
                            }
                        }),
                );
                if self.show_edit {
                    right = right.child(
                        Button::new("titlebar-edit")
                            .icon(AppIcon::Edit)
                            .label("Edit")
                            .selected(self.pane_mode == PaneMode::Edit)
                            .ghost()
                            .on_click({
                                let app = self.app.clone();
                                move |_, _, cx| {
                                    app.update(cx, |app, cx| {
                                        app.set_pane_mode(PaneMode::Edit);
                                        cx.notify();
                                    });
                                }
                            }),
                    );
                }
            }
            AppPage::Settings => {
                right = right.child(
                    Button::new("titlebar-back")
                        .icon(AppIcon::Back)
                        .label("Back")
                        .ghost()
                        .on_click({
                            let app = self.app.clone();
                            move |_, _, cx| {
                                app.update(cx, |app, cx| {
                                    app.open_workspace_page();
                                    cx.notify();
                                });
                            }
                        }),
                );
            }
        }

        TitleBar::new().child(
            h_flex()
                .w_full()
                .justify_between()
                .items_center()
                .pr_2()
                .child(
                    h_flex().items_center().gap_3().child(left).child(
                        div()
                            .text_color(muted)
                            .text_size(gpui::px(11.0))
                            .child(self.page_title),
                    ),
                )
                .child(right),
        )
    }
}
