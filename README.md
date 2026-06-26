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
