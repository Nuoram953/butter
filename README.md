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
