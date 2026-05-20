# Urban Intelligence Examples

This folder contains a larger planning case study for Urban Intelligence.

## Contents

- `urban_intelligence_domain.rddl`: domain-level definitions.
- `urban_intelligence_XX_YY.rddl`: problem instances with different parameter combinations.

The current set includes 30 numbered instances (from `01_03` to `03_30`) plus the domain file.

## Purpose

Use these models to:

- stress-test parser and runtime on bigger inputs,
- validate handling of realistic planning structures,
- benchmark changes against a non-trivial scenario family.

## How To Run

From the repository root:

```bash
cargo test --test examples ui_
```

To run all example categories together:

```bash
cargo test --test examples
```