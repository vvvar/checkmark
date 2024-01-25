use std::path::{Path, PathBuf};

pub fn run(static_dir: &PathBuf) {
    let static_dir = static_dir.clone();
    println!("Serving files from {}", static_dir.display());
    println!("Open http://localhost:8000 in your browser. Press Ctrl+C to stop.");
    open::that("http://localhost:8000").ok();
    rouille::start_server("localhost:8000", move |request| {
        let response = rouille::match_assets(&request, &static_dir);
        if response.is_success() {
            return response;
        } else {
            let default_file = Path::new("/").join("README.html");
            return rouille::Response::redirect_302(default_file.display().to_string());
        }
    })
}
