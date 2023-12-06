use markdown;
use markdown::mdast;
use markdown::mdast::{AlignKind, Node};
// use similar::{ChangeTag, TextDiff};

struct ListContext {
    nesting_level: usize,
    is_ordered: bool,
    num_item: u32,
}

struct BlockQuoteContext {
    depth: usize,
}

/// Current rendering context
enum Context {
    Document,
    List(ListContext),
    BlockQuote(BlockQuoteContext),
}

/// It is possible to pass single "~" and it wold be interpreted
/// as a delete which shall be interpreted as a superscript
/// https://github.com/markdown-it/markdown-it-sup
fn is_superscript(d: &mdast::Delete, source: &str) -> bool {
    let mut superscript = false;
    if let Some(position) = &d.position {
        let line_number = position.start.line;
        let col_start = position.start.column;
        let col_end = position.end.column;

        if let Some(line) = source.lines().nth(line_number - 1) {
            let slice = &line[col_start - 1..col_end - 1];
            if slice.matches("~").count() == 2 {
                superscript = true;
            }
        }
    }
    return superscript;
}

/// There are two types of string(bold) - underscore(__) and asterisk(**)
/// We can determine a type of it from the original file
fn is_string_underscored(d: &mdast::Strong, source: &str) -> bool {
    let mut underscored = false;
    if let Some(position) = &d.position {
        let line_number = position.start.line;
        let col_start = position.start.column;
        let col_end = position.end.column;

        if let Some(line) = source.lines().nth(line_number - 1) {
            let slice = &line[col_start - 1..col_end - 1];
            if slice.matches("__").count() == 2 {
                underscored = true;
            }
        }
    }
    return underscored;
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
            if slice.contains("#") {
                atx = true;
            }
        }
    }
    return atx;
}

/// Render Markdown file from AST
fn to_md(node: &mdast::Node, mut buffer: &mut String, context: &Context, source: &str) {
    match node {
        Node::Root(r) => {
            for child in &r.children {
                to_md(&child, &mut buffer, &context, &source);
                buffer.push_str("\n");
            }
        }
        Node::Heading(heading) => {
            if is_heading_atx(&heading, &source) {
                buffer.push_str("#".repeat(heading.depth as usize).as_str());
                buffer.push_str(" ");
            }
            for child in &heading.children {
                to_md(&child, &mut buffer, &context, &source);
            }
            buffer.push_str("\n");
            if !is_heading_atx(&heading, &source) {
                if heading.depth == 1 {
                    buffer.push_str("=============");
                } else {
                    buffer.push_str("-------------");
                }
            }
        }
        Node::Text(t) => match context {
            Context::BlockQuote(ctx) => {
                buffer.push_str(&t.value.replace("\n", &format!("\n{}", "> ".repeat(ctx.depth).as_str())))
            }
            _ => buffer.push_str(&t.value),
        },
        Node::Paragraph(p) => {
            for child in &p.children {
                to_md(&child, &mut buffer, &context, &source);
            }
            buffer.push_str("\n");
        }
        Node::List(l) => {
            let mut start = if l.start.is_some() {
                l.start.unwrap()
            } else {
                0
            };
            let mut nesting_level = 0;
            match context {
                Context::List(ctx) => nesting_level = ctx.nesting_level + 1,
                _ => {}
            }
            for child in &l.children {
                match context {
                    Context::BlockQuote(_) => {
                        if &child != &l.children.first().unwrap() {
                            buffer.push_str("> ");
                        }
                    }
                    _ => {}
                }
                to_md(
                    &child,
                    &mut buffer,
                    &Context::List(ListContext {
                        nesting_level: nesting_level,
                        is_ordered: l.ordered,
                        num_item: start,
                    }),
                    &source,
                );
                start += 1;
                if l.spread {
                    buffer.push_str("\n");
                }
            }
        }
        Node::ListItem(li) => match context {
            Context::List(ctx) => {
                buffer.push_str(&"  ".repeat(ctx.nesting_level));
                if ctx.is_ordered {
                    buffer.push_str(&format!("{}. ", ctx.num_item));
                } else {
                    buffer.push_str("+ ");
                }
                for child in &li.children {
                    match child {
                        Node::Paragraph(_) => {
                            if &child != &li.children.first().unwrap() {
                                buffer.push_str("\n");
                                buffer.push_str(&"    ");
                                to_md(&child, &mut buffer, &context, &source);
                            } else {
                                to_md(&child, &mut buffer, &context, &source);
                            }
                        }
                        _ => to_md(&child, &mut buffer, &context, &source),
                    }
                }
            }
            _ => {}
        },
        Node::Code(c) => match context {
            Context::BlockQuote(_) => buffer.push_str(
                &format!(
                    "```{}\n{}\n```\n",
                    c.lang.as_ref().unwrap_or(&String::new()),
                    c.value
                )
                .replace("\n", &format!("\n> ")),
            ),
            _ => buffer.push_str(&format!(
                "```{}\n{}\n```\n",
                c.lang.as_ref().unwrap_or(&String::new()),
                c.value
            )),
        },
        Node::InlineCode(c) => {
            buffer.push_str(&format!("`{}`", &c.value));
        }
        Node::Emphasis(e) => {
            buffer.push_str("*");
            for child in &e.children {
                to_md(&child, &mut buffer, &context, &source);
            }
            buffer.push_str("*");
        }
        Node::Strong(s) => {
            if is_string_underscored(&s, &source) {
                buffer.push_str("__");
            } else {
                buffer.push_str("**");
            }
            for child in &s.children {
                to_md(&child, &mut buffer, &context, &source);
            }
            if is_string_underscored(&s, &source) {
                buffer.push_str("__");
            } else {
                buffer.push_str("**");
            }
        }
        Node::Delete(d) => {
            if is_superscript(&d, &source) {
                buffer.push_str("~");
            } else {
                buffer.push_str("~~");
            }
            for child in &d.children {
                to_md(&child, &mut buffer, &context, &source);
            }
            if is_superscript(&d, &source) {
                buffer.push_str("~");
            } else {
                buffer.push_str("~~");
            }
        }
        Node::Break(_) => {
            buffer.push_str("\n");
        }
        Node::Link(l) => {
            buffer.push_str("[");
            for child in &l.children {
                to_md(&child, &mut buffer, &context, &source);
            }
            buffer.push_str("]");
            buffer.push_str(&format!("({}", &l.url.clone().as_str()));
            if let Some(title) = &l.title {
                buffer.push_str(&format!(" \"{}\"", &title));
            }
            buffer.push_str(")");
        }
        Node::Image(i) => {
            buffer.push_str(&format!("![{}]({}", &i.alt, &i.url));
            if let Some(title) = &i.title {
                buffer.push_str(&format!(" \"{}\"", &title));
            }
            buffer.push_str(")");
        }
        Node::BlockQuote(b) => {
            for child in &b.children {
                buffer.push_str("> ");
                if &child != &b.children.first().unwrap() {
                    buffer.push_str("\n> ");
                }
                match &context {
                    Context::BlockQuote(ctx) => to_md(
                        &child,
                        &mut buffer,
                        &Context::BlockQuote(BlockQuoteContext {
                            depth: ctx.depth + 1,
                        }),
                        &source,
                    ),
                    _ => to_md(
                        &child,
                        &mut buffer,
                        &Context::BlockQuote(BlockQuoteContext { depth: 1 }),
                        &source,
                    ),
                }
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
                if &child == &f.children.first().unwrap() {
                    to_md(&child, &mut buffer, &context, &source);
                } else {
                    let mut tmp_buffer = String::from("");
                    to_md(&child, &mut tmp_buffer, &context, &source);
                    if let Some(position) = child.position() {
                        for line in tmp_buffer.lines() {
                            buffer.push_str(&" ".repeat(position.clone().start.column));
                            buffer.push_str(&line);
                            buffer.push_str("\n");
                        }
                    }
                }
            }
        }
        Node::Table(t) => {
            for child in &t.children {
                if &child == &t.children.first().unwrap() {
                    to_md(&child, &mut buffer, &context, &source);
                    buffer.push_str("|");
                    for align in &t.align {
                        match align {
                            AlignKind::Left => buffer.push_str(" :-- |"),
                            AlignKind::Right => buffer.push_str(" --: |"),
                            AlignKind::Center => buffer.push_str(" :-: |"),
                            AlignKind::None => buffer.push_str(" --- |"),
                        }
                    }
                    buffer.push_str("\n");
                } else {
                    to_md(&child, &mut buffer, &context, &source);
                }
            }
        }
        Node::TableCell(tc) => {
            for child in &tc.children {
                to_md(&child, &mut buffer, &context, &source);
            }
            buffer.push_str(" | ");
        }
        Node::TableRow(tr) => {
            buffer.push_str("| ");
            for child in &tr.children {
                to_md(&child, &mut buffer, &context, &source);
            }
            buffer.push_str("\n");
        }
        _ => panic!("Unexpected node type {node:#?}"),
    }
}

/// Return formatted Markdown file
pub fn fmt_markdown(file: &common::MarkDownFile) -> common::MarkDownFile {
    let mut buffer: String = String::from("");
    let ast = markdown::to_mdast(&file.content, &markdown::ParseOptions::gfm()).unwrap();
    dbg!(&ast);
    to_md(&ast, &mut buffer, &Context::Document, &file.content);
    if buffer.ends_with("\n\n\n") {
        buffer = buffer.strip_suffix("\n\n").unwrap().to_string();
    } else if buffer.ends_with("\n\n") {
        buffer = buffer.strip_suffix("\n").unwrap().to_string();
    }
    if !file.content.ends_with("\n") && buffer.ends_with("\n") {
        buffer = buffer.strip_suffix("\n").unwrap().to_string();
    }
    common::MarkDownFile {
        path: file.path.clone(),
        content: buffer,
    }
}

pub fn check_md_format(file: &common::MarkDownFile) -> Vec<common::CheckIssue> {
    let mut issues: Vec<common::CheckIssue> = vec![];

    let formatted_file = fmt_markdown(&file);

    if !file.content.eq(&formatted_file.content) {
        issues.push(
            common::CheckIssueBuilder::default()
                .set_category(common::IssueCategory::Formatting)
                .set_file_path(file.path.clone())
                .set_row_num_start(0)
                .set_row_num_end(file.content.lines().count())
                .set_col_num_start(0)
                .set_col_num_end(0)
                .set_message(String::from(
                    "Formatting is incorrect! Please run fmt to fix it",
                ))
                .set_fixes(vec![])
                .build(),
        );
    }

    return issues;
}
