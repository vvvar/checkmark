use crate::violation::{Violation, ViolationBuilder};
use common::{for_each, parse, MarkDownFile};
use markdown::mdast::{Code, Node};

fn violation_builder() -> ViolationBuilder {
    ViolationBuilder::default()
        .code("MD014")
        .message("Dollar signs used before commands without showing output")
        .doc_link("https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md014.md")
        .rationale("It is easier to copy/paste and less noisy if the dollar signs are omitted when they are not needed")
        .push_additional_link("https://cirosantilli.com/markdown-style-guide#dollar-signs-in-shell-code")
        .push_fix("Remove unnecessary blank line")
        .is_fmt_fixable(true)
}

fn is_code_start_always_with_dollar(code: &Code) -> bool {
    code.value.lines().all(|line| line.starts_with("$"))
}

fn to_issue(code: &Code) -> Violation {
    violation_builder()
        .position(&code.position)
        .push_fix(&format!(
            "The dollar signs are unnecessary in this situation, and should not be included"
        ))
        .build()
}

pub fn md014_dollar_sign_before_command_without_output(file: &MarkDownFile) -> Vec<Violation> {
    log::debug!("[MD014] File: {:#?}", &file.path);
    let ast = parse(&file.content).unwrap();
    let mut code_blocks: Vec<&Code> = vec![];
    for_each(&ast, |node| {
        if let Node::Code(c) = node {
            code_blocks.push(c);
        }
    });
    code_blocks
        .iter()
        .filter(|c| is_code_start_always_with_dollar(c))
        .map(|c| to_issue(c))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    pub fn md014() {
        let file = common::MarkDownFile {
            path: String::from("this/is/a/dummy/path/to/a/file.md"),
            content: "# H1

```
$ ls
$ cat foo
$ less bar
```"
            .to_string(),
            issues: vec![],
        };

        assert_eq!(
            vec![violation_builder()
                .position(&Some(markdown::unist::Position::new(3, 1, 6, 7, 4, 39)))
                .build(),],
            md014_dollar_sign_before_command_without_output(&file)
        );
    }
}
