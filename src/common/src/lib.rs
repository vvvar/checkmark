//! # Checkmark Common
//!
//! `common` is a crate that collects various functionality shared between other internal crates.
//!
//! This crate provides the following main structures:
//!
//! ## `MarkDownFile`
//!
//! Represents a single markdown file under check. It contains the following fields:
//!
//! - `path`: The path to the markdown file.
//! - `content`: The content of the markdown file.
//! - `issues`: A vector of `CheckIssue` that occurred while checking the file.
//!
//! ## `IssueCategory`
//!
//! Represents the type of issue that occurred while checking the markdown file. It is an enum with the following variants:
//!
//! - `Formatting`: Issue with how the document has been formatted.
//! - `Linting`: Issue indicates violation of some linting rule.
//! - `LinkChecking`: Issue reaching a link from a file (either unreachable local file or URL).
//! - `Spelling`: Issue with word spelling.
//! - `Grammar`: Issue with grammar.
//!
//! This crate provides functionality to check a markdown file for these issues and report them for further action.
//!
//! `# Panics`
//! CheckIssueBuilder::build() panics if any of the required fields has not been set.
//! CheckIssue::to_sarif_result() panics if any of the required fields has not been set.

/// Represents single markdown file under check
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MarkDownFile {
    pub path: String,
    pub content: String,
    pub issues: Vec<CheckIssue>,
}

/// Represents type of issue that occurred while check
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IssueCategory {
    /// Issue with how document has been formatted
    Formatting,
    /// Issue indicates violation of some linting rule
    Linting,
    /// Issue reaching a link from a file(either unreachable local file or URL)
    LinkChecking,
    /// Issue with word spelling
    Spelling,
    /// Grammar
    Grammar,
    /// Documentation review suggestion
    Review,
}

/// Represent how critical issue is
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IssueSeverity {
    /// Highest level, bug
    Bug,
    /// Highest level, but no necessarily a bug
    Error,
    /// Warning, could be skipped but it is highly advisable to fix it
    Warning,
    /// Just a note, completely optional, lowest level
    Note,
    /// Hint
    Help,
}

/// Represents issue found by checking markdown file
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CheckIssue {
    /// Category of the issue
    pub category: IssueCategory,
    /// How critical the issue is
    pub severity: IssueSeverity,
    /// Path to the file that has an issue
    pub file_path: String,
    /// Line number where issue starts
    pub row_num_start: usize,
    /// Line number where issue ends
    pub row_num_end: usize,
    /// Column number where issue starts
    pub col_num_start: usize,
    /// Column number where issue ends
    pub col_num_end: usize,
    /// Character index from where issue begins
    pub offset_start: usize,
    /// Character index from where issue ends
    pub offset_end: usize,
    /// Message that describes an issue
    pub message: String,
    /// Possible fixes
    pub fixes: Vec<String>,
}

/// Builder for `CheckIssue` struct
#[derive(Default)]
pub struct CheckIssueBuilder {
    pub category: Option<IssueCategory>,
    pub severity: Option<IssueSeverity>,
    pub file_path: Option<String>,
    pub row_num_start: Option<usize>,
    pub row_num_end: Option<usize>,
    pub col_num_start: Option<usize>,
    pub col_num_end: Option<usize>,
    pub offset_start: Option<usize>,
    pub offset_end: Option<usize>,
    pub message: Option<String>,
    pub fixes: Vec<String>,
}

impl CheckIssueBuilder {
    #[inline]
    pub const fn set_category(mut self, category: IssueCategory) -> Self {
        self.category = Some(category);
        self
    }

    #[inline]
    pub const fn set_severity(mut self, severity: IssueSeverity) -> Self {
        self.severity = Some(severity);
        self
    }

    #[inline]
    pub fn set_file_path(mut self, file_path: String) -> Self {
        self.file_path = Some(file_path);
        self
    }

    #[inline]
    pub const fn set_row_num_start(mut self, row_num_start: usize) -> Self {
        self.row_num_start = Some(row_num_start);
        self
    }

    #[inline]
    pub const fn set_row_num_end(mut self, row_num_end: usize) -> Self {
        self.row_num_end = Some(row_num_end);
        self
    }

    #[inline]
    pub const fn set_col_num_start(mut self, col_num_start: usize) -> Self {
        self.col_num_start = Some(col_num_start);
        self
    }

    #[inline]
    pub const fn set_col_num_end(mut self, col_num_end: usize) -> Self {
        self.col_num_end = Some(col_num_end);
        self
    }

    #[inline]
    pub const fn set_offset_start(mut self, offset_start: usize) -> Self {
        self.offset_start = Some(offset_start);
        self
    }

    #[inline]
    pub const fn set_offset_end(mut self, offset_end: usize) -> Self {
        self.offset_end = Some(offset_end);
        self
    }

    #[inline]
    pub fn set_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    /// Replace fix
    #[inline]
    pub fn set_fixes(mut self, fixes: Vec<String>) -> Self {
        self.fixes = fixes;
        self
    }

    /// Push fix
    #[inline]
    pub fn push_fix(mut self, fix: &str) -> Self {
        self.fixes.push(fix.to_owned());
        self
    }

    #[inline]
    pub fn build(self) -> CheckIssue {
        CheckIssue {
            category: self.category.expect("Category has not been set, use set_category() method before building an instance"),
            severity: self.severity.expect("Issue severity was not set, use set_severity() method before building an instance"),
            file_path: self.file_path.expect("File path has not been set, use set_file_path() method before building an instance"),
            row_num_start: self.row_num_start.expect("Row number start has not been set, use set_row_num_start() method before building an instance"),
            row_num_end: self.row_num_end.expect("Row number end has not been set, use set_row_num_end() method before building an instance"),
            col_num_start: self.col_num_start.expect("Col number start has not been set, use set_col_num_start() method before building an instance"),
            col_num_end: self.col_num_end.expect("Col end start has not been set, use set_col_num_end() method before building an instance"),
            offset_start: self.offset_start.expect("Issue offset start has not been set, use set_offset_start() method before building an instance"),
            offset_end: self.offset_end.expect("Issue offset end has not been set, use set_offset_end() method before building an instance"),
            message: self.message.expect("Message has not been set, use set_message() method before building an instance"),
            fixes: self.fixes,
        }
    }
}

impl CheckIssue {
    /// Convert `CheckIssue` to the sarif-compatible result
    #[inline]
    pub fn to_sarif_result(&self) -> serde_sarif::sarif::Result {
        let artifact_location = serde_sarif::sarif::ArtifactLocationBuilder::default()
            .uri(String::from(&self.file_path))
            .build()
            .unwrap();

        let region = serde_sarif::sarif::RegionBuilder::default()
            .start_line(self.row_num_start as i64)
            .end_line(self.row_num_end as i64)
            .start_column(self.col_num_start as i64)
            .end_column(self.col_num_end as i64)
            .build()
            .unwrap();

        let physical_location = serde_sarif::sarif::PhysicalLocationBuilder::default()
            .artifact_location(artifact_location.clone())
            .region(region)
            .build()
            .unwrap();

        let location = serde_sarif::sarif::LocationBuilder::default()
            .physical_location(physical_location)
            .build()
            .unwrap();

        let message = serde_sarif::sarif::MessageBuilder::default()
            .text(&self.message)
            .build()
            .unwrap();

        let mut fixes: Vec<serde_sarif::sarif::Fix> = vec![];
        for issue_fix in &self.fixes {
            let artifact_content = serde_sarif::sarif::ArtifactContentBuilder::default()
                .text(&issue_fix.clone())
                .build()
                .unwrap();

            let region = serde_sarif::sarif::RegionBuilder::default()
                .snippet(artifact_content.clone())
                .start_line(self.row_num_start as i64)
                .build()
                .unwrap();

            let replacement = serde_sarif::sarif::ReplacementBuilder::default()
                .deleted_region(region)
                .inserted_content(artifact_content.clone())
                .build()
                .unwrap();

            let changes = vec![serde_sarif::sarif::ArtifactChangeBuilder::default()
                .replacements(vec![replacement])
                .artifact_location(artifact_location.clone())
                .build()
                .unwrap()];

            let description = serde_sarif::sarif::MessageBuilder::default()
                .text(&self.message)
                .build()
                .unwrap();

            let fix = serde_sarif::sarif::FixBuilder::default()
                .description(description)
                .artifact_changes(changes)
                .build()
                .unwrap();

            fixes.push(fix);
        }

        let severity = match self.severity {
            IssueSeverity::Bug => "error",
            IssueSeverity::Error => "error",
            IssueSeverity::Warning => "warning",
            IssueSeverity::Note => "note",
            IssueSeverity::Help => "help",
        };
        let kind = match self.category {
            IssueCategory::Formatting => "formatting",
            IssueCategory::Linting => "linting",
            IssueCategory::LinkChecking => "links",
            IssueCategory::Spelling => "spelling",
            IssueCategory::Grammar => "grammar",
            IssueCategory::Review => "review",
        };
        serde_sarif::sarif::ResultBuilder::default()
            .level(severity)
            .kind(kind)
            .locations(vec![location])
            .message(message)
            .fixes(fixes)
            .build()
            .unwrap()
    }
}

// Collect nodes of type from provided AST
pub fn for_each<'node_lifetime>(
    ast: &'node_lifetime markdown::mdast::Node,
    mut f: impl FnMut(&'node_lifetime markdown::mdast::Node),
) {
    let mut stack: Vec<&markdown::mdast::Node> = vec![];
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

pub fn filter<'node_lifetime>(
    ast: &'node_lifetime markdown::mdast::Node,
    mut predicate: impl FnMut(&'node_lifetime markdown::mdast::Node) -> bool,
) -> Vec<&'node_lifetime markdown::mdast::Node> {
    let mut stack: Vec<&markdown::mdast::Node> = vec![];
    for_each(ast, |node| {
        if predicate(node) {
            stack.push(node);
        }
    });
    stack
}

pub fn filter_text_nodes(ast: &markdown::mdast::Node) -> Vec<&markdown::mdast::Text> {
    let mut text_nodes: Vec<&markdown::mdast::Text> = vec![];
    for_each(ast, |node| {
        if let markdown::mdast::Node::Text(t) = node {
            text_nodes.push(t)
        }
    });
    text_nodes
}

pub fn filter_paragraph_nodes(ast: &markdown::mdast::Node) -> Vec<&markdown::mdast::Paragraph> {
    let mut p_nodes: Vec<&markdown::mdast::Paragraph> = vec![];
    for_each(ast, |node| {
        if let markdown::mdast::Node::Paragraph(t) = node {
            p_nodes.push(t)
        }
    });
    p_nodes
}

/// Find index of substring in source string
pub fn find_index(source: &str, sub_str: &str) -> core::ops::Range<usize> {
    let mut index_start = 0;
    let mut index_end = source.len();
    log::debug!("Searching {:#?}", &sub_str);
    if let Some(index) = source.find(sub_str) {
        log::debug!("Found exact index: {:#?}", &index);
        index_start = index;
        index_end = sub_str.len() + index_start;
    } else {
        log::debug!("Unable to find exact index, trying to guess");
        for line in source.lines() {
            if strsim::sorensen_dice(sub_str, line) > 0.5 {
                index_start = source.find(line).unwrap();
                index_end = source.len();
                log::debug!(
                    "Found the best guess line on index {:#?}:\n{:#?}",
                    &index_start,
                    &line
                );
            }
        }
    }
    core::ops::Range {
        start: index_start,
        end: index_end,
    }
}
