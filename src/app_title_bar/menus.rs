use gpui::{Corner, Entity, IntoElement, ParentElement, Styled};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    menu::{DropdownMenu, PopupMenu, PopupMenuItem},
    Selectable, Sizable,
};

use crate::{
    app::{MdrsApp, PaneMode},
    app_icon::AppIcon,
};

pub(super) fn render_windows_controls(
    app: Entity<MdrsApp>,
    sidebar_toggleable: bool,
) -> impl IntoElement {
    h_flex()
        .items_center()
        .gap_1()
        .child(
            Button::new("titlebar-menu")
                .icon(AppIcon::Menu)
                .xsmall()
                .compact()
                .ghost()
                .on_click({
                    let app = app.clone();
                    move |_, _, cx| {
                        if !sidebar_toggleable {
                            return;
                        }
                        app.update(cx, |app, cx| {
                            app.toggle_sidebar();
                            cx.notify();
                        });
                    }
                }),
        )
        .child(render_file_menu(app.clone()))
        .child(render_edit_menu())
}

pub(super) fn render_macos_controls(
    app: Entity<MdrsApp>,
    sidebar_toggleable: bool,
) -> impl IntoElement {
    h_flex()
        .items_center()
        .gap_1()
        .child(
            Button::new("titlebar-menu")
                .icon(AppIcon::Menu)
                .xsmall()
                .compact()
                .ghost()
                .on_click({
                    let app = app.clone();
                    move |_, _, cx| {
                        if !sidebar_toggleable {
                            return;
                        }
                        app.update(cx, |app, cx| {
                            app.toggle_sidebar();
                            cx.notify();
                        });
                    }
                }),
        )
        .child(render_file_menu(app.clone()))
        .child(render_edit_menu())
}

pub(super) fn render_workspace_actions(
    app: Entity<MdrsApp>,
    pane_mode: PaneMode,
    show_edit: bool,
) -> impl IntoElement {
    let mut right = h_flex().items_center().gap_1();
    right = right.child(
        Button::new("titlebar-preview")
            .icon(AppIcon::Preview)
            .label("Preview")
            .xsmall()
            .compact()
            .selected(pane_mode == PaneMode::Preview)
            .ghost()
            .on_click({
                let app = app.clone();
                move |_, _, cx| {
                    app.update(cx, |app, cx| {
                        app.set_pane_mode(PaneMode::Preview);
                        cx.notify();
                    });
                }
            }),
    );

    if show_edit {
        right = right.child(
            Button::new("titlebar-edit")
                .icon(AppIcon::Edit)
                .label("Edit")
                .xsmall()
                .compact()
                .selected(pane_mode == PaneMode::Edit)
                .ghost()
                .on_click(move |_, _, cx| {
                    app.update(cx, |app, cx| {
                        app.set_pane_mode(PaneMode::Edit);
                        cx.notify();
                    });
                }),
        );
    }

    right
}

pub(super) fn render_settings_back(app: Entity<MdrsApp>) -> impl IntoElement {
    Button::new("titlebar-back")
        .icon(AppIcon::Back)
        .label("Back")
        .xsmall()
        .compact()
        .ghost()
        .on_click(move |_, _, cx| {
            app.update(cx, |app, cx| {
                app.open_workspace_page();
                cx.notify();
            });
        })
}

fn render_file_menu(app: Entity<MdrsApp>) -> impl IntoElement {
    Button::new("titlebar-file-menu")
        .label("File")
        .xsmall()
        .compact()
        .ghost()
        .dropdown_menu_with_anchor(Corner::BottomLeft, move |menu: PopupMenu, _window, _cx| {
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
            .item(PopupMenuItem::new("Save File").on_click({
                let app = app.clone();
                move |_, window, cx| {
                    app.update(cx, |app, cx| {
                        app.save_document(window, cx);
                    });
                }
            }))
            .separator()
            .item(
                PopupMenuItem::new("Settings")
                    .icon(AppIcon::Settings)
                    .on_click({
                        let app = app.clone();
                        move |_, _, cx| {
                            app.update(cx, |app, cx| {
                                app.open_settings();
                                cx.notify();
                            });
                        }
                    }),
            )
        })
}

fn render_edit_menu() -> impl IntoElement {
    Button::new("titlebar-edit-menu")
        .label("Edit")
        .xsmall()
        .compact()
        .ghost()
        .dropdown_menu_with_anchor(Corner::BottomLeft, |menu: PopupMenu, _window, _cx| {
            menu.item(PopupMenuItem::new("Undo").disabled(true))
                .item(PopupMenuItem::new("Redo").disabled(true))
                .separator()
                .item(PopupMenuItem::new("Cut").disabled(true))
                .item(PopupMenuItem::new("Copy").disabled(true))
                .item(PopupMenuItem::new("Paste").disabled(true))
        })
}
