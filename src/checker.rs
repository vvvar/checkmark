use crate::link_checker;
use crate::prettier;

pub struct Issue {
    pub id: String,
    pub file_path: String,
    pub category: String,
    pub description: String,
    pub suggestion: String
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
    return Ok(issues);
}
