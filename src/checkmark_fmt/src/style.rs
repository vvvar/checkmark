#[derive(Debug, Clone, Default, PartialEq)]
#[allow(dead_code)]
pub enum ListSignStyle {
    /// "+"
    Plus,
    /// "-"
    #[default]
    Minus,
    /// "*"
    Asterisk,
}

impl ListSignStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Asterisk => "*",
            Self::Plus => "+",
            Self::Minus => "-",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum HeaderStyle {
    /// "#"-prefixed header
    #[default]
    Atx,
    /// Header that followed by "======" or "-----"(depends on depth)
    SetExt,
}

#[derive(Debug, Clone, Default, PartialEq)]
#[allow(dead_code)]
pub enum StrongStyle {
    /// Wrapped with "__"
    Underscore,
    /// Wrapped with "**"
    #[default]
    Asterisk,
}

#[derive(Debug, Clone, Default, PartialEq)]
#[allow(dead_code)]
pub struct HeaderOptions {
    pub style: HeaderStyle,
}

#[derive(Debug, Clone, Default, PartialEq)]
#[allow(dead_code)]
pub struct ListOptions {
    pub sign_style: ListSignStyle,
    pub num_spaces_after_list_marker: u8,
}

#[derive(Debug, Clone, Default, PartialEq)]
#[allow(dead_code)]
pub struct StrongOptions {
    pub style: StrongStyle,
}

#[derive(Debug, Clone, Default, PartialEq)]
#[allow(dead_code)]
pub struct CodeBlockOptions {
    pub default_language: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FormattingOptions {
    pub header: HeaderOptions,
    pub list: ListOptions,
    pub strong: StrongOptions,
    pub code_block: CodeBlockOptions,
}

impl FormattingOptions {
    pub fn from(config: &common::Config, source: &common::MarkDownFile) -> Self {
        Self {
            list: ListOptions {
                sign_style: match config.style.unordered_lists {
                    common::UnorderedListStyle::Consistent => {
                        log::debug!("Detect unordered list style in {:#?}", &source.path);
                        let ast = common::ast::parse(&source.content).unwrap();
                        let unordered_list_items = common::ast::BfsIterator::from(&ast)
                            .filter_map(|n| common::ast::try_cast_to_list(n))
                            .filter(|l| l.ordered == false) // We only care about unordered lists
                            .flat_map(|l| {
                                // Get all list items from them
                                let mut items = vec![];
                                for child in &l.children {
                                    if let markdown::mdast::Node::ListItem(li) = child {
                                        items.push(li);
                                    }
                                }
                                items
                            })
                            .collect::<Vec<_>>();
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
                                &source.content[offset_start..offset_end].trim();
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
                num_spaces_after_list_marker: config
                    .style
                    .num_spaces_after_list_marker
                    .unwrap_or(1),
            },
            header: HeaderOptions {
                style: match config.style.headings {
                    common::HeadingStyle::Consistent => {
                        log::debug!("Detecting heading style from the file {:#?}", &source.path);
                        let ast = common::ast::parse(&source.content).unwrap();
                        let headings = common::ast::BfsIterator::from(&ast)
                            .filter_map(|n| common::ast::try_cast_to_heading(n))
                            .collect::<Vec<&markdown::mdast::Heading>>();
                        if let Some(first_heading) = headings.first() {
                            log::debug!("First heading in a file: {:#?}", &first_heading);

                            if is_heading_atx(first_heading, &source.content) {
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
                            &source.path
                        );

                        let ast = common::ast::parse(&source.content).unwrap();
                        let strong_els = common::ast::BfsIterator::from(&ast)
                            .filter_map(|n| common::ast::try_cast_to_strong(n))
                            .collect::<Vec<&markdown::mdast::Strong>>();
                        if let Some(first_strong_el) = strong_els.first() {
                            log::debug!("First bold(strong) el in a file: {:#?}", &first_strong_el);

                            if is_string_underscored(first_strong_el, &source.content) {
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
            code_block: CodeBlockOptions {
                default_language: config
                    .style
                    .default_code_block_language
                    .clone()
                    .unwrap_or(String::from("text")),
            },
        }
    }
}

/// It is possible to pass single "~" and it wold be interpreted
/// as a delete which shall be interpreted as a superscript
/// https://github.com/markdown-it/markdown-it-sup
pub fn is_superscript(d: &markdown::mdast::Delete, source: &str) -> bool {
    let mut superscript = false;
    if let Some(position) = &d.position {
        let slice = &source[position.start.offset..position.end.offset];
        superscript = slice.matches('~').count() == 2;
    }
    superscript
}

/// There are two types of string(bold) - underscore(__) and asterisk(**)
/// We can determine a type of it from the original file
/// TODO - use it when we will support context-based detection of style
#[allow(dead_code)]
pub fn is_string_underscored(d: &markdown::mdast::Strong, source: &str) -> bool {
    let mut underscored = false;
    if let Some(position) = &d.position {
        let slice = &source[position.start.offset..position.end.offset];
        underscored = slice.matches("__").count() == 2;
    }
    underscored
}

/// There are two types of string(bold) - underscore(__) and asterisk(**)
/// We can determine a type of it from the original file
/// TODO - use it when we will support context-based detection of style
#[allow(dead_code)]
pub fn is_heading_atx(d: &markdown::mdast::Heading, source: &str) -> bool {
    let mut atx = false;
    if let Some(position) = &d.position {
        let slice = &source[position.start.offset..position.end.offset];
        atx = slice.contains('#');
    }
    atx
}

/// Returns true when link it either auto or bare
/// Autolink: "<http://example.com>"
/// Bare link: "http://example.com"
pub fn is_auto_or_bare_link(l: &markdown::mdast::Link) -> bool {
    // Detect this structure
    //    Link {
    //       children: [ Text {} ]
    //       title: None
    //    }
    if l.title.is_none() && l.children.len() == 1 {
        if let markdown::mdast::Node::Text(t) = l.children.first().unwrap() {
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
