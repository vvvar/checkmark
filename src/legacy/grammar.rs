use crate::checker::Issue;
use reqwest;
use std::collections::HashMap;
use std::fs;

pub async fn check(
    path: &String,
    api_key: &String,
) -> Result<Vec<Issue>, Box<dyn std::error::Error>> {
    let original = fs::read_to_string(path)?;
    let mut issues = Vec::<Issue>::new();
    for (num_line, line) in original.lines().enumerate() {
        let text_only = markdown_to_text::convert(&line);
        if !text_only.is_empty() {
            let client = reqwest::Client::new();
            let mut map = HashMap::new();
            map.insert("key", api_key);
            map.insert("text", &text_only);
            map.insert("session_id", &path);
            let resp: serde_json::Value = client
                .post("https://api.sapling.ai/api/v1/edits")
                .json(&map)
                .send()
                .await?
                .json()
                .await?;
            match resp["edits"].as_array() {
                Some(edits) => {
                    for edit in edits {
                        let general_error_type = edit["general_error_type"]
                            .as_str()
                            .expect("Sapling haven't sent correction type");
                        let sentence = edit["sentence"]
                            .as_str()
                            .expect("Sapling haven't sent a sentence to correct");
                        let replacement = edit["replacement"]
                            .as_str()
                            .expect("Sapling haven't sent a replacement");
                        let replacement_start = edit["start"].as_number().expect("Sapling haven't sent where it expect a start of the replacement").as_u64().expect("Cannot make a Number-->i64 conversion of the replacement index from Sapling") as usize;
                        let replacement_end = edit["end"].as_number().expect("Sapling haven't sent where it expect an end of the replacement").as_u64().expect("Cannot make a Number-->i64 conversion of the replacement index from Sapling") as usize;
                        let problematic_part_of_phrase =
                            &sentence[replacement_start..replacement_end];
                        issues.push(Issue {
                            id: String::from("MD003"),
                            file_path: format!("{}:{}", &path, &num_line + 1),
                            category: format!("{}", &general_error_type),
                            description: format!(
                                "{:?} has {} error in a phrase {:?}",
                                &sentence,
                                &general_error_type.to_ascii_lowercase(),
                                &problematic_part_of_phrase
                            ),
                            issue_in_code: None,
                            suggestions: vec![format!(
                                "Replace {:?} in a {:?} with {:?}, so it will be {:?}",
                                &problematic_part_of_phrase,
                                &sentence,
                                &replacement,
                                &sentence.replace(&problematic_part_of_phrase, &replacement)
                            )],
                        });
                    }
                }
                None => {}
            }
        }
    }
    Ok(issues)
}
