mod args;
mod checker;
mod grammar;
mod link_checker;
mod linter;
mod md;
mod prettier;
mod spell_checker;

use colored::Colorize;
use env_logger;
use log::warn;
use markdown;
use markdown::mdast;
use markdown::mdast::AlignKind;
use md::list;
use mdast::Node;
use predicates::reflection::PredicateReflection;
use spinners::{Spinner, Spinners};
use std::fmt;
use std::fs;
use similar::{ChangeTag, TextDiff};
use console::Style;

#[derive(Debug)]
struct AppError {
    message: String,
}

impl std::error::Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Print GCC style error: https://www.gnu.org/prep/standards/html_node/Errors.html#Errors
fn print_gcc_style_error(issues: &Vec<checker::Issue>) {
    for issue in issues {
        eprintln!(
            "{}: {}: {}: {}",
            &issue.file_path.bold(),
            "error".bright_red().bold(),
            &issue.category.bold(),
            &issue.description
        );
        match &issue.issue_in_code {
            Some(issue_in_code) => {
                eprintln!("{}", issue_in_code);
            }
            None => {}
        }
        for suggestion in &issue.suggestions {
            eprintln!(
                "   {}{} {}",
                "Suggestion".blue(),
                ":".blue(),
                &suggestion.blue()
            );
        }
        eprintln!();
    }
}

/// Print custom style error
#[allow(dead_code)]
fn print_custom_style_error(issues: &Vec<checker::Issue>) {
    for issue in issues {
        println!(
            "❗️{}: {}: {}: {}",
            &issue.file_path.bold(),
            "Error".bright_red().bold(),
            &issue.category.bold(),
            &issue.description
        );
        for suggestion in &issue.suggestions {
            println!("└→💡{}: {}", "Suggestion".yellow(), &suggestion);
        }
    }
}

fn render_list_node(
    node: &mdast::Node,
    mut buffer: &mut String,
    nesting_level: usize,
    is_ordered: bool,
    num_item: u32,
    is_in_block_quote: bool
) {
    match node {
        Node::List(list) => {
            let mut start = if list.start.is_some() {
                list.start.unwrap()
            } else {
                0
            };
            for child in &list.children {
                render_list_node(&child, &mut buffer, nesting_level + 1, is_ordered, start, is_in_block_quote);
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
                dbg!(&child);
                render_list_node(&child, &mut buffer, nesting_level, is_ordered, num_item, is_in_block_quote);
                buffer.push_str("\n");
            }
        }
        Node::Paragraph(paragraph) => {
            for child in &paragraph.children {
                render_list_node(&child, &mut buffer, nesting_level, is_ordered, num_item, is_in_block_quote);
            }
        }
        Node::Text(text) => {
            buffer.push_str(&text.value.replace("\n", &format!("\n   ")));
        }
        _ => travel_md_ast(&node, &mut buffer, is_in_block_quote)
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
            dbg!(&l);
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
                buffer.push_str(&format!(
                    "```{}\n{}\n```\n",
                    c.lang.as_ref().unwrap_or(&String::new()),
                    c.value
                ).replace("\n", &format!("\n> ")));
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
            // dbg!(&b);
            // buffer.push_str("> ");
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
                    for align in &t.align  {
                        match align {
                            AlignKind::Left => buffer.push_str(" :-- |"),
                            AlignKind::Right => buffer.push_str(" --: |"),
                            AlignKind::Center => buffer.push_str(" :-: |"),
                            AlignKind::None => buffer.push_str(" --- |")
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
        },
        Node::TableRow(tr) => {
            buffer.push_str("| ");
            for child in &tr.children {
                travel_md_ast(&child, &mut buffer, is_in_block_quote);
            }
            buffer.push_str("\n");
        },
        _ => panic!("Unexpected node type {node:#?}"),
    }
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    env_logger::init();
    let arguments = args::read();
    let mut has_any_issue = false;
    for file in md::list(&arguments.root) {
        let original = fs::read_to_string(&file).unwrap();
        let mut buffer: String = String::from("");
        let ast = markdown::to_mdast(&original, &markdown::ParseOptions::gfm()).unwrap();
        // dbg!(&ast);
        travel_md_ast(&ast, &mut buffer, false);
        // println!("{}", &buffer);

        fs::write("/Users/vvoinov/Documents/repos/md-checker/rendered.md", &buffer).expect("Cannot save result md");

        let diff = TextDiff::from_lines(
            &original,
            &buffer,
        );
        for op in diff.ops() {
            for change in diff.iter_changes(op) {
                let (sign, style) = match change.tag() {
                    ChangeTag::Delete => ("-", Style::new().red()),
                    ChangeTag::Insert => ("+", Style::new().green()),
                    ChangeTag::Equal => (" ", Style::new()),
                };
                print!("{}{}", style.apply_to(sign).bold(), style.apply_to(change));
            }
        }

        // if arguments.autoformat {
        //     if prettier::auto_format(&file) {
        //         println!("✅{}: {}", "Auto-format".cyan().bold(), &file);
        //     } else {
        //         println!("❎{}: {}", "Auto-format".cyan().bold(), &file);
        //     }
        // } else {
        //     let mut sp = Spinner::new(
        //         Spinners::Point,
        //         format!("{}: {}", "Check".cyan().bold(), &file).into(),
        //     );
        //     if let Ok(issues) = checker::check(&file).await {
        //         if issues.is_empty() {
        //             sp.stop_with_symbol("✅");
        //         } else {
        //             sp.stop_with_symbol("❌");
        //             print_gcc_style_error(&issues);
        //             has_any_issue = true;
        //         }
        //     } else {
        //         sp.stop_with_symbol("❌");
        //         has_any_issue = true;
        //         warn!("Unexpected error while checking a file");
        //     }
        // }
    }
    if has_any_issue {
        Err(AppError {
            message: String::from("Some files failed a check"),
        })
    } else {
        Ok(())
    }
}
