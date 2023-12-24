use markdown::mdast;
use markdown::mdast::{AlignKind, Node};

/// Represents the context of a list in a markdown document.
#[derive(Debug)]
struct ListContext {
    nesting_level: usize,
    is_ordered: bool,
    num_item: u32,
}

/// Represents the context of a block quote in a markdown document.
#[derive(Debug)]
struct BlockQuoteContext {
    depth: usize,
}

/// Represents the context of a block quote within a list in a markdown document.
#[derive(Debug)]
struct BlockQuoteInListContext {
    list_ctx: ListContext,
    block_quote_ctx: BlockQuoteContext,
}

/// Represents the current rendering context of a markdown document.
#[derive(Debug)]
enum Context {
    Document,
    List(ListContext),
    BlockQuote(BlockQuoteContext),
    BlockQuoteInList(BlockQuoteInListContext),
}

/// It is possible to pass single "~" and it wold be interpreted
/// as a delete which shall be interpreted as a superscript
/// https://github.com/markdown-it/markdown-it-sup
fn is_superscript(d: &mdast::Delete, source: &str) -> bool {
    let mut superscript = false;
    if let Some(position) = &d.position {
        let slice = &source[position.start.offset..position.end.offset];
        if slice.matches('~').count() == 2 {
            superscript = true;
        }
    }
    superscript
}

/// There are two types of string(bold) - underscore(__) and asterisk(**)
/// We can determine a type of it from the original file
fn is_string_underscored(d: &mdast::Strong, source: &str) -> bool {
    let mut underscored = false;
    if let Some(position) = &d.position {
        let slice = &source[position.start.offset..position.end.offset];
        if slice.matches("__").count() == 2 {
            underscored = true;
        }
    }
    underscored
}

/// There are two types of string(bold) - underscore(__) and asterisk(**)
/// We can determine a type of it from the original file
fn is_heading_atx(d: &mdast::Heading, source: &str) -> bool {
    let mut atx = false;
    if let Some(position) = &d.position {
        let line_number = position.start.line;
        let col_start = position.start.column;
        let col_end = position.end.column;

        if let Some(line) = source.lines().nth(line_number - 1) {
            let slice = &line[col_start - 1..col_end - 1];
            if slice.contains('#') {
                atx = true;
            }
        }
    }
    atx
}

/// Removes trailing new-line and spaces from the end of the string
fn remove_trailing_newline_and_space(s: &str) -> String {
    let mut result = String::from(s);
    while result.ends_with('\n') || result.ends_with(' ') {
        if let Some(s) = result.strip_suffix('\n') {
            result = s.to_string()
        }
        if let Some(s) = result.strip_suffix(' ') {
            result = s.to_string()
        }
    }
    result
}

/// Render Markdown file from AST
fn to_md(node: &mdast::Node, buffer: &mut String, context: &Context, source: &str) {
    match node {
        Node::Root(r) => {
            for child in &r.children {
                to_md(child, buffer, context, source);
                buffer.push('\n');
            }
        }
        Node::Heading(heading) => {
            if is_heading_atx(heading, source) {
                buffer.push_str("#".repeat(heading.depth as usize).as_str());
                buffer.push(' ');
            }
            for child in &heading.children {
                to_md(child, buffer, context, source);
            }
            buffer.push('\n');
            if !is_heading_atx(heading, source) {
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
                to_md(child, buffer, context, source);
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
                    buffer.push_str("+ ");
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
                        to_md(child, buffer, context, source);
                    } else {
                        to_md(child, buffer, context, source);
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
                to_md(child, buffer, context, source);
            }
            buffer.push('*');
        }
        Node::Strong(s) => {
            if is_string_underscored(s, source) {
                buffer.push_str("__");
            } else {
                buffer.push_str("**");
            }
            for child in &s.children {
                to_md(child, buffer, context, source);
            }
            if is_string_underscored(s, source) {
                buffer.push_str("__");
            } else {
                buffer.push_str("**");
            }
        }
        Node::Delete(d) => {
            if is_superscript(d, source) {
                buffer.push('~');
            } else {
                buffer.push_str("~~");
            }
            for child in &d.children {
                to_md(child, buffer, context, source);
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
                to_md(child, buffer, context, source);
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
                    ),
                    _ => to_md(
                        child,
                        buffer,
                        &Context::BlockQuote(BlockQuoteContext { depth: 1 }),
                        source,
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
                    to_md(child, buffer, context, source);
                } else {
                    let mut tmp_buffer = String::from("");
                    to_md(child, &mut tmp_buffer, context, source);
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
                    to_md(child, buffer, context, source);
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
                    to_md(child, buffer, context, source);
                }
            }
        }
        Node::TableCell(tc) => {
            for child in &tc.children {
                to_md(child, buffer, context, source);
            }
            buffer.push_str(" | ");
        }
        Node::TableRow(tr) => {
            buffer.push_str("| ");
            for child in &tr.children {
                to_md(child, buffer, context, source);
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
    to_md(&ast, &mut buffer, &Context::Document, &file.content);
    buffer = remove_trailing_newline_and_space(&buffer);
    buffer.push('\n');
    common::MarkDownFile {
        path: file.path.clone(),
        content: buffer,
        issues: vec![],
    }
}

pub fn check_md_format(file: &mut common::MarkDownFile) {
    if !file.content.eq(&fmt_markdown(file).content) {
        file.issues.push(
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
                .set_fixes(vec![format!(
                    "Run \"checkmark fmt {}\" to fix it",
                    file.path.clone()
                )])
                .build(),
        );
    }
}
