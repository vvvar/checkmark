mod context;
mod style;
mod utils;

use context::*;
use style::*;
use utils::*;

use colored::Colorize;
use markdown::mdast;
use markdown::mdast::{AlignKind, Node};

/// Takes a table, walks through it's cells, calculates max size of each column
/// and returns a vector of sizes that represents maximum possible size of each column(per all rows)
/// Useful to get know what is the expected size of each column to align them later
fn calculate_max_col_len(table: &mdast::Table, source: &str) -> Vec<usize> {
    let mut max_col_len = std::collections::BTreeMap::<usize, usize>::new();
    for child in &table.children {
        if let Node::TableRow(tr) = child {
            for (i, child) in tr.children.iter().enumerate() {
                let offset_start = child.position().as_ref().unwrap().start.offset;
                let offset_end = child.position().as_ref().unwrap().end.offset;
                let col_len = source[offset_start..offset_end]
                    .trim_matches(' ')
                    .trim_matches('|')
                    .trim_matches(' ')
                    .len();
                if let Some(max_len) = max_col_len.get(&i) {
                    if col_len > *max_len {
                        max_col_len.insert(i, col_len);
                    }
                } else {
                    max_col_len.insert(i, col_len);
                }
            }
        }
    }
    max_col_len.into_values().collect()
}

fn render_table_row(
    node: &mdast::Node,
    buffer: &mut String,
    context: &Context,
    source: &str,
    options: &FormattingOptions,
    expected_col_lengths: &[usize],
) {
    // Render a heading of the table
    buffer.push_str("| ");
    // Rows
    for (i, child) in node.children().unwrap().iter().enumerate() {
        // Cols
        let len_before = buffer.len();
        for child in child.children().unwrap() {
            to_md(child, buffer, context, source, options);
        }
        let len_after = buffer.len();
        let expected_len = len_before + expected_col_lengths.get(i).unwrap();
        if len_after < expected_len {
            // Fill missing with white spaces
            buffer.push_str(&" ".repeat(expected_len - len_after));
        }
        buffer.push_str(" | ");
    }
    buffer.push('\n');
    trim_trailing_space_before_newline(buffer); // Dirty hack to fix trailing space
                                                // Follow it with a separator
                                                // Remember lengths of each column
                                                // to align them later
}

fn render_table_heading_separator(
    table: &mdast::Table,
    buffer: &mut String,
    expected_col_lengths: &[usize],
) {
    buffer.push('|');
    for (i, align) in table.align.iter().enumerate() {
        let padding = if let Some(len) = expected_col_lengths.get(i) {
            len - 1
        } else {
            // Default padding, to avoid overflow
            2
        };
        match align {
            AlignKind::Left => buffer.push_str(&format!(" :{} |", &"-".repeat(padding))),
            AlignKind::Right => buffer.push_str(&format!(" {}: |", &"-".repeat(padding))),
            AlignKind::Center => buffer.push_str(&format!(" :{}: |", &"-".repeat(padding - 1))),
            AlignKind::None => buffer.push_str(&format!(" -{}- |", &"-".repeat(padding - 1))),
        }
    }
    buffer.push('\n');
}

/// This function escapes special characters that has a special meaning Markdown.
/// Although there's no official specification for how to escape special characters
/// it is better to do so to avoid any possible issues with engines.
/// Based on discussion here: https://talk.commonmark.org/t/can-we-have-formal-escaping-rules/2624
/// and Markdown cookbook: https://bookdown.org/yihui/rmarkdown-cookbook/special-chars.html
/// and Perforce recommendation: https://www.perforce.com/manuals/v18.2/swarm/Content/Swarm/basics.markdown.html#:~:text=Use%20the%20backslash%20character%20%5C%20to,Exclamation%20point%20%5C!
/// also this post: https://stackoverflow.com/a/45766624
/// Do not escape: "-", "+", "!", "#", "{", "}", "(", ")", "_", and "." because render engines are mostly fine with them although they have a special meaning
fn escape_special_characters(str: &str) -> String {
    str.replace('\t', " ")
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('*', "\\*")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('>', "\\>")
        .replace('<', "\\<")
}

/// Returns true when link it either auto or bare
/// Autolink: "<http://example.com>"
/// Bare link: "http://example.com"
fn is_auto_or_bare_link(l: &mdast::Link) -> bool {
    // Detect this structure
    //    Link {
    //       children: [ Text {} ]
    //       title: None
    //    }
    if l.title.is_none() && l.children.len() == 1 {
        if let Node::Text(t) = l.children.first().unwrap() {
            // Check that text is === as url
            // "mailto:" stripped because parser adds it
            // for all e-mails
            if let Some(email) = l.url.strip_prefix("mailto:") {
                t.value.eq(&email)
            } else {
                t.value.eq(&l.url)
            }
        } else {
            false
        }
    } else {
        false
    }
}

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
                // Only when HTML is on a Root-level
                // we want to add extra newline
                if let Node::Html(_) = child {
                    buffer.push('\n');
                }
            }
        }
        Node::Heading(heading) => {
            if HeaderStyle::Atx == options.header.style {
                buffer.push_str("#".repeat(heading.depth as usize).as_str());
                buffer.push(' ');
            }
            for child in &heading.children {
                to_md(child, buffer, context, source, options);
            }
            buffer.push('\n');
            if HeaderStyle::SetExt == options.header.style {
                if heading.depth == 1 {
                    buffer.push_str("=============");
                } else {
                    buffer.push_str("-------------");
                }
            }
        }
        Node::Text(t) => {
            let text = escape_special_characters(&t.value);
            match context {
                Context::BlockQuote(ctx) => buffer.push_str(
                    &text.replace('\n', &format!("\n{}", "> ".repeat(ctx.depth).as_str())),
                ),
                Context::List(ctx) => {
                    // Very special case - when we have list with text that has ne lines
                    // we want to align them
                    if ctx.is_ordered {
                        buffer.push_str(&text.replace(
                            '\n',
                            &format!("\n{}", "   ".repeat(ctx.nesting_level + 1).as_str()),
                        ));
                    } else {
                        buffer.push_str(&text.replace(
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
                    if ctx.list_ctx.is_ordered {
                        // The only difference is in additional space
                        buffer.push_str(&text.replace(
                            '\n',
                            &format!(
                                "\n{} > ",
                                "  ".repeat(ctx.list_ctx.nesting_level + 1).as_str()
                            ),
                        ));
                    } else {
                        buffer.push_str(&text.replace(
                            '\n',
                            &format!(
                                "\n{}> ",
                                "  ".repeat(ctx.list_ctx.nesting_level + 1).as_str()
                            ),
                        ));
                    }
                }
                _ => buffer.push_str(&text),
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
                if let Context::List(ctx) = context {
                    if ctx.is_ordered {
                        if ctx.spread && child == l.children.first().unwrap() {
                            buffer.push('\n');
                        }
                        buffer.push(' ');
                    }
                }
                to_md(
                    child,
                    buffer,
                    &Context::List(ListContext {
                        nesting_level,
                        is_ordered: l.ordered,
                        num_item: start,
                        spread: l.spread,
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
                    buffer.push_str(&format!("{}.", ctx.num_item));
                } else {
                    buffer.push_str(options.list.sign_style.as_str());
                }
                buffer.push_str(&" ".repeat(options.list.num_spaces_after_list_marker as usize));
                if let Some(is_checked) = li.checked {
                    match is_checked {
                        true => buffer.push_str("[x] "),
                        false => buffer.push_str("[ ] "),
                    }
                }
                if li.children.is_empty() {
                    // Special case when there's a list item without content
                    // e.g.:
                    //   - First
                    //   -
                    //   - Third
                    // Since we're relying on "\n" to be in the Paragraph node
                    // and there's none we have to add it manually here
                    buffer.push('\n');
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
                Context::List(ctx) => {
                    // Add horizontal padding to align code block with list
                    let padding_left = if ctx.is_ordered {
                        format!("\n{}", "    ".repeat(ctx.nesting_level + 1))
                    } else {
                        format!("\n{}", "   ".repeat(ctx.nesting_level + 1))
                    };
                    let mut code_block = format!("\n```{}\n{}\n```\n", syntax_highlight, c.value);
                    code_block = code_block.replace('\n', &padding_left);
                    code_block = remove_trailing_newline_and_space(&code_block);
                    code_block.push('\n');
                    buffer.push_str(&code_block);
                }
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
            match options.strong.style {
                StrongStyle::Asterisk => buffer.push_str("**"),
                StrongStyle::Underscore => buffer.push_str("__"),
            }
            for child in &s.children {
                to_md(child, buffer, context, source, options);
            }
            match options.strong.style {
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
            if is_auto_or_bare_link(l) {
                buffer.push('<');
                buffer.push_str(&l.url);
                buffer.push('>');
            } else {
                buffer.push('[');
                for child in &l.children {
                    if let Node::Link(sub_link) = child {
                        // Although not explicitly prohibited, this is kinda a blank spot in CommonMark.
                        // Technically, it is valid to have "[http://google.com](http://github.com)".
                        // This will result in parser creating two nested links "Link { Link { Text }, Text }".
                        // If we'll render it as-is it will result in a broken Markdown because
                        // it will became "[[http://google.com](http://google.com)](http://github.com)".
                        // Every time we format it will add another layer of nesting.
                        // We want to avoid it and render inner link as a plain text.
                        for child in &sub_link.children {
                            to_md(child, buffer, context, source, options);
                        }
                    } else {
                        to_md(child, buffer, context, source, options);
                    }
                }
                buffer.push(']');
                buffer.push_str(&format!("({}", &l.url.clone().as_str()));
                if let Some(title) = &l.title {
                    buffer.push_str(&format!(" \"{}\"", &title));
                }
                buffer.push(')');
            }
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
                                spread: ctx.spread,
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
                                spread: ctx.list_ctx.spread,
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
            if let Some(label) = &d.label {
                buffer.push_str(&format!("[{}]: {}", label, d.url));
            } else {
                buffer.push_str(&format!("[{}]: {}", d.identifier, d.url));
            }
            if let Some(title) = &d.title {
                buffer.push_str(&format!(" \"{}\"", &title));
            }
        }
        Node::LinkReference(lr) => {
            buffer.push('[');
            for child in &lr.children {
                match child {
                    Node::Text(_) => to_md(child, buffer, context, source, options),
                    Node::Link(l) => {
                        buffer.push_str(&l.url);
                    }
                    _ => to_md(child, buffer, context, source, options),
                }
            }
            buffer.push(']');
            if let Some(label) = &lr.label {
                buffer.push_str(&format!("[{}]", &label));
            } else {
                buffer.push_str(&format!("[{}]", &lr.identifier));
            }
        }
        Node::FootnoteReference(fr) => {
            buffer.push_str(&format!("[^{}]", &fr.identifier));
        }
        Node::FootnoteDefinition(fd) => {
            buffer.push_str(&format!("[^{}]: ", &fd.identifier));
            for child in &fd.children {
                if child == fd.children.first().unwrap() {
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
            let headers_cols_lengths = calculate_max_col_len(t, source);
            for child in &t.children {
                render_table_row(
                    child,
                    buffer,
                    context,
                    source,
                    options,
                    &headers_cols_lengths,
                );
                if child == t.children.first().unwrap() {
                    render_table_heading_separator(t, buffer, &headers_cols_lengths);
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
        Node::Yaml(yaml) => {
            buffer.push_str("---\n");
            buffer.push_str(&yaml.value);
            buffer.push_str("\n---\n");
        }
        _ => panic!("Unexpected node type {node:#?}"),
    }
}

/// Return formatted Markdown file
pub fn fmt_markdown(file: &common::MarkDownFile, config: &common::Config) -> common::MarkDownFile {
    log::debug!("Format {:#?} with config: {:#?}", &file.path, &config);

    log::debug!("Constructing formatting options");
    let fmt_options = FormattingOptions {
        list: ListOptions {
            sign_style: match config.style.unordered_lists {
                common::UnorderedListStyle::Consistent => {
                    log::debug!("Detect unordered list style in {:#?}", &file.path);

                    let ast = common::parse(&file.content).unwrap();
                    let mut unordered_list_items: Vec<&mdast::ListItem> = vec![];
                    common::for_each(&ast, |node| {
                        if let mdast::Node::List(l) = node {
                            if !l.ordered {
                                for child in &l.children {
                                    if let mdast::Node::ListItem(li) = child {
                                        unordered_list_items.push(li);
                                    }
                                }
                            }
                        }
                    });
                    if let Some(first_unordered_list_item) = unordered_list_items.first() {
                        log::debug!(
                            "First unordered list item: {:#?}",
                            &first_unordered_list_item
                        );

                        let offset_start = first_unordered_list_item
                            .position
                            .as_ref()
                            .unwrap()
                            .start
                            .offset;
                        let offset_end = first_unordered_list_item
                            .position
                            .as_ref()
                            .unwrap()
                            .end
                            .offset;
                        let first_unordered_list_item_str =
                            &file.content[offset_start..offset_end].trim();
                        log::debug!(
                            "Extracted first unordered list item from file: {:#?}",
                            &first_unordered_list_item_str
                        );

                        if first_unordered_list_item_str.starts_with('*') {
                            log::debug!("First unordered list item has asterisk style");
                            ListSignStyle::Asterisk
                        } else if first_unordered_list_item_str.starts_with('+') {
                            log::debug!("First unordered list item has plus style");
                            ListSignStyle::Plus
                        } else {
                            log::debug!("First unordered list style is neither asterisk nor plus, defaulting to dash");
                            ListSignStyle::Minus
                        }
                    } else {
                        log::debug!("File has no unordered lists, defaulting to dash");
                        ListSignStyle::Minus
                    }
                }
                common::UnorderedListStyle::Asterisk => ListSignStyle::Asterisk,
                common::UnorderedListStyle::Plus => ListSignStyle::Plus,
                common::UnorderedListStyle::Dash => ListSignStyle::Minus,
            },
            num_spaces_after_list_marker: config.style.num_spaces_after_list_marker.unwrap_or(1),
        },
        header: HeaderOptions {
            style: match config.style.headings {
                common::HeadingStyle::Consistent => {
                    log::debug!("Detecting heading style from the file {:#?}", &file.path);

                    let ast = common::parse(&file.content).unwrap();
                    let mut headings: Vec<&mdast::Heading> = vec![];
                    common::for_each(&ast, |node| {
                        if let mdast::Node::Heading(h) = node {
                            headings.push(h);
                        }
                    });
                    if let Some(first_heading) = headings.first() {
                        log::debug!("First heading in a file: {:#?}", &first_heading);

                        if is_heading_atx(first_heading, &file.content) {
                            log::debug!("First heading has ATX style");
                            HeaderStyle::Atx
                        } else {
                            log::debug!("First heading has SetExt style");
                            HeaderStyle::SetExt
                        }
                    } else {
                        log::debug!("There are no headings in a file, defaulting to ATX");
                        HeaderStyle::Atx
                    }
                }
                common::HeadingStyle::Setext => HeaderStyle::SetExt,
                common::HeadingStyle::Atx => HeaderStyle::Atx,
            },
        },
        strong: StrongOptions {
            style: match config.style.bold {
                common::BoldStyle::Consistent => {
                    log::debug!(
                        "Detecting bold(strong) style from the file {:#?}",
                        &file.path
                    );

                    let ast = common::parse(&file.content).unwrap();
                    let mut strong_els: Vec<&mdast::Strong> = vec![];
                    common::for_each(&ast, |node| {
                        if let mdast::Node::Strong(s) = node {
                            strong_els.push(s);
                        }
                    });
                    if let Some(first_strong_el) = strong_els.first() {
                        log::debug!("First bold(strong) el in a file: {:#?}", &first_strong_el);

                        if is_string_underscored(first_strong_el, &file.content) {
                            log::debug!("First bold(strong) el is underscored");
                            StrongStyle::Underscore
                        } else {
                            log::debug!(
                                "First bold(strong) not underscored, defaulting to the asterisk"
                            );
                            StrongStyle::Asterisk
                        }
                    } else {
                        log::debug!(
                            "There are no bold(strong) els in a file, defaulting to asterisk"
                        );
                        StrongStyle::Asterisk
                    }
                }
                common::BoldStyle::Asterisk => StrongStyle::Asterisk,
                common::BoldStyle::Underscore => StrongStyle::Underscore,
            },
        },
    };
    log::debug!("Formatting options: {:#?}", &fmt_options);

    let mut buffer: String = String::from("");
    let ast = common::parse(&file.content).unwrap();
    to_md(
        &ast,
        &mut buffer,
        &Context::Document,
        &file.content,
        &fmt_options,
    );

    log::debug!("Removing trailing newlines and spaces");
    buffer = remove_trailing_newline_and_space(&buffer);
    buffer.push('\n');

    common::MarkDownFile {
        path: file.path.clone(),
        content: buffer,
        issues: vec![],
    }
}

pub fn check_md_format(
    file: &common::MarkDownFile,
    config: &common::Config,
) -> Vec<common::CheckIssue> {
    let mut issues: Vec<common::CheckIssue> = vec![];
    let formatted = &fmt_markdown(file, config);
    if !file.content.eq(&formatted.content) {
        let mut issue = common::CheckIssueBuilder::default()
            .set_category(common::IssueCategory::Formatting)
            .set_severity(common::IssueSeverity::Error)
            .set_file_path(file.path.clone())
            .set_row_num_start(1)
            .set_row_num_end(file.content.lines().count())
            .set_col_num_start(1)
            .set_col_num_end(1)
            .set_offset_start(0)
            .set_offset_end(file.content.len())
            .set_message(String::from("Incorrect file formatting"));
        issue = issue.push_fix(&format!(
            "🧠 {}  {}",
            "Rationale".cyan(),
            "Consistent formatting makes it easier to understand a document"
        ));
        if !config.fmt.show_diff {
            issue = issue.push_fix(&format!("💡 {} Run \"checkmark fmt --check --show-diff {}\" to see a diff between expected formatting and your", "Suggestion".cyan(), &file.path));
        }
        issue = issue.push_fix(&format!(
            "🚀 {}   checkmark fmt {}",
            "Auto-fix".cyan(),
            &file.path
        ));
        if config.fmt.show_diff {
            issue = issue.push_fix(&format!(
                "📌 {}\n\n{}\n\n",
                "Diff".cyan(),
                get_diff(&file.content, &formatted.content)
            ));
        }
        issues.push(issue.build());
    }
    issues
}
