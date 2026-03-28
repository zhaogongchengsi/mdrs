use gpui::{div, Context, IntoElement, ParentElement, Styled};
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    scroll::ScrollableElement,
    v_flex, ActiveTheme, Icon, Selectable, Sizable, Size,
};

use crate::{app::MdrsApp, app_icon::AppIcon};

const SIDEBAR_SECTION_INSET_X: f32 = 8.0;
const FILE_ROW_ICON_SLOT: f32 = 16.0;

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
                file_list = file_list.child(render_workspace_file_item(
                    ("workspace-file", index),
                    file.label(),
                    is_selected,
                    fg,
                    muted,
                    {
                        let entity = entity.clone();
                        move |_, window, cx| {
                            let file_path = file_path.clone();
                            entity.update(cx, |app, cx| {
                                app.open_file(file_path, window, cx);
                            });
                        }
                    },
                ));
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
                        v_flex().w_full().px_2().py_3().gap_2().child(
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
                                .px_2()
                                .pb_4()
                                .gap_2()
                                .overflow_y_scrollbar()
                                .child(sidebar_section_label("FILES", muted))
                                .child(file_list),
                        ),
                    ),
            )
    }
}

fn render_workspace_file_item(
    id: impl Into<gpui::ElementId>,
    label: impl Into<String>,
    is_selected: bool,
    fg: gpui::Hsla,
    muted: gpui::Hsla,
    on_click: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> Button {
    let text_color = if is_selected { fg } else { muted };
    let icon_color = if is_selected { fg } else { muted.opacity(0.9) };

    Button::new(id)
        .selected(is_selected)
        .ghost()
        .w_full()
        .px_0()
        .py_0p5()
        .justify_start()
        .child(
            h_flex()
                .w_full()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .w(gpui::px(FILE_ROW_ICON_SLOT))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            Icon::from(AppIcon::File)
                                .with_size(Size::Small)
                                .text_color(icon_color),
                        ),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_size(gpui::px(12.5))
                        .text_color(text_color)
                        .font_weight(if is_selected {
                            gpui::FontWeight::MEDIUM
                        } else {
                            gpui::FontWeight::NORMAL
                        })
                        .child(label.into()),
                ),
        )
        .on_click(on_click)
}

fn sidebar_section_label(label: impl Into<String>, muted: gpui::Hsla) -> gpui::Div {
    div()
        .px(gpui::px(SIDEBAR_SECTION_INSET_X))
        .text_color(muted)
        .text_size(gpui::px(10.5))
        .child(label.into())
}
