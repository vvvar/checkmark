use crate::formatter;

use colored::Colorize;
use console::Style;
use similar::{ChangeTag, TextDiff};
use std::fs;

pub fn check_md_format(file_path: &str) -> Vec<serde_sarif::sarif::Result> {
    let file_content = fs::read_to_string(&file_path).unwrap();
    let formatted = formatter::fmt_markdown(&file_content);

    let artifact_location = serde_sarif::sarif::ArtifactLocationBuilder::default()
        .uri(String::from(file_path))
        .build()
        .unwrap();

    let message = serde_sarif::sarif::MessageBuilder::default()
        .text("Formatting is incorrect")
        .build()
        .unwrap();

    let physical_location = serde_sarif::sarif::PhysicalLocationBuilder::default()
        .artifact_location(artifact_location.clone())
        .build()
        .unwrap();

    let location = serde_sarif::sarif::LocationBuilder::default()
        .physical_location(physical_location)
        .build()
        .unwrap();

    let mut fixes: Vec<serde_sarif::sarif::Fix> = vec![];

    let diff = TextDiff::from_lines(&file_content, &formatted);
    for op in diff.ops() {
        let mut replacement = serde_sarif::sarif::ReplacementBuilder::default();

        let mut text_to_delete = String::from("");
        let mut text_to_insert = String::from("");
        
        let mut delete_line_number: usize = 0;
        
        for change in diff.iter_changes(op) {
            // Old code from example that prints a diff
            // let (sign, style) = match change.tag() {
            //     ChangeTag::Delete => ("-", Style::new().red()),
            //     ChangeTag::Insert => ("+", Style::new().green()),
            //     ChangeTag::Equal => ("", Style::new()),
            // };
            // format!("{}{}", style.apply_to(sign).bold(), style.apply_to(change));
            // print!("{}{}", style.apply_to(sign).bold(), style.apply_to(change));

            match change.tag() {
                ChangeTag::Delete => {
                    text_to_delete += &change.value();
                    if let Some(num) = change.old_index() {
                        delete_line_number = num;
                    };
                },
                ChangeTag::Insert => text_to_insert += &change.value(),
                ChangeTag::Equal => {},
            };
        }
       

        if !text_to_delete.is_empty() {
            let artifact_content: serde_sarif::sarif::ArtifactContent = serde_sarif::sarif::ArtifactContentBuilder::default()
                .text(text_to_delete)
                .build()
                .unwrap();
            let region = serde_sarif::sarif::RegionBuilder::default()
                .snippet(artifact_content)
                .start_line(delete_line_number as i64)
                .build()
                .unwrap();
            replacement.deleted_region(region);

            let artifact_content = serde_sarif::sarif::ArtifactContentBuilder::default()
                .text(text_to_insert)
                .build()
                .unwrap();
            replacement.inserted_content(artifact_content);


            let replacements = vec![replacement.build().unwrap()];
            let changes = vec![serde_sarif::sarif::ArtifactChangeBuilder::default()
                .replacements(replacements)
                .artifact_location(artifact_location.clone())
                .build()
                .unwrap()];
            let fix = serde_sarif::sarif::FixBuilder::default()
                .artifact_changes(changes)
                .build()
                .unwrap();
            fixes.push(fix);
        }
    }

    let result = serde_sarif::sarif::ResultBuilder::default()
        .locations(vec![location.clone()])
        .analysis_target(artifact_location.clone())
        .message(message)
        .fixes(fixes)
        .build()
        .unwrap();

    return vec![result];
}
