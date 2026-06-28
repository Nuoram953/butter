use schemars::JsonSchema;
use std::path::PathBuf;

use serde::Deserialize;

use crate::{
    config, git,
    rules::{
        Level,
        result::{Failure, RuleResult, get_rule_result_status, render_message},
    },
};

/// Checks that all files in group are modified.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FileGroupRuleConfig {
    /// Name of the rule.
    pub name: String,

    /// List of files that all needs to be changed at the same time.
    pub group: Vec<String>,

    /// Message displayed when the rule fails.
    pub message: String,

    /// Severity of the rule.
    pub level: Level,
}

impl Default for FileGroupRuleConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            group: Vec::new(),
            message: String::new(),
            level: Level::Warn,
        }
    }
}

impl FileGroupRuleConfig {
    pub fn evaluate_files(&self, files: &[PathBuf]) -> RuleResult {
        let mut failures: Vec<Failure> = Vec::new();

        let has_group_members = files.iter().any(|file| {
            self.group
                .iter()
                .any(|group_file| file.to_str().unwrap_or("").contains(group_file))
        });

        if has_group_members {
            self.group.iter().for_each(|file_group| {
                let found = files
                    .iter()
                    .any(|file| file.to_str().unwrap_or("").contains(file_group));

                if !found {
                    let reason = render_message(&self.message, &[("file", file_group)]);

                    failures.push(Failure {
                        file: Some(PathBuf::from(file_group)),
                        reason,
                    });
                }
            });
        }

        RuleResult {
            name: self.name.clone(),
            status: get_rule_result_status(failures.len(), &self.level),
            failures,
        }
    }

    pub fn evaluate(&self, branch: Option<&str>) -> RuleResult {
        let config = config::load_config();
        let files = git::get_changed_files(branch.unwrap_or(&config.unwrap().default_branch));
        self.evaluate_files(&files)
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::result::Status;

    use super::*;
    use std::path::PathBuf;

    #[test]
    fn returns_true_when_all_files_from_group_have_changed() {
        let rule = FileGroupRuleConfig {
            group: vec!["dev.tf".into(), "stage.tf".into(), "qa.tf".into()],
            ..Default::default()
        };

        let files = vec![
            PathBuf::from("dev.tf"),
            PathBuf::from("stage.tf"),
            PathBuf::from("qa.tf"),
        ];

        let result = rule.evaluate_files(&files);

        assert_eq!(result.status, Status::Success);
        assert_eq!(result.failures.len(), 0);
    }

    #[test]
    fn returns_false_when_file_is_missing_from_group() {
        let rule = FileGroupRuleConfig {
            group: vec!["dev.tf".into(), "stage.tf".into(), "qa.tf".into()],
            ..Default::default()
        };

        let files = vec![PathBuf::from("dev.tf"), PathBuf::from("stage.tf")];

        let result = rule.evaluate_files(&files);

        assert_eq!(result.status, Status::Warning);
        assert_eq!(result.failures.len(), 1);
    }

    #[test]
    fn returns_false_when_multiple_files_are_missing_from_group() {
        let rule = FileGroupRuleConfig {
            group: vec![
                "dev.tf".into(),
                "stage.tf".into(),
                "qa.tf".into(),
                "prod.tf".into(),
            ],
            ..Default::default()
        };

        let files = vec![PathBuf::from("dev.tf"), PathBuf::from("stage.tf")];

        let result = rule.evaluate_files(&files);

        assert_eq!(result.status, Status::Warning);
        assert_eq!(result.failures.len(), 2);
    }

    #[test]
    fn returns_true_when_no_files_from_group_changed() {
        let rule = FileGroupRuleConfig {
            group: vec![
                "dev.tf".into(),
                "stage.tf".into(),
                "qa.tf".into(),
                "prod.tf".into(),
            ],
            ..Default::default()
        };

        let files = vec![PathBuf::from("readme.md")];

        let result = rule.evaluate_files(&files);

        assert_eq!(result.status, Status::Success);
        assert_eq!(result.failures.len(), 0);
    }
}
