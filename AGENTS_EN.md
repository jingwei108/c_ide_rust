<!-- From: d:\code\c_ide_rust\AGENTS.md -->
# Cide Project Agent Guide

## Project Overview

Cide is a cross-platform C/C++ teaching IDE consisting of:

- **Frontend**: Flutter (Android + Desktop Windows) — using self-developed `CideEditor` + `flutter_riverpod` state management
- **Backend**: Shared Rust native compiler / VM (`cide_native`)
- **Compiler pipeline**: Lexer → Parser → TypeChecker → BytecodeGen → CideVM
- **Bridge**: flutter_rust_bridge v2 (`native/src/api/cide.rs` → `CideFlutter/lib/src/rust`)
- **No git commits without permission**
- **Honest records**: This project is a teaching C/CPP subset, with Clang as the standard. Any deviation between this project and the standard must be recorded.

## Tech Stack

| Layer | Technology |
|:------|:-----------|
| Android | Flutter + self-developed `CideEditor` + CustomPainter visualization |
| Desktop | Flutter + self-developed `CideEditor` + CustomPainter visualization |
| Native | **Rust 1.95.0**, Cargo, cdylib/staticlib/rlib |
| VM | Custom bytecode interpreter, 1 MB linear memory |
| Bridge | flutter_rust_bridge v2.12.0 (SSE codec) |

## Key Directories

```
native/src/compiler/    Lexer, Parser, TypeChecker, BytecodeGen, AST, CFG, DataFlow, IntentInference (Rust)
native/src/vm/          CideVM bytecode interpreter (Rust)
native/src/unified/     Unified mode / time-travel engine (Rust)
native/src/engine/      Compiler pipeline and tools (Rust)
native/src/capi/        C API (MAUI compatibility layer) (Rust)
native/src/api/         FRB API (flutter_rust_bridge) (Rust)
native/src/diagnostics/ Structured diagnostics, auto-fix suggestions, knowledge graph, teaching reasoning (Rust)
CideFlutter/            Flutter cross-platform frontend (Android + Desktop Windows)
docs/                   Design documents, incident reports
```

For the Flutter frontend testing framework, see [`docs/current/FLUTTER_TESTING_EN.md`](docs/current/FLUTTER_TESTING_EN.md).

## Rust Migration Progress (Completed ✅)

| Phase | Module | Status |
|:------|:-------|:-------|
| Phase 0 | Rust skeleton + C API stubs + Session types | ✅ Done |
| Phase 1 | VM migration (CideVM + host funcs) | ✅ Done |
| Phase 2a | Lexer | ✅ Done |
| Phase 2b | AST | ✅ Done |
| Phase 2c | Parser | ✅ Done |
| Phase 2d | TypeChecker | ✅ Done |
| Phase 2e | BytecodeGen | ✅ Done |
| Phase 2f | C API `cide_compile_all` wiring | ✅ Done |
| Phase 3 | ~~C# frontend~~ → Flutter frontend end-to-end tests | ✅ Done |
| Phase 4 | Android target build (cargo-ndk) | ✅ Done |
| Phase 5 | Clean up legacy C++ / CMake files | ✅ Done |
| Phase 6 | Comprehensive review: compiler warning cleanup + security hardening + test coverage expansion | ✅ Done |
| Phase 7 | Desktop memory leak fixes + sizeof/scanf subset expansion | ✅ Done |
| Phase 8 | Full `float` pipeline support (Lexer→Parser→TypeChecker→BytecodeGen→VM) + diagnostics expansion | ✅ Done |
| Phase 9 | Flutter frontend built from scratch: IDE UI + editor + debug panel + algorithm visualization | ✅ Done |
| Phase 10 | Memory-mapped Canvas + algorithm visualization event FRB integration + interaction enhancements | ✅ Done |
| Phase 11 | Code review fixes + engineering standards (`rustfmt.toml`/`CHANGELOG.md`) + full Flutter frontend modularization | ✅ Done |
| Phase 12 | Full `union` pipeline support (Lexer→Parser→TypeChecker→BytecodeGen→VM) + `sizeof(union U)` | ✅ Done |
| Phase 13 | Unified mode / time travel: VM snapshot/restore, checkpoint manager, Seek progress bar, exception rollback | ✅ Done |
| Phase 14 | Heap memory visualization enhancement: malloc line tracking, external fragmentation visualization, leak detection reports | ✅ Done |
| Phase 15 | Pointer tracking animation: `PointerSnapshot` four states (Valid/Freed/Null/Dangling) real-time arrow rendering | ✅ Done |
| Phase 16 | Algorithm step semantic annotation: 27 predefined algorithm step templates, runtime teaching description inference | ✅ Done |
| Phase 17 | Code template parameterization + interactive tutorial: parameter placeholders, `TemplateTutorialPanel` line-by-line guidance | ✅ Done |
| Phase 18 | 6-04 carpet review: P0 soundness fixes, VM optimization, DRY refactoring, clippy 0 warnings | ✅ Done |
| Phase 19 | Use-After-Free / Double-Free runtime detection: `freed_logs` instruction-level checks, knowledge cards E3060/E3061 | ✅ Done |
| Phase 20 | Cognitive reasoning P0: `TraceAnalyzer` trace slicing + 5-class Trap root-cause inference, `RootCauseHint` | ✅ Done |
| Phase 21 | Cognitive reasoning P1: `MisconceptionPattern` 6-pattern detection + `LearningPath` recommendation engine | ✅ Done |
| Phase 22 | Cognitive reasoning P2: `KnowledgeGraph` 24 concept nodes + 30+ relationship edges, `ConceptGraphView` | ✅ Done |
| Phase 23 | Cognitive reasoning P3: `ControlFlowGraph` + `DataFlow` + `IntentInference` code intent inference | ✅ Done |
| Phase 24 | Semantic intelligent completion v2: `CompletionEngine` five context-aware completion types | ✅ Done |
| Phase 25 | Template JIT (Trace-based Loop Accelerator): hot-loop trace recording + pre-optimized function pointer sequence | ✅ Done |
| Phase 26 | Flutter Bridge communication optimization: Stream mode, differential encoding `StepPayloadDelta`, symbol table dedup | ✅ Done |
| Phase 27 | Data structure syntax expansion P0+P1: array decay, `unsigned` full pipeline, `const`, `extern`, VLA full pipeline | ✅ Done |
| Phase 28 | CLI debugging tool `cide_cli`: `compile`/`run`/`step`/`unified`, supporting stdin pipes for quick testing | ✅ Done |
| Phase 29 | Bytecode Libc productization: build-time precompilation + fixed index segment + ctype/abs via bytecode path | ✅ Done |
| Phase 30 | P0 syntax expansion: generic comma operator, Designated Initializer, offsetof + regression fixes | ✅ Done |
| Phase 31 | C++ extension P0: Lexer/Parser/AST keyword and node expansion | ✅ Done |
| Phase 32 | C++ extension P1: TypeChecker classes/inheritance/template monomorphization | ✅ Done |
| Phase 33 | C++ extension P2: BytecodeGen virtual functions/this pointer/constructors/destructors | ✅ Done |
| Phase 34 | C++ extension Stage 0.5: container收口 (list<int>/vector<char>/sort_int), C++ three-tier CI inclusion, CPP_FAILURES.md | ✅ Done |
| Phase 35 | C++ extension Stage 2: stack object RAII (default ctor auto-call / scope exit / return / break / continue auto-dtor) | ✅ Done |
| Phase 36 | C++ extension Stage 3: `new[]/delete[]` element construction/destruction (base[-4] stores count, reverse dtor, temp slot expanded to 4) | ✅ Done |
| Phase 37 | C++ extension Stage 4: reference declaration and basic semantics (`int& r = x` full pipeline; `T&` parameter/return; reference auto-deref; implicit address-of; returning reference lvalue recognition) | ✅ Done |
| Phase 38 | C++ extension Stage 5: implicit move constructor auto-generation (class with pointer/resource fields auto-generates `__ctor__{Class}__move`; `std::move` initialization calls move ctor; source pointer fields nulled to prevent double-free) | ✅ Done |
| Phase 39 | C++ extension Stage 6: simplified `unique_ptr<T>` dogfooding + constructor initialization syntax `Type name(args);` + constructor overload/implicit default ctor | ✅ Done |
| Phase 40 | C++ extension M6: test defense wrap-up — 59 new C++ E2E regression cases (core language / container algorithms / teaching OJ), `test_cide_e2e_cpp` in CI, Golden generated by Clang++ | ✅ Done |
| Phase 41 | C++ built-in container layout decoupling: `.cpp` interface declarations as single source of truth + JSON loader, zero Rust hard-coding | ✅ Done |
| Phase 42 | P0 syntax/stdlib expansion + code review report progress + performance optimization ([Unreleased]) | 🚧 In progress |

## Test Defenses

Cide adopts **five layers of collaborative test defenses**. Core philosophy: *tests are not for boasting pass rates, but for honestly discovering potential problems*. Any failure must be recorded truthfully; modifying test expectations to beautify data is prohibited.

```
┌────────────────────────────────────────────────────────────────┐
│  Defense 5: CI integration and consistency monitoring (Phase F) │
│  └─ Run all defenses automatically on PR; cross-validate *_FAILURES.md with test results │
├────────────────────────────────────────────────────────────────┤
│  Defense 4: Fuzz stress testing (Phase E)                       │
│  └─ Random memory state + random standard library call sequences, verify safety detection does not leak │
├────────────────────────────────────────────────────────────────┤
│  Defense 3: Three-tier contract verification (Phase A~C)        │
│  ├─ 3a Host Contract: Rust unit tests directly verify Host Func boundary behavior │
│  ├─ 3b Bytecode Self-Consistency: C source → Clang vs Cide self-hosting │
│  └─ 3c Differential Stress: cross-compare multiple implementations of the same feature │
├────────────────────────────────────────────────────────────────┤
│  Defense 2: K&R real-program regression (existing) + LeetCode (started, Phase 4) │
│  └─ K&R verifies "can real-world code run"; LeetCode easy problems gradually being filled │
├────────────────────────────────────────────────────────────────┤
│  Defense 1: Shadow Verification (existing)                      │
│  └─ Verify "behavior consistency with Clang"                    │
└────────────────────────────────────────────────────────────────┘
```

### Defense 1: Shadow Verification

The same C source is compiled and executed by both **Clang** and **Cide**, and stdout outputs are compared for exact match. Golden outputs must come from Clang, never from Cide itself.

- **Coverage**: 298 Baseline cases + 82 template-generated cases + 76 K&R cases + 92 LeetCode problems (562 C Shadow Verification cases total, 556 matched); 83 C++ cases (C++ Shadow Verification, 83/83 green; measured 2026-06-18)
- **Drivers**: `python native/tests/shadow_verification/shadow_verify.py`, `python scripts/shadow_verify_cpp.py`
- **Reports**: `native/tests/shadow_verification/reports/`

### Defense 2: K&R Real-Program Regression (existing) + LeetCode (planned)

Collect real teaching/competition code as end-to-end regression cases to verify "can real-world code run".

- **Baseline**: `native/tests/cases/baseline/` (298 cases, all green)
- **K&R**: *The C Programming Language* exercises (69 cases, 69 green, 0 known failures)
- **Template Generated**: algorithm template batch generation (82 cases, 78 green, 4 known failures)
- **LeetCode**: Phase 4 + Phase 5 fully implemented; current 92 problems all pass, see `native/tests/LEETCODE_FAILURES.md`
- **Reports**: `native/tests/TEST_REPORT.md`, `KR_FAILURES.md`, `E2E_FAILURES.md`, `LEETCODE_FAILURES.md`

### Defense 3: Three-Tier Contract Verification

The same feature may simultaneously exist as VM Builtin, Rust Host, and Bytecode Libc implementations, which must be independently verified for consistency.

| Sub-layer | Goal | Key files |
|:----------|:-----|:----------|
| **3a Host Contract** | Verify Layer B Host Func boundary conditions and safety injection (UAF/Double-Free/Buffer Overflow) | `native/tests/host_contract_tests.rs` |
| **3b Bytecode Self-Consistency** | Can Cide compiler + VM correctly compile and run "its own standard library" | `native/tests/bytecode_libc_consistency.rs` + `bytecode_libc_consistency/src/*.c` |
| **3c Differential Stress** | Cross-validate Host and Bytecode versions of the same feature; results must always match | `native/tests/differential_stress.rs` |

- **Failure records**: `HOST_CONTRACT_FAILURES.md`, `BYTECODE_LIBC_FAILURES.md`, `DIFFERENTIAL_FAILURES.md`

### Defense 4: Fuzz Stress Testing

Use a **deterministic RNG** to generate random memory states and random standard library call sequences, verifying that safety detection does not leak or crash.

| Scenario | Coverage |
|:---------|:---------|
| **Fuzz A** | malloc/free/realloc random sequences + UAF/Double-Free detection verification |
| **Fuzz B** | strcpy/strcat/strncpy/memcpy/memmove + Buffer Overflow (E3070) |
| **Fuzz C** | printf/scanf/getchar/putchar random formats and inputs |
| **Fuzz D** | Mixed malicious sequences (memory/strings/IO/rand cross) |
| **Fuzz E** | Random allocation + partial release, verify leak report accuracy |

- **Driver**: `cargo test --test fuzz_stress_test`
- **Records**: `native/tests/FUZZ_FAILURES.md`

### Defense 5: CI Integration and Consistency Monitoring

`.github/workflows/ci.yml` automatically runs all defenses above on every Push/PR, and executes `scripts/ci_three_tier_check.py` for consistency checks:

- If a test marked `KNOWN_FAILURE` in `*_FAILURES.md` now passes → **error prompting document update**
- If a test fails but has no corresponding record in the document → **error prompting adding a record**
- Generates `reports/three_tier_report.md` as a CI artifact upload

---

## Coding Conventions

### Rust (native)
- AST uses enum instead of C++ polymorphic class hierarchy: `Expr` / `Stmt` enums + `Box<Expr>` / `Vec<Box<Expr>>`
- `SourceLoc` has `Copy` derive (two `i32`s, pass-by-value has no overhead)
- Parser zero-progress guard: `if pos_ == checkpoint { self.advance(); }`
- Error handling: no panic; collect into `Vec<Error>` and return uniformly
- Borrow checker conflict resolution pattern: clone data first, then call methods requiring `&mut self`

### Dart / Flutter (frontend)
- State management: `flutter_riverpod` (`StateNotifier` + `StateNotifierProvider`)
- Editor: self-developed `CideEditor` (`EditableText` + `CustomPaint` implementation), not CodeMirror / not `re_editor`
- Rust calls via `flutter_rust_bridge`: `rust.compile()` / `rust.stepNext()` etc.
- UI thread: `Future.delayed` / `async-await`, no explicit main-thread switching needed
- Custom components: algorithm validation, memory map, linked list visualization, tutorial guidance, etc. are all CustomPainter / Widget implementations

## C Teaching Subset Overview

The C teaching subset supported by this project covers **Phase 1 ~ Phase 5+** capabilities (including comma operator, Designated Initializer, `offsetof`, VFS file I/O, etc.). Detailed spec: [`docs/current/C_SUBSET_SPEC.md`](docs/current/C_SUBSET_SPEC.md). Core support includes:

**Data types**: `int`, `char`, `float`, `double`, `unsigned`, `_Bool`/`bool`, `int*`, `char*`, `float*`, `double*`, `int[]`, `char[]`, `double[]`, `struct` (including return by value), `union`, `enum`, `typedef`

**Arrays**: fixed-size arrays (1D/multi-dimensional), **VLA variable-length arrays** (`int a[n]` / `int a[n][3]`, local scope, runtime stack allocation), array/string initializer lists, array parameter decay semantics

**Pointers**: address-of `&`, dereference `*`, pointer arithmetic (automatic step scaling), **multi-level pointers** (`int**`, `struct Node**`), explicit cast `(int*)p`, function pointers (including indirect calls, struct members, typedef)

**Statements**: variable declarations (including multiple variables, block scope), `if/else`, `while`, `do...while`, `for` (C99-style variable declaration), `switch/case/default`, `break`, `continue`, `return`

**Expressions**: arithmetic, comparison, logical (short-circuit evaluation), bitwise `& | ^ ~ << >>`, assignment (including compound assignment), ternary `?:`, array index, function call, `&`, `*`, struct access `.` / `->`, `++` / `--`, `sizeof`

**Functions**: definition/call/recursion/forward declaration, **functions returning struct by value** (Hidden Return Pointer ABI)

**Memory**: `malloc`/`free`/`realloc`

**I/O**: `printf`/`scanf`/`sprintf`/`snprintf`/`sscanf`/`fprintf`/`fgets`/`fputs`/`puts`/`getchar`/`putchar`/`ungetc`; VFS sandbox file I/O: `fopen`/`fread`/`fwrite`/`fclose`/`feof`/`fgetc`/`fputc`/`fseek`/`ftell`/`rewind`

**Strings**: `strlen`, `strcpy`, `strncpy`, `strcmp`, `strncmp`, `strcat`, `strncat`, `memcpy`, `memmove`, `memcmp`, `strchr`, `strrchr`, `strstr`, `memchr`, `strdup`, `atoi`

**Math**: `sin`/`cos`/`tan`/`sqrt`/`pow`/`atan`/`log`/`log10`/`exp`/`fabs`/`abs`/`ceil`/`floor`/`round`/`fmod` (via `libm`, `double` precision)

**Type system**: `typedef`, `sizeof`, `const`, `static` (local+global+function), `extern`, `volatile`, `restrict`, `inline`, `register`, `auto`

**Headers**: `#include <stdio.h>` / `<stdlib.h>` / `<ctype.h>` / `<math.h>` / `<string.h>` stub declaration loading

**Others**: `rand`/`srand`, `memset`, `exit`, `qsort`, `calloc`, `bsearch`, `atof`/`atol`, `#define` macros (object macros / parameterized macros / nested calls)

**Character classification**: `isdigit`/`isalpha`/`islower`/`isupper`/`isalnum`/`isspace`/`isprint`/`iscntrl`/`isxdigit`/`tolower`/`toupper` (`ctype.h`, some paths go through Bytecode Libc)

**C++ classes and templates (Phase 31+)**: `class`, member access control, `this` pointer, virtual functions, template class monomorphization, stack object RAII (auto ctor/dtor), constructor initialization syntax `Type name(args);`, implicit default/move constructors, `std::move`, simplified `unique_ptr<T>` dogfooding (construction/`get`/`release`/`reset`/dtor/ownership transfer)

**Explicitly not supported**: bitfield, `va_list` variadics, full preprocessor (only `#define` constant macros + `#include` standard library stubs)

## Known Limitations

### Currently Not Supported
- **Parameterized macro calls followed by semicolon** — forms like `SWAP(int,x,y);`, if the macro body already contains braces `{ ... }`, expand into `{ ... };` (compound statement + empty statement), which the current Parser cannot parse correctly. Workaround: do not add an extra semicolon after macro calls, or use the `do { ... } while(0)` pattern
- **VLA bounds checking** — VLA array indexes do not yet support compile-time/runtime `TrapBounds` bounds checking (`bound_size` is 0, checks are skipped)
- **`#include` non-standard library paths** — only stub loading for `<stdio.h>` / `<stdlib.h>` / `<ctype.h>` / `<math.h>` / `<string.h>` is supported; custom headers or `"header.h"` form are not yet supported
- **`va_list` / `va_start` / `va_arg` / `va_end`** — custom variadic functions are not yet supported (`printf`/`scanf` are built-in)
- **Global VLA** — variable-length arrays in global/static scope are not yet supported
- **VFS text mode newline conversion (fixed)** — as of 2026-06-15, Windows text-mode newline conversion is fully implemented: in `"r"`/`"w"` mode, `\n` is expanded to `\r\n` on write and collapsed to `\n` on read; `fseek`/`ftell` distinguish logical/physical cursor to match Windows CRT behavior. `vfs_io_extensions.c` and `file_fread.c` are restored to matching.

### Known Behavioral Differences Between Cide and Clang (Honest Records)

The following inconsistencies between Cide and Clang were discovered during LeetCode defense filling:

- **Compound side-effect array indexing** — forms like `a[++i] = b[j--]` (containing `++`/`--` side effects on two different objects) behave correctly under Clang/GCC, but may incorrectly trigger "accessing NULL pointer" traps in the Cide VM (see `native/tests/LEETCODE_FAILURES.md` for `lc_232` record). Workaround: split the increment/decrement into separate statements.
- **Function returning `double` value is incorrect** — when a function's return type is `double`, the caller may receive `0.0` (discovered in the original `lc_4` implementation: `double findMedianSortedArrays(...)` returns the correct median under Clang, but prints all `0.00000` under Cide VM). Declaring `double x = 2.5;` and printing it works correctly, indicating the issue is in the function return path rather than the `double` type itself. Workaround: use integer-scaled return values, or output the floating-point result via a pointer from the caller.

> Historical feature details and bug-fix records are in [`CHANGELOG.md`](CHANGELOG.md) and [`docs/current/C_SUBSET_SPEC.md`](docs/current/C_SUBSET_SPEC.md).

## Build Commands

```bash
# Daily build (Desktop Debug)
python scripts/build_flutter.py

# Build and run desktop Release
python scripts/build_flutter.py -c Release --run

# Full Android build (.so + APK)
python scripts/build_flutter.py -t Android

# Build + install + launch + logs (mobile full pipeline)
python scripts/test_mobile.py --install --run --logcat

# Release build
python scripts/build_release.py

# Run tests and lint before build
python scripts/build_flutter.py --test

# Flutter offline build (no network)
python scripts/build_flutter.py --offline

# Clean Flutter build artifacts
python scripts/build_flutter.py --clean

# --- Manual commands (fallback when scripts are unavailable) ---

# Build native DLL (Release Desktop)
cd native && cargo build --release
# Output: native/target/release/cide_native.dll

# Build Android .so (arm64-v8a + armeabi-v7a)
cd native
cargo ndk -t aarch64-linux-android --platform 21 build --release
cargo ndk -t armv7-linux-androideabi --platform 21 build --release

# Build and run Flutter desktop (manual)
cd CideFlutter
flutter pub get --offline
flutter build windows --debug
flutter run -d windows

# Build Flutter Android APK (manual)
cd CideFlutter
flutter build apk --release

# Install and launch (manual)
adb install -r "build/app/outputs/flutter-apk/app-release.apk"
adb shell monkey -p com.cide.app -c android.intent.category.LAUNCHER 1
```

## Debugging Tips

### Native Layer Debugging (Rust)
1. Project properties → Debug → **Enable native code debugging**
2. Set breakpoints in `cide_compile_all` / `cide_run` in `native/src/capi/mod.rs`
3. PDB warning (`apphost.pdb` missing) can be safely ignored

### Memory Leak Localization
- Managed vs native: VS Memory Analyzer looks at "managed memory"; if growth is small there but Task Manager shows large memory growth → leak is in native heap
- Parser infinite loop symptom: memory grows slowly and continuously (~100MB/s), AST nodes or error messages keep accumulating

## CLI Debugging Tool

The project provides an independent command-line debugging tool `cide_cli`, which can directly operate the Rust backend compiler/VM without launching the Flutter frontend.

### Build

```bash
cd native && cargo build --release --bin cide_cli
```

### Commands

| Command | Description |
|:--------|:------------|
| `compile <file>` | Compile and display diagnostics (error codes + fix suggestions) |
| `run <file>` | Compile and run at full speed |
| `step <file>` | Interactive step debugging (supports `p` print variables, `o` print output, `r` run to end, `q` quit) |
| `unified <file>` | Unified mode (time-travel engine) batch execution and summary output |
| `export <file1> [file2 ...] -o <out.json>` | Precompile to bytecode artifact (multiple files + `--builtin-libc` option) |

### Options and Special Filenames

- `-i <file>`: read standard input from file (multi-line input for `scanf`/`fgets`)
- `-`: read source code from standard input for quick code snippet testing

### Quick Test Examples

```bash
# Pipe directly
echo '#include <stdio.h>
int main() { printf("hello\n"); return 0; }' | cide_cli run -

# here-document compile
cide_cli compile - <<'EOF'
#include <stdio.h>
int main() {
    int a = 10, b = 20;
    printf("%d\n", a + b);
    return 0;
}
EOF

# Run with input file
cide_cli run sum.c -i input.txt

# Unified mode execution
cide_cli unified hello.c

# Precompile bytecode artifact (with Bytecode Libc)
cide_cli export main.c libc_helper.c -o bundle.json --builtin-libc
```

Full documentation: [`docs/current/CIDE_CLI.md`](docs/current/CIDE_CLI.md).
