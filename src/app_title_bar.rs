use gpui::{div, prelude::*, Entity, IntoElement, ParentElement, RenderOnce, Window};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex, ActiveTheme, Selectable, TitleBar,
};

use crate::app::{AppPage, MdrsApp, PaneMode};

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

        let mut left = h_flex().items_center().gap_2();
        if cfg!(target_os = "windows") {
            left = left.child(Button::new("titlebar-menu").label("M").ghost().on_click({
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
            }));
        }

        left = left.child(
            div()
                .text_color(fg)
                .font_weight(gpui::FontWeight::MEDIUM)
                .child("mdrs"),
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
