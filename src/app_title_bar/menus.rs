use std::rc::Rc;

use gpui::{
    anchored, deferred, div, px, App, ClickEvent, Context, Corner, DismissEvent, Entity,
    Focusable, InteractiveElement, IntoElement, MouseButton, MouseDownEvent, ParentElement,
    RenderOnce, SharedString, Styled, Window,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex, input,
    menu::{PopupMenu, PopupMenuItem},
    Selectable, Sizable,
};

use crate::{
    actions,
    app::{MdrsApp, PaneMode},
    app_icon::AppIcon,
};

pub(super) fn render_windows_controls(
    app: Entity<MdrsApp>,
    sidebar_toggleable: bool,
) -> impl IntoElement {
    render_platform_menus(app, sidebar_toggleable)
}

pub(super) fn render_macos_controls(
    app: Entity<MdrsApp>,
    sidebar_toggleable: bool,
) -> impl IntoElement {
    render_platform_menus(app, sidebar_toggleable)
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
    TitlebarDropdown::new(
        "titlebar-file-menu",
        titlebar_menu_button("titlebar-file-menu", "File"),
        move |menu: PopupMenu, window, cx| {
            let action_context = popup_action_context(&app, window, cx);

            menu.action_context(action_context)
                .item(
                    PopupMenuItem::new("Open File…")
                        .icon(AppIcon::File)
                        .action(Box::new(actions::OpenFile)),
                )
                .item(
                    PopupMenuItem::new("Open Folder…")
                        .icon(AppIcon::Folder)
                        .action(Box::new(actions::OpenFolder)),
                )
                .separator()
                .item(PopupMenuItem::new("Save").action(Box::new(actions::SaveFile)))
                .separator()
                .item(
                    PopupMenuItem::new("Settings")
                        .icon(AppIcon::Settings)
                        .action(Box::new(actions::OpenSettings)),
                )
        },
    )
}

fn render_edit_menu(app: Entity<MdrsApp>) -> impl IntoElement {
    TitlebarDropdown::new(
        "titlebar-edit-menu",
        titlebar_menu_button("titlebar-edit-menu", "Edit"),
        move |menu: PopupMenu, window, cx| {
            let action_context = popup_action_context(&app, window, cx);

            menu.action_context(action_context)
                .item(PopupMenuItem::new("Undo").action(Box::new(input::Undo)))
                .item(PopupMenuItem::new("Redo").action(Box::new(input::Redo)))
                .separator()
                .item(PopupMenuItem::new("Cut").action(Box::new(input::Cut)))
                .item(PopupMenuItem::new("Copy").action(Box::new(input::Copy)))
                .item(PopupMenuItem::new("Paste").action(Box::new(input::Paste)))
                .separator()
                .item(PopupMenuItem::new("Select All").action(Box::new(input::SelectAll)))
        },
    )
}

fn render_view_menu(app: Entity<MdrsApp>, sidebar_toggleable: bool) -> impl IntoElement {
    TitlebarDropdown::new(
        "titlebar-view-menu",
        titlebar_menu_button("titlebar-view-menu", "View"),
        move |menu: PopupMenu, window, cx| {
            let action_context = popup_action_context(&app, window, cx);
            let sidebar_visible = app.read(cx).sidebar_visible();

            menu.action_context(action_context).item(
                PopupMenuItem::new("Toggle Sidebar")
                    .checked(sidebar_visible)
                    .disabled(!sidebar_toggleable)
                    .action(Box::new(actions::ToggleSidebar)),
            )
        },
    )
}

fn render_platform_menus(app: Entity<MdrsApp>, sidebar_toggleable: bool) -> impl IntoElement {
    h_flex()
        .h_full()
        .items_center()
        .gap_1()
        .child(titlebar_icon_button("titlebar-menu").on_click({
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
        }))
        .child(render_file_menu(app.clone()))
        .child(render_edit_menu(app.clone()))
        .child(render_view_menu(app, sidebar_toggleable))
}

fn titlebar_icon_button(id: &'static str) -> Button {
    Button::new(id)
        .icon(AppIcon::Menu)
        .xsmall()
        .compact()
        .ghost()
        .h_full()
}

fn titlebar_menu_button(id: &'static str, label: &'static str) -> Button {
    Button::new(id)
        .label(label)
        .xsmall()
        .compact()
        .ghost()
        .h_full()
}

fn popup_action_context(
    app: &Entity<MdrsApp>,
    window: &Window,
    cx: &gpui::App,
) -> gpui::FocusHandle {
    window
        .focused(cx)
        .unwrap_or_else(|| app.read(cx).menu_action_context(cx))
}

#[derive(Default)]
struct TitlebarDropdownState {
    menu: Option<Entity<PopupMenu>>,
}

#[derive(IntoElement)]
struct TitlebarDropdown {
    id: SharedString,
    trigger: Button,
    builder: Rc<dyn Fn(PopupMenu, &mut Window, &mut Context<PopupMenu>) -> PopupMenu>,
}

impl TitlebarDropdown {
    fn new(
        id: impl Into<SharedString>,
        trigger: Button,
        builder: impl Fn(PopupMenu, &mut Window, &mut Context<PopupMenu>) -> PopupMenu + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            trigger,
            builder: Rc::new(builder),
        }
    }
}

impl RenderOnce for TitlebarDropdown {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let menu_state =
            window.use_keyed_state(self.id.clone(), cx, |_, _| TitlebarDropdownState::default());
        let is_open = menu_state.read(cx).menu.is_some();
        let builder = self.builder.clone();
        let trigger = self
            .trigger
            .selected(is_open)
            .on_mouse_down(
                MouseButton::Left,
                |_: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                    // Stop propagation so the title bar does not start dragging.
                    window.prevent_default();
                    cx.stop_propagation();
                },
            )
            .on_click({
                let menu_state = menu_state.clone();
                move |_: &ClickEvent, window: &mut Window, cx: &mut App| {
                    if menu_state.read(cx).menu.is_some() {
                        menu_state.update(cx, |state, _| {
                            state.menu = None;
                        });
                        return;
                    }

                    let builder = builder.clone();
                    let menu =
                        PopupMenu::build(window, cx, move |menu, window, cx| builder(menu, window, cx));
                    menu.read(cx).focus_handle(cx).focus(window);

                    window
                        .subscribe(&menu, cx, {
                            let menu_state = menu_state.clone();
                            move |_, _: &DismissEvent, _: &mut Window, cx: &mut App| {
                                menu_state.update(cx, |state, _| {
                                    state.menu = None;
                                });
                            }
                        })
                        .detach();

                    menu_state.update(cx, |state, _| {
                        state.menu = Some(menu);
                    });
                }
            });

        let mut root = div().relative().child(trigger);
        if let Some(menu) = menu_state.read(cx).menu.clone() {
            root = root.child(deferred(
                anchored()
                    .anchor(Corner::TopLeft)
                    .snap_to_window_with_margin(px(8.))
                    .child(div().size_full().occlude().child(menu)),
            ));
        }

        root
    }
}
