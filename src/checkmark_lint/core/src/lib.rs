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
use common::{ast::parse, CheckIssue, CheckIssueBuilder, IssueCategory, IssueSeverity};
use rayon::prelude::*;

fn convert_into_check_issue(
    MarkDownFile { path, .. }: &MarkDownFile,
    Metadata {
        code,
        documentation,
        additional_links,
        requirement,
        rationale,
        is_fmt_fixable,
    }: &Metadata,
    Violation {
        message,
        assertion,
        fixes,
        position,
        ..
    }: &Violation,
) -> CheckIssue {
    let mut issue = CheckIssueBuilder::default()
        .set_category(IssueCategory::Linting)
        .set_severity(IssueSeverity::Error)
        .set_file_path(path.clone())
        .set_row_num_start(position.start.line)
        .set_row_num_end(position.end.line)
        .set_col_num_start(position.start.column)
        .set_col_num_end(position.end.line)
        .set_offset_start(position.start.offset)
        .set_offset_end(position.end.offset);

    if assertion.is_empty() {
        issue = issue.set_message(format!("{code} - {message}."));
    } else {
        issue = issue.set_message(format!("{code} - {message}. {assertion}."));
    }

    issue = issue.push_fix(&format!("📝 {}   {requirement}.", "Requirement".cyan()));

    issue = issue.push_fix(&format!("🧠 {}     {rationale}.", "Rationale".cyan()));

    for fix in fixes {
        let prefix = "Suggestion".cyan();
        issue = issue.push_fix(&format!("💡 {prefix}    {fix}.",));
    }

    if *is_fmt_fixable {
        let prefix = "Auto-fix".cyan();
        issue = issue.push_fix(&format!("🚀 {prefix}      checkmark fmt {path}"));
    }

    issue = issue.push_fix(&format!("📚 {} {documentation}", "Documentation".cyan()));

    for link in additional_links {
        let prefix = "Also see".cyan();
        issue = issue.push_fix(&format!("🔗 {prefix}      {link}"));
    }

    issue.build()
}

pub fn lint(file: &MarkDownFile, config: &Config) -> Vec<CheckIssue> {
    let ast = parse(&file.content).expect("unable to parse markdown file");
    let rules: Vec<Box<dyn Rule>> = vec![
        Box::new(MD001),
        Box::new(MD003),
        Box::new(MD004),
        Box::new(MD005),
        Box::new(MD007),
        Box::new(MD009),
        Box::new(MD010),
        Box::new(MD011),
        Box::new(MD012),
        Box::new(MD014),
        Box::new(MD018),
        Box::new(MD019),
        Box::new(MD020),
        Box::new(MD021),
        Box::new(MD022),
        Box::new(MD023),
        Box::new(MD024),
        Box::new(MD025),
        Box::new(MD026),
        Box::new(MD027),
        Box::new(MD028),
        Box::new(MD029),
        Box::new(MD030),
        Box::new(MD031),
        Box::new(MD033),
        Box::new(MD046),
        Box::new(MD051),
    ];
    rules
        .into_par_iter()
        .filter(|rule| rule.is_enabled(config))
        .map(|rule| (rule.metadata(), rule.check(&ast, file, config)))
        .flat_map(|(metadata, violations)| {
            violations
                .into_par_iter()
                .map(move |violation| convert_into_check_issue(file, &metadata, &violation))
        })
        .collect::<Vec<_>>()
}
