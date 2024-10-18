use markdown::mdast::{Blockquote, Code, Heading, Html, Link, List, ListItem, Node, Strong, Text};

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
        Some(self.values[self.index - 1])
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

/// Return the strong node if the provided generic node is a string.
/// Meant to be used in a filter_map statement to filter strong nodes
/// from a generic AST.
/// Example:
/// ```
/// # use markdown::mdast::{Strong, Node};
/// let ast = common::ast::parse("**Strong**").unwrap();
/// let strong_elements = common::ast::BfsIterator::from(&ast)
///                  .filter_map(|n| common::ast::try_cast_to_strong(n))
///                  .collect::<Vec<&Strong>>();
/// ```
pub fn try_cast_to_strong(node: &Node) -> Option<&Strong> {
    match node {
        Node::Strong(e) => Some(e),
        _ => None,
    }
}

/// Return the text node if the provided generic node is an text node.
/// Meant to be used in a filter_map statement to filter text nodes
/// from a generic AST.
/// Example:
/// ```
/// # use markdown::mdast::{Text, Node};
/// let ast = common::ast::parse("Text").unwrap();
/// let text_nodes = common::ast::BfsIterator::from(&ast)
///                     .filter_map(|n| common::ast::try_cast_to_text(n))
///                     .collect::<Vec<&Text>>();
/// ```
pub fn try_cast_to_text(node: &Node) -> Option<&Text> {
    match node {
        Node::Text(e) => Some(e),
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

/// Return the link node if the provided generic node is a link.
/// Meant to be used in a filter_map statement to filter link nodes
/// from a generic AST.
/// Example:
/// ```
/// # use markdown::mdast::{Link, Node};
/// let ast = common::ast::parse("[Text](http://example.com)").unwrap();
/// let links = common::ast::BfsIterator::from(&ast)
///                  .filter_map(|n| common::ast::try_cast_to_link(n))
///                  .collect::<Vec<&Link>>();
/// ```
pub fn try_cast_to_link(node: &Node) -> Option<&Link> {
    match node {
        Node::Link(e) => Some(e),
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

/// Return true only when given node is ListItem
pub fn is_list_item(node: &Node) -> bool {
    matches!(node, Node::ListItem(_))
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
/// # use markdown::mdast::{Blockquote, Node};
/// let ast = common::ast::parse("> Block Quote").unwrap();
/// let block_quotes = common::ast::BfsIterator::from(&ast)
///                  .filter_map(|n| common::ast::try_cast_to_block_quote(n))
///                  .collect::<Vec<&Blockquote>>();
/// ```
pub fn try_cast_to_block_quote(node: &Node) -> Option<&Blockquote> {
    match node {
        Node::Blockquote(e) => Some(e),
        _ => None,
    }
}

/// Parse Markdown file into an AST
pub fn parse(source: &str) -> Result<Node, markdown::message::Message> {
    let options = markdown::ParseOptions {
        constructs: markdown::Constructs {
            frontmatter: true,
            ..markdown::Constructs::gfm()
        },
        ..markdown::ParseOptions::gfm()
    };
    markdown::to_mdast(source, &options)
}
