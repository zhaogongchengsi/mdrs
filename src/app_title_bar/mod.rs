mod menus;

use gpui::{div, prelude::*, Entity, IntoElement, ParentElement, RenderOnce, Window};
use gpui_component::{h_flex, TitleBar};

use crate::app::{AppPage, MdrsApp, PaneMode};

#[derive(IntoElement)]
pub struct MdrsTitleBar {
    pub app: Entity<MdrsApp>,
    pub pane_mode: PaneMode,
    pub current_page: AppPage,
    pub show_edit: bool,
    pub sidebar_toggleable: bool,
}

impl RenderOnce for MdrsTitleBar {
    fn render(self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        let mut left = h_flex().h_full().items_center().gap_1();
        if cfg!(target_os = "windows") {
            left = left.child(menus::render_windows_controls(
                self.app.clone(),
                self.sidebar_toggleable,
            ));
        } else if cfg!(target_os = "macos") {
            left = left.child(menus::render_macos_controls(
                self.app.clone(),
                self.sidebar_toggleable,
            ));
        }

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
                .h_full()
                .justify_between()
                .items_center()
                .px_1p5()
                .pr_1p5()
                .child(div().h_full().child(left))
                .child(right),
        )
    }
}
