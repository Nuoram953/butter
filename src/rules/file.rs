use log::{debug, info};
use serde::Deserialize;

use crate::{git, rules::Level};

#[derive(Debug, Deserialize)]
pub struct FileRuleConfig {
    pub name: String,
    pub when: Vec<String>,
    #[serde(default)]
    pub unless: Vec<String>,
    pub message: String,
    pub level: Level,
}

impl FileRuleConfig {
    pub fn evaluate(&self, branch: Option<&str>) -> bool {
        let files = git::get_changed_files(branch.unwrap_or("main"));

        let file_match_when = files.iter().any(|file| {
            self.when
                .iter()
                .any(|pattern| file.to_str().unwrap_or("").contains(pattern))
        });

        if !file_match_when {
            info!("No files match the pattern of when");
            return true;
        }

        debug!("Files matching when pattern {file_match_when}");

        let file_match_unless = files.iter().any(|file| {
            self.unless
                .iter()
                .any(|pattern| file.to_str().unwrap_or("").contains(pattern))
        });

        debug!("Files matching unless pattern {file_match_unless}");

        return file_match_unless;
    }
}
