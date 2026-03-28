mod menus;

use std::path::PathBuf;

use gpui::{div, img, prelude::*, Entity, IntoElement, ParentElement, RenderOnce, Window};
use gpui_component::{h_flex, ActiveTheme, TitleBar};

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
        let logo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/assets/logo.png");

        let mut left = h_flex().items_center().gap_2();
        if cfg!(target_os = "windows") {
            left = left.child(menus::render_windows_controls(
                self.app.clone(),
                self.sidebar_toggleable,
            ));
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

        let right = match self.current_page {
            AppPage::Workspace => {
                menus::render_workspace_actions(self.app.clone(), self.pane_mode, self.show_edit)
                    .into_any_element()
            }
            AppPage::Settings => menus::render_settings_back(self.app.clone()).into_any_element(),
        };

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
