#[derive(Debug, Clone, PartialEq)]
pub struct Violation {
    /// Assertion message in a format "Expected X, got X".
    pub assertion: String,
    /// List of possible fixes for the violation.
    pub fixes: Vec<String>,
    /// Message that describes what went wrong.
    pub message: String,
    /// Position where issue was found.
    pub position: markdown::unist::Position,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ViolationBuilder {
    pub assertion: Option<String>,
    pub fixes: Vec<String>,
    pub message: Option<String>,
    pub position: Option<markdown::unist::Position>,
}

impl ViolationBuilder {
    pub fn assertion(mut self, assertion: &str) -> ViolationBuilder {
        self.assertion = Some(assertion.to_owned());
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

    pub fn set_fixes(mut self, fixes: Vec<String>) -> ViolationBuilder {
        self.fixes = fixes.clone();
        self
    }

    pub fn push_fix(mut self, fix: &str) -> ViolationBuilder {
        self.fixes.push(fix.to_owned());
        self
    }

    pub fn build(self) -> Violation {
        Violation {
            assertion: self
                .assertion
                .expect("ViolationBuilder.assertion is not set"),
            message: self.message.expect("ViolationBuilder.message is not set"),
            position: self.position.expect("ViolationBuilder.position is not set"),
            fixes: self.fixes,
        }
    }
}
