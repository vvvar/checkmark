use checkmark_lint_common::*;
use checkmark_lint_macro::*;

use common::ast::{try_cast_to_code, BfsIterator};

#[rule(
    requirement = "Code blocks should not have all lines starting with a dollar sign",
    rationale = "It is easier to copy/paste and less noisy if the dollar signs are omitted when they are not needed",
    documentation = "https://github.com/DavidAnson/markdownlint/blob/v0.32.1/doc/md014.md",
    additional_links = ["https://cirosantilli.com/markdown-style-guide#dollar-signs-in-shell-code"],
    is_fmt_fixable = false,
)]
fn md014(ast: &Node, _: &MarkDownFile, _: &Config) -> Vec<Violation> {
    BfsIterator::from(ast)
        .filter_map(|n| try_cast_to_code(n))
        .filter(|c| is_all_lines_starts_with_dollar_sign(c))
        .map(to_issue)
        .collect()
}

fn is_all_lines_starts_with_dollar_sign(code: &Code) -> bool {
    code.value.lines().all(|line| line.starts_with('$'))
}

fn to_issue(code: &Code) -> Violation {
    ViolationBuilder::default()
        .message("All lines in a code block starts with dollar sign")
        .assertion("Expected to have a command output, got all lines with a dollar sign")
        .push_fix("The dollar signs are unnecessary in this situation, and should not be included")
        .position(&code.position)
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[rule_test(markdown = "# H1

```
$ ls
$ cat foo
$ less bar
```")]
    fn detects_dollar_only_code_block_without_output(
        ast: &Node,
        file: &MarkDownFile,
        config: &Config,
    ) {
        assert_eq!(
            vec![ViolationBuilder::default()
            .message("All lines in a code block starts with dollar sign")
            .assertion("Expected to have a command output, got all lines with a dollar sign")
            .push_fix("The dollar signs are unnecessary in this situation, and should not be included")
                .position(&Some(Position::new(3, 1, 6, 7, 4, 39)))
                .build(),],
            MD014.check(ast, file, config)
        );
    }
}
