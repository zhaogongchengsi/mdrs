use gpui::{div, AnyElement, Context, IntoElement, ParentElement, Styled};
use gpui_component::{h_flex, input::Input, scroll::ScrollableElement, v_flex, ActiveTheme};

use crate::app::{LaunchContext, MdrsApp};

impl MdrsApp {
    fn render_preview_content(&self, cx: &mut Context<Self>) -> AnyElement {
        let muted = cx.theme().colors.muted_foreground;
        if self.is_loading {
            return div()
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .text_color(muted)
                .child("Loading preview...")
                .into_any_element();
        }

        if let Some(error) = &self.load_error {
            return div()
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .text_color(muted)
                .child(error.clone())
                .into_any_element();
        }

        if self.current_path.is_none() && self.launch_context == LaunchContext::Folder {
            return div()
                .flex()
                .flex_1()
                .items_center()
                .justify_center()
                .text_color(muted)
                .child("Select a Markdown file to preview it.")
                .into_any_element();
        }

        div()
            .flex_1()
            .min_h_0()
            .overflow_y_scrollbar()
            .child(self.preview.clone())
            .into_any_element()
    }

    pub(super) fn render_editor_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let border = cx.theme().colors.border;
        let fg = cx.theme().colors.foreground;
        let muted = cx.theme().colors.muted_foreground;

        div()
            .flex_1()
            .h_full()
            .min_h_0()
            .overflow_hidden()
            .border_r_1()
            .border_color(border)
            .child(
                v_flex()
                    .size_full()
                    .min_h_0()
                    .child(panel_header(
                        "Editor",
                        &self.document_name(),
                        fg,
                        muted,
                        border,
                    ))
                    .child(
                        div()
                            .flex_1()
                            .min_h_0()
                            .px_4()
                            .py_3()
                            .child(Input::new(&self.editor).h_full()),
                    ),
            )
    }

    pub(super) fn render_preview_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let border = cx.theme().colors.border;
        let fg = cx.theme().colors.foreground;
        let muted = cx.theme().colors.muted_foreground;

        div().flex_1().h_full().min_h_0().overflow_hidden().child(
            v_flex()
                .size_full()
                .min_h_0()
                .child(panel_header(
                    "Preview",
                    &self.document_name(),
                    fg,
                    muted,
                    border,
                ))
                .child(
                    div()
                        .flex_1()
                        .min_h_0()
                        .overflow_hidden()
                        .px_4()
                        .py_3()
                        .child(self.render_preview_content(cx)),
                ),
        )
    }
}

fn panel_header(
    title: &str,
    subtitle: &str,
    fg: gpui::Hsla,
    muted: gpui::Hsla,
    border: gpui::Hsla,
) -> gpui::Div {
    div().border_b_1().border_color(border).child(
        h_flex()
            .w_full()
            .justify_between()
            .items_center()
            .px_4()
            .py_3()
            .child(
                div()
                    .text_color(fg)
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(title.to_string()),
            )
            .child(
                div()
                    .text_color(muted)
                    .text_size(gpui::px(12.0))
                    .child(subtitle.to_string()),
            ),
    )
}
