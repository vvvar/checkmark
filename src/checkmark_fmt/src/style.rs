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
pub struct FormattingOptions {
    pub list_sign_style: ListSignStyle,
    pub header_style: HeaderStyle,
    pub strong_style: StrongStyle,
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
