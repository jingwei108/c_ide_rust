# Cide 全面地毯式代码审阅报告

**日期**: 2026-06-05  
**审阅范围**: 全部代码（Rust 后端 + Flutter 前端 + 构建脚本 + CI）  
**版本**: 0.1.0  
**代码规模**: Rust 手写 ~20,600 行 + FRB 生成 ~4,900 行 + Dart ~17,600 行 + Python ~950 行 = 总计 ~44,000 行  
**测试规模**: 199 个 E2E 测试 + 96 个集成/单元测试 (`native/tests/`) + 36 个内联单元测试 = **331 个测试**

---

## 第一部分：错误勘误

### 一、本次审阅确认的待修复问题

#### BUG-001 [P3] `write_cstring` 边界检查注释可更明确
- **文件**: `native/src/vm/vm.rs:564-570`
- **现象**: 代码逻辑实际正确：`a + bytes.len() < self.memory.len()` 已隐含为 null 终止符预留了 1 字节空间（等价于 `a + bytes.len() + 1 <= len`）。当 `addr + bytes.len() == MEM_SIZE` 时拒绝写入是正确行为，否则 `memory[a + bytes.len()] = 0` 会越界。
- **修复**: 在注释中显式说明该检查已为 null 终止符预留空间，避免后续审阅者误判。无需修改 `<` 条件。

#### BUG-002 [P2] `MemoryState::default()` 中 `heap_offset` 默认值为 0
- **文件**: `native/src/session.rs:211`
- **现象**: `#[derive(Default)]` 将 `heap_offset: u32` 设为 0，依赖外部调用 `reset_runtime()` 手动设置为 `0x5000`。若任何代码路径在未调用 reset_runtime 时访问 heap_offset（如 `allocate_raw`），分配行为未定义。
- **修复**: 为 `MemoryState` 手动实现 `Default` trait，设置 `heap_offset = crate::vm::vm::HEAP_START`。

#### BUG-003 [P3] `gen_addr` 对未声明变量仍生成 PushConst(0)（防御性不足）
- **文件**: `native/src/compiler/bytecode_gen.rs:1638-1641`
- **现象**: 字节码生成阶段对未声明变量 emit `PushConst(0)`。但编译管线末端的 `has_errors()` 守卫（同文件 line 270 附近）会拦截并丢弃错误字节码，该错误字节码**不会进入 VM 执行**。
- **修复**: 为增强防御性，可在 `gen_addr` 中直接 emit `Trap` 指令，或改为提前返回，避免生成无意义指令。

#### BUG-004 [P3] `session_ops.rs` 硬编码 `0x5000` 而非引用常量
- **文件**: `native/src/engine/session_ops.rs:57`
- **现象**: `session.memory.heap_offset = 0x5000;` 使用字面量，与 `vm/vm.rs:14` 定义的 `pub const HEAP_START: u32 = 0x5000;` 无引用关系。修改 HEAP_START 时可能遗漏此处的同步更新。
- **修复**: 改为 `session.memory.heap_offset = crate::vm::vm::HEAP_START;`。

### 二、代码审计中确认的架构层面潜在风险

#### RISK-001 [P1] 全局单例 Session 架构 — 不支持多 Tab/并发
- **文件**: `native/src/flutter_bridge.rs`
- **现象**: `SESSIONS` 使用 `LazyLock<Mutex<HashMap<u64, &'static Mutex<Session>>>>` + `Box::leak` 分配，会话销毁后内存永不释放。不支持同一 App 内同时打开多个独立文件集。
- **影响**: 当前单用户单会话场景可用，但架构不可扩展。

#### RISK-002 [P1] `cstr_to_str` 已修复但仍需注意 FFI 安全性
- **文件**: `native/src/capi/mod.rs:15-20`
- **当前状态**: 已改为返回 `Option<String>`（拷贝），不是 `&'static str`。使用 `CStr::from_ptr` + `to_str()` + `to_owned()`，安全正确。
- **建议**: 无需修复。本次审阅确认该问题已在历史版本中解决。

#### RISK-003 [P2] VM `reset()` 已正确重置 `qsort_depth`
- **文件**: `native/src/vm/vm.rs:207`
- **当前状态**: `self.qsort_depth = 0;` 已存在。本次审阅确认该问题已在历史版本中解决。

#### RISK-004 [P2] 解析器 `typedef_names` 回滚已实现
- **文件**: `native/src/compiler/parser.rs:1229, 1240`
- **当前状态**: `let typedef_snapshot = self.typedef_names.clone();` + `self.typedef_names = typedef_snapshot;` 已存在。本次审阅确认该问题已在历史版本中解决。

#### RISK-005 [P2] `is_type_token` 未包含 `LongLiteral`
- **文件**: `native/src/compiler/parser.rs:184-197`
- **当前状态**: `is_type_token` 检查 `Int/Void/Char/Float/Double/Struct/Enum/Unsigned/Long/Short/Signed/Const/Union/Identifier`，不包含 `LongLiteral`。Behavior 正确无误。本次审阅确认没有此 bug。

### 三、历史 Bug 修复验证

历史审阅（2026-05-10 ~ 2026-06-04）中共计 31 个 P0/Critical 级别 Bug，**29 个已修复并通过回归验证**（详见 CODE_REVIEW_REPORT_5_17.md 和 REVIEW_REPORT_2026-06-04.md）。包括：

| 类别 | 数量 | 代表性修复 |
|------|------|-----------|
| VM 核心 | 7 | `call_user_function` 循环计数、`restore` panic、`PushConstF` 符号扩展、`qsort_depth` 重置 |
| 编译器前端 | 8 | 结构体字段偏移、字符数组初始化、`peek()` UTF-8、`LongLiteral` 误判 |
| Flutter 桥接 | 4 | `printf` 格式修饰符栈、`unsigned char` 映射、`reset()` 异步 |
| 内存安全 | 3 | 编译管线内存边界、`cstr_to_str` 悬垂引用 |
| CI/工程 | 7 | 构建脚本、E2E 测试扩展、代码重复消除 |

---

## 第二部分：代码优化

### 一、性能热点

#### OPT-001 [P3] VM `last_accessed_vars` 诊断 HashMap 查找（非热路径）
- **文件**: `native/src/vm/vm.rs`
- **问题**: `LoadLocal`/`StoreLocal`/`LoadGlobal`/`StoreGlobal` 执行时通过 `local_sym_map`/`global_sym_map` 做变量名反查，仅用于统一模式的 `last_accessed_vars` 诊断跟踪。
- **实际**: 内存访问本身已是 O(1) 直接偏移寻址（`frame.locals_base + operand` / `GLOBAL_START + operand`），不存在 O(n) 符号表扫描。
- **建议**: 若诊断跟踪成为瓶颈，可将变量名反查惰性化或移除；当前无需优化。

#### OPT-002 [P2] `make_token` 中 `chars().count()` 逐 token 计算列偏移
- **文件**: `native/src/compiler/lexer.rs`
- **问题**: 每个 token 生成都调用 O(n) 的 `chars().count()`，1000 行源码产生 ~5000 次全字符遍历。
- **建议**: 在 `advance()` 中同步维护 `char_count` 计数器。

#### OPT-003 [P2] 内存边界检查逻辑在 6 个 load/store 函数中重复
- **文件**: `native/src/vm/vm.rs:568-647`
- **问题**: `load_i32`/`store_i32`/`load_f64`/`store_f64`/`load_i64`/`store_i64` 有相同的 `check_mem_access` 调用模式。
- **建议**: 提取 `fn checked_mem_op(addr, size) -> Option<&mut [u8]>` 统一方法。

#### OPT-004 [P3] 首次适配分配在 free_list 增长后性能退化
- **文件**: `native/src/session.rs:218-248`
- **问题**: `allocate_raw` 对 free_list 做 O(n) 线性扫描。
- **建议**: 按大小分桶或使用 buddy allocator。

### 二、代码重复与可维护性

#### OPT-005 `gen_expr` 函数过长
- **文件**: `native/src/compiler/bytecode_gen.rs:917`
- **建议**: 拆分为 `gen_binary_op`、`gen_unary_op`、`gen_member_access` 等独立方法。

#### OPT-006 18 个内置函数类型检查器高度重复
- **文件**: `native/src/compiler/type_checker.rs`
- **建议**: 使用声明式 `BuiltinSig` 结构 + `macro_rules!` 表格驱动。

#### OPT-007 `host_printf_0/1/2` 可被 `host_printf_n` 替代
- **文件**: `native/src/vm/host_funcs.rs`
- **建议**: 删除 3 个重复变体，统一为 `host_printf_n`。

#### OPT-008 Call/CallPtr 帧设置代码重复
- **文件**: `native/src/vm/vm.rs`
- **建议**: 提取 `push_call_frame()` 函数。

### 三、内存与资源管理

#### OPT-009 `Box::leak` 模式 — Session/Engine 内存永不释放
- **文件**: `native/src/flutter_bridge.rs`
- **说明**: 当前单会话场景可接受，但架构上应记录为已知限制。
- **远期建议**: 重构为 `Arc<RwLock<HashMap<u64, Arc<Mutex<Session>>>>>`。

#### OPT-010 VM 快照每次复制完整 1MB
- **文件**: `native/src/vm/snapshot.rs`
- **问题**: 50 个检查点 × 1MB = 50MB。对移动端不可接受。
- **建议**: 增量快照（仅保存修改页）。

#### OPT-011 `frame_cache` 无界增长
- **文件**: `native/src/unified/engine.rs`
- **问题**: 长程序（>10 万步）累积的 StepPayload 可达数百 MB。
- **建议**: LRU 淘汰或仅保留最近 N 步 + 检查点。

#### OPT-012 `format_type` 高频 `.to_string()` 分配
- **文件**: `native/src/compiler/ast.rs`
- **建议**: 使用 `Display` trait 的 `write!` 宏消除中间分配。

### 四、安全性增强

#### OPT-013 18 处 `unwrap()` 需要保护性处理
- **文件**: `vm/vm.rs`(2), `vm/host_funcs.rs`(1), `unified/trace_analyzer.rs`(3), `compiler/parser.rs`(1), `compiler/type_checker.rs`(1), `compiler/cfg.rs`(6), `compiler/data_flow.rs`(2), `compiler/intent.rs`(1), `diagnostics/misconception_patterns.rs`(1)
- **建议**: 审计每个 `unwrap()`，替换为 `unwrap_or_else(|| log + default)` 或 `?` 传播。

#### OPT-014 `host_func_id.rs` 中文件 I/O 常量注释过时
- **文件**: `native/src/vm/host_func_id.rs`
- **问题**: `FOPEN`/`FREAD`/`FWRITE`/`FCLOSE`/`FEOF` 常量顶部注释写着"当前沙盒中未实现"，且带有 `#[allow(dead_code)]`。但实际上这些 host 函数已在 `vm/host_funcs.rs` 完整实现，并配套 VFS 层和 E2E 测试通过。
- **修复**: 删除过时的"未实现"注释和多余的 `#[allow(dead_code)]` 属性。

---

## 第三部分：框架迭代

### 一、架构层面结构性建议

#### ARC-001 [高优] 三层 API 嵌套 — 胶水代码占比过高
- **影响范围**: `capi/mod.rs`(1041行) → `flutter_bridge.rs`(643行) → `api/cide.rs`(335行)
- **问题**: 三个层次各有 Session 管理、编译调用、运行逻辑的重复实现。
- **建议**: 提取 `core_api` 模块（纯 Rust API），C API 和 FRB API 仅做薄封装层。

```
目标架构:
  engine/core_api/          ← 纯 Rust API（compile, run, debug 核心逻辑）
  capi/mod.rs               ← 薄 FFI 封装（仅 extern "C" 转换层）
  api/cide.rs               ← 薄 FRB 封装（仅 FRB 类型转换层）
```

#### ARC-002 [中优] 编译管线过程式 — 缺乏管线抽象
- **文件**: `native/src/engine/compile_pipeline.rs`
- **问题**: 单体函数包含 7 个子步骤，无管线抽象。
- **建议**: 使用 Builder 模式或类型状态模式：
  ```rust
  CompilePipeline::new(sources)
      .lex()?.parse()?.type_check()?
      .codegen()?.detect_algorithm()?
      .build()
  ```

#### ARC-003 [中优] `Type` 枚举幽灵字段 — 数千种非法状态
- **文件**: `native/src/compiler/ast.rs`
- **问题**: 扁平枚举每个变体携带所有可能字段（Pointer 含 dims、Int 含 name、Void 含 array_size），需要 `self.make_*()` 方法掩盖复杂性。
- **建议**: 嵌套类型表示，参考 rustc `TyKind`：
  ```rust
  enum TypeKind {
      Void, Int(IntFlags), Float(FloatFlags),
      Char(CharFlags), Bool,
      Pointer(PointerType), Array(ArrayType),
      Struct(String), Union(String),
      Function(FunctionType),
  }
  ```

#### ARC-004 [中优] `Result<(), String>` → 结构化错误类型
- **影响范围**: 全项目
- **建议**: 引入 `thiserror`，定义 `CompileError`/`RuntimeError`/`VmError` 枚举层次。

#### ARC-005 [中优] VM 提取为独立 crate
- **建议**: `cide_vm` 可独立编译、测试、benchmark，与编译器/前端解耦。

### 二、超长文件拆分建议

| 当前文件 | 行数 | 建议拆分 |
|----------|------|----------|
| `vm/vm.rs` | 1,987 | `vm/executor.rs` + `vm/memory.rs` + `vm/trap.rs` + `vm/debug.rs` |
| `compiler/bytecode_gen.rs` | 1,885 | `codegen/expr.rs` + `codegen/stmt.rs` + `codegen/declare.rs` |
| `compiler/type_checker.rs` | 1,616 | `typeck/builtins.rs` + `typeck/expr.rs` + `typeck/stmt.rs` |
| `compiler/parser.rs` | 1,467 | `parser/expr.rs` + `parser/decl.rs` + `parser/ty.rs` |
| `engine/completion.rs` | 1,159 | `completion/context.rs` + `completion/candidates.rs` |

---

## 第四部分：之前方案中未实现功能的罗列

### 一、设计文档规划 vs 实际代码对比

| # | 功能 | 来源文档 | 状态 | 优先级 |
|---|------|----------|------|--------|
| F-01 | **智能检查点**（循环边界/函数调用/数组交换自动保存） | `UNIFIED_MODE_DESIGN.md`; `checkpoint.rs:32` TODO | `smart_mode=true` 但实现仅用固定间隔 | P1 |
| F-02 | **增量快照**（写时复制/差异编码替代全量 1MB 复制） | `UNIFIED_MODE_DESIGN.md` | 未实现 | P1 |
| F-03 | **更多算法模板**（堆排序、BFS/DFS、DP 等） | `ALGORITHM_DATASTRUCTURE_DESIGN.md` | 仅有 8 种内置模板 | P1 |
| F-04 | **本地持久化**（项目文件保存/加载/自动恢复） | `LOCAL_PERSISTENCE_PLAN.md` | 仅有 SharedPreferences 存学习进度，无项目文件存储 | P1 |
| F-05 | **链表/树可视化动画**（节点增删过渡动画、交互式拖拽） | `ROADMAP.md` Stage 8 | 基础静态可视化存在，无动画 | P2 |
| F-06 | **iOS 目标支持** | `ROADMAP.md` Stage 8 | 未开始 | P2 |
| F-07 | **图像输入集成**（OCR/相机拍照识别代码） | `IMAGE_INPUT_INTEGRATION_PLAN.md` | 未实现 | P2 |
| F-08 | **完整自定义键盘**（全符号布局、快捷键栏、沉浸式编辑） | `CUSTOM_KEYBOARD.md` | 仅有基础 CustomKeyboard widget 框架 | P2 |
| F-09 | **编辑器长按菜单增强**（完整上下文菜单） | `EDITOR_LONG_PRESS_MENU.md` | 基础复制/粘贴已实现，缺少完整菜单 | P2 |
| F-10 | **面板拖拽自由布局**（磁吸对齐、多区域拖放） | `PANEL_DRAG_GESTURE_DESIGN.md` | 基础 floating orb 拖拽已实现，面板间拖拽受限 | P2 |
| F-11 | **递归类型系统完整重构** | `RECURSIVE_TYPE_SYSTEM_REFACTOR.md` | Type 枚举仍有幽灵字段 | P2 |
| F-12 | **Android Release 配置**（签名、ProGuard、正式包名） | 5-18 Review E3 | 仅 Debug 构建可用 | P2 |
| F-13 | **属性测试 / Fuzzing / Benchmark CI** | 5-16 Review F8; 6-04 Review §3.6 | 未实现 | P2 |
| F-14 | **增量编译** | 5-16 Review F9 | 每次全量编译 | P3 |
| F-15 | **LSP 协议集成** | 5-16 Review F10 | 有 completion 引擎但未实现 LSP | P3 |
| F-16 | **WASM 编译目标** | 5-16 Review F13 | 未开始 | P3 |
| F-17 | **社区算法模板系统** | `ROADMAP.md` Stage 8 | 无可扩展框架 | P3 |
| F-18 | **C 子集扩展**（位域、volatile 等） | 5-16 Review F11 | 函数指针已完成，位域/volatile 未实现 | P3 |
| F-19 | **复杂数据结构可视化**（图、哈希表） | `ALGORITHM_DATASTRUCTURE_DESIGN.md` | 仅有数组/链表/树基础可视化 | P3 |
| F-20 | **检查点写入策略优化**（仅在控制流边界保存） | `UNIFIED_MODE_DESIGN.md` | 无条件每 20 步保存 | P2 |

### 二、文档与代码现状的差异

| 文档 | 声明 | 实际 | 建议 |
|------|------|------|------|
| `ROADMAP.md` (line 158) | "~30 条指令" | 106 条 OpCode | 更新 |
| `ROADMAP.md` | "知识图谱系统 - 设计阶段，未开始" | `knowledge_graph.rs` 有 482 行完整实现 | 更新状态 |
| `ROADMAP.md` | "Desktop Release - 进行中" | 仅 Debug 构建可用，Release 未完整 | 更新状态 |
| `AGENTS.md` (Phase 11) | "240 个单元测试" | 实际 331 个（199 E2E + 96 集成/单元 + 36 内联单元） | 更新 |
| `DESIGN.md` | C++ 伪代码 (`vector<Token>`, `unique_ptr`) | 全部 Rust 实现 | 更新为 Rust 示例 |
| `DESIGN.md` | "不支持 union" | Union 已完整实现（5 个 E2E 测试） | 更新 |

---

## 第五部分：三个后续问题分析

> 后续问题建立在发挥本项目自研框架（CideVM + 零侵入可视化 + 中文认知推理诊断）的优势以及放大竞品优势的基础上。

### 竞品优势回顾

本项目的三个核心竞品壁垒：
1. **CideVM** — 1MB 线性内存自研 VM（106 指令 + JIT 加速 + 时间旅行）。竞品使用本地 GCC/Clang 或 wasm3，无运行时细粒度可视化能力。
2. **零侵入可视化框架** — 编译期 AST 模式识别 + 运行时数据采集，学生无需添加任何标注。
3. **中文认知推理诊断** — 56 种错误码 + 中文解释 + 自动修复 + 6 种认知误区检测 + 24 节点知识图谱。竞品仅提供 GCC 英文错误原文。

### 问题 1：之前方案中未实现的功能是否需要实现？

| 功能 | 判断 | 理由 |
|------|------|------|
| **F-01 智能检查点** | **必须** | 时间旅行调试核心体验。固定间隔策略在循环中浪费空间（循环 10000 次=500 个快照=500MB），智能模式可将内存降至 ~10MB。对移动端关键。 |
| **F-02 增量快照** | **必须** | 同上。50 个 1MB 全量快照 = 50MB，移动端不可接受。增量快照可降至 ~5MB。 |
| **F-03 更多算法模板** | **必须** | 算法教学是核心场景。8 种算法严重不足，需扩展到 20+。直接放大零侵入可视化优势。 |
| **F-04 本地持久化** | **建议** | IDE 的基本能力，缺失严重降低可用性。 |
| **F-05 可视化动画** | **建议** | 链表/树是 C 教学核心难点。动画展示指针操作本质，竞品不具备。 |
| **F-06 iOS 目标** | **建议** | 学校 iPad 普及率高。Flutter 跨平台使成本可控。 |
| **F-07 图像输入** | **暂缓** | OCR 精度有限，体验不如模板系统。 |
| **F-08~F-10 编辑器体验** | **建议** | 直接影响移动端可用性。 |
| **F-11~F-20 其余** | **暂缓** | 当前用户规模下优先级低 |

### 问题 2：后续需要深化优化的功能

| # | 方向 | 理由 | 优先级 |
|---|------|------|--------|
| T-01 | **时间旅行调试体验** | 定义性特性。智能检查点 + 增量快照 + 差异编码流。竞品只能"运行→输出"。 | 最高 |
| T-02 | **零侵入可视化覆盖范围** | 从 8 种 → 20+ 算法。可视化事件时间线回放。算法步骤动画。 | 最高 |
| T-03 | **认知推理诊断升级** | 从单句提示 → 交互式卡片（原因 + 概念 + 示例 + 练习）。学习路径从静态 → 动态适应。中文认知诊断是竞品完全不具备的。 | 高 |
| T-04 | **VM 性能** | O(n) 符号查找 → O(1) 直接寻址。热循环 JIT 扩展。性能是产品可用性底线。 | 高 |
| T-05 | **移动端体验** | 自定义键盘完善、Flutter 前端已知问题（8 项）、触控优化。移动端是 P0 目标。 | 中 |
| T-06 | **多文件项目体验** | 头文件/实现分离模板、模块依赖可视化、跨文件补全。C 教学必经之路。 | 中 |

### 问题 3：需要实现的新功能

| # | 功能 | 差异化价值 | 所需基础设施 | 优先级 |
|---|------|-----------|-------------|--------|
| N-01 | **教学练习模式** | 不是"通过/不通过"，而是展示每一步在做什么。 | 编译管线 + 算法检测 + 运行时可视化 + 中文诊断（全部已有） | 战略级 |
| N-02 | **代码回放与分享** | 老师录制完整 trace → 学生逐步回放。不是看答案，是看解题过程的每一步。 | StepPayload 流 + seek/播放控制（已有） | 战略级 |
| N-03 | **变量/条件断点** | 当变量改变时暂停、`i==5` 时暂停。竞品调试器通常只支持行断点。 | VM 每步执行 + 变量快照 + 断点系统（已有） | 高 |
| N-04 | **内存可视化教学引导** | 堆/栈增长实时动画、碎片形成演示、指针连线。C 教学最大难点用动画解决。 | 内存区域跟踪 + free_list（已有） | 高 |
| N-05 | **代码质量评分** | 不判对错，告诉学生怎么改进。从"教学工具"到"智能辅导"。 | AST 分析 + 错误码 + 算法检测（已有） | 中 |
| N-06 | **C 语言概念沙盒** | 拖拽探索指针/数组/内存。因有完整 VM，可在受控环境中有意展示错误行为。 | VM + 内存可视化 + trap 处理（已有） | 中 |
| N-07 | **教师控制台** | 查看全班统计、识别常见错误、推送示例。 | 需要服务端，超出单机 App 范围 | 低 |

---

## 附录 A：测试覆盖评估

| 类别 | 已测试 | 缺口 |
|------|--------|------|
| 基本语法 | 全面 | — |
| 控制流 | 全面 | — |
| 数组/结构体/联合体 | 全面 | union 数组、static 变量、extern |
| 浮点/双精度 | 全面（含 epsilon 比较） | — |
| 函数指针 | 14 个测试 | — |
| 内存安全 | 7 个测试（泄漏/UAF/double-free） | 堆/栈碰撞检测 |
| 算法可视化 | 8 种算法 E2E | 可视化事件正确性验证 |
| 认知推理 | 36 个内联单元测试 | E2E 测试 |
| 编译错误 | 部分 | 缺少 `assert_compile_error!` 框架 |

## 附录 B：快速修复清单

| # | 文件 | 问题 | 难度 |
|---|------|------|------|
| 1 | `vm/vm.rs:567` | `write_cstring` 边界条件 `<` 应改为考虑 null 终止符 | 1 行 |
| 2 | `session.rs` | 手动实现 `Default` 设 `heap_offset = HEAP_START` | 5 行 |
| 3 | `session_ops.rs:57` | `0x5000` → `crate::vm::vm::HEAP_START` | 1 行 |
| 4 | `ROADMAP.md:158` | "~30 条指令" → "106 条指令" | 1 行 |
| 5 | `AGENTS.md` Phase 11 | "240 个单元测试" → "331 个测试" | 1 行 |

---

**报告结束**

*本次审阅基于对全部 ~44,000 行源代码（手写 Rust ~20,600 + 生成 ~4,900 + Dart ~17,600 + Python ~950）、61 份历史文档、7 份历史审阅报告的地毯式分析。历史 Bug 修复状态均已确认。部分初始判断（BUG-001、OPT-001、OPT-014、测试统计）在后续逐行代码验证中被修正，详见正文标注。*
