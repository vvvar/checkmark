use lychee_lib::Result;

pub async fn check(path: &str) -> Result<bool> {
    println!("Checking links in {}...", path);
    let response = lychee_lib::check(path).await?;
    println!("{}", response);
    return Ok(response.status().is_success());
}