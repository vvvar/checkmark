mod md001_heading_level_should_increment_by_one_level_at_time;
use md001_heading_level_should_increment_by_one_level_at_time::MD001;

mod md003_heading_style;
use md003_heading_style::MD003;

mod md004_unordered_list_style;
use md004_unordered_list_style::MD004;

mod md005_consistent_list_items_indentation;
use md005_consistent_list_items_indentation::MD005;

mod md007_unordered_list_indentation;
use md007_unordered_list_indentation::MD007;

mod md009_trailing_spaces;
use md009_trailing_spaces::MD009;

mod md010_hard_tabs;
use md010_hard_tabs::MD010;

mod md011_reversed_link_syntax;
use md011_reversed_link_syntax::MD011;

mod md012_multiple_blank_lines;
use md012_multiple_blank_lines::MD012;

mod md014_dollar_sign_before_command_without_output;
use md014_dollar_sign_before_command_without_output::MD014;

mod md018_no_space_after_hash_in_atx_heading;
use md018_no_space_after_hash_in_atx_heading::MD018;

mod md019_multiple_spaces_after_hash_in_atx_heading;
use md019_multiple_spaces_after_hash_in_atx_heading::MD019;

mod md020_no_space_inside_hashes_on_closed_atx_heading;
use md020_no_space_inside_hashes_on_closed_atx_heading::MD020;

mod md021_multiple_spaces_inside_hashes_on_closed_atx_heading;
use md021_multiple_spaces_inside_hashes_on_closed_atx_heading::MD021;

mod md022_headings_should_be_surrounded_by_blank_lines;
use md022_headings_should_be_surrounded_by_blank_lines::MD022;

mod md023_headings_must_start_at_the_beginning_of_the_line;
use md023_headings_must_start_at_the_beginning_of_the_line::MD023;

mod md024_multiple_headings_with_the_same_content;
use md024_multiple_headings_with_the_same_content::MD024;

mod md025_multiple_top_level_headings;
use md025_multiple_top_level_headings::MD025;

mod md026_trailing_punctuation_in_heading;
use md026_trailing_punctuation_in_heading::MD026;

mod md027_multiple_spaces_after_block_quote_symbol;
use md027_multiple_spaces_after_block_quote_symbol::MD027;

mod md028_blank_line_inside_block_quote;
use md028_blank_line_inside_block_quote::MD028;

mod md029_ordered_list_item_prefix;
use md029_ordered_list_item_prefix::MD029;

mod md030_spaces_after_list_markers;
use md030_spaces_after_list_markers::MD030;

mod md031_fenced_code_blocks_surrounded_with_blank_lines;
use md031_fenced_code_blocks_surrounded_with_blank_lines::MD031;

mod md033_inline_html;
use md033_inline_html::MD033;

mod md046_code_block_style;
use md046_code_block_style::MD046;

mod md051_link_fragments_should_be_valid;
use md051_link_fragments_should_be_valid::MD051;

use checkmark_lint_common::*;
use colored::Colorize;
use common::*;
use rayon::prelude::*;

type Task = dyn Fn(&Node, &MarkDownFile, &Config) -> (Metadata, Vec<Violation>) + Send + Sync;

fn create_task<T: Rule>() -> Box<Task> {
    Box::new(|ast: &Node, file: &MarkDownFile, config: &Config| {
        let rule = T::default();
        (rule.metadata(), rule.check(ast, file, config))
    })
}

/// Return formatted Markdown file
pub fn lint(file: &MarkDownFile, config: &Config) -> Vec<CheckIssue> {
    let ast = common::ast::parse(&file.content).expect("unable to parse markdown file");
    vec![
        create_task::<MD001>(),
        create_task::<MD003>(),
        create_task::<MD004>(),
        create_task::<MD005>(),
        create_task::<MD007>(),
        create_task::<MD009>(),
        create_task::<MD010>(),
        create_task::<MD011>(),
        create_task::<MD012>(),
        create_task::<MD014>(),
        create_task::<MD018>(),
        create_task::<MD019>(),
        create_task::<MD020>(),
        create_task::<MD021>(),
        create_task::<MD022>(),
        create_task::<MD023>(),
        create_task::<MD024>(),
        create_task::<MD025>(),
        create_task::<MD026>(),
        create_task::<MD027>(),
        create_task::<MD028>(),
        create_task::<MD029>(),
        create_task::<MD030>(),
        create_task::<MD031>(),
        create_task::<MD033>(),
        create_task::<MD046>(),
        create_task::<MD051>(),
    ]
    .into_par_iter()
    .map(|f| f(&ast, file, config))
    .collect::<Vec<_>>()
    .iter()
    .flat_map(|(metadata, violations)| {
        violations.iter().map(|violation| {
            let mut issue = common::CheckIssueBuilder::default()
                .set_category(common::IssueCategory::Linting)
                .set_severity(common::IssueSeverity::Error)
                .set_file_path(file.path.clone())
                .set_row_num_start(violation.position.start.line)
                .set_row_num_end(violation.position.end.line)
                .set_col_num_start(violation.position.start.column)
                .set_col_num_end(violation.position.end.line)
                .set_offset_start(violation.position.start.offset)
                .set_offset_end(violation.position.end.offset);
            if violation.assertion.is_empty() {
                issue = issue.set_message(format!("{} - {}.", metadata.code, violation.message));
            } else {
                issue = issue.set_message(format!(
                    "{} - {}. {}.",
                    metadata.code, violation.message, violation.assertion
                ));
            }
            issue = issue.push_fix(&format!(
                "📝 {}   {}.",
                "Requirement".cyan(),
                &metadata.requirement
            ));
            issue = issue.push_fix(&format!(
                "🧠 {}     {}.",
                "Rationale".cyan(),
                &metadata.rationale
            ));
            for fix in &violation.fixes {
                issue = issue.push_fix(&format!("💡 {}    {}.", "Suggestion".cyan(), fix));
            }
            if metadata.is_fmt_fixable {
                issue = issue.push_fix(&format!(
                    "🚀 {}      checkmark fmt {}",
                    "Auto-fix".cyan(),
                    &file.path
                ));
            }
            issue = issue.push_fix(&format!(
                "📚 {} {}",
                "Documentation".cyan(),
                metadata.documentation
            ));
            for link in &metadata.additional_links {
                issue = issue.push_fix(&format!("🔗 {}      {}", "Also see".cyan(), link));
            }
            issue.build()
        })
    })
    .collect::<Vec<CheckIssue>>()
}
