use crate::violation::{Violation, ViolationBuilder};
use common::MarkDownFile;
use markdown::mdast::Heading;

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD001")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md001.md")
        .rationale("Headings represent the structure of a document and can be confusing when skipped - especially for accessibility scenarios")
        .push_additional_link("https://www.w3.org/WAI/tutorials/page-structure/headings/")
}

pub fn md001_heading_level_should_increment_by_one_level_at_time(
    file: &MarkDownFile,
) -> Vec<Violation> {
    log::debug!("[MD001] File: {:#?}", &file.path);
    let ast = common::ast::parse(&file.content).unwrap();
    let headings = common::ast::BfsIterator::from(&ast)
        .filter_map(|n| common::ast::try_cast_to_heading(n))
        .collect::<Vec<&Heading>>();

    headings
        .iter()
        .zip(headings.iter().skip(1))
        .filter(|(prev, current)| current.depth > prev.depth + 1)
        .map(|(prev, current)| violation_builder()
            .message(&format!("Heading level incremented by more then one level at a time. Expected {:#?} or less, got {:#?}", "#".repeat(prev.depth as usize + 1), "#".repeat(current.depth as usize)))
            .push_fix(&format!("Decrease the heading level so it will be {:#?}, {:#?} or less", "#".repeat(prev.depth as usize + 1), "#".repeat(prev.depth as usize)))
            .position(&current.position).build())
        .collect::<Vec<Violation>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use markdown::unist::Position;
    use pretty_assertions::assert_eq;

    #[test]
    fn md001() {
        let file = MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "# H1
        
## H2

#### H4

### H3

## H2

### H3"
                .to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![violation_builder()
                .message("Heading level incremented by more then one level at a time. Expected \"###\" or less, got \"####\"")
                .position(&Some(Position::new(5, 1, 21, 5, 8, 28)))
                .build()],
            md001_heading_level_should_increment_by_one_level_at_time(&file),
        );
    }
}
