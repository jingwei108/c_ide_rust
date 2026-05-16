# Changelog

All notable changes to the Cide project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **Flutter bottom panel UI polish**:
  - Output tab empty state now shows `terminal_outlined` icon + "等待执行" text instead of plain text.
  - Diagnostics tab empty state now shows `check_circle_outline` icon + "无诊断信息" text.
  - Algorithm tab empty state now shows `auto_graph_outlined` icon + "未检测到算法模式" text.
  - Copy button in output tab now has a background container (adapts to dark/light theme) and no longer overlaps text (right padding added to scroll view).
  - Removed unused "+" button from bottom tab bar.

### Added
- Host function ID unified constant module (`vm/host_func_id.rs`) to prevent ID mismatch between compile-time and runtime.
- Unified compilation pipeline `run_compile_pipeline()` in `engine/compile_pipeline.rs` to eliminate ~100 lines of DRY violation between `flutter_bridge.rs` and `capi/mod.rs`.
- `rustfmt.toml` for consistent Rust code formatting across the project.
- `CHANGELOG.md` for tracking project evolution.
- **44 unit tests** across all compiler phases (`lexer_unit_test.rs`, `parser_unit_test.rs`, `type_checker_unit_test.rs`, `bytecode_gen_unit_test.rs`, `vm_memory_safety_test.rs`).
- **Flutter frontend modularization**: extracted all tab widgets (`AlgorithmTab`, `WatchTab`, `PointerVisTab`, `ArrayVisTab`, `MemoryTab`, `VariablesTab`, `CallstackTab`, `KnowledgeCardTab`), visualizers (`ArrayVisualizer`, `KnowledgeCardItem`), and layout components (`Toolbar`, `SymbolBar`, `TemplateBar`, `HeightResizablePanel`, `DraggablePanelTab`) from `ide_screen.dart` (2004 → 471 lines).
- **Flutter provider split**: extracted `IdeNotifier` to `providers/ide_notifier.dart` (`ide_provider.dart` 726 → 7 lines).

### Fixed
- `cide_session_load` now restores VM state via `setup_vm()` instead of overwriting with a blank VM.
- `call_user_function` no longer incorrectly pops stack value on `Trap`; returns `None` instead.
- Hex literal overflow check relaxed from `i32::MAX` to `u32::MAX` (`0x80000000` now accepted).
- Algorithm detector now collects all matching patterns per function instead of returning only the first match.
- `call_user_function` temporarily disables breakpoints to prevent internal `Paused` from terminating `run()`.
- `Type::is_scalar()` now includes `Float`, consistent with `TypeChecker::is_scalar()`.
- `malloc(0)` emits a pedagogical warning about implementation-defined behavior.

### Changed
- `TypeChecker` now uses `#[derive(Default)]`; `TypeChecker::new()` removed.
- Temp test files (`temp_nested_struct_test.rs`, `temp_ptr_array_test.rs`, `tmp_struct_copy_test.rs`) merged or removed; tests consolidated into `end_to_end_extra_test.rs`.
- `CODE_REVIEW_REPORT.md` updated to reflect actual fix status.

## [0.1.0] - 2026-05-14

### Added
- **Full C subset compiler pipeline** (Lexer → Parser → TypeChecker → BytecodeGen → CideVM).
- **Float type support** across the entire pipeline (Lexer/Parser/TypeChecker/BytecodeGen/VM).
- **Host functions**: `printf`, `scanf`, `malloc`, `free`, `realloc`, `strlen`, `strcpy`, `strcmp`, `strcat`, `memset`, `getchar`, `putchar`, `rand`, `srand`, `atoi`, `exit`, `fprintf`, `qsort`.
- **C language features**: `struct`/`typedef struct`, `enum`, arrays (multi-dimensional), pointers (arithmetic, dereference, cast), `#define` macros, function forward declarations, `sizeof`, explicit casts, compound assignments (`+=`, `-=`, etc.), ternary operator, bitwise operators (`& | ^ ~ << >>`).
- ** pedagogical diagnostics**: Chinese error messages with emoji, fix suggestions, error catalog with explanations.
- **Algorithm visualization**: Bubble sort, selection sort, insertion sort, quick sort, merge sort, binary search detection with visual event hooks.
- **Memory map visualization**: 256KB VM memory grid with color-coded regions.
- **Flutter frontend**: IDE screen with `re_editor`, console, variable watch, step debugging, algorithm animation panel.
- **Session save/load**: `serde_json`-based snapshot of compile/runtime/memory state.
- **CI/CD**: GitHub Actions workflow for Rust build/test/clippy + C# build/test.

### Fixed
- Parser zero-progress deadlocks (`struct*`, `ParseBlock`, `parse_case_stmt`).
- VM security hardening: u32 overflow on addr arithmetic, step_count overflow, heap limit closure capture, jump target bounds, value stack limits.
- `char` array initialization using `StoreMemByte` instead of `StoreLocal`.
- Implicit cast hint system with severity levels (warning vs hint).
- UTF-8 safety in Lexer (`chars().nth()` instead of `as_bytes()[i] as char`).

### Security
- `compile_pipeline.rs` unsafe string write bounds validated.
- C API naked pointers documented with lifetime contracts.

---

## Migration History

- **Phase 0** (2025-10): Rust skeleton + C API stubs.
- **Phase 1** (2025-10): VM migration (CideVM + host functions).
- **Phase 2** (2025-11): Compiler frontend migration (Lexer/Parser/TypeChecker/BytecodeGen).
- **Phase 3–5** (2025-11): C# frontend E2E tests, Android builds, C++/CMake cleanup.
- **Phase 6–8** (2025-12–2026-01): Warning cleanup, float support, diagnostic system expansion.
- **Phase 9–10** (2026-02–2026-05): Flutter frontend from scratch, memory canvas, algorithm visualization FRB integration.
