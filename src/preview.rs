use gpui::{
    Context, IntoElement, ParentElement, Render, Styled, Window, div, px,
};
use gpui_component::{ActiveTheme, v_flex};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

/// A single rendered block in the Markdown preview.
#[derive(Debug, Clone)]
enum MdBlock {
    /// A heading (h1–h6) with its text content.
    Heading { level: u8, text: String },
    /// A paragraph with inline-formatted runs.
    Paragraph { spans: Vec<Span> },
    /// A fenced or indented code block.
    CodeBlock { code: String, lang: Option<String> },
    /// A horizontal rule.
    Rule,
    /// A block-level quote.
    Blockquote { spans: Vec<Span> },
    /// A list (ordered or unordered) with item text.
    List {
        ordered: bool,
        items: Vec<Vec<Span>>,
    },
}

/// An inline text span with optional formatting.
#[derive(Debug, Clone)]
struct Span {
    text: String,
    bold: bool,
    italic: bool,
    code: bool,
}

/// Parses a Markdown string into a list of [`MdBlock`]s.
fn parse_markdown(src: &str) -> Vec<MdBlock> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(src, options);

    let mut blocks: Vec<MdBlock> = Vec::new();

    // Inline accumulator state
    let mut current_spans: Vec<Span> = Vec::new();
    let mut current_text = String::new();
    let mut bold = false;
    let mut italic = false;

    let mut in_heading: Option<u8> = None;
    let mut in_blockquote = false;
    let mut list_stack: Vec<(bool, Vec<Vec<Span>>)> = Vec::new(); // (ordered, items)
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
            // ── Tags: open ──────────────────────────────────────────────────
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
                        if s.is_empty() { None } else { Some(s) }
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
            Event::Start(Tag::Link { .. }) => {
                // Links are rendered as plain text for simplicity.
            }
            Event::Start(Tag::Image { .. }) => {
                // Images are rendered as placeholder text.
                current_text.push_str("[image]");
            }

            // ── Tags: close ─────────────────────────────────────────────────
            Event::End(TagEnd::Heading(_)) => {
                flush_span(&mut current_spans, &mut current_text, bold, italic);
                let text = current_spans
                    .iter()
                    .map(|s| s.text.as_str())
                    .collect::<String>();
                if let Some(level) = in_heading.take() {
                    blocks.push(MdBlock::Heading { level, text });
                }
                current_spans.clear();
            }
            Event::End(TagEnd::Paragraph) => {
                flush_span(&mut current_spans, &mut current_text, bold, italic);
                let spans = std::mem::take(&mut current_spans);
                if in_blockquote {
                    // Blockquote paragraphs are collected by the outer End(BlockQuote).
                    // Re-push so we can capture them later.
                    current_spans = spans;
                } else if in_list_item {
                    // Paragraph within a list item — handled by End(Item).
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
                if let Some((_, ref mut items)) = list_stack.last_mut() {
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

            // ── Leaf events ─────────────────────────────────────────────────
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

// ── GPUI view ───────────────────────────────────────────────────────────────

/// The read-only Markdown preview pane.
pub struct MarkdownPreview {
    blocks: Vec<MdBlock>,
}

impl MarkdownPreview {
    pub fn new(src: &str) -> Self {
        Self {
            blocks: parse_markdown(src),
        }
    }

    pub fn set_markdown(&mut self, src: &str) {
        self.blocks = parse_markdown(src);
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
                    // Show language label if present.
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
                        div()
                            .p_3()
                            .font_family("monospace")
                            .text_size(px(13.0))
                            .text_color(fg)
                            .child(code.trim_end_matches('\n').to_string()),
                    );
                    container = container.child(code_block);
                }

                MdBlock::Rule => {
                    container = container.child(
                        div()
                            .w_full()
                            .h(px(1.0))
                            .bg(border)
                            .my_2(),
                    );
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
                            "• ".to_string()
                        };
                        list_div = list_div.child(
                            div()
                                .flex()
                                .flex_row()
                                .gap_1()
                                .child(
                                    div()
                                        .text_color(muted)
                                        .flex_shrink_0()
                                        .child(bullet),
                                )
                                .child(render_spans(item_spans, fg, muted, code_bg)),
                        );
                    }
                    container = container.child(list_div);
                }
            }
        }

        container
    }
}

/// Returns (font_size_px, font_weight) for a given heading level.
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

/// Renders a slice of [`Span`]s as a horizontal-wrapping flex container.
fn render_spans(
    spans: &[Span],
    fg: gpui::Hsla,
    muted: gpui::Hsla,
    code_bg: gpui::Hsla,
) -> gpui::Div {
    let mut row = div().flex().flex_row().flex_wrap().gap_x_1().text_color(fg);
    for span in spans {
        // Only skip spans that are purely empty (not whitespace that matters).
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
