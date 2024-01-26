use common::{Config, MarkDownFile};
use markdown::{to_html_with_options, Options};

use crate::themes::Themes;

pub fn md_to_html(file: &MarkDownFile, config: &Config) -> String {
    let html = to_html_with_options(
        &file.content,
        &Options {
            compile: markdown::CompileOptions {
                allow_dangerous_html: true,
                allow_dangerous_protocol: true,
                ..markdown::CompileOptions::gfm()
            },
            ..markdown::Options::gfm()
        },
    )
    .expect("Unable to parse Markdown file")
    .replace(".md", ".html");
    use html_editor::operation::*;
    let css = html_editor::Node::Text(Themes::create().get(&config.rendering.theme));
    let style: html_editor::Node = html_editor::Node::new_element("style", vec![], vec![css]);
    let head: html_editor::Node = html_editor::Node::new_element("head", vec![], vec![style]);
    let content = html_editor::Node::Text(html);
    let body: html_editor::Node = html_editor::Node::new_element("body", vec![], vec![content]);
    let document = html_editor::Node::new_element("html", vec![], vec![head, body]);
    document.html()
}
