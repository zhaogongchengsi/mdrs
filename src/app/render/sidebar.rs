use gpui::{div, Context, IntoElement, ParentElement, Styled};
use gpui_component::{
    button::{Button, ButtonVariants},
    scroll::ScrollableElement,
    v_flex, ActiveTheme, Selectable,
};

use crate::{app::MdrsApp, app_icon::AppIcon};

impl MdrsApp {
    pub(super) fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let border = cx.theme().colors.border;
        let fg = cx.theme().colors.foreground;
        let muted = cx.theme().colors.muted_foreground;
        let entity = cx.entity();
        let current_path = self.current_path.clone();

        let mut file_list = v_flex().w_full().gap_1();
        if self.workspace_files.is_empty() {
            file_list = file_list.child(
                div()
                    .px_3()
                    .py_2()
                    .text_color(muted)
                    .text_size(gpui::px(12.0))
                    .child("No Markdown files found"),
            );
        } else {
            for (index, file) in self.workspace_files.iter().enumerate() {
                let file_path = file.path.clone();
                let is_selected = current_path.as_ref() == Some(&file.path);
                file_list = file_list.child(
                    Button::new(("workspace-file", index))
                        .icon(AppIcon::File)
                        .label(file.label())
                        .selected(is_selected)
                        .ghost()
                        .w_full()
                        .text_size(gpui::px(13.0))
                        .on_click({
                            let entity = entity.clone();
                            move |_, window, cx| {
                                let file_path = file_path.clone();
                                entity.update(cx, |app, cx| {
                                    app.open_file(file_path, window, cx);
                                });
                            }
                        }),
                );
            }
        }

        div()
            .w(gpui::px(248.0))
            .h_full()
            .min_h_0()
            .border_r_1()
            .border_color(border)
            .child(
                v_flex()
                    .size_full()
                    .min_h_0()
                    .child(
                        div().px_4().py_3().border_b_1().border_color(border).child(
                            v_flex()
                                .gap_1()
                                .child(
                                    div()
                                        .text_color(muted)
                                        .text_size(gpui::px(11.0))
                                        .child("WORKSPACE"),
                                )
                                .child(
                                    div()
                                        .text_color(fg)
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .child(self.workspace_name()),
                                ),
                        ),
                    )
                    .child(
                        v_flex().w_full().px_3().py_3().gap_2().child(
                            Button::new("workspace-switch")
                                .label("Switch Workspace")
                                .ghost()
                                .on_click({
                                    let entity = entity.clone();
                                    move |_, window, cx| {
                                        entity.update(cx, |app, cx| {
                                            app.prompt_open_folder(window, cx);
                                        });
                                    }
                                }),
                        ),
                    )
                    .child(
                        div().flex_1().min_h_0().overflow_hidden().child(
                            v_flex()
                                .min_h_0()
                                .w_full()
                                .px_3()
                                .pb_4()
                                .gap_2()
                                .overflow_y_scrollbar()
                                .child(
                                    div()
                                        .px_2()
                                        .text_color(muted)
                                        .text_size(gpui::px(11.0))
                                        .child("FILES"),
                                )
                                .child(file_list),
                        ),
                    ),
            )
    }
}
