# Changelog

All notable changes to the Cide project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed (2026-05-18 审查报告修复)
- **Rust 后端 P0 Bug（5 个严重问题）**：
  - `call_user_function` 循环次数错误：拆分 `arg_count` 为 `param_count`（参数个数）和 `param_words`（总 word 数）
  - `restore()` 快照恢复：`.copy_from_slice()` → 安全边界拷贝，防止不同内存配置下 panic
  - 复编译时 `f64_constants` 残留：添加 `clear()` 防止旧常量污染
  - 常量索引越界：`.unwrap_or(0)` → `trap` 报告越界错误
  - `PushConstF` 符号扩展：`operand as u64` → `operand as u32 as u64`，修复负 float 值损坏
- **VM 安全加固**：
  - `TrapBounds`：栈为空时 `trap` 而非静默返回 0
  - C API `cide_get_call_frame`：`vm.as_ref().unwrap()` → 安全匹配
  - `write_cstring`：移除 `#[allow(clippy::int_plus_one)]`，改写边界条件
- **代码质量与重构**：
  - 统一宿主函数名→ID 映射：`host_func_id::by_user_name()` / `is_builtin()` 消除 3 处重复
  - 合并 `gen_struct_copy` / `gen_struct_copy_to_local` → `gen_struct_copy_common`
  - 合并 `parse_abstract_declarator` / `parse_declarator_node`（新增 `is_abstract` 标志）
  - 删除 `Session.errors_buffer` 冗余字段
  - `insert_implicit_cast`：`std::mem::replace` + dummy Literal → `std::mem::take`
  - 删除未使用的 `parse_call_expr`
  - `cargo clippy -- -D warnings` 完全通过（无手动抑制）
- **工程化**：
  - 检查点内存上限：默认最大 50 个快照，防止长程序内存无限增长
  - 字符串字面量上限：`0x8000` (32KB) → `MEM_SIZE / 16` (64KB)
  - CI 新增 Release 构建验证 + Flutter 测试
  - Android `applicationId`：`com.example.cide` → `com.cide.app`
  - `re_editor` 锁定确切版本 `0.8.0`，添加私有 API 依赖注释
  - NDK 配置添加环境变量说明
- **文档同步**：
  - `DESIGN.md`：指令集 `~30 条` → `106 条`，C++ 伪代码 → Rust
  - `AGENTS.md` / `CHANGELOG.md`：测试数量 `44` → `238`
  - `ROADMAP.md`：知识图谱标记为未启动，函数指针标记为已完成
  - `CideFlutter/README.md`：重写为项目说明
- **Flutter 前端加固**：
  - `LinkedListVisualizer` / `TreeVisualizer`：异步 `setState()` 前加 `mounted` 检查
  - `LinkedListVisualizer`：内存上限改为 `rust.getMemorySize()` 动态获取
  - `MemoryTab`：`StatelessWidget` → `StatefulWidget` 缓存 Future
  - `IdeScreen`：键盘状态同步从 `build()` 移至 `didChangeDependencies`

### Added
- **键盘弹出时沉浸编辑模式**（Flutter）：
  - 自定义键盘或系统键盘弹出时，顶部工具栏、模板栏、底部面板通过 `SizeTransition` 平滑收起，编辑器自动拉伸占满剩余空间。
  - 键盘收起后上下栏自动弹出恢复。
  - 系统键盘真实可见性通过 `MediaQuery.viewInsets.bottom` 检测，收起后自动同步状态。
- **编辑器手势优化**（Flutter）：
  - 点击代码字符处：打开键盘。
  - 点击空白处（空行、行尾之后、尾部空白区域）：关闭键盘。
  - 上下滑动（位移 >100px 且垂直方向为主）：关闭键盘。
  - 长按（>600ms）仍弹出上下文菜单，不受单击/滑动逻辑影响。
  - 空白检测通过 `addPostFrameCallback` 延迟到 `re_editor` 内部更新光标位置后执行，避免依赖内部私有 API。
- **Panel drag-and-drop swap logic** (Flutter):
  - All drag interactions now perform **swap** instead of add/remove/move. Both regions (bottom tabs + floating orb) maintain fixed element counts.
  - Cross-region swap: `swapBottomWithFloatingItem(bottomPanelId, floatingIndex)` and `swapFloatingWithBottomItem(floatingPanelId, bottomIndex)` in `ide_notifier.dart`.
  - Item-level `DragTarget` for each floating menu item (`floating_orb_widget.dart`), enabling precise swap with the hovered item.
  - Hover feedback: blue border + shadow on both bottom tabs and floating menu items when a draggable hovers over them.
  - Edge detection: dropping on empty padding/orb area shows a SnackBar "未识别到可交换的目标位置".
  - Same-region filtering: floating menu item `DragTarget` only accepts drags from `PanelLocation.bottom`, preventing accidental same-region swaps.
- **Floating orb menu direction**: menu now prefers expanding **upward** whenever space allows (`_pos.dy >= menuHeight + 28`), making it easier to drag bottom tabs upward into the menu for swapping.

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
- **240 unit tests** across all compiler phases (`lexer_unit_test.rs`, `parser_unit_test.rs`, `type_checker_unit_test.rs`, `bytecode_gen_unit_test.rs`, `vm_memory_safety_test.rs`, `compile_pipeline_test.rs`, `end_to_end_test.rs`, `end_to_end_extra_test.rs`, `test_snapshot.rs`).
- **Flutter frontend modularization**: extracted all tab widgets (`AlgorithmTab`, `WatchTab`, `PointerVisTab`, `ArrayVisTab`, `MemoryTab`, `VariablesTab`, `CallstackTab`, `KnowledgeCardTab`), visualizers (`ArrayVisualizer`, `KnowledgeCardItem`), and layout components (`Toolbar`, `SymbolBar`, `TemplateBar`, `HeightResizablePanel`, `DraggablePanelTab`) from `ide_screen.dart` (2004 → 471 lines).
- **Flutter provider split**: extracted `IdeNotifier` to `providers/ide_notifier.dart` (`ide_provider.dart` 726 → 7 lines).
- **数组排序实时条形图可视化**（Flutter + Rust）：
  - Rust: `CideVM::get_array_snapshots()` 遍历符号表识别 `Type::Array`，从 VM 内存逐元素读取（支持 int/char/float/double/long long）。
  - `StepPayload` 新增 `array_snapshots: Vec<ArraySnapshot>`，`StepCollector` 每步自动收集。
  - Flutter: `ArrayVisTab` 从 `unifiedProvider` 零延迟读取；`ArrayVisualizer` 绘制条形图，高度表示数值，负值红色/正值蓝色。
  - VisEvent 比较事件（如 `arr[i]:arr[j]`）自动高亮对应条形（琥珀色 + 发光阴影）。
- **变量级高亮（读/写标记）**（Flutter + Rust）：
  - Rust: `CideVM::step()` 中 `LoadLocal`/`StoreLocal`/`LoadGlobal`/`StoreGlobal` 自动记录 `VariableAccess`（Read/Write）。
  - `StepPayload` 新增 `accessed_vars`。
  - Flutter: `VariablesTab` 被读取变量显示蓝色边框+「读」徽章，被写入显示橙色边框+「写」徽章。
- **编辑器行号区域变量访问指示**：统一模式下当前执行行的行号旁追加 `a=W b=R` 标记。
- **运行时异常智能诊断匹配**（Flutter）：
  - `KnowledgeCard` 新增 `relatedTrapKeywords` 字段和 `findByTrapMessage()` 方法。
  - 新增 5 张运行时异常知识卡片：数组越界、NULL 指针解引用、除零、栈溢出、访问已释放内存。
  - `ExecutionControlPanel` 异常提示条新增「查看帮助」按钮，点击弹出 BottomSheet 展示匹配的知识卡片。
- **学习进度追踪（统一模式）**（Flutter）：
  - `LearningProgress` 新增 `totalUnifiedRuns`/`totalStepsExecuted`/`totalTraps`/`totalSeeks`/`maxStepsInSingleRun`。
  - `IdeNotifier` 新增 `recordUnifiedRun()` / `recordSeek()`。
  - `ProgressTab` 新增「调试探索」卡片，显示运行次数/总步数/异常/Seek/峰值步数。
- **算法检测信息在前端展示**（Flutter）：`ExecutionControlPanel` 顶部显示检测到的算法名称（如「冒泡排序」）+ 时间复杂度说明。
- **IDE 热键支持（Desktop）**（Flutter）：F5 运行/继续、Shift+F5 停止、F10 单步、F9 切换断点；`EditorPanelState` 新增 `getCurrentLine()`。
- **变量值变化检测**（Flutter）：`VariablesTab` 比较当前步与上一步变量值，数值增加显示绿色 ↑，减少显示红色 ↓，非数值变化显示黄色 •。
- **断点列表管理面板**（Flutter）：新增 `BreakpointsTab`，显示所有断点行号+源码预览，支持点击跳转和删除。
- **代码覆盖率统计**（Flutter）：`ExecutionControlPanel` 显示覆盖率百分比（已执行行数/总行数），颜色分级（≥80%绿/≥50%橙/<50%红）。
- **算法事件指示条**（Flutter）：`ExecutionControlPanel` 顶部紫色渐变条显示当前步 VisEvent 上下文（如 `arr[i]:arr[i+1]`）。
- **函数指针高级语法支持**（Rust Parser + TypeChecker + BytecodeGen）：
  - 多级函数指针：`int (**pp)(int) = &fp;` — `interpret_declarator_node` 的 `Function` 分支递归解释 `ptr_inner` 为"以函数指针为基础类型的声明符"。
  - 返回指针的函数指针：`int *(*fp)(int) = greet;`。
  - `sizeof` 函数指针类型：`sizeof(int (*)(int))` — 新增 `parse_abstract_declarator()` 支持抽象声明符（括号、多级指针、数组后缀、函数参数列表）。
  - `typedef` 函数指针：`typedef int (*Op)(int, int);` — `parse_typedef` 改用完整 `parse_declarator()` 替代简陋的 `parse_type_only()`。
  - `static` 局部变量：`static int arr[3] = {1,2,3};` — `parse_statement` 识别 `static` 存储类说明符并跳过。

### Fixed
- **Flutter Overlay popup Material missing**: `FloatingPanelPopup` now wraps its content with `Material(type: MaterialType.transparency)`, eliminating the yellow underline artifacts on text and the red `No Material widget found` crash when opening `WatchTab` (which contains `TextField`) or `ProgressTab` (which contains `TextButton`) from the floating orb.
- **Flutter run/step auto-compile**: `IdeNotifier.run()` and `IdeNotifier.step()` now automatically call `compile()` before executing if the session is not already running. Previously, clicking the play button without manually compiling first resulted in a silent `"程序尚未编译"` error because `state.error` was never displayed in the UI.
- **Flutter error visibility**: `IdeScreen` now listens to `state.error` via `ref.listen` and shows a floating `SnackBar` when a new error occurs, preventing silent failures.
- `printf`/`fprintf` format specifiers now correctly skip width/precision/length modifiers (e.g. `%6d`, `%.2f`, `%ld`), preventing stack imbalance from mis-counted arguments. Shared logic extracted into `parse_format_specs()` + `format_printf_string()` in `host_funcs.rs`.
- `scanf` format parsing now also skips modifiers via `parse_format_specs()`, fixing the same miscount bug.
- Comma-separated multi-variable array declarations now preserve per-variable dimensions (`int a[10], b[20];`). `parse_declarator()` extracted; `Stmt::VarDecl.extra_vars` changed to `Vec<(Type, String, Option<Expr>)>`.
- `unsigned char` no longer mapped to `unsigned int`; now correctly preserves `TypeKind::Char` with `is_unsigned: true`.
- Flutter `IdeNotifier.reset()` is now `async` and properly `await`s `rust.resetSession()`, eliminating the race condition.
- `cide_get_runtime_error()` now uses `error_buffer` snapshot pattern (same as `cide_get_compile_errors()`), eliminating dangling pointer risk across FFI boundary.
- `cide_session_load` now restores VM state via `setup_vm()` instead of overwriting with a blank VM.
- `call_user_function` no longer incorrectly pops stack value on `Trap`; returns `None` instead.
- Hex literal overflow check relaxed from `i32::MAX` to `u32::MAX` (`0x80000000` now accepted).
- Algorithm detector now collects all matching patterns per function instead of returning only the first match.
- `call_user_function` temporarily disables breakpoints to prevent internal `Paused` from terminating `run()`.
- `Type::is_scalar()` now includes `Float`, consistent with `TypeChecker::is_scalar()`.
- `malloc(0)` emits a pedagogical warning about implementation-defined behavior.
- Lexer `make_token` column calculation now uses `text.chars().count()` instead of `text.len()`, fixing multi-byte UTF-8 character inaccuracy.
- **统一模式下断点暂停支持**（Rust + Flutter）：`AutoStepResult` 新增 `paused` 字段；`UnifiedEngine::run_batch` 正确传递 `self.is_paused`；Flutter 端 `_collectBatch` 检测到 `paused` 后取消 Timer 并切换到 `paused` 状态。
- **算法可视化事件 context 修复**（Rust）：`vm.rs` 中 `StepEvent` 生成 `VisEvent` 时 `context` 为空；`CideVM.vis_event_lines` 扩展为 `Vec<(i32, i32, String)>` 保留 context，`compile_pipeline.rs` 传递 `ev.context` 到 VM。
- `cargo clippy` 8 处警告自动修复（`useless_format!` → `.to_string()`，`manual_range_contains` → `(32..=126).contains(&b)`）。

### Changed
- `TypeChecker` now uses `#[derive(Default)]`; `TypeChecker::new()` removed.
- Temp test files (`temp_nested_struct_test.rs`, `temp_ptr_array_test.rs`, `tmp_struct_copy_test.rs`) merged or removed; tests consolidated into `end_to_end_extra_test.rs`.
- `CODE_REVIEW_REPORT.md` updated to reflect actual fix status.
- Lexer internal representation changed from `source: String` (byte-indexed) to `chars: Vec<char>` (char-indexed), making `peek()` and `advance()` O(1) instead of O(n).
- `merge_free_list()` extracted in `host_funcs.rs` to eliminate ~20 lines of duplication between `host_free` and `host_realloc`.
- `push_one()` extracted in `compile_pipeline.rs` to eliminate ~100 lines of duplication between `push_diagnostics` / `push_warnings` / `push_hints`.
- `parse_declarator()` extracted in `parser.rs` to share declarator parsing between `parse_type_and_name()` and comma-separated extra variables.

## [0.1.0] - 2026-05-14

### Added
- **Full C subset compiler pipeline** (Lexer → Parser → TypeChecker → BytecodeGen → CideVM).
- **Float type support** across the entire pipeline (Lexer/Parser/TypeChecker/BytecodeGen/VM).
- **Host functions**: `printf`, `scanf`, `malloc`, `free`, `realloc`, `strlen`, `strcpy`, `strcmp`, `strcat`, `memset`, `getchar`, `putchar`, `rand`, `srand`, `atoi`, `exit`, `fprintf`, `qsort`.
- **C language features**: `struct`/`typedef struct`, `enum`, arrays (multi-dimensional), pointers (arithmetic, dereference, cast), `#define` macros, function forward declarations, `sizeof`, explicit casts, compound assignments (`+=`, `-=`, etc.), ternary operator, bitwise operators (`& | ^ ~ << >>`).
- ** pedagogical diagnostics**: Chinese error messages with emoji, fix suggestions, error catalog with explanations.
- **Algorithm visualization**: Bubble sort, selection sort, insertion sort, quick sort, merge sort, binary search detection with visual event hooks.
- **Memory map visualization**: 1MB VM memory grid with color-coded regions.
- **Flutter frontend**: IDE screen with `re_editor`, console, variable watch, step debugging, algorithm animation panel.
- **Session save/load**: `serde_json`-based snapshot of compile/runtime/memory state.
- **CI/CD**: GitHub Actions workflow for Rust build/test/clippy + C# build/test.

### Fixed
- Parser zero-progress deadlocks (`struct*`, `ParseBlock`, `parse_case_stmt`).
- VM security hardening: u32 overflow on addr arithmetic, step_count overflow, heap limit closure capture, jump target bounds, value stack limits.
- `char` array initialization using `StoreMemByte` instead of `StoreLocal`.
- Implicit cast hint system with severity levels (warning vs hint).
- UTF-8 safety in Lexer (`chars().nth()` instead of `as_bytes()[i] as char`).
- `printf`/`fprintf` format modifiers (`%6d`, `%.2f`, `%ld`) no longer cause stack unbalance.
- Comma-separated multi-variable array declarations (`int a[10], b[20];`) now preserve per-variable dimensions.
- `unsigned char` no longer incorrectly mapped to `unsigned int`.
- `cide_get_runtime_error` dangling pointer: now uses buffer snapshot pattern.
- `call_user_function` return_ip uses `HOST_CALLBACK_SENTINEL` instead of `code.len()`.
- `session.rs` removed misleading `#![forbid(unsafe_code)]`.
- `host_realloc` in-place shrink when old block is at heap boundary.
- `host_qsort` recursion depth limited to `MAX_QSORT_DEPTH = 8`, preventing stack overflow from indirect recursive qsort calls.
- `host_scanf` `%c` no longer skips whitespace (matches standard C semantics).
- `compute_stride` zero-dimension fallback fixed: `dims[i] == 0` now produces stride 0 instead of 1.
- Algorithm validation regex no longer matches `int main(` inside string literals or comments.
- `flutter_riverpod` upgraded from `^3.3.2-dev.2` to stable `^3.3.1`.
- **多维数组初始化回归**：`bytecode_gen.rs` 中 `InitList` 处理在 `elements` 数量少于 `count` 时（如 `{{1,2,3},{4,5,6}}` 的顶层只有两个内层列表，总元素为6），`else` 分支错误 push `0` 而非 `values[i]`，导致数组元素全零。

### Changed
- `host_memset` now uses slice `.fill()` instead of per-byte `store_i8` for large blocks.
- `host_realloc` supports in-place shrink when the old block is at heap boundary.
- `RuntimeState::output()` replaces 13 repeated `output_lines.join("\n")` calls in `flutter_bridge.rs`.
- `TrapBounds` VM instruction now performs full bounds check in a single instruction (was ~15 instructions via manual `Ge`/`Lt`/`JumpIfZero` chain). `gen_index` bytecode shrunk by ~73%.
- `host_memset` now uses slice `.fill()` instead of per-byte `store_i8` for large blocks.

### Refactored
- `Expr::loc()`/`ty()`/`set_ty()` deduplicated with `macro_rules! expr_field!`.
- `merge_free_list()` extracted to eliminate duplication between `host_free` and `host_realloc`.
- `push_one()` unifies `push_diagnostics`/`push_warnings`/`push_hints`.
- `TypeChecker::visit_call()` split into 19 `check_builtin_xxx()` methods + `check_user_func()`.
- `format_type()` in `capi/mod.rs` removed; uses `Type::to_string()` instead.
- FRB duplicate data structures unified: `VisEvent`/`AlgorithmMatch`/`CompileResult`/`RunResult`/`StepResult`/`StepStatus` now single-source in `session.rs`, re-exported by `api/cide.rs`.
- `OpCode::from_u8` auto-generated via `define_opcode!` macro, eliminating manual repr/match maintenance.
- `Lexer::new` takes `&str` instead of `String`, removing `.to_string()` clones in compile pipeline and all tests.
- `flutter_bridge.rs` breakpoint API batchified: `setBreakpoints(Vec<i32>)` replaces N+1 FFI calls.
- `api/cide.rs` now re-exports FRB types from `session.rs`, eliminating duplicate struct definitions between `flutter_bridge.rs` and `api/cide.rs`.

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
