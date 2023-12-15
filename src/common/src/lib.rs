/// Represents single markdown file under check
#[derive(Debug, PartialEq)]
pub struct MarkDownFile {
    pub path: String,
    pub content: String,
}

/// Represents type of issue that occurred while check
#[derive(Debug, PartialEq)]
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

/// Represents issue found by checking markdown file
#[derive(Debug, PartialEq)]
pub struct CheckIssue {
    /// Category of the issue
    pub category: IssueCategory,
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
    /// Message that describes an issue
    pub message: String,
    /// Possible fixes
    pub fixes: Vec<String>,
}

/// Builder for CheckIssue struct
pub struct CheckIssueBuilder {
    pub category: Option<IssueCategory>,
    pub file_path: Option<String>,
    pub row_num_start: Option<usize>,
    pub row_num_end: Option<usize>,
    pub col_num_start: Option<usize>,
    pub col_num_end: Option<usize>,
    pub message: Option<String>,
    pub fixes: Vec<String>,
}

impl CheckIssueBuilder {
    pub fn set_category(mut self, category: IssueCategory) -> Self {
        self.category = Some(category);
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
            file_path: self.file_path.expect("File path has not been set, use set_file_path() method before building an instance"),
            row_num_start: self.row_num_start.expect("Row number start has not been set, use set_row_num_start() method before building an instance"),
            row_num_end: self.row_num_end.expect("Row number end has not been set, use set_row_num_end() method before building an instance"),
            col_num_start: self.col_num_start.expect("Col number start has not been set, use set_col_num_start() method before building an instance"),
            col_num_end: self.col_num_end.expect("Col end start has not been set, use set_col_num_end() method before building an instance"),
            message: self.message.expect("Message has not been set, use set_message() method before building an instance"),
            fixes: self.fixes,
        }
    }

    pub fn default() -> Self {
        CheckIssueBuilder {
            category: None,
            file_path: None,
            row_num_start: None,
            row_num_end: None,
            col_num_start: None,
            col_num_end: None,
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
        #[allow(unused)]
        for issue_fix in &self.fixes {
            let artifact_content = serde_sarif::sarif::ArtifactContentBuilder::default()
                .text("")
                .build()
                .unwrap();
            let region = serde_sarif::sarif::RegionBuilder::default()
                .snippet(artifact_content)
                .start_line(self.row_num_start as i64)
                .build()
                .unwrap();

            let artifact_content = serde_sarif::sarif::ArtifactContentBuilder::default()
                .text("")
                .build()
                .unwrap();

            let replacement = serde_sarif::sarif::ReplacementBuilder::default()
                .deleted_region(region)
                .inserted_content(artifact_content)
                .build()
                .unwrap();

            let changes = vec![serde_sarif::sarif::ArtifactChangeBuilder::default()
                .replacements(vec![replacement])
                .artifact_location(artifact_location.clone())
                .build()
                .unwrap()];

            let fix = serde_sarif::sarif::FixBuilder::default()
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
