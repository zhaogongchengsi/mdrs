use gpui::{div, px, AnyElement, Context, IntoElement, ParentElement, Styled};
use gpui_component::{input::Input, scroll::ScrollableElement, v_flex, ActiveTheme};

use crate::app::MdrsApp;

impl MdrsApp {
    fn render_preview_content(&self, cx: &mut Context<Self>) -> AnyElement {
        let fg = cx.theme().colors.foreground;
        let muted = cx.theme().colors.muted_foreground;
        let border = cx.theme().colors.border;
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

        let is_editor_empty = self.editor.read(cx).value().trim().is_empty();
        if self.current_path.is_none() && is_editor_empty {
            return render_preview_empty_state(fg, muted, border).into_any_element();
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

        div()
            .flex_1()
            .h_full()
            .min_h_0()
            .overflow_hidden()
            .border_r_1()
            .border_color(border)
            .child(
                v_flex().size_full().min_h_0().child(
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
        div().flex_1().h_full().min_h_0().overflow_hidden().child(
            v_flex().size_full().min_h_0().child(
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

fn render_preview_empty_state(fg: gpui::Hsla, muted: gpui::Hsla, border: gpui::Hsla) -> gpui::Div {
    v_flex()
        .flex_1()
        .h_full()
        .items_center()
        .justify_center()
        .child(
            v_flex()
                .w(gpui::px(440.0))
                .max_w_full()
                .px_6()
                .py_8()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .text_color(fg)
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_size(gpui::px(22.0))
                        .child("New Markdown Document"),
                )
                .child(
                    div()
                        .w(gpui::px(72.0))
                        .h(px(1.0))
                        .bg(border),
                )
                .child(
                    div()
                        .text_color(muted)
                        .text_size(gpui::px(13.0))
                        .text_center()
                        .line_height(gpui::relative(1.6))
                        .child("Select a file from the workspace, or start writing in the editor to begin."),
                ),
        )
}
