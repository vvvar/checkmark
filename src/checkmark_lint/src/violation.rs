#[derive(Debug, Clone)]
pub struct Violation {
    /// Code of the violation(e.g. MD001, MD009, etc.)
    pub code: String,
    /// Message text of the violation
    pub message: String,
    /// Position where issue was found
    pub position: markdown::unist::Position,
    /// Link to the documentation of the violation
    pub doc_link: String,
    // Explanation why this rule matters
    pub rationale: String,
    /// List of possible fixes for the violation
    pub fixes: Vec<String>,
    /// List of additional useful links such as link to Markdown reference
    pub additional_links: Vec<String>,
    /// Is it possible to fix the violation automatically with fmt?
    pub is_fmt_fixable: bool,
}

/// Custom Eq implementation since we
/// won't compare the message and the fixes
/// to simplify testing
impl PartialEq for Violation {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
            && self.message == other.message
            && self.position == other.position
            && self.doc_link == other.doc_link
            && self.rationale == other.rationale
            && self.is_fmt_fixable == other.is_fmt_fixable
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ViolationBuilder {
    pub code: Option<String>,
    pub message: Option<String>,
    pub position: Option<markdown::unist::Position>,
    pub doc_link: Option<String>,
    pub rationale: Option<String>,
    pub fixes: Vec<String>,
    pub additional_links: Vec<String>,
    pub is_fmt_fixable: bool,
}

impl ViolationBuilder {
    pub fn code(mut self, code: &str) -> ViolationBuilder {
        self.code = Some(code.to_owned());
        self
    }

    pub fn message(mut self, message: &str) -> ViolationBuilder {
        self.message = Some(message.to_owned());
        self
    }

    pub fn position(mut self, position: &Option<markdown::unist::Position>) -> ViolationBuilder {
        self.position = Some(position.clone().unwrap());
        self
    }

    pub fn doc_link(mut self, doc_link: &str) -> ViolationBuilder {
        self.doc_link = Some(doc_link.to_owned());
        self
    }

    pub fn rationale(mut self, rationale: &str) -> ViolationBuilder {
        self.rationale = Some(rationale.to_owned());
        self
    }

    pub fn set_fixes(mut self, fixes: Vec<String>) -> ViolationBuilder {
        self.fixes = fixes.clone();
        self
    }

    pub fn push_fix(mut self, fix: &str) -> ViolationBuilder {
        self.fixes.push(fix.to_owned());
        self
    }

    pub fn push_additional_link(mut self, additional_link: &str) -> ViolationBuilder {
        self.additional_links.push(additional_link.to_owned());
        self
    }

    pub fn is_fmt_fixable(mut self, is_fmt_fixable: bool) -> ViolationBuilder {
        self.is_fmt_fixable = is_fmt_fixable;
        self
    }

    pub fn build(self) -> Violation {
        Violation {
            code: self.code.expect("ViolationBuilder.code is not set"),
            message: self.message.expect("ViolationBuilder.message is not set"),
            position: self.position.expect("ViolationBuilder.position is not set"),
            doc_link: self.doc_link.expect("ViolationBuilder.doc_link is not set"),
            rationale: self
                .rationale
                .expect("ViolationBuilder.rationale is not set"),
            fixes: self.fixes,
            additional_links: self.additional_links,
            is_fmt_fixable: self.is_fmt_fixable,
        }
    }
}
