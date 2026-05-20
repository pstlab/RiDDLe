# Types Examples

This folder groups examples focused on type-system behavior and typing-related constructs.

## Structure

- `rr/`: examples `rr_0.rddl` to `rr_3.rddl`
- `sv/`: examples `sv_0.rddl` to `sv_3.rddl`

## Purpose

Use these files to verify:

- primitive and user-defined type usage,
- compatibility and assignment checks,
- parser stability for type-centric model variations.

## How To Run

From the repository root:

```bash
cargo test --test examples types_
```

To run the full integration suite:

```bash
cargo test --test examples
```