use checkmark_lint_common::*;
use checkmark_lint_macro::*;

use regex::Regex;

#[rule(
    requirement="Heading style should be consistent",
    rationale="Consistent style makes it easier to understand a document",
    documentation="https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md003.md",
    additional_links=["https://www.markdownguide.org/basic-syntax/#headings"],
    is_fmt_fixable=true
)]
fn md003(ast: &Node, file: &MarkDownFile, config: &Config) -> Vec<Violation> {
    let style = HeadingStyle::from(config);

    let headings = common::ast::BfsIterator::from(ast)
        .filter_map(|n| common::ast::try_cast_to_heading(n))
        .collect::<Vec<&Heading>>();

    let preferred_style = match style {
        HeadingStyle::Consistent => get_first_heading_style(&headings, &file.content),
        HeadingStyle::Atx => HeadingStyle::Atx,
        HeadingStyle::SetExt => HeadingStyle::SetExt,
    };

    headings
        .iter()
        .filter_map(|h| {
            if get_heading_style(h, &file.content).ne(&preferred_style) {
                let mut violation = ViolationBuilder::default();
                // To have more descriptive messages, we're giving
                // more precise messages based on heading style
                if style.eq(&HeadingStyle::Consistent) {
                    violation = violation
                        .message("Inconsistent headings style")
                        .assertion(&format!("First heading in this file is {:#?}, but this one is {:#?}", preferred_style.as_str(), get_heading_style(h, &file.content).as_str()));
                } else {
                    violation = violation
                        .message("Wrong heading style")
                        .assertion(&format!("Expected {:#?}, got {:#?}", style.as_str(), get_heading_style(h, &file.content).as_str()));
                }
                Some(violation
                    .push_fix(&format!("Change heading style to {:#?}", preferred_style.as_str()))
                    .push_fix("Alternatively, you can enforce specific heading style via either \"headings\" option from the \"[style]\" section in config file or via \"--style-headings\" CLI option")
                    .position(&h.position)
                    .build())
            } else {
                None
            }
        })
        .collect::<Vec<Violation>>()
}

#[derive(Debug, Clone, PartialEq)]
pub enum HeadingStyle {
    Consistent,
    Atx,
    SetExt,
}

impl HeadingStyle {
    pub fn as_str(&self) -> &str {
        match self {
            HeadingStyle::Consistent => "consistent",
            HeadingStyle::Atx => "ATX",
            HeadingStyle::SetExt => "SetExt",
        }
    }
}

impl From<&Config> for HeadingStyle {
    fn from(config: &Config) -> Self {
        match config.style.headings {
            common::HeadingStyle::Consistent => HeadingStyle::Consistent,
            common::HeadingStyle::Atx => HeadingStyle::Atx,
            common::HeadingStyle::Setext => HeadingStyle::SetExt,
        }
    }
}

// Detect whether heading is ATX or SetExt style
fn get_heading_style(h: &Heading, source: &str) -> HeadingStyle {
    let offset_start = h.position.as_ref().unwrap().start.offset;
    let offset_end = h.position.as_ref().unwrap().end.offset;
    let heading = source.get(offset_start..offset_end).unwrap_or("");
    // Pattern: starts with zero or more whitespace followed by one or more hash characters
    // This shall capture ATX headings on root and nested level
    if Regex::new(r"\s*#+").unwrap().is_match(heading) {
        HeadingStyle::Atx
    } else {
        HeadingStyle::SetExt
    }
}

// When first heading exist, detects it's style
// Otherwise - fallback to Atx
fn get_first_heading_style(headings: &[&Heading], source: &str) -> HeadingStyle {
    if let Some(h) = headings.first() {
        get_heading_style(h, source)
    } else {
        HeadingStyle::Atx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rule_test(markdown = "# H1

H2
-----")]
    fn detect_inconsistent_heading_style(ast: &Node, file: &MarkDownFile, config: &mut Config) {
        config.style.headings = common::HeadingStyle::Consistent;
        assert_eq!(
            vec![ViolationBuilder::default()
                .message("Inconsistent headings style")
                .assertion("First heading in this file is \"ATX\", but this one is \"SetExt\"")
                .position(&Some(Position::new(3, 1, 6, 4, 6, 14)))
                .set_fixes(vec![
                    String::from("Change heading style to \"ATX\""),
                    String::from("Alternatively, you can enforce specific heading style via either \"headings\" option from the \"[style]\" section in config file or via \"--style-headings\" CLI option")
                ])
                .build()],
            MD003.check(ast, file, config),
        );
    }

    #[rule_test(markdown = "# H1")]
    fn detect_atx_when_setext_is_forced(ast: &Node, file: &MarkDownFile, config: &mut Config) {
        config.style.headings = common::HeadingStyle::Setext;
        assert_eq!(
            vec![ViolationBuilder::default()
                .message("Wrong heading style")
                .assertion("Expected \"SetExt\", got \"ATX\"")
                .position(&Some(Position::new(1, 1, 0, 1, 5, 4)))
                .set_fixes(vec![
                    String::from("Change heading style to \"SetExt\""),
                    String::from("Alternatively, you can enforce specific heading style via either \"headings\" option from the \"[style]\" section in config file or via \"--style-headings\" CLI option")
                ])
                .build()],
            MD003.check(ast, file, config),
        );
    }

    #[rule_test(markdown = "H1
===========")]
    fn detect_setext_when_atx_is_forced(ast: &Node, file: &MarkDownFile, config: &mut Config) {
        config.style.headings = common::HeadingStyle::Atx;
        assert_eq!(
            vec![ViolationBuilder::default()
                .message("Wrong heading style")
                .assertion("Expected \"ATX\", got \"SetExt\"")
                .position(&Some(Position::new(1, 1, 0, 2, 12, 14)))
                .set_fixes(vec![
                    String::from("Change heading style to \"ATX\""),
                    String::from("Alternatively, you can enforce specific heading style via either \"headings\" option from the \"[style]\" section in config file or via \"--style-headings\" CLI option")
                ])
                .build()],
            MD003.check(ast, file, config),
        );
    }
}
