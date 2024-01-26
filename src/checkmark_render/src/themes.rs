use std::collections::HashMap;

pub struct Themes {
    pub preinstalled_themes: HashMap<String, String>,
}

impl Themes {
    pub fn create() -> Self {
        let mut themes = HashMap::<String, String>::new();
        themes.insert(
            "black".to_string(),
            include_str!("themes/black.css").to_string(),
        );
        themes.insert(
            "book".to_string(),
            include_str!("themes/book.css").to_string(),
        );
        themes.insert(
            "classic".to_string(),
            include_str!("themes/classic.css").to_string(),
        );
        themes.insert(
            "funky".to_string(),
            include_str!("themes/funky.css").to_string(),
        );
        themes.insert(
            "gfm".to_string(),
            include_str!("themes/gfm.css").to_string(),
        );
        themes.insert(
            "grayscale".to_string(),
            include_str!("themes/grayscale.css").to_string(),
        );
        themes.insert(
            "newspaper".to_string(),
            include_str!("themes/newspaper.css").to_string(),
        );
        themes.insert(
            "paper".to_string(),
            include_str!("themes/paper.css").to_string(),
        );
        themes.insert(
            "publication".to_string(),
            include_str!("themes/publication.css").to_string(),
        );
        themes.insert(
            "tiny".to_string(),
            include_str!("themes/tiny.css").to_string(),
        );
        themes.insert(
            "typewriter".to_string(),
            include_str!("themes/typewriter.css").to_string(),
        );
        themes.insert(
            "whiteboard".to_string(),
            include_str!("themes/whiteboard.css").to_string(),
        );
        Self {
            preinstalled_themes: themes,
        }
    }

    /// Get theme CSS by name
    /// If theme does not exist - default is returned
    pub fn get(&self, theme: &Option<String>) -> String {
        let default_theme = "typewriter";
        match theme {
            Some(name) => match self.preinstalled_themes.get(&name.to_lowercase()) {
                Some(css) => css.clone().to_string(),
                None => self
                    .preinstalled_themes
                    .get(default_theme)
                    .unwrap()
                    .to_string(),
            },
            None => self
                .preinstalled_themes
                .get(default_theme)
                .unwrap()
                .to_string(),
        }
    }
}
