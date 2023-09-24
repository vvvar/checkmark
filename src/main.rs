mod args;
mod md;
mod prettier;

use std::fs;

fn main() {
    match md::list(&args::read().root) {
        Ok(files) => {
            for file in files {
                println!("Processing {:?}...", file);
                match fs::read_to_string(file) {
                    Ok(content) => {
                        println!("Before: {:?}", content);
                        println!("After: {:?}", prettier::format(&content));
                    },
                    Err(_e) => {}
                }
                
            }
        },
        Err(e) => println!("{:?}", e)
    }
}
