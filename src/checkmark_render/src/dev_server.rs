use std::path::Path;

pub fn run(static_dir: &Path) {
    let static_dir = static_dir.to_path_buf();
    println!("Serving files from {}", static_dir.display());
    println!("Open http://localhost:8000 in your browser. Press Ctrl+C to stop.");
    open::that("http://localhost:8000").ok();
    rouille::start_server("localhost:8000", move |request| {
        let response = rouille::match_assets(request, &static_dir);
        if response.is_success() {
            response
        } else {
            let default_file = Path::new("/").join("README.html");
            return rouille::Response::redirect_302(default_file.display().to_string());
        }
    })
}
