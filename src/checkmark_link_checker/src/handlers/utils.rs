use common::MarkDownFile;
use std::ops::Range;

pub fn find_all_links_in_file(file: &MarkDownFile, uri: &str) -> Vec<Range<usize>> {
    file.content
        .match_indices(&uri)
        .into_iter()
        .map(|(offset, matched_str)| Range {
            start: offset,
            end: offset + matched_str.len(),
        })
        .collect()
}
