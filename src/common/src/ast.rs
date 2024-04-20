use markdown::mdast::{BlockQuote, Code, Heading, Html, List, ListItem, Node, Paragraph, Text};

#[derive(Debug)]
pub struct BfsIterator<'a> {
    values: Vec<&'a Node>,
    index: usize,
}

impl<'a> BfsIterator<'a> {
    pub fn from(ast: &'a Node) -> Self {
        let mut bfs = Self {
            values: vec![],
            index: 0,
        };
        let mut stack: Vec<&Node> = vec![];
        stack.push(ast);
        while let Some(current) = stack.pop() {
            bfs.values.push(current);
            if let Some(children) = current.children() {
                for child in children.iter().rev() {
                    stack.push(child);
                }
            }
        }
        bfs
    }
}

impl<'a> Iterator for BfsIterator<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.values.len() {
            return None;
        }
        self.index += 1;
        Some(&self.values[self.index - 1])
    }
}

/// Return the heading node if the provided generic node is a heading
/// Meant to be used in a filter_map statement to filter heading nodes
/// from a generic AST
/// Example:
/// ```
/// # use markdown::mdast::{Heading, Node};
/// let ast = common::ast::parse("# Heading").unwrap();
/// let headings = common::ast::BfsIterator::from(&ast)
///                  .filter_map(|n| common::ast::try_cast_to_heading(n))
///                  .collect::<Vec<&Heading>>();
/// ```
pub fn try_cast_to_heading(node: &Node) -> Option<&Heading> {
    match node {
        Node::Heading(e) => Some(e),
        _ => None,
    }
}

/// Return the HTML node if the provided generic node is an HTML node.
/// Meant to be used in a filter_map statement to filter HTML nodes
/// from a generic AST.
/// Example:
/// ```
/// # use markdown::mdast::{Html, Node};
/// let ast = common::ast::parse("<span>Text</span>").unwrap();
/// let html_nodes = common::ast::BfsIterator::from(&ast)
///                  .filter_map(|n| common::ast::try_cast_to_html(n))
///                  .collect::<Vec<&Html>>();
/// ```
pub fn try_cast_to_html(node: &Node) -> Option<&Html> {
    match node {
        Node::Html(e) => Some(e),
        _ => None,
    }
}

/// Return the list node if the provided generic node is a list
/// Meant to be used in a filter_map statement to filter list nodes
/// from a generic AST
/// Example:
/// ```
/// # use markdown::mdast::{List, Node};
/// let ast = common::ast::parse("- Item\n\n- Item\n\n").unwrap();
/// let lists = common::ast::BfsIterator::from(&ast)
///                  .filter_map(|n| common::ast::try_cast_to_list(n))
///                  .collect::<Vec<&List>>();
/// ```
pub fn try_cast_to_list(node: &Node) -> Option<&List> {
    match node {
        Node::List(e) => Some(e),
        _ => None,
    }
}

/// Return the list item node if the provided generic node is a list item
/// Meant to be used in a filter_map statement to filter list item nodes
/// from a generic AST
/// Example:
/// ```
/// # use markdown::mdast::{ListItem, Node};
/// let ast = common::ast::parse("- Item\n\n- Item\n\n").unwrap();
/// let list_items = common::ast::BfsIterator::from(&ast)
///                  .filter_map(|n| common::ast::try_cast_to_list_item(n))
///                  .collect::<Vec<&ListItem>>();
/// ```
pub fn try_cast_to_list_item(node: &Node) -> Option<&ListItem> {
    match node {
        Node::ListItem(e) => Some(e),
        _ => None,
    }
}

/// Return the code node if the provided generic node is a code
/// Meant to be used in a filter_map statement to filter code nodes
/// from a generic AST
/// Example:
/// ```
/// # use markdown::mdast::{Code, Node};
/// let ast = common::ast::parse("   Code Block").unwrap();
/// let code_nodes = common::ast::BfsIterator::from(&ast)
///                  .filter_map(|n| common::ast::try_cast_to_code(n))
///                  .collect::<Vec<&Code>>();
/// ```
pub fn try_cast_to_code(node: &Node) -> Option<&Code> {
    match node {
        Node::Code(e) => Some(e),
        _ => None,
    }
}

/// Return the block quote node if the provided generic node is a block quote.
/// Meant to be used in a filter_map statement to filter block quote nodes
/// from a generic AST.
/// Example:
/// ```
/// # use markdown::mdast::{BlockQuote, Node};
/// let ast = common::ast::parse("> Block Quote").unwrap();
/// let block_quotes = common::ast::BfsIterator::from(&ast)
///                  .filter_map(|n| common::ast::try_cast_to_block_quote(n))
///                  .collect::<Vec<&BlockQuote>>();
/// ```
pub fn try_cast_to_block_quote(node: &Node) -> Option<&BlockQuote> {
    match node {
        Node::BlockQuote(e) => Some(e),
        _ => None,
    }
}

// Collect nodes of type from provided AST
pub fn for_each<'a>(ast: &'a Node, mut f: impl FnMut(&'a Node)) {
    let mut stack: Vec<&Node> = vec![];
    stack.push(ast);
    while let Some(current) = stack.pop() {
        f(current);
        if let Some(children) = current.children() {
            for child in children.iter().rev() {
                stack.push(child);
            }
        }
    }
}

pub fn filter<'a>(ast: &'a Node, mut p: impl FnMut(&'a Node) -> bool) -> Vec<&'a Node> {
    let mut stack: Vec<&Node> = vec![];
    for_each(ast, |node| {
        if p(node) {
            stack.push(node);
        }
    });
    stack
}

pub fn filter_text_nodes(ast: &Node) -> Vec<&Text> {
    let mut text_nodes: Vec<&Text> = vec![];
    for_each(ast, |node| {
        if let Node::Text(t) = node {
            text_nodes.push(t)
        }
    });
    text_nodes
}

pub fn filter_paragraph_nodes(ast: &Node) -> Vec<&Paragraph> {
    let mut p_nodes: Vec<&Paragraph> = vec![];
    for_each(ast, |node| {
        if let Node::Paragraph(t) = node {
            p_nodes.push(t)
        }
    });
    p_nodes
}

/// Parse Markdown file into an AST
pub fn parse(source: &str) -> Result<Node, String> {
    let options = markdown::ParseOptions {
        constructs: markdown::Constructs {
            frontmatter: true,
            ..markdown::Constructs::gfm()
        },
        ..markdown::ParseOptions::gfm()
    };
    markdown::to_mdast(source, &options)
}
