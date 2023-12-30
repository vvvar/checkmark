mod context;
mod style;
mod utils;

use context::*;
use style::*;
use utils::*;

use markdown::mdast;
use markdown::mdast::{AlignKind, Node};

/// Render Markdown file from AST
fn to_md(
    node: &mdast::Node,
    buffer: &mut String,
    context: &Context,
    source: &str,
    options: &FormattingOptions,
) {
    match node {
        Node::Root(r) => {
            for child in &r.children {
                to_md(child, buffer, context, source, options);
                buffer.push('\n');
            }
        }
        Node::Heading(heading) => {
            if HeaderStyle::Atx == options.header_style {
                buffer.push_str("#".repeat(heading.depth as usize).as_str());
                buffer.push(' ');
            }
            for child in &heading.children {
                to_md(child, buffer, context, source, options);
            }
            buffer.push('\n');
            if HeaderStyle::SetExt == options.header_style {
                if heading.depth == 1 {
                    buffer.push_str("=============");
                } else {
                    buffer.push_str("-------------");
                }
            }
        }
        Node::Text(t) => {
            match context {
                Context::BlockQuote(ctx) => buffer.push_str(
                    &t.value
                        .replace('\n', &format!("\n{}", "> ".repeat(ctx.depth).as_str())),
                ),
                Context::List(ctx) => {
                    // Very special case - when we have list with text that has ne lines
                    // we want to align them
                    if ctx.is_ordered {
                        buffer.push_str(&t.value.replace(
                            '\n',
                            &format!("\n{}", "   ".repeat(ctx.nesting_level + 1).as_str()),
                        ));
                    } else {
                        buffer.push_str(&t.value.replace(
                            '\n',
                            &format!("\n{}", "  ".repeat(ctx.nesting_level + 1).as_str()),
                        ));
                    }
                }
                Context::BlockQuoteInList(ctx) => {
                    // Very special case - we have a block quote inside a list
                    // we want to align it with list so it will be rendered
                    // by engines like a quote inside a list.
                    // Otherwise - it will be rendered outside
                    buffer.push_str(&t.value.replace(
                        '\n',
                        &format!(
                            "\n{}> ",
                            "  ".repeat(ctx.list_ctx.nesting_level + 1).as_str()
                        ),
                    ));
                }
                _ => buffer.push_str(&t.value),
            }
        }
        Node::Paragraph(p) => {
            for child in &p.children {
                to_md(child, buffer, context, source, options);
            }
            buffer.push('\n');
        }
        Node::List(l) => {
            let mut start = if l.start.is_some() {
                l.start.unwrap()
            } else {
                0
            };
            let mut nesting_level = 0;
            if let Context::List(ctx) = context {
                nesting_level = ctx.nesting_level + 1
            }
            for child in &l.children {
                if let Context::BlockQuote(_) = context {
                    if child != l.children.first().unwrap() {
                        buffer.push_str("> ");
                    }
                }
                to_md(
                    child,
                    buffer,
                    &Context::List(ListContext {
                        nesting_level,
                        is_ordered: l.ordered,
                        num_item: start,
                    }),
                    source,
                    options,
                );
                start += 1;
                // Spread list(also called loose in CommonMark) is when
                // at least one element is new-line separated. We force
                // to be consistent and add newlines everywhere except
                // last element because it will have newline anyways
                if l.spread && child != l.children.last().unwrap() {
                    buffer.push('\n');
                }
            }
        }
        Node::ListItem(li) => {
            if let Context::List(ctx) = context {
                buffer.push_str(&"  ".repeat(ctx.nesting_level));
                if ctx.is_ordered {
                    buffer.push_str(&format!("{}. ", ctx.num_item));
                } else {
                    buffer.push_str(&format!("{} ", options.list_sign_style.as_str()));
                }
                for child in &li.children {
                    // When there's 2+ paragraphs in a list item
                    // then we want to align then with list
                    if child != li.children.first().unwrap() {
                        if let Node::Paragraph(_) = &child {
                            if ctx.is_ordered {
                                buffer.push_str(&format!(
                                    "\n{}",
                                    "   ".repeat(ctx.nesting_level + 1).as_str()
                                ));
                            } else {
                                buffer.push_str(&format!(
                                    "\n{}",
                                    "  ".repeat(ctx.nesting_level + 1).as_str()
                                ));
                            }
                        } else if let Node::BlockQuote(_) = &child {
                            if ctx.is_ordered {
                                buffer.push_str("   ");
                            } else {
                                buffer.push_str("  ");
                            }
                        }
                        to_md(child, buffer, context, source, options);
                    } else {
                        to_md(child, buffer, context, source, options);
                    }
                }
            }
        }
        Node::Code(c) => {
            let mut syntax_highlight = "text";
            if let Some(lang) = &c.lang {
                syntax_highlight = &lang;
            }
            match context {
                Context::BlockQuote(_) => buffer.push_str(
                    &format!("```{}\n{}\n```\n", syntax_highlight, c.value,).replace('\n', "\n> "),
                ),
                _ => buffer.push_str(&format!("```{}\n{}\n```\n", syntax_highlight, c.value)),
            }
        }
        Node::InlineCode(c) => {
            buffer.push_str(&format!("`{}`", &c.value));
        }
        Node::Emphasis(e) => {
            buffer.push('*');
            for child in &e.children {
                to_md(child, buffer, context, source, options);
            }
            buffer.push('*');
        }
        Node::Strong(s) => {
            match options.strong_style {
                StrongStyle::Asterisk => buffer.push_str("**"),
                StrongStyle::Underscore => buffer.push_str("__"),
            }
            for child in &s.children {
                to_md(child, buffer, context, source, options);
            }
            match options.strong_style {
                StrongStyle::Asterisk => buffer.push_str("**"),
                StrongStyle::Underscore => buffer.push_str("__"),
            }
        }
        Node::Delete(d) => {
            if is_superscript(d, source) {
                buffer.push('~');
            } else {
                buffer.push_str("~~");
            }
            for child in &d.children {
                to_md(child, buffer, context, source, options);
            }
            if is_superscript(d, source) {
                buffer.push('~');
            } else {
                buffer.push_str("~~");
            }
        }
        Node::Break(_) => {
            buffer.push('\n');
        }
        Node::Link(l) => {
            buffer.push('[');
            for child in &l.children {
                to_md(child, buffer, context, source, options);
            }
            buffer.push(']');
            buffer.push_str(&format!("({}", &l.url.clone().as_str()));
            if let Some(title) = &l.title {
                buffer.push_str(&format!(" \"{}\"", &title));
            }
            buffer.push(')');
        }
        Node::Image(i) => {
            buffer.push_str(&format!("![{}]({}", &i.alt, &i.url));
            if let Some(title) = &i.title {
                buffer.push_str(&format!(" \"{}\"", &title));
            }
            buffer.push(')');
        }
        Node::BlockQuote(b) => {
            for child in &b.children {
                buffer.push_str(&match &context {
                    Context::BlockQuote(ctx) => "> ".repeat(ctx.depth),
                    Context::BlockQuoteInList(ctx) => "> ".repeat(ctx.block_quote_ctx.depth),
                    _ => "> ".to_string(),
                });
                match &context {
                    Context::BlockQuote(ctx) => to_md(
                        child,
                        buffer,
                        &Context::BlockQuote(BlockQuoteContext {
                            depth: ctx.depth + 1,
                        }),
                        source,
                        options,
                    ),
                    Context::List(ctx) => to_md(
                        child,
                        buffer,
                        &Context::BlockQuoteInList(BlockQuoteInListContext {
                            list_ctx: ListContext {
                                nesting_level: ctx.nesting_level,
                                is_ordered: ctx.is_ordered,
                                num_item: ctx.num_item,
                            },
                            block_quote_ctx: BlockQuoteContext { depth: 1 },
                        }),
                        source,
                        options,
                    ),
                    Context::BlockQuoteInList(ctx) => to_md(
                        child,
                        buffer,
                        &Context::BlockQuoteInList(BlockQuoteInListContext {
                            list_ctx: ListContext {
                                nesting_level: ctx.list_ctx.nesting_level,
                                is_ordered: ctx.list_ctx.is_ordered,
                                num_item: ctx.list_ctx.num_item,
                            },
                            block_quote_ctx: BlockQuoteContext {
                                depth: ctx.block_quote_ctx.depth + 1,
                            },
                        }),
                        source,
                        options,
                    ),
                    _ => to_md(
                        child,
                        buffer,
                        &Context::BlockQuote(BlockQuoteContext { depth: 1 }),
                        source,
                        options,
                    ),
                }
                // Add new trailing blank block quote if there's more than one child
                if child != b.children.last().unwrap() {
                    match &context {
                        Context::Document => buffer.push_str(">\n"),
                        Context::BlockQuote(ctx) => buffer.push_str(&format!(
                            "{}\n",
                            "> ".repeat(ctx.depth + 1).strip_suffix(' ').unwrap()
                        )),
                        Context::List(_) => {}
                        Context::BlockQuoteInList(_) => {}
                    }
                }
            }
            // Remove trailing block quote if any
            if buffer.ends_with("\n> ") {
                buffer.truncate(buffer.len() - "\n> ".len());
            }
        }
        Node::ThematicBreak(_) => {
            buffer.push_str("---\n");
        }
        Node::Html(h) => {
            buffer.push_str(&h.value);
        }
        Node::ImageReference(ir) => {
            buffer.push_str(&format!("![{}][{}]", ir.alt, ir.identifier));
        }
        Node::Definition(d) => {
            buffer.push_str(&format!("[{}]: {}", d.identifier, d.url));
            if let Some(title) = &d.title {
                buffer.push_str(&format!(" \"{}\"", &title));
            }
        }
        Node::LinkReference(lr) => {
            buffer.push_str(&format!("[^{}]", &lr.identifier));
        }
        Node::FootnoteReference(f) => {
            buffer.push_str(&format!("[^{}]", &f.identifier));
        }
        Node::FootnoteDefinition(f) => {
            buffer.push_str(&format!("[^{}]: ", &f.identifier));
            for child in &f.children {
                if child == f.children.first().unwrap() {
                    to_md(child, buffer, context, source, options);
                } else {
                    let mut tmp_buffer = String::from("");
                    to_md(child, &mut tmp_buffer, context, source, options);
                    if let Some(position) = child.position() {
                        for line in tmp_buffer.lines() {
                            buffer.push_str(&" ".repeat(position.clone().start.column));
                            buffer.push_str(line);
                            buffer.push('\n');
                        }
                    }
                }
            }
        }
        Node::Table(t) => {
            for child in &t.children {
                if child == t.children.first().unwrap() {
                    to_md(child, buffer, context, source, options);
                    buffer.push('|');
                    for align in &t.align {
                        match align {
                            AlignKind::Left => buffer.push_str(" :-- |"),
                            AlignKind::Right => buffer.push_str(" --: |"),
                            AlignKind::Center => buffer.push_str(" :-: |"),
                            AlignKind::None => buffer.push_str(" --- |"),
                        }
                    }
                    buffer.push('\n');
                } else {
                    to_md(child, buffer, context, source, options);
                }
            }
        }
        Node::TableCell(tc) => {
            for child in &tc.children {
                to_md(child, buffer, context, source, options);
            }
            buffer.push_str(" | ");
        }
        Node::TableRow(tr) => {
            buffer.push_str("| ");
            for child in &tr.children {
                to_md(child, buffer, context, source, options);
            }
            buffer.push('\n');
        }
        _ => panic!("Unexpected node type {node:#?}"),
    }
}

/// Return formatted Markdown file
pub fn fmt_markdown(file: &common::MarkDownFile) -> common::MarkDownFile {
    let mut buffer: String = String::from("");
    let ast = markdown::to_mdast(&file.content, &markdown::ParseOptions::gfm()).unwrap();
    to_md(
        &ast,
        &mut buffer,
        &Context::Document,
        &file.content,
        &FormattingOptions::default(),
    );
    buffer = remove_trailing_newline_and_space(&buffer);
    buffer.push('\n');
    common::MarkDownFile {
        path: file.path.clone(),
        content: buffer,
        issues: vec![],
    }
}

pub fn check_md_format(file: &common::MarkDownFile) -> Vec<common::CheckIssue> {
    let mut issues: Vec<common::CheckIssue> = vec![];
    let formatted = &fmt_markdown(file);
    if !file.content.eq(&formatted.content) {
        issues.push(
            common::CheckIssueBuilder::default()
                .set_category(common::IssueCategory::Formatting)
                .set_severity(common::IssueSeverity::Error)
                .set_file_path(file.path.clone())
                .set_row_num_start(1)
                .set_row_num_end(file.content.lines().count())
                .set_col_num_start(1)
                .set_col_num_end(1)
                .set_offset_start(0)
                .set_offset_end(file.content.len())
                .set_message(String::from("Formatting is incorrect"))
                .set_fixes(vec![
                    format!("Run \"checkmark fmt {}\" to fix it", file.path.clone()),
                    "See an approximated diff below:".to_string(),
                    get_diff(&file.content, &formatted.content),
                ])
                .build(),
        );
    }
    issues
}
