mod associated_files;
mod dev_server;
mod md_to_html;
mod themes;

use common::{Config, MarkDownFile};
use md_to_html::md_to_html;
use rayon::prelude::*;
use std::env::current_dir;
use std::fs::{create_dir_all, remove_dir_all, write};
use std::path::{Path, PathBuf};

fn ensure_output_dir(config: &Config) -> PathBuf {
    let cwd = current_dir().unwrap();
    let output_dir = match &config.rendering.output {
        Some(output_dir) => PathBuf::from(output_dir),
        None => cwd.join("output"),
    };
    remove_dir_all(&output_dir).ok();
    create_dir_all(&output_dir).ok();
    output_dir
}

pub async fn render(files: &Vec<MarkDownFile>, config: &Config) {
    // 1. Ensure output dir exists and it's fresh
    let output_dir = ensure_output_dir(config);
    // 2. Find all associated files(images, assets, etc) and copy them to output dir
    //    Preserve the directory structure
    associated_files::collect_and_copy(files, &output_dir).await;
    // 3. Render markdown files to html
    //    Preserve the directory structure
    files.par_iter().for_each(|file| {
        // 4. Calculate path to output file
        //    cwd + output_dir + file path relative to cwd
        //    Change ext from ".md" to ".html"
        let mut out_file_path = Path::new(&output_dir).join(
            Path::new(&file.path)
                .strip_prefix(current_dir().unwrap())
                .unwrap(),
        );
        out_file_path.set_extension("html");
        // 5. Ensure dir tree exist and finally write the file
        create_dir_all(out_file_path.parent().unwrap()).ok();
        write(&out_file_path, md_to_html(file, config)).unwrap();
    });
    if config.rendering.serve {
        dev_server::run(&output_dir);
    }
}
