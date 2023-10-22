use crate::link_checker;
use crate::prettier;
use crate::grammar;
use std::env;

pub struct Issue {
    pub id: String,
    pub file_path: String,
    pub category: String,
    pub description: String,
    pub suggestions: Vec<String>
}

pub async fn check(path: &String) -> Result<Vec<Issue>, Box<dyn std::error::Error>> {
    // println!("Checking {}...", path);
    let mut issues = Vec::<Issue>::new();
    let mut formatting_issues = prettier::check_format(&path)?;
    if !formatting_issues.is_empty() {
        issues.append(&mut formatting_issues);
    }
    let mut link_check_issues = link_checker::check(&path).await?;
    if  !link_check_issues.is_empty() {
        issues.append(&mut link_check_issues);
    }
    match env::var("SAPLING_API_KEY") {
        Ok(val) => {
            let mut grammar_check_issues = grammar::check(&path, &val).await?;
            if  !grammar_check_issues.is_empty() {
                issues.append(&mut grammar_check_issues);
            }
        },
        Err(_e) => {}
    }
    return Ok(issues);
}
