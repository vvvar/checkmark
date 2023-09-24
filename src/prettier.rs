use js_sandbox::Script;

pub fn format(md: &String) -> String {
    match Script::from_file("/Users/vvoinov/Documents/repos/md-checker/src/js/bundle.js") {
        Ok(mut script) => match script.call("format_markdown", (md,)) {
            Ok(formatted) => return formatted,
            Err(_e) => return String::from(md)
        }
        Err(_e) => return String::from(md)
    }
}