# RiDDLe Language Support for Visual Studio Code

Syntax highlighting and basic editor support for **RiDDLe** (*Rational Domain Definition Language*), an object-oriented domain description language for timeline-based planning.

## Features

- **Syntax Highlighting** for:
  - Keywords (`class`, `predicate`, `predicate!`, `fact`, `goal`, `for`, `or`, `new`, `return`, `this`)
  - Primitive types (`bool`, `int`, `real`, `string`, `void`)
  - Built-in timeline classes & temporal keywords (`StateVariable`, `ReusableResource`, `ConsumableResource`, `Interval`, `Impulse`, `origin`, `horizon`, `start`, `end`, `duration`, `at`, `capacity`, `initial_amount`)
  - Token unification modifiers (`!`, `?`) and named parameter bindings (`param:`)
  - Arithmetic, relational, and logical constraint operators
- **Editor Ergonomics**:
  - Auto-closing brackets `{}`, `[]`, `()`, and double quotes `""`
  - Line (`//`) and block (`/* */`) comment toggling (`Ctrl+/` or `Cmd+/`)

## Supported File Extensions

- `.rdl`
- `.rddl`
- `.riddle`

## Local Installation

1. Copy this folder (`riddle-lang`) into your VS Code extensions directory:
   - **Windows:** `%USERPROFILE%\.vscode\extensions\`
   - **macOS / Linux:** `~/.vscode/extensions/`
2. Reload VS Code (`Ctrl+Shift+P` / `Cmd+Shift+P` → **Developer: Reload Window**).

## Resources

- **Core Rust Implementation:** [github.com/pstlab/RiDDLe](https://github.com/pstlab/RiDDLe)