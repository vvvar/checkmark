use markdown::mdast::{Node, Paragraph, Text};

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
