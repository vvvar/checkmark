use markdown;
use markdown::mdast;
use markdown::mdast::{AlignKind, Node};
// use similar::{ChangeTag, TextDiff};

fn render_list_node(
    node: &mdast::Node,
    mut buffer: &mut String,
    nesting_level: usize,
    is_ordered: bool,
    num_item: u32,
    is_in_block_quote: bool,
) {
    match node {
        Node::List(list) => {
            let mut start = if list.start.is_some() {
                list.start.unwrap()
            } else {
                0
            };
            for child in &list.children {
                render_list_node(
                    &child,
                    &mut buffer,
                    nesting_level + 1,
                    is_ordered,
                    start,
                    is_in_block_quote,
                );
                start += 1;
            }
        }
        Node::ListItem(list_item) => {
            buffer.push_str(&"   ".repeat(nesting_level));
            if is_ordered {
                buffer.push_str(&format!("{}. ", num_item));
            } else {
                buffer.push_str("+ ");
            }
            for child in &list_item.children {
                if &child != &list_item.children.first().unwrap() {
                    buffer.push_str("   ");
                }
                render_list_node(
                    &child,
                    &mut buffer,
                    nesting_level,
                    is_ordered,
                    num_item,
                    is_in_block_quote,
                );
                buffer.push_str("\n");
            }
        }
        Node::Paragraph(paragraph) => {
            for child in &paragraph.children {
                render_list_node(
                    &child,
                    &mut buffer,
                    nesting_level,
                    is_ordered,
                    num_item,
                    is_in_block_quote,
                );
            }
        }
        Node::Text(text) => {
            buffer.push_str(&text.value.replace("\n", &format!("\n   ")));
        }
        _ => travel_md_ast(&node, &mut buffer, is_in_block_quote),
    }
}

fn travel_md_ast(node: &mdast::Node, mut buffer: &mut String, is_in_block_quote: bool) {
    match node {
        Node::Root(r) => {
            for child in &r.children {
                travel_md_ast(&child, &mut buffer, is_in_block_quote);
                buffer.push_str("\n");
            }
        }
        Node::Heading(heading) => {
            buffer.push_str("#".repeat(heading.depth as usize).as_str());
            buffer.push_str(" ");
            for child in &heading.children {
                travel_md_ast(&child, &mut buffer, is_in_block_quote);
            }
            buffer.push_str("\n");
        }
        Node::Text(t) => {
            if is_in_block_quote {
                buffer.push_str(&t.value.replace("\n", &format!("\n> ")));
            } else {
                buffer.push_str(&t.value);
            }
        }
        Node::Paragraph(p) => {
            for child in &p.children {
                travel_md_ast(&child, &mut buffer, is_in_block_quote);
            }
            buffer.push_str("\n");
        }
        Node::List(l) => {
            let mut start = if l.start.is_some() {
                l.start.unwrap()
            } else {
                0
            };
            for child in &l.children {
                if is_in_block_quote && &child != &l.children.first().unwrap() {
                    buffer.push_str("> ");
                }
                render_list_node(&child, &mut buffer, 0, l.ordered, start, is_in_block_quote);
                start += 1;
            }
        }
        Node::ListItem(_) => {
            // Not needed since we're rendering through render_list_node
        }
        Node::Code(c) => {
            if is_in_block_quote {
                buffer.push_str(
                    &format!(
                        "```{}\n{}\n```\n",
                        c.lang.as_ref().unwrap_or(&String::new()),
                        c.value
                    )
                    .replace("\n", &format!("\n> ")),
                );
            } else {
                buffer.push_str(&format!(
                    "```{}\n{}\n```\n",
                    c.lang.as_ref().unwrap_or(&String::new()),
                    c.value
                ));
            }
        }
        Node::InlineCode(c) => {
            buffer.push_str(&format!("`{}`", &c.value));
        }
        Node::Emphasis(e) => {
            buffer.push_str("*");
            for child in &e.children {
                travel_md_ast(&child, &mut buffer, is_in_block_quote);
            }
            buffer.push_str("*");
        }
        Node::Strong(s) => {
            buffer.push_str("**");
            for child in &s.children {
                travel_md_ast(&child, &mut buffer, is_in_block_quote);
            }
            buffer.push_str("**");
        }
        Node::Delete(d) => {
            buffer.push_str("~~");
            for child in &d.children {
                travel_md_ast(&child, &mut buffer, is_in_block_quote);
            }
            buffer.push_str("~~");
        }
        Node::Break(_) => {
            buffer.push_str("\n");
        }
        Node::Link(l) => {
            buffer.push_str("[");
            for child in &l.children {
                travel_md_ast(&child, &mut buffer, is_in_block_quote);
            }
            buffer.push_str("]");
            buffer.push_str(&format!("({})", &l.url.clone().as_str()));
        }
        Node::Image(i) => {
            buffer.push_str(&format!("![{}]({})", &i.alt, &i.url));
        }
        Node::BlockQuote(b) => {
            for child in &b.children {
                buffer.push_str("> ");
                if &child != &b.children.first().unwrap() {
                    buffer.push_str("\n> ");
                }
                travel_md_ast(&child, &mut buffer, true);
            }
        }
        Node::ThematicBreak(_) => {
            buffer.push_str("----\n");
        }
        Node::Html(h) => {
            buffer.push_str(&h.value);
        }
        Node::ImageReference(ir) => {
            buffer.push_str(&format!("![{}][{}]", ir.alt, ir.identifier));
        }
        Node::Definition(d) => {
            buffer.push_str(&format!("[{}]: {}", d.identifier, d.url));
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
                    travel_md_ast(&child, &mut buffer, is_in_block_quote);
                } else {
                    let mut tmp_buffer = String::from("");
                    travel_md_ast(&child, &mut tmp_buffer, is_in_block_quote);
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
                    travel_md_ast(&child, &mut buffer, is_in_block_quote);
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
                    travel_md_ast(&child, &mut buffer, is_in_block_quote);
                }
            }
        }
        Node::TableCell(tc) => {
            for child in &tc.children {
                travel_md_ast(&child, &mut buffer, is_in_block_quote);
            }
            buffer.push_str(" | ");
        }
        Node::TableRow(tr) => {
            buffer.push_str("| ");
            for child in &tr.children {
                travel_md_ast(&child, &mut buffer, is_in_block_quote);
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
    travel_md_ast(&ast, &mut buffer, false);
    match buffer.strip_suffix("\n") {
        Some(stripped) => common::MarkDownFile {
            path: file.path.clone(),
            content: String::from(stripped),
        },
        None => common::MarkDownFile {
            path: file.path.clone(),
            content: buffer,
        }
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

    // let artifact_location = serde_sarif::sarif::ArtifactLocationBuilder::default()
    //     .uri(String::from(&file.path))
    //     .build()
    //     .unwrap();

    // let message = serde_sarif::sarif::MessageBuilder::default()
    //     .text("Formatting is incorrect")
    //     .build()
    //     .unwrap();

    // let physical_location = serde_sarif::sarif::PhysicalLocationBuilder::default()
    //     .artifact_location(artifact_location.clone())
    //     .build()
    //     .unwrap();

    // let location = serde_sarif::sarif::LocationBuilder::default()
    //     .physical_location(physical_location)
    //     .build()
    //     .unwrap();

    // let mut fixes: Vec<serde_sarif::sarif::Fix> = vec![];

    // let diff = TextDiff::from_lines(&file.content, &formatted_file.content);
    // for op in diff.ops() {
    // let mut replacement = serde_sarif::sarif::ReplacementBuilder::default();

    // let mut text_to_delete = String::from("");
    // let mut text_to_insert = String::from("");

    // let mut delete_line_number: usize = 0;

    // for change in diff.iter_changes(op) {
    // Old code from example that prints a diff
    // let (sign, style) = match change.tag() {
    //     ChangeTag::Delete => ("-", Style::new().red()),
    //     ChangeTag::Insert => ("+", Style::new().green()),
    //     ChangeTag::Equal => ("", Style::new()),
    // };
    // format!("{}{}", style.apply_to(sign).bold(), style.apply_to(change));
    // print!("{}{}", style.apply_to(sign).bold(), style.apply_to(change));

    // match change.tag() {
    //     ChangeTag::Delete => {
    //         text_to_delete += &change.value();
    //         if let Some(num) = change.old_index() {
    //             delete_line_number = num;
    //         };
    //     }
    //     ChangeTag::Insert => text_to_insert += &change.value(),
    //     ChangeTag::Equal => {}
    // };
    // }

    // if !text_to_delete.is_empty() {
    //     let artifact_content: serde_sarif::sarif::ArtifactContent =
    //         serde_sarif::sarif::ArtifactContentBuilder::default()
    //             .text(text_to_delete)
    //             .build()
    //             .unwrap();
    //     let region = serde_sarif::sarif::RegionBuilder::default()
    //         .snippet(artifact_content)
    //         .start_line(delete_line_number as i64)
    //         .build()
    //         .unwrap();
    //     replacement.deleted_region(region);

    //     let artifact_content = serde_sarif::sarif::ArtifactContentBuilder::default()
    //         .text(text_to_insert)
    //         .build()
    //         .unwrap();
    //     replacement.inserted_content(artifact_content);

    //     let replacements = vec![replacement.build().unwrap()];
    //     let changes = vec![serde_sarif::sarif::ArtifactChangeBuilder::default()
    //         .replacements(replacements)
    //         .artifact_location(artifact_location.clone())
    //         .build()
    //         .unwrap()];
    //     let fix = serde_sarif::sarif::FixBuilder::default()
    //         .artifact_changes(changes)
    //         .build()
    //         .unwrap();
    //     fixes.push(fix);
    // }
    // }
    // }

    // let result = serde_sarif::sarif::ResultBuilder::default()
    //     .locations(vec![location.clone()])
    //     .analysis_target(artifact_location.clone())
    //     .message(message)
    //     .fixes(fixes)
    //     .build()
    //     .unwrap();

    // return vec![result];
}
