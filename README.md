# RiDDLe

[![Crate](https://img.shields.io/crates/v/riddle-lang?logo=rust)](https://crates.io/crates/riddle-lang)
[![Docs](https://docs.rs/riddle-lang/badge.svg)](https://docs.rs/riddle-lang)
[![Rust](https://img.shields.io/badge/Rust-1.95+-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)
![Build Status](https://github.com/pstlab/RiDDLe/actions/workflows/rust.yml/badge.svg)
[![codecov](https://codecov.io/gh/pstlab/RiDDLe/branch/main/graph/badge.svg)](https://codecov.io/gh/pstlab/RiDDLe)

RiDDLe (Rational Domain Definition Language) is an object-oriented language for defining timeline-based planning domains and problems.

This repository contains a Rust implementation of:

- a lexer,
- a parser that builds an AST,
- a semantic runtime with scopes, classes, predicates, objects, and formulas,
- an execution engine for statements and expressions.

The language is inspired by [DDL.1](https://www.researchgate.net/publication/228818262_DDL_1_A_formal_description_of_a_constraint_representation_language_for_physical_domains), with a stronger object-oriented decomposition to improve modularity and model reuse.

More language-level details are available in the [wiki](https://github.com/ratioSolver/RiDDLe/wiki).

## Status

The project currently exposes RiDDLe as a Rust library crate (published as `riddle-lang` on crates.io, imported as module `riddle`).

- Parsing entry points are stable and covered by tests.
- The runtime includes class/predicate registration, function/constructor execution, and formula instantiation.
- A command line interface is not included in this repository.

## Installation

Requirements:

- Rust toolchain (edition 2024; recent stable Rust is recommended)

Build and test:

```bash
cargo check
cargo test
```

## Library Usage

Main parsing functions exported by the crate:

- `parse_problem`
- `parse_class`
- `parse_constructor`
- `parse_function`
- `parse_predicate`
- `parse_statement`
- `parse_expression`

Example:

```rust
use riddle::parse_problem;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let input = r#"
		class Point {
			int x, y;

			Point(int x, int y): x(x), y(y) {}

			predicate is_origin() {
				x == 0 & y == 0;
			}
		}
	"#;

	let ast = parse_problem(input)?;
	println!("Parsed {} class(es)", ast.classes.len());
	Ok(())
}
```

If you need JSON output, the crate re-exports `serde_json` and provides a `ToJson` trait used by core runtime types.

## What The Language Supports Here

The parser/runtime supports:

- primitive types: `bool`, `int`, `real`, `string`
- classes with:
	- fields (with optional initializers)
	- constructors
	- functions (including typed return values)
	- predicates
	- nested classes
	- inheritance syntax (`class A : B, C`)
- expressions:
	- arithmetic (`+`, `-`, `*`, `/`)
	- logical (`!`, `&`, `|`)
	- relational/equality (`<`, `<=`, `>`, `>=`, `==`, `!=`)
	- function calls
	- qualified identifiers (`a.b.c`)
	- object construction (`new Type(...)`)
- statements:
	- local declarations
	- assignments
	- quantified loops (`for (Type x) { ... }`)
	- disjunction blocks with optional costs
	- `fact`/`goal` formula instantiation
	- `return`
- comments:
	- single line (`// ...`)
	- multi line (`/* ... */`)

## Repository Layout

Core source files:

- `src/lib.rs`: public API entry points.
- `src/lexer.rs`: tokenization and comment handling.
- `src/parser.rs`: recursive-descent parser and AST construction.
- `src/language.rs`: AST definitions and execution/evaluation logic.
- `src/env.rs`: runtime environment, variables, boolean expression normalization.
- `src/scope.rs`: types, classes, predicates, functions, constructors, scope resolution.
- `src/core.rs`: runtime core abstraction and default core wiring.

Examples and tests:

- `examples/`: RiDDLe models grouped by domain (`core`, `types`, `blocks`, `urban_intelligence`).
- `tests/examples.rs`: integration tests that parse all shipped example files.

## Development Notes

The parser and runtime rely heavily on hierarchical scope resolution and dynamic dispatch (`Rc<dyn ...>`). When extending the language, the most important consistency points are:

- update lexer tokens and parser grammar together,
- keep AST changes aligned with evaluation/execution logic,
- preserve type/scope lookup invariants in `scope.rs` and `language.rs`,
- add parser tests plus at least one example-level integration test when introducing new syntax.

## Running The Example Suite

The integration test file parses every `.rddl` model in `examples/`:

```bash
cargo test --test examples
```

This is the fastest way to validate parser compatibility across all bundled domains.

## License

Licensed under the terms of the repository [LICENSE](LICENSE).
