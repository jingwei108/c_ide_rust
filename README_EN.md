# Cide

> [中文版](README.md)

A cross-platform IDE for a restricted C/C++ teaching subset: Rust backend compiler + custom CideVM bytecode virtual machine + trace-based template JIT acceleration, Flutter frontend, algorithm visualization, and time-travel debugging for education.

## What is this

Cide was originally designed as a lightweight IDE for teaching scenarios. In a classroom setting, students can:

- Write code in the C/C++ teaching subset
- Execute step by step and observe memory and variable changes
- Understand algorithms and data structures through visualization
- Receive structured diagnostics and learning suggestions when errors occur

The backend is a complete compiler pipeline (Lexer → Parser → TypeChecker → BytecodeGen → CideVM), and the frontend is based on Flutter, supporting Android and Windows desktop.

## Honest Statement: This is an AI Experiment Field

**Cide was not built from scratch by a team that fully understands every line of code.**

It is the product of human-AI collaboration:

- The project designer participated in overall architecture, feature direction, key decisions, and some detailed adjustments
- A large amount of code, tests, and documentation was generated, refactored, and maintained by AI (including this README)
- The designer cannot guarantee they can answer every question from the community
- Some collaborative interaction transcripts are preserved in `docs/archive`

If you find during use:

- Some code design rationale is unclear
- Some boundary behavior is inconsistent with the standard
- Some documentation lags behind implementation
- Some changes appear to "work around problems just to pass tests"

These are typical signs of an AI experiment field. We would rather record honestly than pretend everything is under control.

## If You Feel Your Time Was Wasted

Feel free to criticize us.

Criticism is real motivation for the project to continue improving. If you are willing, you can point out issues through Issues or PRs; if you just want to vent, we accept that too—after all, a project that cannot take full responsibility for all its code does not deserve everyone's trust.

## Tech Stack

| Layer | Technology |
|:------|:-----------|
| Frontend | Flutter + self-developed `CideEditor` + CustomPainter visualization |
| Backend | Rust 1.95.0 |
| VM | Custom bytecode interpreter (106 instructions) |
| JIT | Trace-based template super-instruction acceleration (hot-loop recognition, safe Rust pre-compiled function sequence) |
| Bridge | flutter_rust_bridge v2.12.0 |
| Build | Python scripts + Cargo + Flutter |

> **Note**: The template JIT is not a traditional machine-code JIT. Because the crate enables `#![forbid(unsafe_code)]`, it cannot dynamically generate machine code. Instead, it compiles hot-loop bytecode traces into pre-compiled Rust function-pointer sequences (super-instructions), skipping interpreter dispatch overhead, and falls back to standard interpretation when traces do not match.

## Current Status

- **C subset**: Covers common teaching syntax, Shadow Verification 562 cases / 556 matched (measured 2026-06-18)
- **C++14 teaching subset extension**: M7 Beta Readiness is ready, 83/83 C++ Shadow Verification green, 61 C++ E2E cases green
- **Editor**: Third-party `re_editor` has been removed; current editor is self-developed `CideEditor` (`CideFlutter/lib/editor/`) (originally derived from `re_editor` and heavily modified)
- **Unified mode / Time travel**: Implemented
- **Algorithm visualization**: Zero-intrusive visualization for array sorting, linked lists, binary trees, etc.

## Project Structure

```
CideFlutter/          Flutter frontend (Android + Windows Desktop)
native/               Rust backend
├── src/
│   ├── compiler/     Compiler (Lexer / Parser / AST / TypeChecker / BytecodeGen)
│   ├── vm/           CideVM bytecode interpreter + JIT template acceleration
│   ├── unified/      Unified mode / time-travel engine
│   ├── diagnostics/  Structured diagnostics, auto-fix, knowledge graph
│   ├── api/          flutter_rust_bridge API
│   ├── capi/         Minimal C API (Shadow Verification / CLI service)
│   └── bin/          cide_cli command-line debugging tool
├── runtime_libc/     Standard library stubs and built-in C++ container templates
└── tests/            Test defenses and cases
docs/                 Design documents, build guides, and specifications
scripts/              Python build and test scripts
```

## Quick Start

The full quick-start guide is in [`docs/current/QUICKSTART_EN.md`](docs/current/QUICKSTART_EN.md).

Common commands:

```bash
# Quick CLI experience (no Flutter required)
cd native
cargo build --release --bin cide_cli
cargo run --release --bin cide_cli -- run tests/cases/baseline/hello_world.c

# Windows desktop build and run
python scripts/build_flutter.py --run

# Run all Rust tests
cd native && cargo test

# C Shadow Verification
python native/tests/shadow_verification/shadow_verify.py

# C++ Shadow Verification
python scripts/shadow_verify_cpp.py

# Flutter frontend tests (unit / widget / integration)
cd CideFlutter && flutter test
cd CideFlutter && flutter test -d windows integration_test/
```

For the Flutter testing framework, see [`docs/current/FLUTTER_TESTING_EN.md`](docs/current/FLUTTER_TESTING_EN.md).

For environment requirements, full build commands, and development conventions, see [`AGENTS_EN.md`](AGENTS_EN.md).

## Test Defenses

The project adopts five layers of collaborative test defenses. The core philosophy is: *tests are not for boasting about pass rates, but for honestly discovering problems*.

1. **Shadow Verification**: Compare stdout output with Clang/Clang++
2. **K&R + LeetCode real-program regression**: K&R 69 exercises, LeetCode 92 problems
3. **Three-tier contract verification**: Host / Bytecode Libc / Differential Stress
4. **Fuzz stress testing**
5. **CI integration and consistency monitoring**

Current status:

- C Shadow Verification: **562 cases, 556 matched** (differences are honestly recorded known limitations; measured 2026-06-18)
- C++ Shadow Verification: **83/83 green, 0 gap**
- C++ E2E regression cases: **61/61 green**
- Full `cargo test`: **719 passed, 0 failed**

## License

[MIT](LICENSE)
