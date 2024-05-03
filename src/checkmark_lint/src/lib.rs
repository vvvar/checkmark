mod md001_heading_level_should_increment_by_one_level_at_time;
mod md003_heading_style;
mod md004_unordered_list_style;
mod md005_consistent_list_items_indentation;
mod md007_unordered_list_identation;
mod md009_trailing_spaces;
mod md010_hard_tabs;
mod md011_reversed_link_syntax;
mod md012_multiple_blank_lines;
mod md014_dollar_sign_before_command_without_output;
mod md018_no_space_after_hash_in_atx_heading;
mod md019_multiple_spaces_after_hash_in_atx_heading;
mod md020_no_space_inside_hashes_on_closed_atx_heading;
mod md021_multiple_spaces_inside_hashes_on_closed_atx_heading;
mod md022_headings_should_be_surrounded_by_blank_lines;
mod md023_headings_must_start_at_the_beginning_of_the_line;
mod md024_multiple_headings_with_the_same_content;
mod md025_multiple_top_level_headings;
mod md026_trailing_punctuation_in_heading;
mod md027_multiple_spaces_after_block_quote_symbol;
mod md028_blank_line_inside_block_quote;
mod md029_ordered_list_item_prefix;
mod md030_spaces_after_list_markers;
mod md031_fenced_code_blocks_surrounded_with_blank_lines;
mod md033_inline_html;
mod md046_code_block_style;
mod md051_link_fragments_should_be_valid;
mod violation;

use colored::Colorize;
use common::{CheckIssue, Config, MarkDownFile};
use md001_heading_level_should_increment_by_one_level_at_time::*;
use md003_heading_style::*;
use md004_unordered_list_style::*;
use md005_consistent_list_items_indentation::*;
use md007_unordered_list_identation::*;
use md009_trailing_spaces::*;
use md010_hard_tabs::*;
use md011_reversed_link_syntax::*;
use md012_multiple_blank_lines::*;
use md014_dollar_sign_before_command_without_output::*;
use md018_no_space_after_hash_in_atx_heading::*;
use md019_multiple_spaces_after_hash_in_atx_heading::*;
use md020_no_space_inside_hashes_on_closed_atx_heading::*;
use md021_multiple_spaces_inside_hashes_on_closed_atx_heading::*;
use md022_headings_should_be_surrounded_by_blank_lines::*;
use md023_headings_must_start_at_the_beginning_of_the_line::*;
use md024_multiple_headings_with_the_same_content::*;
use md025_multiple_top_level_headings::*;
use md026_trailing_punctuation_in_heading::*;
use md027_multiple_spaces_after_block_quote_symbol::*;
use md028_blank_line_inside_block_quote::*;
use md029_ordered_list_item_prefix::*;
use md030_spaces_after_list_markers::*;
use md031_fenced_code_blocks_surrounded_with_blank_lines::*;
use md033_inline_html::*;
use md046_code_block_style::*;
use md051_link_fragments_should_be_valid::*;
use rayon::prelude::*;

/// Return formatted Markdown file
pub fn lint(file: &MarkDownFile, config: &Config) -> Vec<CheckIssue> {
    vec![
        md001_heading_level_should_increment_by_one_level_at_time(file),
        md003_heading_style(
            file,
            &match config.style.headings {
                common::HeadingStyle::Consistent => HeadingStyle::Consistent,
                common::HeadingStyle::Atx => HeadingStyle::Atx,
                common::HeadingStyle::Setext => HeadingStyle::SetExt,
            },
        ),
        md004_unordered_list_style(
            file,
            &match config.style.unordered_lists {
                common::UnorderedListStyle::Consistent => UnorderedListStyle::Consistent,
                common::UnorderedListStyle::Dash => UnorderedListStyle::Dash,
                common::UnorderedListStyle::Plus => UnorderedListStyle::Plus,
                common::UnorderedListStyle::Asterisk => UnorderedListStyle::Asterisk,
            },
        ),
        md005_consistent_list_items_indentation(file),
        md007_unordered_list_indentation(file, 2),
        md009_trailing_spaces(file),
        md010_hard_tabs(file),
        md011_reversed_link_syntax(file),
        md012_multiple_blank_lines(file),
        md014_dollar_sign_before_command_without_output(file),
        md018_no_space_after_hash_in_atx_heading(file),
        md019_multiple_spaces_after_hash_on_atx_style_heading(file),
        md020_no_space_inside_hashes_on_closed_atx_heading(file),
        md021_multiple_spaces_inside_hashes_on_closed_atx_heading(file),
        md022_headings_should_be_surrounded_by_blank_lines(file),
        md023_headings_must_start_at_the_beginning_of_the_line(file),
        md024_multiple_headings_with_the_same_content(file),
        md025_multiple_top_level_headings(file),
        md026_trailing_punctuation_in_heading(file),
        md027_multiple_spaces_after_block_quote_symbol(file),
        md028_blank_line_inside_block_quote(file),
        md029_ordered_list_item_prefix(file),
        md030_spaces_after_list_markers(
            file,
            match config.style.num_spaces_after_list_marker {
                Some(n) => n,
                None => DEFAULT_NUM_SPACES_AFTER_MARKER,
            },
        ),
        md031_fenced_code_blocks_surrounded_with_blank_lines(file, config.linter.md031_list_items),
        md033_inline_html(file, &config.linter.md033_allowed_html_tags),
        md046_code_block_style(file, &CodeBlockStyle::Consistent),
        md051_link_fragments_should_be_valid(file),
    ]
    .into_par_iter()
    .flatten()
    .map(|violation| {
        let mut issue = common::CheckIssueBuilder::default()
            .set_category(common::IssueCategory::Linting)
            .set_severity(common::IssueSeverity::Error)
            .set_file_path(file.path.clone())
            .set_row_num_start(violation.position.start.line)
            .set_row_num_end(violation.position.end.line)
            .set_col_num_start(violation.position.start.column)
            .set_col_num_end(violation.position.end.line)
            .set_offset_start(violation.position.start.offset)
            .set_offset_end(violation.position.end.offset)
            .set_message(format!("{} - {}", violation.code, violation.message));
        issue = issue.push_fix(&format!(
            "🧠 {}  {}",
            "Rationale".cyan(),
            &violation.rationale
        ));
        for fix in &violation.fixes {
            issue = issue.push_fix(&format!("💡 {} {}", "Suggestion".cyan(), fix));
        }
        if violation.is_fmt_fixable {
            issue = issue.push_fix(&format!(
                "🚀 {}   checkmark fmt {}",
                "Auto-fix".cyan(),
                &file.path
            ));
        }
        for link in &violation.additional_links {
            issue = issue.push_fix(&format!("🔗 {}        {}", "See".cyan(), link));
        }
        issue = issue.push_fix(&format!(
            "📚 {}       {}",
            "Docs".cyan(),
            violation.doc_link
        ));
        issue.build()
    })
    .collect::<Vec<CheckIssue>>()
}
