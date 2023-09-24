mod args;
mod md;
mod link_checker;
mod prettier;

async fn check(path: &String) -> Result<bool, Box<dyn std::error::Error>> {
    println!("Checking {}...", path);
    return Ok(link_checker::check(&path).await? && prettier::check_format(&path)?);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = args::read();
    for file in md::list(&arguments.root).expect("Failed to read Markdown files") {
        let result = check(&file).await;
        if result? {
            println!("OK: {:?}", &file);
        } else {
            println!("ERROR: {:?}", &file);
        }
    }
    return Ok(());
}
