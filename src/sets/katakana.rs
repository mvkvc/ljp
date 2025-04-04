use crate::{StudyItem, StudySetLoader};
use include_dir::{include_dir, Dir, File};

static ASSETS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets");

#[derive(Debug, Clone)]
pub struct KatakanaStudySet;

impl StudySetLoader for KatakanaStudySet {
    fn name(&self) -> String {
        "katakana".to_string()
    }

    fn load(&self) -> Vec<StudyItem> {
        let katakana_file: &File = ASSETS_DIR
            .get_file("katakana.csv")
            .expect("katakana.csv not found in assets directory");

        let data = katakana_file
            .contents_utf8()
            .expect("Failed to read katakana.csv as UTF-8");

        let mut items = Vec::new();

        for line in data.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 2 {
                items.push(StudyItem {
                    front: parts[0].trim().to_string(),
                    back: parts[1].trim().to_string(),
                });
            } else {
                eprintln!("Warning: Skipping malformed line in katakana.csv: {}", line);
            }
        }

        items
    }
}
