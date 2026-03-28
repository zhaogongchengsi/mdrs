use gpui::{div, px, Context, IntoElement, ParentElement, Render, Styled, Window};
use gpui_component::{scroll::ScrollableElement, v_flex, ActiveTheme};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::file_loader::format_bytes;

const MAX_PREVIEW_BYTES: usize = 512 * 1024;

#[derive(Debug, Clone)]
enum MdBlock {
    Heading {
        level: u8,
        text: String,
    },
    Paragraph {
        spans: Vec<Span>,
    },
    CodeBlock {
        code: String,
        lang: Option<String>,
    },
    Rule,
    Blockquote {
        spans: Vec<Span>,
    },
    List {
        ordered: bool,
        items: Vec<Vec<Span>>,
    },
    Notice {
        text: String,
    },
}

#[derive(Debug, Clone)]
struct Span {
    text: String,
    bold: bool,
    italic: bool,
    code: bool,
}

fn parse_markdown(src: &str) -> Vec<MdBlock> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(src, options);

    let mut blocks: Vec<MdBlock> = Vec::new();
    let mut current_spans: Vec<Span> = Vec::new();
    let mut current_text = String::new();
    let mut bold = false;
    let mut italic = false;

    let mut in_heading: Option<u8> = None;
    let mut in_blockquote = false;
    let mut list_stack: Vec<(bool, Vec<Vec<Span>>)> = Vec::new();
    let mut in_list_item = false;
    let mut in_code_block = false;
    let mut code_block_text = String::new();
    let mut code_block_lang: Option<String> = None;

    let flush_span = |spans: &mut Vec<Span>, text: &mut String, bold: bool, italic: bool| {
        if !text.is_empty() {
            spans.push(Span {
                text: std::mem::take(text),
                bold,
                italic,
                code: false,
            });
        }
    };

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                let lvl = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                in_heading = Some(lvl);
                current_spans.clear();
                current_text.clear();
            }
            Event::Start(Tag::Paragraph) => {
                current_spans.clear();
                current_text.clear();
            }
            Event::Start(Tag::BlockQuote(_)) => {
                in_blockquote = true;
                current_spans.clear();
                current_text.clear();
            }
            Event::Start(Tag::List(start)) => {
                list_stack.push((start.is_some(), Vec::new()));
            }
            Event::Start(Tag::Item) => {
                in_list_item = true;
                current_spans.clear();
                current_text.clear();
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                code_block_text.clear();
                code_block_lang = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                        let s = lang.to_string();
                        if s.is_empty() {
                            None
                        } else {
                            Some(s)
                        }
                    }
                    pulldown_cmark::CodeBlockKind::Indented => None,
                };
            }
            Event::Start(Tag::Emphasis) => {
                flush_span(&mut current_spans, &mut current_text, bold, italic);
                italic = true;
            }
            Event::Start(Tag::Strong) => {
                flush_span(&mut current_spans, &mut current_text, bold, italic);
                bold = true;
            }
            Event::Start(Tag::Link { .. }) => {}
            Event::Start(Tag::Image { .. }) => {
                current_text.push_str("[image]");
            }
            Event::End(TagEnd::Heading(_)) => {
                flush_span(&mut current_spans, &mut current_text, bold, italic);
                let text = current_spans
                    .iter()
                    .map(|span| span.text.as_str())
                    .collect::<String>();
                if let Some(level) = in_heading.take() {
                    blocks.push(MdBlock::Heading { level, text });
                }
                current_spans.clear();
            }
            Event::End(TagEnd::Paragraph) => {
                flush_span(&mut current_spans, &mut current_text, bold, italic);
                let spans = std::mem::take(&mut current_spans);
                if in_blockquote || in_list_item {
                    current_spans = spans;
                } else if !spans.is_empty() {
                    blocks.push(MdBlock::Paragraph { spans });
                }
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                flush_span(&mut current_spans, &mut current_text, bold, italic);
                let spans = std::mem::take(&mut current_spans);
                in_blockquote = false;
                if !spans.is_empty() {
                    blocks.push(MdBlock::Blockquote { spans });
                }
            }
            Event::End(TagEnd::Item) => {
                flush_span(&mut current_spans, &mut current_text, bold, italic);
                let item_spans = std::mem::take(&mut current_spans);
                in_list_item = false;
                if let Some((_, items)) = list_stack.last_mut() {
                    items.push(item_spans);
                }
            }
            Event::End(TagEnd::List(_)) => {
                if let Some((ordered, items)) = list_stack.pop() {
                    blocks.push(MdBlock::List { ordered, items });
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                let code = std::mem::take(&mut code_block_text);
                let lang = code_block_lang.take();
                blocks.push(MdBlock::CodeBlock { code, lang });
            }
            Event::End(TagEnd::Emphasis) => {
                flush_span(&mut current_spans, &mut current_text, bold, italic);
                italic = false;
            }
            Event::End(TagEnd::Strong) => {
                flush_span(&mut current_spans, &mut current_text, bold, italic);
                bold = false;
            }
            Event::Text(text) => {
                if in_code_block {
                    code_block_text.push_str(&text);
                } else {
                    current_text.push_str(&text);
                }
            }
            Event::Code(text) => {
                flush_span(&mut current_spans, &mut current_text, bold, italic);
                current_spans.push(Span {
                    text: text.to_string(),
                    bold: false,
                    italic: false,
                    code: true,
                });
            }
            Event::SoftBreak | Event::HardBreak => {
                if !in_code_block {
                    current_text.push('\n');
                }
            }
            Event::Rule => {
                blocks.push(MdBlock::Rule);
            }
            _ => {}
        }
    }

    blocks
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PreviewStats {
    pub truncated: bool,
    pub rendered_bytes: usize,
    pub total_bytes: usize,
}

fn preview_slice(src: &str) -> (&str, PreviewStats) {
    let total_bytes = src.len();
    if total_bytes <= MAX_PREVIEW_BYTES {
        return (
            src,
            PreviewStats {
                truncated: false,
                rendered_bytes: total_bytes,
                total_bytes,
            },
        );
    }

    let mut end = MAX_PREVIEW_BYTES;
    while end > 0 && !src.is_char_boundary(end) {
        end -= 1;
    }

    (
        &src[..end],
        PreviewStats {
            truncated: true,
            rendered_bytes: end,
            total_bytes,
        },
    )
}

fn build_preview(src: &str) -> (Vec<MdBlock>, PreviewStats) {
    let (preview_src, stats) = preview_slice(src);
    let mut blocks = parse_markdown(preview_src);
    if stats.truncated {
        blocks.push(MdBlock::Notice {
            text: format!(
                "Preview limited to {} of {} to keep large Markdown files responsive.",
                format_bytes(stats.rendered_bytes as u64),
                format_bytes(stats.total_bytes as u64)
            ),
        });
    }

    (blocks, stats)
}

pub struct MarkdownPreview {
    blocks: Vec<MdBlock>,
    stats: PreviewStats,
}

impl MarkdownPreview {
    pub fn new(src: &str) -> Self {
        let (blocks, stats) = build_preview(src);
        Self { blocks, stats }
    }

    pub fn set_markdown(&mut self, src: &str) -> PreviewStats {
        let (blocks, stats) = build_preview(src);
        self.blocks = blocks;
        self.stats = stats;
        stats
    }

    pub fn stats(&self) -> PreviewStats {
        self.stats
    }
}

impl Render for MarkdownPreview {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let fg = theme.colors.foreground;
        let muted = theme.colors.muted_foreground;
        let border = theme.colors.border;
        let code_bg = theme.colors.secondary;
        let accent = theme.colors.accent;

        let mut container = v_flex().p_4().gap_3().w_full();

        for block in &self.blocks {
            match block {
                MdBlock::Heading { level, text } => {
                    let (size, weight) = heading_style(*level);
                    container = container.child(
                        div()
                            .text_color(fg)
                            .font_weight(weight)
                            .text_size(px(size))
                            .pb_1()
                            .border_b_1()
                            .border_color(border)
                            .child(text.clone()),
                    );
                }
                MdBlock::Paragraph { spans } => {
                    container = container.child(render_spans(spans, fg, muted, code_bg));
                }
                MdBlock::CodeBlock { code, lang } => {
                    let mut code_block = div()
                        .rounded_md()
                        .bg(code_bg)
                        .border_1()
                        .border_color(border)
                        .overflow_hidden();
                    if let Some(language) = lang {
                        code_block = code_block.child(
                            div()
                                .px_3()
                                .py_1()
                                .border_b_1()
                                .border_color(border)
                                .text_size(px(11.0))
                                .text_color(muted)
                                .child(language.clone()),
                        );
                    }
                    code_block = code_block.child(
                        div().overflow_x_scrollbar().child(
                            div()
                                .p_3()
                                .font_family("monospace")
                                .text_size(px(13.0))
                                .text_color(fg)
                                .child(code.trim_end_matches('\n').to_string()),
                        ),
                    );
                    container = container.child(code_block);
                }
                MdBlock::Rule => {
                    container = container.child(div().w_full().h(px(1.0)).bg(border).my_2());
                }
                MdBlock::Blockquote { spans } => {
                    container = container.child(
                        div()
                            .pl_3()
                            .border_l_2()
                            .border_color(accent)
                            .text_color(muted)
                            .child(render_spans(spans, muted, muted, code_bg)),
                    );
                }
                MdBlock::List { ordered, items } => {
                    let mut list_div = v_flex().gap_1().pl_4();
                    for (idx, item_spans) in items.iter().enumerate() {
                        let bullet = if *ordered {
                            format!("{}. ", idx + 1)
                        } else {
                            "- ".to_string()
                        };
                        list_div = list_div.child(
                            div()
                                .flex()
                                .flex_row()
                                .gap_1()
                                .child(div().text_color(muted).flex_shrink_0().child(bullet))
                                .child(render_spans(item_spans, fg, muted, code_bg)),
                        );
                    }
                    container = container.child(list_div);
                }
                MdBlock::Notice { text } => {
                    container = container.child(
                        div()
                            .rounded_md()
                            .border_1()
                            .border_color(border)
                            .bg(code_bg)
                            .p_3()
                            .text_size(px(12.0))
                            .text_color(muted)
                            .child(text.clone()),
                    );
                }
            }
        }

        container
    }
}

fn heading_style(level: u8) -> (f32, gpui::FontWeight) {
    match level {
        1 => (32.0, gpui::FontWeight::BOLD),
        2 => (26.0, gpui::FontWeight::BOLD),
        3 => (22.0, gpui::FontWeight::SEMIBOLD),
        4 => (18.0, gpui::FontWeight::SEMIBOLD),
        5 => (16.0, gpui::FontWeight::MEDIUM),
        _ => (14.0, gpui::FontWeight::MEDIUM),
    }
}

fn render_spans(
    spans: &[Span],
    fg: gpui::Hsla,
    muted: gpui::Hsla,
    code_bg: gpui::Hsla,
) -> gpui::Div {
    let mut row = div().flex().flex_row().flex_wrap().gap_x_1().text_color(fg);
    for span in spans {
        if span.text.is_empty() {
            continue;
        }
        let text = span.text.clone();
        let mut elem = div().child(text);
        if span.bold {
            elem = elem.font_weight(gpui::FontWeight::BOLD);
        }
        if span.italic {
            elem = elem.italic();
        }
        if span.code {
            elem = elem
                .font_family("monospace")
                .text_size(px(13.0))
                .px_1()
                .rounded_sm()
                .bg(code_bg)
                .text_color(muted);
        }
        row = row.child(elem);
    }
    row
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_limits_large_documents_without_breaking_utf8() {
        let large = "你好，Markdown\n".repeat((MAX_PREVIEW_BYTES / 18) + 4096);
        let (preview_src, stats) = preview_slice(&large);

        assert!(stats.truncated);
        assert!(stats.rendered_bytes <= MAX_PREVIEW_BYTES);
        assert!(large.is_char_boundary(preview_src.len()));
    }
}
