use gpui::{div, Context, IntoElement, ParentElement, Styled};
use gpui_component::{scroll::ScrollableElement, v_flex, ActiveTheme};

use crate::app::MdrsApp;

impl MdrsApp {
    pub(super) fn render_settings_page(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let border = cx.theme().colors.border;
        let fg = cx.theme().colors.foreground;
        let muted = cx.theme().colors.muted_foreground;

        v_flex()
            .flex_1()
            .min_h_0()
            .overflow_y_scrollbar()
            .child(
                v_flex()
                    .w_full()
                    .max_w(gpui::px(880.0))
                    .mx_auto()
                    .px_6()
                    .py_6()
                    .gap_4()
                    .child(
                        div()
                            .text_color(fg)
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_size(gpui::px(24.0))
                            .child("Settings"),
                    )
                    .child(settings_section(
                        "Window Chrome",
                        "Windows uses a custom VS Code-style title bar. macOS keeps the native traffic light placement.",
                        border,
                        fg,
                        muted,
                    ))
                    .child(settings_section(
                        "Editor Behavior",
                        "Scratch launches open directly in edit mode. Single files and workspace files open in preview mode first.",
                        border,
                        fg,
                        muted,
                    ))
                    .child(settings_section(
                        "Preview Performance",
                        "Large files use buffered loading, and preview rendering is capped for responsiveness.",
                        border,
                        fg,
                        muted,
                    )),
            )
    }
}

fn settings_section(
    title: &str,
    description: &str,
    border: gpui::Hsla,
    fg: gpui::Hsla,
    muted: gpui::Hsla,
) -> gpui::Div {
    div().w_full().border_1().border_color(border).child(
        v_flex()
            .w_full()
            .px_4()
            .py_4()
            .gap_2()
            .child(
                div()
                    .text_color(fg)
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(title.to_string()),
            )
            .child(
                div()
                    .text_color(muted)
                    .text_size(gpui::px(13.0))
                    .line_height(gpui::relative(1.6))
                    .child(description.to_string()),
            ),
    )
}
