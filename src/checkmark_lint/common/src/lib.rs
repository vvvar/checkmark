pub use common::{Config, MarkDownFile};
pub use markdown::mdast::*;
pub use markdown::unist::*;

use url::Url;

pub trait Rule
where
    Self: Default,
{
    fn metadata(&self) -> Metadata;

    fn check(&self, ast: &Node, file: &MarkDownFile, config: &Config) -> Vec<Violation>;
}

pub struct Metadata {
    pub additional_links: Vec<Url>,
    pub code: &'static str,
    pub documentation: Url,
    pub is_fmt_fixable: bool,
    pub rationale: &'static str,
    pub requirement: &'static str,
}

#[derive(Debug)]
pub struct Context {
    pub ast: Node,
    pub file: MarkDownFile,
    pub config: Config,
}

impl Context {
    pub fn new(file: &MarkDownFile, config: &Config) -> Self {
        let ast = common::ast::parse(&file.content).unwrap();
        Self {
            ast,
            file: file.clone(),
            config: config.clone(),
        }
    }
}

#[macro_export]
macro_rules! make_test_context {
    ($ast:ident, $file:ident, $config:ident, $context:ident, $markdown_str: tt) => {
        let $file = MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: String::from($markdown_str),
            issues: vec![],
        };
        let $ast = common::ast::parse(&$file.content).unwrap();
        let $config = Config::default();
        let $context = Context {
            ast: &$ast,
            file: &$file,
            config: &$config,
        };
    };
}

// #[cfg(test)]
// #[macro_export]
// macro_rules! make_test_context_with_custom_config {
//     ($ast:ident, $file:ident, $config:ident, $context:ident, $markdown_str: tt) => {
//         let $file = MarkDownFile {
//             path: String::from("this/is/a/dummy/path/to/a/file.md"),
//             content: String::from($markdown_str),
//             issues: vec![],
//         };
//         let $ast = common::ast::parse(&$file.content).unwrap();
//         let $context = Context {
//             ast: &$ast,
//             file: &$file,
//             config: &$config,
//         };
//     };
// }

// // #[cfg(test)]
// #[macro_export]
// macro_rules! make_rule_test {
//     ($rule_under_test:ident, $test_case_name:ident, $expected_violations:expr, $input_markdown_str: tt) => {
//         paste::item! {
//             #[test]
//             fn [< $test_case_name:lower:snake >] () {
//                 use super::*;
//                 make_test_context!(ast, file, config, context, $input_markdown_str);
//                 pretty_assertions::assert_eq!($rule_under_test::default().check(context), Vec::from($expected_violations));
//             }
//         }
//     };
// }

// #[cfg(test)]
// pub(crate) use make_test_context;

// #[cfg(test)]
// pub(crate) use make_test_context_with_custom_config;

// #[cfg(test)]
// pub(crate) use make_rule_test;

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

// #[macro_export]
// macro_rules! rule {
//     (
//         meta {
//             code: $rule_code:tt,
//             requirement: $requirement:tt,
//             rationale: $rationale:tt,
//             documentation: $documentation:tt,
//             additional_links: [
//                 $($additional_links:tt)*
//             ],
//             is_fmt_fixable: $is_fmt_fixable:tt
//         }

//         $block:block

//         $(test {
//             id: $test_case_id:tt,
//             expect: $expected_violations:expr,
//             input: $markdown_str:tt
//         })*

//         $(test_with_custom_config {
//             id: $config_test_case_id:tt,
//             config_builder: $config_builder:expr,
//             expect: $config_expected_violations:expr,
//             input: $config_markdown_str:tt
//         })*

//     ) => {
//         paste::item! {
//             #[derive(Debug, Default)]
//             pub struct [<$rule_code:upper>];

//             impl Rule for [<$rule_code:upper>] {
//                 fn metadata(&self) -> Metadata {
//                     Metadata {
//                         code: $rule_code,
//                         requirement: $requirement,
//                         rationale: $rationale,
//                         documentation: make_url!($documentation),
//                         additional_links: vec![
//                             $(
//                                 make_url!($additional_links)
//                             )*
//                         ],
//                         is_fmt_fixable: $is_fmt_fixable,
//                     }
//                 }

//                 fn check(&self, context: Context) -> Vec<Violation> {
//                     $block
//                 }
//             }

//             #[cfg(test)]
//             mod tests {
//                 $(
//                     #[test]
//                     fn [<$rule_code:lower _ $test_case_id:lower:snake>]() {
//                         use super::*;
//                         make_test_context!(ast, file, config, context, $markdown_str);
//                         pretty_assertions::assert_eq!([<$rule_code:upper>]::default().check(context), Vec::from($expected_violations));
//                     }
//                 )*

//                 $(
//                     #[test]
//                     fn [<$rule_code:lower _ $config_test_case_id:lower:snake >]() {
//                         use super::*;
//                         let config = $config_builder();
//                         make_test_context_with_custom_config!(ast, file, config, context, $config_markdown_str);
//                         pretty_assertions::assert_eq!([<$rule_code:upper>]::default().check(context), Vec::from($config_expected_violations));
//                     }
//                 )*
//             }
//         }
//     };
// }

// pub(crate) use rule;
