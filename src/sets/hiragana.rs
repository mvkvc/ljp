use crate::{StudyItem, StudySetLoader};
use include_dir::{include_dir, Dir, File};

static ASSETS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets");

#[derive(Debug, Clone)]
pub struct HiraganaStudySet;

impl StudySetLoader for HiraganaStudySet {
    fn name(&self) -> String {
        "hiragana".to_string()
    }

    fn load(&self) -> Vec<StudyItem> {
        let hiragana_file: &File = ASSETS_DIR
            .get_file("hiragana.csv")
            .expect("hiragana.csv not found in assets directory");

        let data = hiragana_file
            .contents_utf8()
            .expect("Failed to read hiragana.csv as UTF-8");

        let mut items = Vec::new();

        for line in data.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 2 {
                items.push(StudyItem {
                    front: parts[1].trim().to_string(),
                    back: parts[0].trim().to_string(),
                });
            } else {
                eprintln!("Warning: Skipping malformed line in hiragana.csv: {}", line);
            }
        }

        items
    }
}
