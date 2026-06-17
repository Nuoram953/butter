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

## `file_content` _(not yet implemented — proposed)_

For cases where touching all the right files isn't enough — you also want to confirm the files agree on something (e.g. the same version string, the same instance count). This would require a way to extract and compare a value from each file, which is meaningfully more complex than path matching and likely needs engine support beyond YAML (e.g. a regex/key extractor per file).

**Sketch**

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

Not included in the working config — flagged here only as the natural next step if path-level checks turn out to be insufficient.

---

## Field Summary Across Types

| Field     | `file` | `file_group` | `file_content` (proposed) |
| --------- | ------ | ------------ | ------------------------- |
| `name`    | ✓      | ✓            | ✓                         |
| `type`    | ✓      | ✓            | ✓                         |
| `when`    | ✓      | —            | —                         |
| `group`   | —      | ✓            | ✓                         |
| `require` | —      | ✓            | ✓                         |
| `extract` | —      | —            | ✓                         |
| `message` | ✓      | ✓            | ✓                         |
| `level`   | ✓      | ✓            | ✓                         |
