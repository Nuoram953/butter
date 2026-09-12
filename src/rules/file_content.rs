use std::collections::HashMap;
use std::path::PathBuf;

use log::debug;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    config, git,
    rules::{
        Level,
        result::{Failure, RuleResult, get_rule_result_status, render_message},
    },
};

/// Policy for comparing extracted values across files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContentRequirePolicy {
    /// All extracted values across the files must be identical.
    #[default]
    Identical,
}

/// Verifies consistency of extracted values across a group of files.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FileContentRuleConfig {
    /// Name of the rule.
    pub name: String,

    /// List of files that are expected to have consistent extracted content.
    pub group: Vec<String>,

    /// Regular expression used to extract values. If a capture group is present, group 1 is extracted; otherwise, the full match is used.
    pub extract: String,

    /// Policy for comparing extracted values. Defaults to `identical`.
    #[serde(default)]
    pub require: ContentRequirePolicy,

    /// If true, always check all files in the group even if none of them were changed in git. Defaults to false.
    #[serde(default)]
    pub always: bool,

    /// Message displayed when the rule fails. Supports {{values}} and {{file}} placeholders.
    pub message: String,

    /// Severity of the rule.
    pub level: Level,
}

impl Default for FileContentRuleConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            group: Vec::new(),
            extract: String::new(),
            require: ContentRequirePolicy::Identical,
            always: false,
            message: String::new(),
            level: Level::Warn,
        }
    }
}

impl FileContentRuleConfig {
    pub fn evaluate_files(&self, files: &[PathBuf]) -> RuleResult {
        self.evaluate_files_with_reader(files, |path| std::fs::read_to_string(path))
    }

    pub fn evaluate_files_with_reader<F>(&self, files: &[PathBuf], mut reader: F) -> RuleResult
    where
        F: FnMut(&str) -> std::io::Result<String>,
    {
        let mut failures: Vec<Failure> = Vec::new();

        if self.group.is_empty() {
            return RuleResult {
                name: self.name.clone(),
                status: get_rule_result_status(0, &self.level),
                failures,
            };
        }

        let should_run = self.always
            || files.iter().any(|file| {
                self.group
                    .iter()
                    .any(|group_file| file.to_str().unwrap_or("").contains(group_file))
            });

        if !should_run {
            return RuleResult {
                name: self.name.clone(),
                status: get_rule_result_status(0, &self.level),
                failures,
            };
        }

        let re = match Regex::new(&self.extract) {
            Ok(r) => r,
            Err(err) => {
                failures.push(Failure {
                    file: None,
                    reason: format!("Invalid regex pattern '{}': {}", self.extract, err),
                });
                return RuleResult {
                    name: self.name.clone(),
                    status: get_rule_result_status(failures.len(), &self.level),
                    failures,
                };
            }
        };

        let mut extracted_values: Vec<(&String, String)> = Vec::new();

        for group_file in &self.group {
            let content = match reader(group_file) {
                Ok(c) => c,
                Err(err) => {
                    debug!("Could not read file {:?}: {}", group_file, err);
                    failures.push(Failure {
                        file: Some(PathBuf::from(group_file)),
                        reason: format!("Could not read file '{}': {}", group_file, err),
                    });
                    continue;
                }
            };

            match re.captures(&content) {
                Some(caps) => {
                    let val = caps
                        .get(1)
                        .or_else(|| caps.get(0))
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default();
                    extracted_values.push((group_file, val));
                }
                None => {
                    debug!(
                        "Pattern {:?} not found in file {:?}",
                        self.extract, group_file
                    );
                    failures.push(Failure {
                        file: Some(PathBuf::from(group_file)),
                        reason: format!(
                            "Pattern '{}' not found in file '{}'",
                            self.extract, group_file
                        ),
                    });
                }
            }
        }

        if extracted_values.len() >= 2 {
            match self.require {
                ContentRequirePolicy::Identical => {
                    let all_identical = extracted_values
                        .windows(2)
                        .all(|w| w[0].1 == w[1].1);

                    if !all_identical {
                        let values_summary = extracted_values
                            .iter()
                            .map(|(file, val)| format!("{file}: \"{val}\""))
                            .collect::<Vec<_>>()
                            .join(", ");

                        let mut counts: HashMap<&str, usize> = HashMap::new();
                        for (_, val) in &extracted_values {
                            *counts.entry(val.as_str()).or_insert(0) += 1;
                        }

                        let max_count = counts.values().copied().max().unwrap_or(0);
                        let majority_count = counts.values().filter(|&&c| c == max_count).count();

                        for (file, val) in &extracted_values {
                            let is_deviant = if majority_count == 1 {
                                counts.get(val.as_str()).copied().unwrap_or(0) < max_count
                            } else {
                                true
                            };

                            if is_deviant {
                                let reason = render_message(
                                    &self.message,
                                    &[
                                        ("file", file),
                                        ("values", &values_summary),
                                    ],
                                );
                                failures.push(Failure {
                                    file: Some(PathBuf::from(file)),
                                    reason,
                                });
                            }
                        }
                    }
                }
            }
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
    use std::io;

    use crate::rules::result::Status;

    use super::*;

    #[test]
    fn returns_success_when_all_values_are_identical() {
        let rule = FileContentRuleConfig {
            group: vec!["dev.tf".into(), "prod.tf".into()],
            extract: r#"instance_count\s*=\s*(\d+)"#.into(),
            message: "Instance counts differ: {{values}}".into(),
            ..Default::default()
        };

        let files = vec![PathBuf::from("dev.tf")];
        let result = rule.evaluate_files_with_reader(&files, |path| {
            match path {
                "dev.tf" => Ok("instance_count = 2".into()),
                "prod.tf" => Ok("instance_count = 2".into()),
                _ => Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
            }
        });

        assert_eq!(result.status, Status::Success);
        assert_eq!(result.failures.len(), 0);
    }

    #[test]
    fn returns_failure_when_values_differ() {
        let rule = FileContentRuleConfig {
            group: vec!["dev.tf".into(), "prod.tf".into()],
            extract: r#"instance_count\s*=\s*(\d+)"#.into(),
            message: "Instance counts differ: {{values}}".into(),
            ..Default::default()
        };

        let files = vec![PathBuf::from("dev.tf")];
        let result = rule.evaluate_files_with_reader(&files, |path| {
            match path {
                "dev.tf" => Ok("instance_count = 1".into()),
                "prod.tf" => Ok("instance_count = 2".into()),
                _ => Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
            }
        });

        assert_eq!(result.status, Status::Warning);
        assert_eq!(result.failures.len(), 2);
        assert!(result.failures[0].reason.contains("Instance counts differ"));
        assert!(result.failures[0].reason.contains("dev.tf: \"1\""));
        assert!(result.failures[0].reason.contains("prod.tf: \"2\""));
    }

    #[test]
    fn returns_failure_when_pattern_not_found_in_a_file() {
        let rule = FileContentRuleConfig {
            group: vec!["dev.tf".into(), "prod.tf".into()],
            extract: r#"instance_count\s*=\s*(\d+)"#.into(),
            message: "Instance counts differ: {{values}}".into(),
            ..Default::default()
        };

        let files = vec![PathBuf::from("dev.tf")];
        let result = rule.evaluate_files_with_reader(&files, |path| {
            match path {
                "dev.tf" => Ok("instance_count = 1".into()),
                "prod.tf" => Ok("other_setting = true".into()),
                _ => Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
            }
        });

        assert_eq!(result.status, Status::Warning);
        assert_eq!(result.failures.len(), 1);
        assert_eq!(result.failures[0].file, Some(PathBuf::from("prod.tf")));
        assert!(result.failures[0].reason.contains("Pattern 'instance_count"));
    }

    #[test]
    fn returns_failure_when_file_cannot_be_read() {
        let rule = FileContentRuleConfig {
            group: vec!["dev.tf".into(), "prod.tf".into()],
            extract: r#"instance_count\s*=\s*(\d+)"#.into(),
            message: "Instance counts differ: {{values}}".into(),
            ..Default::default()
        };

        let files = vec![PathBuf::from("dev.tf")];
        let result = rule.evaluate_files_with_reader(&files, |path| {
            match path {
                "dev.tf" => Ok("instance_count = 1".into()),
                _ => Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
            }
        });

        assert_eq!(result.status, Status::Warning);
        assert_eq!(result.failures.len(), 1);
        assert_eq!(result.failures[0].file, Some(PathBuf::from("prod.tf")));
        assert!(result.failures[0].reason.contains("Could not read file"));
    }

    #[test]
    fn skips_when_always_is_false_and_no_files_changed() {
        let rule = FileContentRuleConfig {
            group: vec!["dev.tf".into(), "prod.tf".into()],
            extract: r#"instance_count\s*=\s*(\d+)"#.into(),
            always: false,
            ..Default::default()
        };

        let files = vec![PathBuf::from("README.md")];
        let result = rule.evaluate_files_with_reader(&files, |_| {
            panic!("Should not read files when skipped");
        });

        assert_eq!(result.status, Status::Success);
        assert_eq!(result.failures.len(), 0);
    }

    #[test]
    fn evaluates_when_always_is_true_even_if_no_files_changed() {
        let rule = FileContentRuleConfig {
            group: vec!["dev.tf".into(), "prod.tf".into()],
            extract: r#"instance_count\s*=\s*(\d+)"#.into(),
            always: true,
            ..Default::default()
        };

        let files = vec![PathBuf::from("README.md")];
        let result = rule.evaluate_files_with_reader(&files, |path| {
            match path {
                "dev.tf" => Ok("instance_count = 1".into()),
                "prod.tf" => Ok("instance_count = 2".into()),
                _ => Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
            }
        });

        assert_eq!(result.status, Status::Warning);
        assert_eq!(result.failures.len(), 2);
    }

    #[test]
    fn extracts_full_match_when_no_capture_group() {
        let rule = FileContentRuleConfig {
            group: vec!["a.txt".into(), "b.txt".into()],
            extract: r#"version-\d+"#.into(),
            message: "Mismatch: {{values}}".into(),
            always: true,
            ..Default::default()
        };

        let files = vec![];
        let result = rule.evaluate_files_with_reader(&files, |path| {
            match path {
                "a.txt" => Ok("header version-1 footer".into()),
                "b.txt" => Ok("header version-1 footer".into()),
                _ => Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
            }
        });

        assert_eq!(result.status, Status::Success);
        assert_eq!(result.failures.len(), 0);
    }

    #[test]
    fn flags_minority_file_when_majority_matches() {
        let rule = FileContentRuleConfig {
            group: vec!["dev.tf".into(), "stage.tf".into(), "prod.tf".into()],
            extract: r#"version\s*=\s*"([^"]+)""#.into(),
            message: "Version mismatch: {{values}}".into(),
            always: true,
            ..Default::default()
        };

        let files = vec![];
        let result = rule.evaluate_files_with_reader(&files, |path| {
            match path {
                "dev.tf" => Ok(r#"version = "1.0.0""#.into()),
                "stage.tf" => Ok(r#"version = "1.0.0""#.into()),
                "prod.tf" => Ok(r#"version = "0.9.0""#.into()),
                _ => Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
            }
        });

        assert_eq!(result.status, Status::Warning);
        assert_eq!(result.failures.len(), 1);
        assert_eq!(result.failures[0].file, Some(PathBuf::from("prod.tf")));
    }
}
