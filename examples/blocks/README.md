# Blocks Examples

This folder contains the block-world style examples.

## Contents

- `blocks_domain.rddl`: shared domain definitions.
- `blocks_01.rddl` to `blocks_03.rddl`: problem instances.

## Purpose

Use these files to validate object/class/predicate modeling patterns in a compact but expressive domain.

## How To Run

From the repository root:

```bash
cargo test --test examples blocks_
```

To run every integration example:

```bash
cargo test --test examples
```
