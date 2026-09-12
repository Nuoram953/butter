<p align="center">
  <img src="assets/logo_butter.png" width="180" />
</p>

<h1 align="center">Butter</h1>

<p align="center">
  A rule engine that validates changed files using YAML rules to prevent past mistakes.
</p>

<p align="center">
  <img src="https://github.com/Nuoram953/butter/actions/workflows/ci.yml/badge.svg" />
  <img src="https://img.shields.io/github/v/release/Nuoram953/butter" />
  <img src="https://img.shields.io/badge/status-active-green" />
  <img src="https://img.shields.io/badge/license-MIT-blue" />
  <img src="https://img.shields.io/badge/rust-1.70+-orange" />
</p>

## Why?

As a software developer working mostly in a monorepo, there can be many things to remember.

> If you change file X, you should also make sure Y was updated

Butter helps prevent repeating the same mistakes.

## Overview

Butter is a CLI tool that runs configurable checks. It helps you enforce simple safety rules in your codebase and catch mistakes early.

## Installation

Other ways to install will be added in the future

```bash
cargo build --release
```

## Usage

Run checks against your current changes:

```bash

butter check
```

## Configuration

Butter reads a `rules.yml`` file from its config directory.

Example:

```yaml
rules:
  - name: deploy_change_requires_traffic
    type: file
    when:
      - deploy.yml
    unless:
      - traffic.yml
    message: "Deploy script has changed. Did you update the traffic script?"
    level: warn
```

## Rules

<!-- SCHEMA:file:START -->
### `file`

Fails if any changed file matches a `when` pattern unless a changed file also matches a corresponding `unless` pattern (e.g. "editing `src` requires also editing `test`").

| Field | Type | Required | Description |
|---|---|---|---|
| `level` | string (`warn, error`) | yes | Severity of the rule. |
| `message` | string | yes | Message displayed when the rule fails. |
| `name` | string | yes | Name of the rule. |
| `unless` | array | no | If the rule is triggered, at least one changed file must match one of these for the rule to pass. Defaults to empty if omitted. |
| `when` | array | yes | Pattern if any changed file path contains one of these, the rule is triggered. |


<!-- SCHEMA:file:END -->

<!-- SCHEMA:file_name:START -->
### `file_name`

Checks that filenames in a given directory match a naming pattern (regex).

| Field | Type | Required | Description |
|---|---|---|---|
| `directory` | string | yes | Directory to search. |
| `level` | string (`warn, error`) | yes | Severity of the rule. |
| `message` | string | yes | Message displayed when the rule fails. |
| `name` | string | yes | Name of the rule. |
| `pattern` | string | yes | Regular expression used to match files. |


<!-- SCHEMA:file_name:END -->

<!-- SCHEMA:file_group:START -->
### `file_group`

Checks that all files in group are modified.

| Field | Type | Required | Description |
|---|---|---|---|
| `group` | array | yes | List of files that all needs to be changed at the same time. |
| `level` | string (`warn, error`) | yes | Severity of the rule. |
| `message` | string | yes | Message displayed when the rule fails. |
| `name` | string | yes | Name of the rule. |


<!-- SCHEMA:file_group:END -->

<!-- SCHEMA:file_content:START -->
### `file_content`

Verifies consistency of extracted values across a group of files.

| Field | Type | Required | Description |
|---|---|---|---|
| `always` | boolean | no | If true, always check all files in the group even if none of them were changed in git. Defaults to false. |
| `extract` | string | yes | Regular expression used to extract values. If a capture group is present, group 1 is extracted; otherwise, the full match is used. |
| `group` | array | yes | List of files that are expected to have consistent extracted content. |
| `level` | string (`warn, error`) | yes | Severity of the rule. |
| `message` | string | yes | Message displayed when the rule fails. Supports {{values}} and {{file}} placeholders. |
| `name` | string | yes | Name of the rule. |
| `require` | string (`identical`) | no | Policy for comparing extracted values. Defaults to `identical`. |


<!-- SCHEMA:file_content:END -->

<!-- SCHEMA:removed_variable:START -->
### `removed_variable`

Checks that variables whose declarations were removed in the git diff are no longer referenced in the file.

| Field | Type | Required | Description |
|---|---|---|---|
| `level` | string (`warn, error`) | yes | Severity of the rule. |
| `message` | string | yes | Message displayed when the rule fails. Supports {{variable}}, {{file}}, {{line}}, and {{line_content}} placeholders. |
| `name` | string | yes | Name of the rule. |
| `pattern` | string | no | Regular expression used to extract variable names from removed lines. Must contain at least one capture group for the variable name. Defaults to matching JS/TS const, let, var, and function declarations. |
| `when` | array | no | Patterns to match changed file paths (e.g. [".js", ".ts"]). If empty, checks all changed files. |


<!-- SCHEMA:removed_variable:END -->
