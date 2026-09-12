# Config Rule Types

Reference for the rule `type` values supported in the config YAML, what each is for, and the fields it expects.

---

## `file`

The original/basic rule type. Fires when **any** of the listed paths change. No relationship between files is considered — it's a simple "did one of these show up in the diff" check.

**Fields**

| Field     | Required | Description                                                            |
| --------- | -------- | ---------------------------------------------------------------------- |
| `name`    | yes      | Unique identifier for the rule                                         |
| `type`    | yes      | `file`                                                                 |
| `when`    | yes      | List of file/path patterns. Rule fires if **any** match a changed file |
| `message` | yes      | Message shown when the rule fires                                      |
| `level`   | yes      | `warn` or `error`                                                      |

**Example**

```yaml
- name: deploy_change_requires_traffic
  type: file
  when:
    - deploy
    - scripts/deploy
  message: "Deploy script has change. Did you update the traffic script?"
  level: warn
```

**Use when:** you want a heads-up or gate on a single file or an unordered set of files, with no concern for which other files did or didn't change alongside it.

---

## `file_group`

Detects **partial changes within a related set of files** — i.e. when some, but not all, members of a group changed together. This is the type needed for propagation/consistency checks (e.g. "if one environment's `.tf` file changes, the others should too").

Unlike `file`, this type compares the changed-set against the full group rather than just checking for any overlap.

**Fields**

| Field     | Required | Description                                                                                    |
| --------- | -------- | ---------------------------------------------------------------------------------------------- |
| `name`    | yes      | Unique identifier for the rule                                                                 |
| `type`    | yes      | `file_group`                                                                                   |
| `group`   | yes      | List of files that are expected to change **together**                                         |
| `require` | yes      | Policy describing the expected relationship (see below)                                        |
| `message` | yes      | Message shown when the rule fires. Supports `{{changed}}` / `{{unchanged}}` template variables |
| `level`   | yes      | `warn` or `error`                                                                              |

**`require` policies**

| Value         | Fires when                                                                          |
| ------------- | ----------------------------------------------------------------------------------- |
| `all_or_none` | Some but not all group members changed (a strict, non-empty, non-full subset)       |
| `any_or_none` | _(reserved — no partial state possible; included for schema symmetry)_              |
| `at_least_n`  | Fewer than `n` members changed, where `n` is a sibling field (e.g. `at_least_n: 2`) |

**Example**

```yaml
- name: tf_envs_must_propagate_together
  type: file_group
  group:
    - dev.tf
    - stage.tf
    - prod.tf
  require: all_or_none
  message: >
    {{changed}} changed but {{unchanged}} did not.
    dev.tf, stage.tf, and prod.tf must change together to avoid
    environment drift.
  level: error
```

**Use when:** you have a fixed set of files that should rise and fall together (environment configs, parallel migrations, mirrored schemas) and want to flag drift — i.e., changes that touch some but not all of them.

**Caveat:** this only catches _partial_ changes by construction. It cannot verify content is _consistent_ across the files (e.g. that `prod.tf` was updated to match `dev.tf`'s new value) — only that all three were touched in the same change. Content-level consistency would require a different rule type (see below) or a script-based check outside this config.

---

## `file_content`

For cases where touching all the right files isn't enough — you also want to confirm the files agree on something (e.g. the same version string, the same instance count). This extracts and compares values from a group of files using a regular expression.

**Fields**

| Field     | Required | Description                                                                                                   |
| --------- | -------- | ------------------------------------------------------------------------------------------------------------- |
| `name`    | yes      | Unique identifier for the rule                                                                                |
| `type`    | yes      | `file_content`                                                                                                |
| `group`   | yes      | List of files expected to have consistent extracted content                                                   |
| `extract` | yes      | Regular expression used to extract values. If a capture group `(...)` is present, group 1 is used; else match |
| `require` | no       | Policy for comparing extracted values. Defaults to `identical`                                                |
| `always`  | no       | If `true`, checks group even if no files changed in git. Defaults to `false`                                  |
| `message` | yes      | Message shown when values mismatch. Supports `{{values}}` and `{{file}}` template variables                   |
| `level`   | yes      | `warn` or `error`                                                                                             |

**Example**

```yaml
- name: tf_instance_counts_must_match
  type: file_content
  group:
    - dev.tf
    - stage.tf
    - prod.tf
  extract: 'instance_count\s*=\s*(\d+)'
  require: identical
  message: "Instance counts differ across environment files: {{values}}"
  level: error
```

**Use when:** you need to verify content-level consistency across multiple configuration or source files (such as version strings, environment settings, or replica counts).

---

## `removed_variable`

Parses the git diff for deleted variable declarations and ensures the removed variables are no longer referenced in the file. This prevents accidental leftovers during refactorings or feature flag retirements.

If a variable was removed on a `-` line but re-declared on a `+` line (e.g. `let x` refactored to `const x`), it is ignored as a safe modification rather than a deletion.

**Fields**

| Field     | Required | Description                                                                                                                                                                          |
| --------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `name`    | yes      | Unique identifier for the rule                                                                                                                                                       |
| `type`    | yes      | `removed_variable`                                                                                                                                                                   |
| `when`    | no       | List of path substrings to filter files (e.g. `[".js", ".ts"]`). If omitted or empty, checks all changed files                                                                      |
| `pattern` | no       | Regex with capture group 1 to extract variable names from diff lines. Defaults to `(?:const\|let\|var\|function)\s+([a-zA-Z_$][a-zA-Z0-9_$]*)`                                      |
| `message` | yes      | Message displayed when leftover references are found. Supports `{{variable}}`, `{{file}}`, `{{line}}`, and `{{line_content}}`                                                        |
| `level`   | yes      | `warn` or `error`                                                                                                                                                                    |

**Example**

```yaml
- name: no_orphaned_removed_variables
  type: removed_variable
  when:
    - .js
    - .ts
  message: "Variable '{{variable}}' was removed, but is still referenced on line {{line}} in {{file}}"
  level: error
```

**Use when:** cleaning up dead code, removing deprecated feature flags, or refactoring modules to ensure you haven't left stray usages behind.

---

## Field Summary Across Types

| Field     | `file` | `file_group` | `file_content` | `removed_variable` |
| --------- | ------ | ------------ | -------------- | ------------------ |
| `name`    | ✓      | ✓            | ✓              | ✓                  |
| `type`    | ✓      | ✓            | ✓              | ✓                  |
| `when`    | ✓      | —            | —              | ✓                  |
| `group`   | —      | ✓            | ✓              | —                  |
| `require` | —      | ✓            | ✓              | —                  |
| `extract` | —      | —            | ✓              | —                  |
| `pattern` | —      | —            | —              | ✓                  |
| `always`  | —      | —            | ✓              | —                  |
| `message` | ✓      | ✓            | ✓              | ✓                  |
| `level`   | ✓      | ✓            | ✓              | ✓                  |


