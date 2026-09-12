use std::collections::HashSet;
use std::path::{Path, PathBuf};

use log::debug;
use regex::Regex;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    config, git,
    rules::{
        Level,
        result::{Failure, RuleResult, get_rule_result_status, render_message},
    },
};

fn default_variable_pattern() -> String {
    r"(?:const|let|var|function)\s+([a-zA-Z_$][a-zA-Z0-9_$]*)".to_string()
}

/// Checks that variables whose declarations were removed in the git diff are no longer referenced in the file.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RemovedVariableRuleConfig {
    /// Name of the rule.
    pub name: String,

    /// Patterns to match changed file paths (e.g. [".js", ".ts"]). If empty, checks all changed files.
    #[serde(default)]
    pub when: Vec<String>,

    /// Regular expression used to extract variable names from removed lines. Must contain at least one capture group for the variable name. Defaults to matching JS/TS const, let, var, and function declarations.
    #[serde(default = "default_variable_pattern")]
    pub pattern: String,

    /// Message displayed when the rule fails. Supports {{variable}}, {{file}}, {{line}}, and {{line_content}} placeholders.
    pub message: String,

    /// Severity of the rule.
    pub level: Level,
}

impl Default for RemovedVariableRuleConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            when: Vec::new(),
            pattern: default_variable_pattern(),
            message: String::new(),
            level: Level::Warn,
        }
    }
}

pub fn extract_removed_variables(diff: &str, re: &Regex) -> Vec<String> {
    let mut removed_vars: HashSet<String> = HashSet::new();
    let mut added_vars: HashSet<String> = HashSet::new();

    for line in diff.lines() {
        if line.starts_with("---") || line.starts_with("+++") || line.starts_with("@@") {
            continue;
        }

        if let Some(removed_line) = line.strip_prefix('-') {
            for caps in re.captures_iter(removed_line) {
                let var = caps
                    .get(1)
                    .or_else(|| caps.get(0))
                    .map(|m| m.as_str().to_string());
                if let Some(v) = var {
                    removed_vars.insert(v);
                }
            }
        } else if let Some(added_line) = line.strip_prefix('+') {
            for caps in re.captures_iter(added_line) {
                let var = caps
                    .get(1)
                    .or_else(|| caps.get(0))
                    .map(|m| m.as_str().to_string());
                if let Some(v) = var {
                    added_vars.insert(v);
                }
            }
        }
    }

    let mut deleted: Vec<String> = removed_vars
        .into_iter()
        .filter(|var| !added_vars.contains(var))
        .collect();
    deleted.sort();
    deleted
}

pub fn find_references_in_line(line: &str, var_name: &str) -> bool {
    let mut search_idx = 0;
    while let Some(pos) = line[search_idx..].find(var_name) {
        let abs_pos = search_idx + pos;
        let before_ok = if abs_pos == 0 {
            true
        } else {
            let prev_char = line[..abs_pos].chars().last().unwrap();
            !prev_char.is_alphanumeric() && prev_char != '_' && prev_char != '$'
        };

        let after_pos = abs_pos + var_name.len();
        let after_ok = if after_pos >= line.len() {
            true
        } else {
            let next_char = line[after_pos..].chars().next().unwrap();
            !next_char.is_alphanumeric() && next_char != '_' && next_char != '$'
        };

        if before_ok && after_ok {
            return true;
        }

        search_idx = abs_pos + 1;
    }
    false
}

impl RemovedVariableRuleConfig {
    pub fn evaluate_files(&self, files: &[PathBuf], branch: Option<&str>) -> RuleResult {
        let config = config::load_config().unwrap();
        let base = branch.unwrap_or(&config.default_branch);

        self.evaluate_files_with_providers(
            files,
            |file| git::get_file_diff(base, file),
            |file| std::fs::read_to_string(file),
        )
    }

    pub fn evaluate_files_with_providers<D, R>(
        &self,
        files: &[PathBuf],
        mut diff_provider: D,
        mut read_file: R,
    ) -> RuleResult
    where
        D: FnMut(&Path) -> String,
        R: FnMut(&Path) -> std::io::Result<String>,
    {
        let mut failures: Vec<Failure> = Vec::new();

        let re = match Regex::new(&self.pattern) {
            Ok(r) => r,
            Err(err) => {
                failures.push(Failure {
                    file: None,
                    reason: format!("Invalid regex pattern '{}': {}", self.pattern, err),
                });
                return RuleResult {
                    name: self.name.clone(),
                    status: get_rule_result_status(failures.len(), &self.level),
                    failures,
                };
            }
        };

        let matching_files: Vec<&PathBuf> = files
            .iter()
            .filter(|file| {
                if self.when.is_empty() {
                    true
                } else {
                    let file_str = file.to_str().unwrap_or("");
                    self.when.iter().any(|pattern| file_str.contains(pattern))
                }
            })
            .collect();

        for file in matching_files {
            let file_str = file.to_str().unwrap_or("");
            let diff = diff_provider(file);
            if diff.is_empty() {
                continue;
            }

            let removed_vars = extract_removed_variables(&diff, &re);
            if removed_vars.is_empty() {
                continue;
            }

            debug!(
                "File {:?} has removed variables: {:?}",
                file, removed_vars
            );

            let content = match read_file(file) {
                Ok(c) => c,
                Err(_) => {
                    // File might have been completely deleted, in which case there are no leftover references.
                    continue;
                }
            };

            for (line_idx, line) in content.lines().enumerate() {
                let line_num = line_idx + 1;
                for var in &removed_vars {
                    if find_references_in_line(line, var) {
                        let line_str = line_num.to_string();
                        let trimmed = line.trim();
                        let default_msg = format!(
                            "Variable '{var}' was removed in diff, but is still referenced on line {line_num}: {trimmed}"
                        );

                        let reason = if self.message.is_empty() {
                            default_msg
                        } else {
                            render_message(
                                &self.message,
                                &[
                                    ("variable", var),
                                    ("file", file_str),
                                    ("line", &line_str),
                                    ("line_content", trimmed),
                                ],
                            )
                        };

                        failures.push(Failure {
                            file: Some(file.clone()),
                            reason,
                        });
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
        let config = config::load_config().unwrap();
        let base = branch.unwrap_or(&config.default_branch);
        let files = git::get_changed_files(base);
        self.evaluate_files(&files, branch)
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use crate::rules::result::Status;

    use super::*;

    #[test]
    fn catches_leftover_reference_when_variable_declaration_is_removed() {
        let rule = RemovedVariableRuleConfig {
            name: "test_rule".into(),
            when: vec![".js".into()],
            ..Default::default()
        };

        let diff = "@@ -10,1 +10,0 @@\n-const isNewCheckoutEnabled = checkFlag('new_checkout');\n";
        let content = "function checkout() {\n  if (isNewCheckoutEnabled) {\n    renderNew();\n  }\n}\n";

        let files = vec![PathBuf::from("src/checkout.js")];
        let result = rule.evaluate_files_with_providers(
            &files,
            |_| diff.to_string(),
            |_| Ok(content.to_string()),
        );

        assert_eq!(result.status, Status::Warning);
        assert_eq!(result.failures.len(), 1);
        assert_eq!(
            result.failures[0].file,
            Some(PathBuf::from("src/checkout.js"))
        );
        assert!(
            result.failures[0]
                .reason
                .contains("Variable 'isNewCheckoutEnabled' was removed")
        );
        assert!(result.failures[0].reason.contains("line 2"));
    }

    #[test]
    fn passes_when_variable_is_completely_removed() {
        let rule = RemovedVariableRuleConfig {
            name: "test_rule".into(),
            when: vec![".js".into()],
            ..Default::default()
        };

        let diff = "@@ -10,1 +10,0 @@\n-const isNewCheckoutEnabled = checkFlag('new_checkout');\n";
        let content = "function checkout() {\n  renderNew();\n}\n";

        let files = vec![PathBuf::from("src/checkout.js")];
        let result = rule.evaluate_files_with_providers(
            &files,
            |_| diff.to_string(),
            |_| Ok(content.to_string()),
        );

        assert_eq!(result.status, Status::Success);
        assert_eq!(result.failures.len(), 0);
    }

    #[test]
    fn passes_when_variable_is_refactored_or_modified() {
        let rule = RemovedVariableRuleConfig {
            name: "test_rule".into(),
            when: vec![".js".into()],
            ..Default::default()
        };

        // let count was replaced with const count
        let diff = "@@ -10,1 +10,1 @@\n-let count = 0;\n+const count = 0;\n";
        let content = "const count = 0;\nconsole.log(count);\n";

        let files = vec![PathBuf::from("src/counter.js")];
        let result = rule.evaluate_files_with_providers(
            &files,
            |_| diff.to_string(),
            |_| Ok(content.to_string()),
        );

        assert_eq!(result.status, Status::Success);
        assert_eq!(result.failures.len(), 0);
    }

    #[test]
    fn does_not_false_positive_on_substrings() {
        let rule = RemovedVariableRuleConfig {
            name: "test_rule".into(),
            when: vec![".js".into()],
            ..Default::default()
        };

        let diff = "@@ -5,1 +5,0 @@\n-const flag = true;\n";
        // 'flags' and 'unflag' contain 'flag' as a substring but are distinct identifiers
        let content = "const flags = [];\nconst unflag = false;\n";

        let files = vec![PathBuf::from("src/test.js")];
        let result = rule.evaluate_files_with_providers(
            &files,
            |_| diff.to_string(),
            |_| Ok(content.to_string()),
        );

        assert_eq!(result.status, Status::Success);
        assert_eq!(result.failures.len(), 0);
    }

    #[test]
    fn skips_files_not_matching_when_filter() {
        let rule = RemovedVariableRuleConfig {
            name: "test_rule".into(),
            when: vec![".js".into()],
            ..Default::default()
        };

        let diff = "@@ -5,1 +5,0 @@\n-const flag = true;\n";
        let content = "flag = true;\n";

        let files = vec![PathBuf::from("README.md")];
        let result = rule.evaluate_files_with_providers(
            &files,
            |_| diff.to_string(),
            |_| Ok(content.to_string()),
        );

        assert_eq!(result.status, Status::Success);
        assert_eq!(result.failures.len(), 0);
    }

    #[test]
    fn handles_custom_regex_pattern() {
        let rule = RemovedVariableRuleConfig {
            name: "python_rule".into(),
            pattern: r"^([a-zA-Z_]\w*)\s*=".into(),
            message: "Leftover var {{variable}} in {{file}} on line {{line}}".into(),
            ..Default::default()
        };

        let diff = "@@ -1,1 +1,0 @@\n-is_active = True\n";
        let content = "def test():\n    if is_active:\n        pass\n";

        let files = vec![PathBuf::from("script.py")];
        let result = rule.evaluate_files_with_providers(
            &files,
            |_| diff.to_string(),
            |_| Ok(content.to_string()),
        );

        assert_eq!(result.status, Status::Warning);
        assert_eq!(result.failures.len(), 1);
        assert_eq!(
            result.failures[0].reason,
            "Leftover var is_active in script.py on line 2"
        );
    }

    #[test]
    fn skips_deleted_file_without_error() {
        let rule = RemovedVariableRuleConfig {
            name: "test_rule".into(),
            ..Default::default()
        };

        let diff = "@@ -1,1 +0,0 @@\n-const old = true;\n";
        let files = vec![PathBuf::from("deleted.js")];
        let result = rule.evaluate_files_with_providers(
            &files,
            |_| diff.to_string(),
            |_| Err(io::Error::new(io::ErrorKind::NotFound, "file deleted")),
        );

        assert_eq!(result.status, Status::Success);
        assert_eq!(result.failures.len(), 0);
    }
}
