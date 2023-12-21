/// Represents single markdown file under check
#[derive(Debug, PartialEq, Clone)]
pub struct MarkDownFile {
    pub path: String,
    pub content: String,
    pub issues: Vec<CheckIssue>,
}

/// Represents type of issue that occurred while check
#[derive(Debug, PartialEq, Clone)]
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
#[derive(Debug, PartialEq, Clone)]
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
#[derive(Debug, PartialEq, Clone)]
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

/// Builder for CheckIssue struct
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
    pub fn set_category(mut self, category: IssueCategory) -> Self {
        self.category = Some(category);
        self
    }

    pub fn set_severity(mut self, severity: IssueSeverity) -> Self {
        self.severity = Some(severity);
        self
    }

    pub fn set_file_path(mut self, file_path: String) -> Self {
        self.file_path = Some(file_path);
        self
    }

    pub fn set_row_num_start(mut self, row_num_start: usize) -> Self {
        self.row_num_start = Some(row_num_start);
        self
    }

    pub fn set_row_num_end(mut self, row_num_end: usize) -> Self {
        self.row_num_end = Some(row_num_end);
        self
    }

    pub fn set_col_num_start(mut self, col_num_start: usize) -> Self {
        self.col_num_start = Some(col_num_start);
        self
    }

    pub fn set_col_num_end(mut self, col_num_end: usize) -> Self {
        self.col_num_end = Some(col_num_end);
        self
    }

    pub fn set_offset_start(mut self, offset_start: usize) -> Self {
        self.offset_start = Some(offset_start);
        self
    }

    pub fn set_offset_end(mut self, offset_end: usize) -> Self {
        self.offset_end = Some(offset_end);
        self
    }

    pub fn set_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    /// Replace fix
    pub fn set_fixes(mut self, fixes: Vec<String>) -> Self {
        self.fixes = fixes;
        self
    }

    /// Push fix
    pub fn push_fix(mut self, fix: &str) -> Self {
        self.fixes.push(fix.to_string());
        self
    }

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

    pub fn default() -> Self {
        CheckIssueBuilder {
            category: None,
            severity: None,
            file_path: None,
            row_num_start: None,
            row_num_end: None,
            col_num_start: None,
            col_num_end: None,
            offset_start: None,
            offset_end: None,
            message: None,
            fixes: vec![],
        }
    }
}

impl CheckIssue {
    /// Convert CheckIssue to the sarif-compatible result
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
        // #[allow(unused)]
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

        serde_sarif::sarif::ResultBuilder::default()
            .level("error")
            .locations(vec![location])
            // .analysis_target(artifact_location)
            .message(message)
            .fixes(fixes)
            .build()
            .unwrap()
    }
}

// Collect nodes of type from provided AST
pub fn for_each<'a>(ast: &'a markdown::mdast::Node, mut f: impl FnMut(&'a markdown::mdast::Node)) {
    let mut stack: Vec<&markdown::mdast::Node> = vec![];
    stack.push(&ast);
    loop {
        if let Some(current) = stack.pop() {
            f(&current);
            match current.children() {
                Some(children) => {
                    for child in children.iter().rev() {
                        stack.push(child);
                    }
                }
                None => {}
            }
        } else {
            break;
        }
    }
}

pub fn filter<'a>(
    ast: &'a markdown::mdast::Node,
    mut predicate: impl FnMut(&'a markdown::mdast::Node) -> bool,
) -> Vec<&'a markdown::mdast::Node> {
    let mut stack: Vec<&markdown::mdast::Node> = vec![];
    for_each(&ast, |node| {
        if predicate(node) {
            stack.push(node);
        }
    });
    return stack;
}

pub fn filter_text_nodes<'a>(ast: &'a markdown::mdast::Node) -> Vec<&'a markdown::mdast::Text> {
    let mut text_nodes: Vec<&markdown::mdast::Text> = vec![];
    for_each(&ast, |node| match node {
        markdown::mdast::Node::Text(t) => text_nodes.push(t),
        _ => {}
    });
    return text_nodes;
}

pub fn filter_paragraph_nodes<'a>(
    ast: &'a markdown::mdast::Node,
) -> Vec<&'a markdown::mdast::Paragraph> {
    let mut p_nodes: Vec<&markdown::mdast::Paragraph> = vec![];
    for_each(&ast, |node| match node {
        markdown::mdast::Node::Paragraph(t) => p_nodes.push(t),
        _ => {}
    });
    return p_nodes;
}

/// Find index of substring in source string
pub fn find_index(source: &str, sub_str: &str) -> std::ops::Range<usize> {
    let mut index_start = 0;
    let mut index_end = source.len();
    log::debug!("Searching {:#?}", &sub_str);
    if let Some(index) = source.find(&sub_str) {
        log::debug!("Found exact index: {:#?}", &index);
        index_start = index;
        index_end = index_start + &sub_str.len();
    } else {
        log::debug!("Unable to find exact index, trying to guess");
        for line in source.lines() {
            if strsim::sorensen_dice(&sub_str, &line) > 0.5 {
                index_start = source.find(&line).unwrap();
                index_end = source.len();
                log::debug!(
                    "Found the best guess line on index {:#?}:\n{:#?}",
                    &index_start,
                    &line
                );
            }
        }
    }
    return std::ops::Range {
        start: index_start,
        end: index_end,
    };
}