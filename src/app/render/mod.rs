mod panels;
mod settings;
mod sidebar;

use gpui::{
    div, AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Render, Styled,
    Window,
};
use gpui_component::{h_flex, v_flex, ActiveTheme};

use crate::actions;
use crate::app_title_bar::MdrsTitleBar;

use super::{AppPage, LaunchContext, MdrsApp, PaneMode};

impl Render for MdrsApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg = cx.theme().colors.background;
        let border = cx.theme().colors.border;
        let muted = cx.theme().colors.muted_foreground;
        let preview_stats = self.preview.read(cx).stats();
        let show_edit = self.can_edit();

        let mut body = h_flex().flex_1().w_full().min_h_0().overflow_hidden();
        let content: AnyElement = match self.current_page {
            AppPage::Settings => self.render_settings_page(cx).into_any_element(),
            AppPage::Workspace => {
                if self.has_sidebar() {
                    body = body.child(self.render_sidebar(cx));
                }

                match self.pane_mode {
                    PaneMode::Preview => h_flex()
                        .flex_1()
                        .w_full()
                        .h_full()
                        .min_h_0()
                        .overflow_hidden()
                        .child(self.render_preview_panel(cx))
                        .into_any_element(),
                    PaneMode::Edit => h_flex()
                        .flex_1()
                        .w_full()
                        .h_full()
                        .min_h_0()
                        .overflow_hidden()
                        .child(self.render_editor_panel(cx))
                        .into_any_element(),
                }
            }
        };
        body = body.child(content);

        // The min-height chain here keeps the bottom status bar visible in windowed mode.
        v_flex()
            .size_full()
            .min_h_0()
            .bg(bg)
            // ── macOS native top-menu action handlers ──────────────────────────
            .on_action(cx.listener(|this, _: &actions::OpenFile, window, cx| {
                this.prompt_open_file(window, cx);
            }))
            .on_action(cx.listener(|this, _: &actions::OpenFolder, window, cx| {
                this.prompt_open_folder(window, cx);
            }))
            .on_action(cx.listener(|this, _: &actions::SaveFile, window, cx| {
                this.save_document(window, cx);
            }))
            .on_action(cx.listener(|this, _: &actions::OpenSettings, _window, cx| {
                this.open_settings();
                cx.notify();
            }))
            .on_action(
                cx.listener(|this, _: &actions::ToggleSidebar, _window, cx| {
                    this.toggle_sidebar();
                    cx.notify();
                }),
            )
            // ──────────────────────────────────────────────────────────────────
            .child(div().w_full().flex_shrink_0().child(MdrsTitleBar {
                app: cx.entity(),
                pane_mode: self.pane_mode,
                current_page: self.current_page,
                show_edit,
                sidebar_toggleable: self.launch_context == LaunchContext::Folder,
            }))
            .child(body)
            .child(
                h_flex()
                    .w_full()
                    .flex_shrink_0()
                    .px_4()
                    .py_2()
                    .justify_between()
                    .border_t_1()
                    .border_color(border)
                    .text_size(gpui::px(12.0))
                    .text_color(muted)
                    .child(div().child(match self.launch_context {
                        LaunchContext::Scratch => "New document",
                        LaunchContext::SingleFile => "Single file",
                        LaunchContext::Folder => "Workspace preview",
                    }))
                    .child(div().child(self.status_label(preview_stats))),
            )
    }
}
