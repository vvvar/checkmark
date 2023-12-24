use markdown::mdast;
use markdown::mdast::Node;

// Document shall start with H1
// Document shall have sequent headers

fn starts_with_header(node: &mdast::Node) -> bool {
    match node {
        Node::Root(r) => {
            if let Some(Node::Heading(h)) = r.children.first() {
                h.depth == 1
            } else {
                false
            }
        }
        _ => false,
    }
}

fn extract_headings(node: &mdast::Node, headings: &mut Vec<mdast::Heading>) {
    match node {
        Node::Root(r) => {
            for child in &r.children {
                extract_headings(child, headings);
            }
        }
        Node::Heading(heading) => {
            headings.push(heading.clone());
            for child in &heading.children {
                extract_headings(child, headings);
            }
        }
        Node::Text(_) => {
            // End of tree
        }
        Node::Paragraph(p) => {
            for child in &p.children {
                extract_headings(child, headings);
            }
        }
        Node::List(l) => {
            for child in &l.children {
                extract_headings(child, headings);
            }
        }
        Node::ListItem(li) => {
            for child in &li.children {
                extract_headings(child, headings);
            }
        }
        Node::Code(_) => {
            // End of tree
        }
        Node::InlineCode(_) => {
            // End of tree
        }
        Node::Emphasis(e) => {
            for child in &e.children {
                extract_headings(child, headings);
            }
        }
        Node::Strong(s) => {
            for child in &s.children {
                extract_headings(child, headings);
            }
        }
        Node::Delete(d) => {
            for child in &d.children {
                extract_headings(child, headings);
            }
        }
        Node::Break(_) => {
            // End of tree
        }
        Node::Link(l) => {
            for child in &l.children {
                extract_headings(child, headings);
            }
        }
        Node::Image(_) => {
            // End of tree
        }
        Node::BlockQuote(b) => {
            for child in &b.children {
                extract_headings(child, headings);
            }
        }
        Node::ThematicBreak(_) => {
            // End of tree
        }
        Node::Html(_) => {
            // End of tree
        }
        Node::ImageReference(_) => {
            // End of tree
        }
        Node::Definition(_) => {
            // End of tree
        }
        Node::LinkReference(_) => {
            // End of tree
        }
        Node::FootnoteReference(_) => {
            // End of tree
        }
        Node::FootnoteDefinition(f) => {
            for child in &f.children {
                extract_headings(child, headings);
            }
        }
        Node::Table(t) => {
            for child in &t.children {
                extract_headings(child, headings);
            }
        }
        Node::TableCell(tc) => {
            for child in &tc.children {
                extract_headings(child, headings);
            }
        }
        Node::TableRow(tr) => {
            for child in &tr.children {
                extract_headings(child, headings);
            }
        }
        _ => panic!("Unexpected node type {node:#?}"),
    }
}

fn has_consequent_headers(node: &mdast::Node) -> bool {
    let mut headings: Vec<mdast::Heading> = vec![];
    extract_headings(node, &mut headings);
    let mut last_heading_level = 0;
    for heading in headings {
        if heading.depth >= last_heading_level {
            last_heading_level = heading.depth;
        } else {
            return false;
        }
    }
    true
}

fn lint_md_ast(node: &mdast::Node, _is_in_block_quote: bool) {
    match node {
        Node::Root(r) => {
            for child in &r.children {
                lint_md_ast(child, _is_in_block_quote);
            }
        }
        Node::Heading(heading) => {
            for child in &heading.children {
                lint_md_ast(child, _is_in_block_quote);
            }
        }
        Node::Text(_) => {
            // End of tree
        }
        Node::Paragraph(p) => {
            for child in &p.children {
                lint_md_ast(child, _is_in_block_quote);
            }
        }
        Node::List(l) => {
            for child in &l.children {
                lint_md_ast(child, _is_in_block_quote);
            }
        }
        Node::ListItem(li) => {
            for child in &li.children {
                lint_md_ast(child, _is_in_block_quote);
            }
        }
        Node::Code(_) => {
            // End of tree
        }
        Node::InlineCode(_) => {
            // End of tree
        }
        Node::Emphasis(e) => {
            for child in &e.children {
                lint_md_ast(child, _is_in_block_quote);
            }
        }
        Node::Strong(s) => {
            for child in &s.children {
                lint_md_ast(child, _is_in_block_quote);
            }
        }
        Node::Delete(d) => {
            for child in &d.children {
                lint_md_ast(child, _is_in_block_quote);
            }
        }
        Node::Break(_) => {
            // End of tree
        }
        Node::Link(l) => {
            for child in &l.children {
                lint_md_ast(child, _is_in_block_quote);
            }
        }
        Node::Image(_) => {
            // End of tree
        }
        Node::BlockQuote(b) => {
            for child in &b.children {
                lint_md_ast(child, true);
            }
        }
        Node::ThematicBreak(_) => {
            // End of tree
        }
        Node::Html(_) => {
            // End of tree
        }
        Node::ImageReference(_) => {
            // End of tree
        }
        Node::Definition(_) => {
            // End of tree
        }
        Node::LinkReference(_) => {
            // End of tree
        }
        Node::FootnoteReference(_) => {
            // End of tree
        }
        Node::FootnoteDefinition(f) => {
            for child in &f.children {
                lint_md_ast(child, _is_in_block_quote);
            }
        }
        Node::Table(t) => {
            for child in &t.children {
                lint_md_ast(child, _is_in_block_quote);
            }
        }
        Node::TableCell(tc) => {
            for child in &tc.children {
                lint_md_ast(child, _is_in_block_quote);
            }
        }
        Node::TableRow(tr) => {
            for child in &tr.children {
                lint_md_ast(child, _is_in_block_quote);
            }
        }
        _ => panic!("Unexpected node type {node:#?}"),
    }
}

/// Return formatted Markdown file
pub fn lint(file: &common::MarkDownFile) -> Vec<common::CheckIssue> {
    let ast = markdown::to_mdast(&file.content, &markdown::ParseOptions::gfm()).unwrap();
    lint_md_ast(&ast, false);
    starts_with_header(&ast);
    has_consequent_headers(&ast);
    vec![]
}
