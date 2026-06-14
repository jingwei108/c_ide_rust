# Cide 项目全面代码审阅报告（修订版）

> 审阅日期：2026-06-13  
> 更新日期：2026-06-14（末轮更新）  
> 审阅范围：D:\code\c_ide_rust 全部源码  
> 覆盖：Rust 后端（native/src）+ Flutter 前端（CideFlutter/lib）+ 脚本 + 测试  
> 说明：本报告已复核原报告中的误判项，补充了 unsafe/FFI、VM 边界一致性、当前验证状态等遗漏维度。2026-06-14 基于本报告推进两轮修复并更新状态，详见第 13 节。

---

## 目录

1. [执行摘要](#1-执行摘要)
2. [验证方法](#2-验证方法)
3. [高危 Bug 勘误](#3-高危-bug-勘误)
4. [中低危 Bug 勘误](#4-中低危-bug-勘误)
5. [性能优化](#5-性能优化)
6. [框架迭代](#6-框架迭代)
7. [死代码消除](#7-死代码消除)
8. [unsafe / FFI 安全审计](#8-unsafe--ffi-安全审计)
9. [VM 边界检查一致性](#9-vm-边界检查一致性)
10. [测试与验证状态](#10-测试与验证状态)
11. [Flutter 测试框架](#11-flutter-测试框架)
12. [修复优先级](#12-修复优先级)
13. [修复推进记录](#13-修复推进记录)
14. [附录 A：原报告误判清单](#14-附录-a原报告误判清单)

---

## 1. 执行摘要

### 1.1 整体健康度

| 维度 | 状态 | 关键指标 |
|------|------|----------|
| Rust 单元/集成测试 | 健康 | 714 个测试全绿，clippy 0 警告 |
| Rust 架构 | 中等 | 核心模块 forbid(unsafe_code)，但 C API / Bridge 层 unsafe 集中 |
| VM 内存安全 | 高 | 核心路径有检查，边界策略已统一（V1-V10 已完成） |
| Shadow Verification | 高 | 463 用例，450 匹配（97.19%），剩余差异见 13.3 节 |
| Flutter 静态分析 | 健康 | 0 issues |
| Flutter 测试 | 健康 | 10 个测试文件、90 个单元/Widget 测试 + 4 个集成测试，全绿 |

### 1.2 关键发现

- 确认真实存在的高危问题 16 个，其中 P0/P1 级别（VM 边界检查、C API 生命周期、Flutter 状态管理、编译器状态泄漏）已全部修复。
- 确认原报告误判 12 处，包括 B2/B3/B8/B9/B11/B12/B19/B23/B24、D1/D2，以及无 CI 流水线等事实错误。
- 重大遗漏：原报告未对 unsafe/FFI、VM 边界检查一致性、当前 Shadow Verification 失败状态进行系统审计；本修订版已补充并推进修复。
- 新发现问题 10+ 个（NEW-R1~R7 等），其中 VM 边界检查类问题已全部修复，C API 返回字符串内部指针悬垂已修复，Flutter use_build_context_synchronously 已修复。
- 中低危 Bug 中已修复：B25/B27/B35/B36/B37/B38/B39/B40/B43/B44/B47/B48/B52；待处理/影响有限：B28/B30/B34/B41/B42。

### 1.3 审阅结论

原报告是一份有参考价值但不完备的审阅产物（约 6/10 分）。建议以本修订版作为修复执行清单，避免按原误判项直接改动代码。

---

## 2. 验证方法

本报告所有结论均基于以下实际验证：

| 工具/脚本 | 命令 | 结果 |
|-----------|------|------|
| Rust Clippy | cargo clippy -- -D warnings | 0 警告 |
| Rust 单元测试 | cargo test | 714 passed / 0 failed |
| Shadow Verification | python native/tests/shadow_verification/shadow_verify.py | 450/463 匹配 |
| Flutter Analyze | flutter analyze lib test | 0 issues |
| 源码核对 | ReadFile / Grep 逐条验证 | 详见各条目 |

---

## 3. 高危 Bug 勘误

### 3.1 Rust 编译器 — 高危 Bug

| # | 文件:行 | 严重性 | 描述 | 状态 |
|---|---------|--------|------|------|
| B1 | compiler/ast.rs:538 | 高 | from_base_kind 将 TemplateId 映射为 Class，丢失模板参数。所有模板实例化都会静默退化为基类。 | 已确认 |
| B2 | compiler/parser/mod.rs:343-366 | 中 | 结构体前瞻解析：当 is_var_decl=false 且 !is_pure_decl 时，会进入 else 分支解析为 class/struct，并非静默丢失。但 else 分支会忽略 struct 后面的非声明 token，存在语法处理不当问题。 | 原描述错误，问题降级 |
| B3 | compiler/parser/mod.rs:496-528 | 低 | C++ 构造函数外部定义：空函数体分支已消费尾部分号（第 511 行 consume(Semicolon)）。原报告误判。 | 误判 |
| B4 | compiler/parser/expr.rs:1126-1131 | 高 | parse_primary：未知 token 返回 Expr::Literal 但 token 未被消费，parse_statement 中可能无限循环。 | 已确认 |
| B5 | compiler/parser/expr.rs:496-517 | 中 | Cast 表达式：parse_type_only() 解析匿名结构体时会修改 anonymous_structs，但回滚时仅恢复 pos 和 typedef_names，状态泄漏。实际触发场景有限。 | 已确认，降级 |
| B6 | compiler/codegen/stmt.rs:42-43 | 中 | 静态局部变量和字符串字面量共享同一分配空间（string_mem_offset），但均正确递增，实际构造用例验证无地址重叠。问题降级为代码结构可读性，非功能性 Bug。 | 待确认/影响有限 |
| B7 | compiler/codegen/mod.rs:1070-1080 | 中 | compute_stride 中维度乘法 stride *= dim 可能溢出 i32（如 [100000][100000]），产生垃圾尺寸。原报告行号引用错误（写成 225-263）。 | 已修复：使用 checked_mul，溢出时返回 0 sentinel |
| B8 | compiler/codegen/expr.rs:1358-1361 | 中 | gen_addr：变量未找到时报错但没有 return/占位指令。原报告继续到 _ => 分支描述错误，但错误路径未生成占位值，可能导致后续字节码缺少操作数。 | 问题降级，描述修正 |
| B9 | compiler/codegen/cpp_this_new_delete.rs:191-254 | 低 | gen_delete：当析构函数未注册时跳过整个 if is_class 块。这是 C++ 正常语义（无析构函数则不释放内部指针），free 对象内存本身无错。 | 误判 |
| B10 | compiler/codegen/cpp_this_new_delete.rs:77-101 | 中 | 数组 new T[n]：构造函数循环中若某个构造失败 / trap，已分配内存永不释放。 | 已修复：新增 `SET_ARRAY_GUARD`/`CLEAR_ARRAY_GUARD` host func + `ArrayConstructionGuard`，trap 时回滚释放 base 内存 |
| B11 | compiler/lexer.rs:880-911 | 低 | string_literal 十六进制转义：xHH 分支调用 4 次 advance()（消费反斜杠 x h1 h2），行为正确。原报告 6 次为误判。 | 误判 |
| B12 | compiler/typeck/mod.rs:217-219 | 低 | merge_out_of_line_method_definitions：对 __ctor__Foo__3，普通方法分支因 class_name 为空不会进入；构造函数由第 235 行专门分支处理。原报告 split 取错段描述不准确。 | 误判 |

### 3.2 Rust VM / 引擎 / 桥接 — 高危 Bug

| # | 文件:行 | 严重性 | 描述 | 状态 |
|---|---------|--------|------|------|
| B13 | vm/vm/mod.rs:842 | 中 | `>` 语义（`addr + size > MEM_SIZE` 时 trap）已正确覆盖 `[0, MEM_SIZE)`；改为 `>=` 会错误拒绝访问最后一个有效字节。原报告例子不成立。executor.rs 内联检查使用相同语义，行为一致。 | 误判 |
| B14 | vm/vm/mod.rs:504 | 中 | call_user_function 使用 debug_assert! 验证参数—— Release 构建下不检查，8 字节类型参数静默损坏内存。 | 已修复：debug_assert! → assert! |
| B15 | vm/jit_templates.rs:580 | 中 | 将函数指针强转为 usize 做比较——Rust 不保证函数指针地址唯一。可移植性差，但非即时安全漏洞。 | 已确认，降级 |
| B16 | flutter_bridge.rs:20-35 | 高 | Box::leak 永久泄漏 Session/Engine 内存。destroy_session 只从 map 移除，实际内存永不释放。 | 已确认 |
| B17 | flutter_bridge.rs:37-48 | 中 | current_unified_engine() 在释放 Map 锁后、获得 Engine 锁前存在竞态窗口。但因 Box::leak，engine 内存不会被释放，实际影响有限。 | 已确认，降级 |
| B18 | capi/mod.rs:197-206 | 高 | cide_get_compile_errors 返回 String::as_ptr() 裸指针——任何编译操作都会使指针悬垂。 | 已确认 |
| B19 | unified/stream.rs:604-606 | 低 | get_sym 使用 sym.get(idx as usize)，已做越界检查，返回空字符串。原报告直接 panic 为误判。 | 误判 |

### 3.3 Flutter 前端 — 高危 Bug

| # | 文件:行 | 严重性 | 描述 | 状态 |
|---|---------|--------|------|------|
| B20 | lib/providers/ide_notifier.dart:175-177 | 高 | compile() 成功后启动统一模式；run() 也调用 compile() 后再 runCode()。存在双重编译/执行风险。 | 已确认 |
| B21 | lib/widgets/editor_panel_v2.dart:496-498 | 高 | build 期间调用 _document.setText()——可能触发 notifyListeners()，属于 Flutter 反模式。 | 已确认 |
| B22 | lib/providers/ide_notifier.dart:14 + unified_notifier.dart:11 | 高 | 使用 Notifier 而非 AutoDisposeNotifier。Notifier 无 dispose 生命周期，UnifiedNotifier.dispose() 为无效 override（Flutter analyze 已报错），TextEditingController 等资源泄漏。 | 已确认 |
| B23 | lib/screens/ide_screen.dart:485-496 | 低 | FloatingActionButton 使用 mini: 参数。Flutter 3.29 中 mini 仍然有效且未废弃。原报告误判。 | 误判 |
| B24 | lib/main.dart:15-21 | 低 | destroySession() 在生命周期 handler 中已 await，handler 返回 Future，系统通道会等待。原报告 fire-and-forget 为误判。 | 误判 |
| NEW-F1 | lib/widgets/learning_path_panel.dart:221 | 高 | Navigator.pop(context) 在 async gap 之后，widget 可能已 unmount。Flutter analyze 已报错 use_build_context_synchronously。 | 新发现 |


## 4. 中低危 Bug 勘误

### 4.1 确认真实的问题

| # | 文件:行 | 严重性 | 描述 |
|---|---------|--------|------|
| B25 | compiler/ast.rs:742-757 | 中 | Deref for InitElement 解引用到 Expr，忽略 designators | 已修复：删除 Deref/DerefMut，23 处调用改为显式 `.value` |
| B27 | compiler/ast.rs:489-491 | 中 | total_elements 在 array_size > 0 但为 VLA 负值时返回负值 | 已修复：多维 VLA array_size 计算避免产生负数 |
| B28 | compiler/lexer.rs:1159 | 低 | #include 拼接修改 self.chars 但不更新 line/column | 待处理：当前实现将 stub 换行替换为空格，line/column 由后续 advance 自然维护；影响有限 |
| B30 | compiler/parser/mod.rs:1301 | 低 | struct Foo 在 Foo 已是 class 时返回 Type::Class 而非 Type::Struct | 待处理：C++ 模式下 class/struct 等价引用的有意设计，需明确是否统一表示 |
| B34 | compiler/codegen/stmt.rs:698 | 中 | DoWhile 未像 While 那样调用 enter_scope，break 的 emit_dtors_for_scope_exit 计算偏移 | 已确认描述不准确：While 与 DoWhile 均未调用 enter_scope，符合 C 标准语义（循环体作用域由 Block 自身管理） |
| B35 | compiler/cfg.rs:272 | 低 | If 块克隆整个 AST 子树（仅需 SourceLoc） | 已修复：条件块只保留条件表达式占位语句 |
| B36 | compiler/cfg.rs:413-418 | 中 | 带 Goto/Branch 终结的块可能被误加 fall-through 边 | 已修复：Return 块不再被误加 fall-through 边；Goto/Branch 原本已正确处理 |
| B37 | compiler/data_flow.rs:44-46 | 中 | analyze_live_variables 边遍历为 O(N*E)，大 CFG 极慢 | 已修复：预计算出边邻接表，单次迭代 O(E) |
| B38 | compiler/intent.rs:141 | 低 | 回边检测 a >= b 依赖块 ID 分配顺序，不可靠 | 已修复：改为复用 cfg.find_loops() 结果 |
| B39 | compiler/typeck/mod.rs:847-888 | 中 | check_pointer_assignable 允许任意指针到任意指针赋值 | 已修复：双指针分支增加 pointee 兼容性检查，不兼容时报告 warning |
| B40 | compiler/typeck/decl.rs:89 | 中 | 构造函数 init 中 this 插入仅处理第一个变量，忽略 extra_vars | 已修复：提取 try_process_ctor_init，统一处理 name 与 extra_vars |
| B43 | vm/host_funcs.rs:508-509 | 中 | host_printf_n 无栈深度验证 | 已修复：host_printf_n/host_scanf_n/host_fprintf_n 均增加前置栈深度校验 |
| B44 | vm/host_funcs.rs:1496 | 中 | host_qsort 逐字节写回大数组极慢 | 已修复：结果写回改为按元素块 write_memory（见 13.2c / O10）
| B46 | engine/compile_pipeline.rs:365-369 | 低 | 无条件 println! 调试输出在生产代码中 | 已删除 |
| B47 | engine/session_ops.rs:82-102 | 中 | take() + 放回模式——若中间 panic 则 VM 永久丢失 | 已修复：execute_run 用 catch_unwind 保护 take 后的 VM，panic 时还回 session |
| B48 | engine/completion/candidates.rs:488-498 | 中 | find_variable_type 用空格 split 参数类型，unsigned int 被错误解析 | 已验证：rsplitn(2, ' ') 已正确处理 unsigned int；新增回归测试 |
| B52 | diagnostics/knowledge_graph.rs:448-459 | 低 | ERROR_CONCEPT_MAP 键 3035 重复——scanf 相关映射被覆盖 | 已修复：删除重复 3035 插入，将 3030-3035 统一映射到 FunctionCall/ParameterPassing |
| NEW-R1 | vm/vm/mod.rs:661 | 高 | write_cstring 只检查上界，不检查 addr < NULL_TRAP_SIZE | 已修复：统一调用 check_mem_access |
| NEW-R2 | vm/vm/executor.rs:381 | 高 | OpCode::Memcpy 只检查 dest/src >= mem_size，不检查 NULL 区 / UAF | 已修复：统一调用 check_mem_access + check_uaf |
| NEW-R3 | vm/vm/executor.rs:428 | 中 | OpCode::Strlen 只检查 start >= mem.len()，不检查 NULL 区 | 已修复：显式检查 addr < NULL_TRAP_SIZE |
| NEW-R4 | vm/vm/executor.rs:403 | 中 | OpCode::Memset 对越界采用静默截断，与 write_memory trap 行为不一致 | 已修复：统一调用 check_mem_access，越界时 trap |
| NEW-R5 | capi/mod.rs:605-650 | 中 | cide_memory_get_value / get_pointer_target 只检查 addr + 4 <= mem.len()，不检查 NULL 区 | 已失效：相关 C API 已作为死代码移除 |
| NEW-R6 | flutter_bridge.rs:448-467 | 中 | read_memory 只检查 offset + 4 <= mem.len()，不检查 NULL 区 | 已修复：显式检查 word_addr >= NULL_TRAP_SIZE |
| NEW-R7 | vm/vm/mod.rs:1011-1101 | 中 | get_array_snapshots 只检查 addr + elem_size > MEM_SIZE，不检查 NULL 区 | 已修复：循环内显式检查 addr < NULL_TRAP_SIZE |

### 4.2 待确认/影响有限

| # | 文件:行 | 说明 |
|---|---------|------|
| B26 | compiler/ast.rs:1226-1263 | compute_type_size 模板类名不匹配时静默返回 0——需确认是否会导致实际用例失败 |
| B29 | compiler/parser/mod.rs:73 | FILE 预注册为 Type::void()，用户定义 struct FILE 会被遮蔽——需确认是否有实际冲突 |
| B31 | compiler/parser/expr.rs:1080-1096 | This 生成 Type::default() 而非指针到类类型——需结合 typeck 后续修正判断 |
| B32 | compiler/codegen/expr.rs:1439 | gen_struct_copy 当 size 非 4 的倍数时 copy 0 词——需验证结构体大小为奇数时行为 |
| B33 | compiler/codegen/expr.rs:399-444 | 逗号运算符 Swap+Pop 在 64 位值和 32 位值混合时栈布局错误——需构造复现用例 |
| B41 | compiler/typeck/cpp_overload.rs:142-143 | 默认构造函数的隐式+显式定义可能误报 E4031 歧义——需具体用例 |
| B42 | compiler/codegen/cpp_lambda.rs:68-73 | ByValue 大型结构体捕获仅推 4 字节——需验证 lambda 捕获语义 |

---

## 5. 性能优化

### 5.1 Rust 性能优化

| # | 文件:行 | 严重性 | 优化建议 |
|---|---------|--------|----------|
| O1 | compiler/ast.rs:258-311 | 高 | mangle_name() 每级递归创建新 String。深嵌套类型产生 O(n^2) 分配。建议用可复用 String buffer。 |
| O2 | compiler/codegen/expr.rs:37-1093 | 高 | gen_expr 为 1056 行的单函数，深层嵌套 match 分支。建议按表达式类型拆分为独立方法。 |
| O3 | compiler/unified/engine.rs:145 | 高 | 每个步骤调用 vm.snapshot() 克隆 1MB 内存——100K 步骤 = 100GB 分配流量。应使用增量快照（已有 snapshot_incremental）。 |
| O4 | vm/vm/mod.rs:316 | 高 | snapshot() 无条件克隆 1MB 内存（同 O3）。 |
| O5 | vm/jit_templates.rs:630-636 | 中 | execute_trace_bulk 条件性终结的 trace 循环仅执行 1 次，JIT 加速完全失效。 |
| O6 | compiler/codegen/expr.rs:656-870 | 中 | Call vs CallPtr 的结构体/复合参数传递逻辑几乎完全重复。建议统一为一个 helper。 |
| O7 | compiler/codegen/mod.rs:900-991 | 中 | flatten_global_init 每次递归调用创建中间 Vec，立即丢弃。 |
| O8 | compiler/algorithm_detector.rs:952-1055 | 中 | expr_to_string 每个表达式节点分配新 String（format!），树遍历中大量字符串分配。 |
| O9 | compiler/data_flow.rs:44-46 | 中 | 边遍历 O(N*E)，建议构建邻接表一次后复用。 |
| O10 | vm/host_funcs.rs:1496 | 中 | host_qsort 逐字节 store_i8 写回——大数组排序极慢。 |
| O11 | unified/algorithm_steps.rs | 低 | 60+ 分支的 match，建议用 HashMap<&str, InferenceFn> 调度表替代。 |
| O12 | compiler/codegen/mod.rs:60-106 | 低 | BytecodeGen 40+ 字段，可拆分为 FunctionContext、GlobalContext、ScopeContext。 |

### 5.2 Flutter 性能优化

| # | 文件:行 | 严重性 | 优化建议 |
|---|---------|--------|----------|
| O13 | lib/editor/editor_painter.dart:47-75 | 高 | 每帧为可见行创建 + layout TextPainter。应预缓存 TextPainter 并仅在文本变化时重建。 |
| O14 | lib/editor/cide_document.dart:276-284 | 高 | _rebuildLineOffsets() 每次按键 O(n) 全文扫描。注释声称局部重建，但实现仍是全量扫描，需同步实现或修正注释。 |
| O15 | lib/widgets/floating_orb_widget.dart:565-827 | 高 | _BreathingOrbPainter.paint() 中每帧创建 MaskFilter.blur + RadialGradient 对象。应预创建并缓存。 |

---

## 6. 框架迭代

### 6.1 Rust 架构问题

| # | 文件:行 | 严重性 | 问题与建议 |
|---|---------|--------|-----------|
| A1 | compiler/ast.rs:58-88 | 高 | Type 枚举中 is_const: bool 在几乎所有变体中重复。建议用 Const(Type) 包装器或外层布尔值。set_const 忽略 RValueRef/Auto/TemplateId，与 is_const() 不一致。 |
| A2 | compiler/typeck/mod.rs:15-54 | 低 | VarSymbol.is_global、MethodSig.is_explicit、MethodSig.is_static 均标注 #[allow(dead_code)] 但实际被使用。标注移除可启用编译器 dead code 检测。 |
| A3 | compiler/cpp_frontend/type_map.rs:12-15 | 中 | builtin_layout_data.json 被两次加载（builtin_layout.rs 和 type_map.rs 各 include_str! 一次），浪费启动时间和内存。 |
| A4 | compiler/cfg.rs / data_flow.rs / intent.rs / algorithm_detector.rs | 中 | 每个模块独立实现 parse_func 测试辅助——无共享测试工具。 |
| A5 | compiler/codegen/expr.rs / stmt.rs | 中 | ExprGen 和 StmtGen trait 仅用于方法分组，非 object-safe。可直接改为 impl BytecodeGen 方法。 |
| A6 | parser/mod.rs:52-61 | 低 | anonymous_structs: Vec<StructDecl> 将匿名结构体概念泄漏到 ProgramNode 结构。 |
| A7 | vm/jit_templates.rs | 低 | JIT 模板函数使用 wrapping_add/wrapping_sub 而主解释器用 checked 运算——溢出行为不一致。 |
| A8 | 全局 | 低 | 中英文混合——错误消息为中文，代码注释混合。对非中文贡献者造成维护摩擦。 |
| A9 | flutter_bridge.rs:80-91 | 高 | current_session() 返回 MutexGuard<'static, Session>，长时间持有；run_auto_steps_stream 后台线程与 UI 线程可能死锁。 |
| A10 | flutter_bridge.rs:39-48, 80-91 | 中 | 对可能 poison 的 Mutex 使用 unwrap_or_else(|e| e.into_inner()) 吞掉 poison 状态，panic 后可能继续使用已损坏的 Session。 |

### 6.2 Flutter 架构问题

| # | 文件:行 | 严重性 | 问题与建议 |
|---|---------|--------|-----------|
| A11 | lib/providers/unified_notifier.dart:18-22 | 高 | @override void dispose() —— Notifier 在 flutter_riverpod 中无 dispose 方法，此 override 无效且误导（Flutter analyze 已报错）。应切换到 AutoDisposeNotifier 或移除。 |
| A12 | lib/providers/* | 中 | IdeNotifier 和 UnifiedNotifier 直接依赖 FFI 调用，难以单元测试。应通过接口/抽象层注入依赖。 |
| A13 | lib/screens/ide_screen.dart | 中 | ide_screen.dart 承载过多职责（全 IDE 布局 + 键盘处理 + 统一模式），建议拆分为多个 compose widget。 |
| A14 | lib/widgets/floating_orb_widget.dart:565-827 | 中 | _BreathingOrbPainter 内联大量绘制逻辑，GPU 开销高。应使用 RepaintBoundary 隔离并考虑简化动画。 |

---

## 7. 死代码消除

### 7.1 确认的死代码

| # | 文件:行 | 状态 | 描述 |
|---|---------|------|------|
| D3 | compiler/ast.rs:844 | 误判 | impl Default for Expr 被 `std::mem::take` 多处使用 |
| D4 | compiler/codegen/mod.rs:46 | 已清理 | ClassVarEntry.name —— 写入但从不读取，已移除字段 |
| D5 | compiler/typeck/mod.rs:1003-1011 | 已清理 | pub(crate) fn get_class_field_type() —— 0 个调用点，已删除 |
| D6 | compiler/cpp_frontend/builtin_layout.rs:29,38,41 | 已清理 | source_file、version、generated_at —— 已移除未使用字段 |
| D7 | compiler/cpp_frontend/type_map.rs:8,11 | 已清理 | version、generated_at —— 已随 type_map.rs 精简移除 |
| D8 | compiler/cpp_frontend/type_map.rs:12-15 + builtin_layout.rs | 已合并 | type_map.rs 改为 re-export builtin_layout.rs 的函数，JSON 只加载一次 |
| D9 | lib/widgets/symbol_bar.dart (82行) | 已删除 | SymbolBar 类 —— 整个项目中零次导入/实例化 |
| D10 | lib/widgets/symbol_chip.dart (33行) | 已删除 | SymbolChip 类 —— 仅被已删除的 SymbolBar 使用 |
| D11 | lib/widgets/call_stack_tree.dart (111行) | 已删除 | CallStackTree 类 —— 从未实例化；CallstackTab 自行构建 UI |
| D12 | lib/widgets/concept_graph_view.dart:346 | 误判 | const y = 12.0 在 _drawLegend 中被直接使用 |
| D13 | lib/providers/unified_notifier.dart:18-22 | 已完成 | 已切换为 AutoDisposeNotifier，dispose 逻辑通过 ref.onDispose 处理 |

### 7.2 误判的死代码

| # | 文件:行 | 说明 |
|---|---------|------|
| D1 | compiler/ast.rs:247-256 | function_pointer() 在 typeck/expr.rs:403 被调用，不是死代码 |
| D2 | compiler/ast.rs:359-364 | array_size() 在 codegen/stmt.rs、typeck/mod.rs 等多处被调用，不是死代码 |

### 7.3 空目录

| 目录 | 建议 |
|------|------|
| CideFlutter/lib/controller/ | 删除或添加 .gitkeep 占位 |
| CideFlutter/third_party/ | 删除 |
| dist/ | 删除 |
| native/tests/cases/leetcode/ | 删除 |


---

## 8. unsafe / FFI 安全审计

### 8.1 总体分布

`native/src` 中共有 **106 处 `unsafe`**，集中在：
- `capi/mod.rs`：C API 封装（49 处）
- `frb_generated.rs`：flutter_rust_bridge 自动生成
- `vm/mod.rs` / `vm/executor.rs` / `vm/jit_templates.rs`：VM 内存操作
- `compiler/mod.rs`、`diagnostics/mod.rs`、`api/mod.rs`：少量 FFI 边界

核心编译器模块（`compiler/lexer.rs`、`compiler/parser/*.rs`、`compiler/typeck/*.rs` 等）均声明 `#![forbid(unsafe_code)]`，整体架构良好。

### 8.2 具体问题

| # | 文件:行 | 风险等级 | 描述 | 修复建议 |
|---|---------|----------|------|----------|
| S1 | capi/mod.rs:197-213 | 中 | cide_get_compile_errors 返回 session.compile.errors.as_ptr() 的内部缓冲区指针。跨 compile 调用后悬垂。 | 已在 Session 中缓存 CString，返回其指针；下次调用或销毁前有效 |
| S2 | capi/mod.rs:446-469 | 中 | cide_get_runtime_error 返回 session.runtime.error_buffer.as_ptr()。跨 run/step 调用后若 error 被改写会悬垂。 | 已在 Session 中缓存 CString，返回其指针；下次调用或销毁前有效 |
| S3 | flutter_bridge.rs:20-25,30-35,52-57,84-88 | 中 | Box::leak(Box::new(Mutex::new(...))) 将 Session/Engine 提升为 'static。destroy_session 只移除 map 条目，内存永不释放。 | 已将 HashMap 改为 Arc<Mutex<T>>；destroy_session 移除后 Arc 引用计数归零即可释放 |
| S4 | flutter_bridge.rs:80-91 | 中 | current_session() 返回 MutexGuard<'static, Session>，长时间持有；与 run_auto_steps_stream 后台线程可能死锁。 | current_session() 改为返回 Arc<Mutex<T>>，调用点按需 lock，锁持有范围更明确 |
| S5 | flutter_bridge.rs:39-48,80-91 | 中 | 对 poison 的 Mutex 使用 unwrap_or_else(|e| e.into_inner()) 吞掉 poison，可能继续使用已损坏状态。 | 明确处理 poison：传播错误或恢复前重置 Session |
| S6 | capi/mod.rs:1 | 低 | #![allow(clippy::missing_safety_doc)] 隐藏缺失的安全文档。 | 已完成：已移除 allow，为 capi/mod.rs 所有 unsafe extern "C" 函数补全 # Safety 文档 |
| S7 | compiler/cpp_frontend/type_map.rs:34,50 | 低 | Box::leak(cide_name.clone().into_boxed_str()) 每次调用都泄漏字符串。 | 已完成：type_map.rs 复用 builtin_layout.rs 的 LazyLock 缓存映射 |
| S8 | compiler/cpp_frontend/builtin_layout.rs:138-139 | 低 | builtin_class_mappings() 每次调用都 Box::leak 新字符串。 | 已完成：改用 LazyLock<HashMap> 一次性缓存 |
| S9 | vm/jit_templates.rs:580,633-634 | 低 | 函数指针转 usize 比较，依赖实现定义行为。 | 使用 OpCode 枚举或其他稳定标识 |

### 8.3 总体评估

| 维度 | 评估 |
|------|------|
| unsafe 集中度 | 高：几乎所有 unsafe 都在 capi/mod.rs 和 FRB 生成代码 |
| C API 裸指针 | 做了普遍的 null 检查与长度检查，但返回 String 内部指针是最大隐患 |
| Bridge 并发 | Box::leak + MutexGuard<'static> 是主要风险点 |
| 危险转换 | 未发现 std::mem::transmute；函数指针转 usize 为低危用法 |
| 整体安全等级 | 中-高：VM 与 C API 是主要攻击面 |

---

## 9. VM 边界检查一致性

核心 `load/store_i32/i64/i8` 通过 `check_mem_access` 统一检查，但多条辅助/快速路径绕过或未完整执行检查：

| # | 文件:行 | 当前行为 | 缺失检查 | 修复建议 |
|---|---------|----------|----------|----------|
| V1 | vm/vm/mod.rs:661-676 | write_cstring 检查上界 | 未检查 addr < NULL_TRAP_SIZE | 已修复：统一调用 check_mem_access |
| V2 | vm/vm/executor.rs:381-401 | OpCode::Memcpy 检查 dest/src >= mem_size | 未检查 NULL_TRAP_SIZE、UAF | 已修复：统一调用 check_mem_access + check_uaf |
| V3 | vm/vm/executor.rs:428-438 | OpCode::Strlen 检查 start >= mem.len() | 未检查 NULL_TRAP_SIZE | 已修复：前置 NULL 区检查 |
| V4 | vm/vm/executor.rs:403-427 | OpCode::Memset 对越界静默截断 | 策略与 write_memory 不一致 | 已修复：统一调用 check_mem_access，越界 trap |
| V5 | capi/mod.rs:605-625 | cide_memory_get_value | 未检查 NULL_TRAP_SIZE | 已失效：相关 C API 已移除 |
| V6 | capi/mod.rs:628-650 | cide_memory_get_pointer_target | 未检查 NULL_TRAP_SIZE | 已失效：相关 C API 已移除 |
| V7 | flutter_bridge.rs:448-467 | read_memory | 未检查 NULL_TRAP_SIZE | 已修复：加入 NULL 区判断 |
| V8 | vm/vm/mod.rs:1011-1101 | get_array_snapshots | 未检查 NULL_TRAP_SIZE | 已修复：加入 NULL 区判断 |
| V9 | vm/vm/mod.rs:459-578 | call_user_function 通过 store_i32 写参数、store_i8 清零局部变量 | 已走统一内存访问路径 | 已确认修复：自然经过 check_mem_access + check_uaf |
| V10 | vm/vm/executor.rs:963-991 | do_call 通过 store_i32 写参数、store_i8 清零局部变量 | 已走统一内存访问路径 | 已确认修复：自然经过 check_mem_access + check_uaf |

**修复原则**：所有内存访问最终都应经过 `check_mem_access(addr, size, loc, is_write)`，并统一 `NULL_TRAP_SIZE`、越界、UAF 三种检查策略。

---

## 10. 测试与验证状态

### 10.1 当前状态

| 防线 | 状态 | 关键数据 |
|------|------|----------|
| Rust 单元/集成测试 | 全绿 | 714 passed / 0 failed |
| Clippy | 全绿 | 0 warnings |
| Shadow Verification | 部分失败 | 463 用例，450 匹配（97.19%）；剩余差异见 13.3 节 |
| C++ Shadow | 健康 | 82/82 匹配，0 gap（历史状态） |
| Fuzz Stress | 未运行 | 上次状态见 native/tests/FUZZ_FAILURES.md |
| Flutter 测试 | 健康 | 10 个测试文件、90 个单元/Widget 测试 + 4 个集成测试，全绿 |

### 10.2 Shadow Verification 缺口详情

运行命令：
```bash
python native/tests/shadow_verification/shadow_verify.py --cases native/tests/shadow_verification/cases --report reports/shadow_report.json
```

结果摘要：
- 总用例数：463
- 完全匹配：450（97.19%）
- 编译缺口：3（0.65%）
- 运行时缺口：3
- 输出差异：3
- cide_better：2（`keyword_compat`、`merge_default`）
- known_issue：2（`function_pointer_sizeof`、`sizeof_array_param`）

主要编译缺口原因：
- `inline_asm`、`static_assert`、`typeof`：各 1 个（均为 Cide 教学子集已知不支持特性）

运行时缺口：
- `bTree_default`：pre-existing 问题，根因为访问 NULL 指针区域（地址 0x0010）
- `infixEvaluation_default`、`spfa_default`：Cide 数组越界检查更严格，触发 `TrapBounds`

输出差异：
- `vfs_io_extensions`：Cide 输出 `Hi! 10 4 0 H i i 4`，Clang 输出 `Hi! 10 5 0 H i i 5`（差异根因：Windows 文本模式下 Clang 将 `\n` 写为 `\r\n`，Cide VFS 未转换）
- `file_fread`：Cide 输出 `hello`，Clang 输出 `Hi!`（根因同上，预设文件内容差异）
- `kr_5_8`：Cide 输出 `-1 0 2.71 3.14 42`，Clang 输出 `0 3.14 42 -1 2.71`（函数指针类型转换 UB 导致 Clang 优化差异）

### 10.3 与 FAILURES.md 的对齐

项目已有 11 个 `*_FAILURES.md` 文件。修复 Bug 后必须同步检查这些文件，确保：
- 已修复的 KNOWN_FAILURE 从文档中移除；
- 新出现的失败在文档中登记。

CI 中的 `scripts/ci_three_tier_check.py` 会自动执行上述一致性检查。

---

## 11. Flutter 测试框架

### 11.1 现状评估

**当前覆盖率：已补齐 Model/Provider/Widget 基础层 + 集成测试入口**

| 现有文件 | 内容 |
|----------|------|
| test/ide_provider_test.dart | 2 个单元测试：IdeState 默认值 + copyWith |
| test/models/ide_state_test.dart | 8 个测试：默认值、copyWith、CodeFile、hasErrors/hasWarnings |
| test/models/unified_state_test.dart | 14 个测试：默认值、copyWith、ExecutionPhase getter 矩阵 |
| test/providers/theme_notifier_test.dart | 3 个测试：默认值、toggle、setDark/setLight |
| test/providers/ide_notifier_test.dart | 30 个测试：build、文件管理、面板管理、watch、进度、教程 |
| test/providers/unified_notifier_test.dart | 8 个测试：build、播放控制、onCodeChanged |
| test/widgets/custom_keyboard_test.dart | 10 个测试：字母/数字/符号模式、配对键、Shift、Space |
| test/widgets/file_tab_bar_test.dart | 6 个测试：渲染、切换、关闭按钮、关闭回调、新建文件 |
| test/widgets/tool_button_test.dart | 4 个测试：图标渲染、点击、禁用、自定义颜色 |
| test/services/learning_progress_service_test.dart | 5 个测试：load/save/clear、非法 JSON 回退 |
| test/models/code_template_test.dart | 7 个测试：占位符替换、默认参数、模型字段 |
| integration_test/app_test.dart | 4 个端到端 smoke 测试：启动、主题切换、底部 Tab 切换、新建文件 |
| test_driver/integration_test.dart | Driver 入口（`integrationDriver()`） |

**Flutter analyze lib test 结果（0 issues）**：
- 前次 4 个 issue 已修复：`Notifier` → `AutoDisposeNotifier` 迁移、异步 context 使用 `mounted` 保护、unused variable 清理。
- 新增测试文件均通过 `flutter_lints` 检查，无 unused import 或 unnecessary import。

### 11.2 说明

原报告声称"无 CI 流水线"、"缺失 flutter_lints/integration_test"与事实不符：
- `.github/workflows/ci.yml` 已存在完整 Flutter CI（pub get、FRB 生成、flutter test、build windows）。
- `pubspec.yaml` 已包含 `flutter_lints: ^5.0.0` 和 `integration_test`。
- `mocktail` 已在本次推进中添加（`^1.0.4`），用于未来 mock Rust API 抽象层。

### 11.3 已落地项

| 层级 | 新增文件数 | 新增测试数 | 状态 |
|------|-----------|-----------|------|
| 单元测试 (Model) | 3 | 29 | ✅ 完成 |
| 单元测试 (Provider 逻辑) | 3 | 41 | ✅ 完成（Notifier 纯状态/Rust 无关路径） |
| 单元测试 (Service) | 1 | 5 | ✅ 完成 |
| Widget 测试 | 3 | 20 | ✅ 完成 |
| 集成测试 | 1 | 4 | ✅ 完成（smoke 级） |
| **总计** | **10** | **+88** | |

本轮关键落地测试：
- ✅ `test/models/ide_state_test.dart`
- ✅ `test/models/unified_state_test.dart`
- ✅ `test/providers/theme_notifier_test.dart`
- ✅ `test/providers/ide_notifier_test.dart`
- ✅ `test/providers/unified_notifier_test.dart`
- ✅ `test/widgets/custom_keyboard_test.dart`
- ✅ `test/widgets/file_tab_bar_test.dart`
- ✅ `test/widgets/tool_button_test.dart`
- ✅ `test/services/learning_progress_service_test.dart`
- ✅ `test/models/code_template_test.dart`

### 11.4 剩余补齐项

| 层级 | 目标文件 | 优先级 |
|------|----------|--------|
| Provider | 对 Rust API 抽象后可 mock 的 `compile()`/`run()` 路径 | 中 |
| Widget | `breakpoints_tab_test.dart` 等剩余面板 | 低 |

---

## 12. 修复优先级

### 12.1 立即修复（P0 — 功能/安全）

1. ~~**V1-V10**：统一 VM 所有内存访问路径的边界检查~~（已完成）
2. ~~**S1/S2**：C API 返回字符串指针改为 buf 拷贝模式~~（已完成：改为 Session 内 CString 缓存）
3. ~~**B1**：`from_base_kind` 模板参数丢失~~（已完成：删除死码）
4. ~~**B4**：`parse_primary` 未知 token 不消费~~（已完成）
5. ~~**B20**：Flutter 双重编译/执行~~（已完成）
6. ~~**B21**：build 期间修改 document~~（已完成）
7. ~~**NEW-F1**：`learning_path_panel.dart:221` async context 问题~~（已完成）
8. ~~**B13**：`check_mem_access` off-by-one~~（已确认误判，`>` 语义正确）
9. ~~**B6**：静态局部变量与字符串字面量地址重叠~~（已确认无实际重叠，降级）

### 12.2 短期内修复（P1 — 稳定性/资源）

1. ~~**B16/B17/S3/S4**：`flutter_bridge.rs` 改为 `Arc<Mutex<T>>`，消除泄漏与死锁~~（已完成）
2. ~~**B22/A11**：`Notifier` → `AutoDisposeNotifier`，正确释放资源~~（已完成）
3. ~~**B14**：`call_user_function` `debug_assert!` → 普通 assert~~（已完成）
4. ~~**B7**：类大小 / stride 计算改用 `i64` 或 `checked_mul`~~（已完成：checked_mul）
5. ~~**B10**：`new T[n]` 构造失败时释放已分配内存~~（已完成）
6. ~~**S5**：处理 Mutex poison 状态~~（已完成：poison 时重置为默认）
7. ~~**B46**：移除生产代码中的 `println!`~~（已完成）

### 12.3 中期改进（P2 — 代码质量/性能）

1. ~~清理确认的死代码（D4-D11，D13），修正 D3/D12 误判~~（已完成）
2. ~~消除 C++ 内置容器映射的重复 JSON 加载与每次调用 Box::leak~~（已完成）
3. ~~移除生产代码残留 `println!` 调试输出~~（已完成，新增 B46 清理）
4. ~~修复 `printf` 输出缓冲行为，与 C 标准一致~~（已完成）
5. ~~修复 struct / enum 若干语法缺口~~（已完成：struct 多字段、typedef enum、匿名 enum、三目运算符数组 decay）
6. ~~**O1**：`Type::mangle_name` buffer 复用~~（已完成）
7. ~~**O4**：VM `snapshot()` 1MB buffer 复用~~（已完成：新增 `snapshot_into` + `UnifiedEngine.pre_step_snap` 复用）
8. ~~**O5**：JIT `execute_trace_bulk` 条件跳转 side-exit 处理错误导致只执行一轮~~（已完成）
9. ~~**B15/S9**：`jit_templates.rs` 函数指针转 `usize` 比较~~（已完成：`JitEntry` 新增 `is_conditional_jump` 标志）
10. ~~**O10**：`host_qsort` 逐字节 store_i8 写回极慢~~（已完成：改用 `write_memory` 按块写回）
11. 性能优化 O2-O3、O6-O9、O11-O15（待后续推进）
12. 架构重构 A1-A14
13. ~~**B25**：删除 `InitElement` 的 `Deref`/`DerefMut`~~（已完成）
14. ~~**B27**：多维 VLA `array_size` 避免负数~~（已完成）
15. ~~**B39**：不兼容指针类型赋值报告 warning~~（已完成）
16. ~~**B43**：`printf`/`scanf`/`fprintf` 参数不足前置校验~~（已完成）
17. ~~**V9/V10**：`call_user_function` / `do_call` 走统一内存访问路径~~（已确认修复）
18. ~~**B35**：If 条件块只保留条件表达式~~（已完成）
19. ~~**B36**：Return 块不再误加 fall-through 边~~（已完成）
20. ~~**B37**：`data_flow` 预计算出边邻接表~~（已完成）
21. ~~**B38**：回边检测复用 `cfg.find_loops()`~~（已完成）
22. ~~**B40**：extra_vars 构造函数 this 指针插入~~（已完成）
23. ~~**B47**：`execute_run` panic 时保护 VM~~（已完成）
24. ~~**B48**：`unsigned int` 参数类型解析~~（已验证并补充测试）
25. ~~**B52**：`ERROR_CONCEPT_MAP` 3035 重复~~（已完成）
26. 为每个 P0/P1 问题补充回归测试（部分完成）
27. ~~补齐 C API `# Safety` 文档~~（已完成）
28. ~~扩展 Flutter 测试金字塔（基础层已落地：6 文件 / 69 测试）~~
29. 分析并修复 Shadow Verification 的剩余缺口（当前 13 个，含已知限制与 pre-existing）

---

## 13. 修复推进记录

> 记录基于本报告于 2026-06-14 执行的修复轮次。

### 13.1 本轮完成项

| 编号 | 问题 | 修复文件 | 修复说明 |
|------|------|----------|----------|
| FIX-1 | 生产代码残留调试输出 | `capi/mod.rs`、`engine/compile_pipeline.rs` | 删除 `CAPI: calling...`、`DUMP: VarDecl...` 等 `println!` 及 `dump_var_decls` 死函数 |
| FIX-2 | `printf` 自动换行 | `session.rs` | `RuntimeState::output()` 从 `join("\n")` 改为 `join("")`，与 C 标准一致 |
| FIX-3 | struct 体不支持多字段声明 | `parser/mod.rs` | `parse_struct_body` 改为 `parse_base_type` + `parse_declarator` + 逗号分隔多字段 |
| FIX-4 | `typedef enum { } Alias;` 不支持 | `parser/mod.rs` | 新增 `parse_typedef_enum_decl` |
| FIX-5 | 匿名 `enum { };` 不支持 | `parser/mod.rs` | 顶层 Enum 分支不再强制消费 `Identifier` |
| FIX-6 | 三目运算符字符串字面量类型不匹配 | `typeck/expr.rs` | 对 Array 分支执行通常转换 decay 为 `char*` |
| FIX-7 | KNOWN_FAILURE 状态滞后 | `cide_e2e.rs`、`E2E_FAILURES.md`、`KR_FAILURES.md` | 移除已通过的 `bellmanFord_default`、`kr_5_16` |
| FIX-8 | Flutter 测试金字塔基础层 | `CideFlutter/test/**`、 `pubspec.yaml` | 新增 6 个测试文件、67 个测试；添加 `mocktail` 依赖 |
| FIX-9 | `closeFloatingPanel` 无法清除 active panel | `lib/models/ide_state.dart`、 `lib/providers/ide_notifier.dart` | `IdeState.copyWith` 新增 `clearActiveFloatingPanel` 参数，修复清除逻辑 |
| FIX-10 | Rust 测试 warnings | `native/tests/b10_new_array_rollback.rs`、 `native/tests/test_utils.rs` | 移除未使用导入；`test_utils.rs` 添加 `#![allow(dead_code)]` 避免共享测试工具被误报 |
| FIX-11 | Flutter 测试金字塔扩展层 | `CideFlutter/test/**` | 新增 4 个测试文件、21 个测试：Service/Model/Widget |
| FIX-12 | Flutter 集成测试入口 + 测试揭露的 UI Bug | `integration_test/app_test.dart`、`lib/models/panel_item.dart`、`lib/widgets/editor_panel_v2.dart`、`lib/main.dart` | 新增 4 个端到端 smoke 测试；修复 `PanelItem` 缺失 `intent`、`EditorPanelV2` 初始 `ScrollController` 未 attach、`RustLib.init` 重复初始化 |
| FIX-13 | CI Windows 构建缓存问题 | `.github/workflows/ci.yml` | Build Windows 前清理 `build/windows/x64`，避免 CMake 使用缓存的 `Visual Studio 16 2019` generator 在仅有 VS2022 的 runner 上失败 |
| FIX-14 | `cide_cli` 诊断级别显示错误 | `native/src/bin/cide_cli.rs` | 将 `severity` 映射从 `1=错误/2=警告/3=提示` 修正为 `0=错误/1=警告/2=提示`，与后端 `push_diagnostics/push_warnings/push_hints` 及 Flutter 前端保持一致 |
| FIX-15 | W3052 数组 decay 过度告警 | `native/src/compiler/typeck/mod.rs` | 移除 `check_array_pointer_assignable` 中对正常数组到指针隐式转换的 warning；保留 `sizeof(数组参数)` 场景下的专门 warning，避免 K&R 标准代码产生大量噪音诊断 |
| FIX-16 | Shadow Verification 输入模式不明确 | `native/src/capi/mod.rs`、`native/tests/shadow_verification/shadow_verify.py` | 新增 `cide_set_input_mode` C API；Shadow 验证脚本在运行前将 Session 设为 Batch 模式，使 `getchar` 在输入耗尽后返回 EOF 而非等待，与 Clang 在无输入时的行为一致 |
| FIX-17 | 全局变量区与字符串字面量区内存重叠 | `native/src/compiler/codegen/mod.rs`、`stmt.rs`、`expr.rs` | 全局初始化中的 `StringLiteral` 延迟到 Pass 1 结束后分配地址；静态局部变量与代码生成中的字符串改用 `next_global_offset` 分配，确保字符串区位于全局变量区之后。修复 K&R `kr_6_1` 中 `struct key keytab[]` 的 `char*` 成员被字符串内容覆盖的问题 |
| FIX-18 | Shadow Verification Clang 头文件冲突 | `native/tests/shadow_verification/shadow_verify.py` | `CLANG_HEADER` 不再包含 `stdlib.h`/`string.h`，避免 K&R 示例中用户自定义的 `itoa`/`qsort` 与标准库声明冲突；隐式声明的 `malloc`/`free`/`strcpy` 等在 Windows CRT 下仍可链接 |
| FIX-19 | K&R 失败记录同步 | `native/tests/cide_e2e.rs`、`native/tests/KR_FAILURES.md` | 将已修复的 `kr_6_1` 从 `KNOWN_KR_FAILURES` 移除，并更新 `KR_FAILURES.md` 条目为已修复状态 |
| FIX-20 | O1: `Type::mangle_name` 临时 String 分配过多 | `native/src/compiler/ast.rs` | 新增 `mangle_name_into(&self, buf: &mut String)`，所有分支直接向 buffer 写入；`mangle_name()` 改为封装调用，接口不变；消除嵌套类型递归中的 O(n²) 小字符串分配 |
| FIX-21 | O4: VM 单步回退快照每步分配 1MB Vec | `native/src/vm/vm/mod.rs`、`native/src/unified/engine.rs` | 新增 `CideVM::snapshot_into` 复用已有 1MB buffer；`UnifiedEngine` 新增 `pre_step_snap: Option<VMSnapshot>` 字段，首次 step 分配后后续复用；统一模式长程序堆分配流量不再随步数线性增长 1MB/步 |
| FIX-22 | O5: JIT Trace 批量执行只跑一轮 + Abort trace 被错误编译 | `native/src/vm/jit_templates.rs`、`native/src/vm/jit_trace.rs`、`native/src/vm/vm/executor.rs` | 修复 `execute_trace_bulk` 条件跳转 side-exit 逻辑：taken 回起点继续循环，未 taken 推进到 `end_ip` 退出；修复 `TraceRecorder::finish` 对 Abort 录制的处理；`JitEntry` 新增 `is_conditional_jump` 标志替代函数指针地址比较 |
| FIX-23 | O10: `host_qsort` 逐字节写回大数组极慢 | `native/src/vm/host_funcs.rs` | 排序结果从临时缓冲区写回 VM 内存由逐字节 `store_i8` 改为按元素块 `write_memory`；失败时 trap 并回退 qsort_depth |
| FIX-24 | B25: `InitElement` 隐式解引用忽略 designators | `native/src/compiler/ast.rs` 等 9 个文件 | 删除 `InitElement` 的 `Deref`/`DerefMut`；23 处调用改为显式 `.value` |
| FIX-25 | B39: 不兼容指针类型赋值无警告 | `native/src/compiler/typeck/mod.rs` | `check_pointer_assignable` 双指针分支增加 pointee 兼容性检查，不兼容时报告 `W3053` warning |
| FIX-26 | B43: `printf`/`scanf` 参数不足时行为不佳 | `native/src/vm/host_funcs.rs` | `host_printf_n` / `host_scanf_n` / `host_fprintf_n` 增加前置栈深度校验，一次性 trap |
| FIX-27 | B27: 多维 VLA `array_size` 可能为负 | `native/src/compiler/parser/mod.rs` | 当 `inner_array_size <= 0` 时避免 `size * inner_array_size` 产生负数 |
| FIX-28 | V9/V10: VM 调用帧初始化绕过统一内存检查 | `native/src/vm/vm/mod.rs`、`vm/vm/executor.rs` | 复核确认 `call_user_function` / `do_call` 已通过 `store_i32`/`store_i8` 走统一内存访问路径 |
| FIX-29 | B52: `ERROR_CONCEPT_MAP` 键 3035 重复 | `native/src/diagnostics/knowledge_graph.rs` | 删除重复 3035 插入；将 3030-3035（printf/scanf family）统一映射到 `FunctionCall`/`ParameterPassing` |
| FIX-30 | B35: If 条件块克隆整棵 AST | `native/src/compiler/cfg.rs` | 条件块 `stmts` 改为只包含 `Stmt::Expr { cond, loc }` 占位，不再克隆完整 If 子树 |
| FIX-31 | B36: Return 块误加 fall-through 边 | `native/src/compiler/cfg.rs` | `build_seq` 的 fall-through 边仅对 `Terminator::FallThrough` 添加，移除 `Return` |
| FIX-32 | B37: live-variable 分析 O(N*E) | `native/src/compiler/data_flow.rs` | 预计算出边邻接表，单次迭代从 O(N*E) 降至 O(E) |
| FIX-33 | B38: 回边检测依赖块 ID 顺序 | `native/src/compiler/intent.rs` | 改为复用 `cfg.find_loops()` 结果，不再用 `a >= b` 判断 |
| FIX-34 | B40: 构造函数 this 只插第一个变量 | `native/src/compiler/typeck/decl.rs` | 提取 `try_process_ctor_init`，统一处理 `name` 与 `extra_vars` 中的类类型变量 |
| FIX-35 | B47: execute_run panic 导致 VM 丢失 | `native/src/engine/session_ops.rs` | `take()` 后用 `catch_unwind` 保护运行阶段，panic 时把 VM 还回 session |
| FIX-36 | B48: `unsigned int` 参数类型解析 | `native/tests/completion_unit_test.rs` | 新增回归测试验证 `find_variable_type` 正确处理带空格类型名 |

### 13.2 测试与文档同步

- **新增 Rust 回归测试 5 个**：
  - `parser_unit_test.rs`：`test_parser_struct_multi_field_decl`、`test_parser_typedef_enum_anon`、`test_parser_enum_anon`
  - `type_checker_unit_test.rs`：`test_type_checker_ternary_string_literals`
  - `end_to_end_extra_test.rs`：`test_e2e_printf_no_auto_newline`
- **新增 Flutter 测试文件 10 个、测试 90 个**：
  - `test/models/ide_state_test.dart`：8 个测试
  - `test/models/unified_state_test.dart`：14 个测试
  - `test/models/code_template_test.dart`：7 个测试
  - `test/providers/theme_notifier_test.dart`：3 个测试
  - `test/providers/ide_notifier_test.dart`：30 个测试
  - `test/providers/unified_notifier_test.dart`：8 个测试
  - `test/widgets/custom_keyboard_test.dart`：10 个测试
  - `test/widgets/file_tab_bar_test.dart`：6 个测试
  - `test/widgets/tool_button_test.dart`：4 个测试
  - `test/services/learning_progress_service_test.dart`：5 个测试
- **修复由测试揭露的 Flutter 逻辑 bug**：`closeFloatingPanel` 因 `copyWith` 的 null 语义无法清除 `activeFloatingPanel`。
- **修复 Rust 测试 warnings**：`native/tests/b10_new_array_rollback.rs` 移除未使用导入；`native/tests/test_utils.rs` 添加 `#![allow(dead_code)]`。
- **更新 `CHANGELOG.md`**：新增"代码审查报告推进"小节记录上述修复。
- **当前测试状态**：Rust 691 passed / 0 failed，Clippy 0 warnings，测试代码 0 warnings；Flutter 90 passed / 0 failed + 4 集成测试 passed，Flutter analyze 0 issues。

### 13.2a 2026-06-14 性能优化轮次补充

- **新增 Rust 回归测试 3 个**：
  - `ast_unit_test.rs`（新增文件）：`test_mangle_name_into_matches_mangle_name`、`test_mangle_name_into_appends_to_buffer`
  - `test_snapshot.rs`：`test_snapshot_into_equivalence_and_reuse`
- **改动文件**：
  - `native/src/compiler/ast.rs`：新增 `mangle_name_into`，改写 `mangle_name`
  - `native/src/vm/vm/mod.rs`：新增 `snapshot_into`
  - `native/src/unified/engine.rs`：新增 `pre_step_snap` 复用字段，修改 `run_batch` 调用 `snapshot_into`
  - `CHANGELOG.md`：新增 `Optimized (性能优化 — 代码审查报告 O1/O4)` 小节
- **当前测试状态（本轮验证后）**：Rust 697 passed / 0 failed，Clippy 0 warnings，测试代码 0 warnings；Flutter analyze 0 issues。
- **新增单独验证**：
  - `kr_5_8`、`kr_5_14` 经 Clang 与 Cide 交叉对比均为 `match`；原报告将其归为 `unknown` 编译缺口的状态已过时。
  - K&R 子集（69 用例）单独 Shadow 验证：`68 match / 0 compile_gap / 0 runtime_gap / 1 output_gap / 0 cide_better`。
  - Baseline 子集（448 用例）单独 Shadow 验证：`442 match / 0 compile_gap / 2 runtime_gap / 2 output_gap / 2 cide_better`。

### 13.2b 2026-06-14 JIT 修复轮次补充

- **新增 Rust 回归测试 1 个**：
  - `jit_templates_test.rs`（新增文件）：`test_jit_trace_bulk_accelerates_loop`
- **修复 JIT 相关既有测试**：`native/tests/jit_unit_test.rs` 的 `test_jit_nested_loop` 此前因 Abort trace 被错误编译而依赖 `ends_with_conditional` 的保守退出逻辑；修复后完整 trace 被正确编译，bulk 执行可覆盖多轮内层循环迭代
- **改动文件**：
  - `native/src/vm/jit_templates.rs`：修复 `execute_trace_bulk` 循环逻辑；`JitEntry` 新增 `is_conditional_jump`
  - `native/src/vm/jit_trace.rs`：`TraceRecorder::finish` 增加 `aborted` 参数，Abort 时丢弃录制内容
  - `native/src/vm/vm/executor.rs`：区分 `Finish` 与 `Abort` 调用 `finish(false)` / `finish(true)`
  - `CHANGELOG.md`：新增 JIT 修复条目
- **当前测试状态（本轮验证后）**：Rust 698 passed / 0 failed，Clippy 0 warnings，测试代码 0 warnings；Flutter analyze 0 issues。

### 13.2c 2026-06-14 host_qsort 优化轮次补充

- **新增 Rust 回归测试 3 个**：
  - `qsort_test.rs`（新增文件）：`test_qsort_int_array`、`test_qsort_byte_array`、`test_qsort_larger_array`
- **改动文件**：
  - `native/src/vm/host_funcs.rs`：`host_qsort` 结果写回由逐字节 `store_i8` 改为按元素块 `write_memory`
  - `CHANGELOG.md`：新增 qsort 优化条目
  - `native/tests/HOST_CONTRACT_FAILURES.md`：追加 O5 推进中发现的"Abort trace 被错误编译"已修复记录，以及 O10 验证中发现的"`qsort` 栈上大数组触发 NULL 指针 trap"待分析问题
- **当前测试状态（本轮验证后）**：Rust 701 passed / 0 failed，Clippy 0 warnings，测试代码 0 warnings；Flutter analyze 0 issues。

### 13.2d 2026-06-14 代码审查报告继续推进

- **新增 Rust 回归测试 3 个**：
  - `type_checker_unit_test.rs`：`test_type_checker_incompatible_pointer_assignment_warns`
  - `host_contract_tests.rs`：`test_printf_n_rejects_insufficient_args`、`test_scanf_n_rejects_insufficient_args`
- **改动文件**：
  - `native/src/compiler/ast.rs`：删除 `InitElement` 的 `Deref`/`DerefMut`，消除 designators 被隐式解引用忽略的风险
  - `native/src/compiler/algorithm_detector.rs`、`codegen/{mod,expr,stmt}.rs`、`data_flow.rs`、`intent.rs`、`typeck/{mod,expr}.rs`：23 处 `InitElement` 隐式解引用改为显式 `.value`
  - `native/src/compiler/typeck/mod.rs`：`check_pointer_assignable` 双指针分支增加 pointee 兼容性检查，不兼容时报告 warning
  - `native/src/compiler/parser/mod.rs`：多维 VLA `array_size` 计算避免产生负数
  - `native/src/vm/host_funcs.rs`：`host_printf_n` / `host_scanf_n` / `host_fprintf_n` 增加栈深度前置校验
  - `CHANGELOG.md`：新增"代码审查报告继续推进"小节
  - `docs/current/code_review_report.md`：更新 B25/B27/B39/B43、NEW-R1~R7、V1~V10 状态，更新测试与 Shadow Verification 数据
- **复核状态**：
  - V9/V10（`call_user_function` / `do_call` 局部变量初始化）：当前实现已通过 `store_i32`/`store_i8` 走统一内存访问路径，无需额外修改
  - NEW-R1~R8 边界检查：均已修复或相关代码已移除
- **新增 Rust 回归测试 10 个**：
  - `native/src/compiler/cfg.rs`（单元测试）：`test_cfg_if_cond_block_does_not_clone_whole_if`、`test_cfg_return_no_fall_through_edge`
  - `native/src/compiler/data_flow.rs`（单元测试）：`test_live_variables_nested_if`
  - `native/src/compiler/intent.rs`（单元测试）：`test_intent_loop_detected_without_relying_on_block_id_order`
  - `native/tests/typeck_cpp_unit_test.rs`：`test_cpp_multi_var_ctor_init_inserts_this_for_all`
  - `native/tests/completion_unit_test.rs`：`test_find_variable_type_parses_unsigned_int_param`
- **改动文件（本轮新增）**：
  - `native/src/diagnostics/knowledge_graph.rs`：修复 `ERROR_CONCEPT_MAP` 3035 重复，统一 3030-3035 映射
  - `native/src/compiler/cfg.rs`：If 条件块只保留条件表达式；Return 块不再误加 fall-through 边
  - `native/src/compiler/data_flow.rs`：预计算出边邻接表优化 live-variable 分析
  - `native/src/compiler/intent.rs`：回边检测复用 `cfg.find_loops()`
  - `native/src/compiler/typeck/decl.rs`：提取 `try_process_ctor_init`，支持 extra_vars 的构造函数 this 插入
  - `native/src/engine/session_ops.rs`：`execute_run` 用 `catch_unwind` 保护 take 后的 VM
  - `native/src/engine/completion/mod.rs`：`mod candidates` 改为 `pub mod candidates`，使 `find_variable_type` 可被集成测试访问
- **当前测试状态（本轮验证后）**：Rust 714 passed / 0 failed，Clippy 0 warnings，测试代码 0 warnings；Flutter analyze 0 issues。
- **Shadow Verification 实测**：450 match / 3 compile_gap / 3 runtime_gap / 3 output_gap；`bTree_default` 为 pre-existing runtime_gap（HEAD 提交下同样失败），与本次改动无关。

### 13.2e 2026-06-14 代码审查报告本轮收尾

- **状态复核**：
  - B25（`InitElement` 隐式解引用）、B27（多维 VLA `array_size` 负值）、B35~B40（CFG / 数据流 / 意图推断 / 指针赋值 / 构造函数 this 插入）均已修复并验证。
  - B43（`printf`/`scanf`/`fprintf` 栈深度校验）、B47（`execute_run` panic 时 VM 恢复）、B48（`unsigned int` 参数类型解析）、B52（`ERROR_CONCEPT_MAP` 3035 重复）均已修复并验证。
  - B44（`host_qsort` 逐字节写回）已在 13.2c 中修复，4.1 节状态同步更新。
  - V9/V10（`call_user_function` / `do_call` 局部变量初始化）已确认通过统一 `store_i32`/`store_i8` 路径，自然经过 `check_mem_access` + `check_uaf`，无需额外修改。
- **新增/更新回归测试**：本轮新增 10 个 Rust 回归测试（详见 13.2d），覆盖 CFG、live-variable、意图推断、C++ 构造函数 this 插入、`unsigned int` 参数解析等场景。
- **当前测试状态（本轮最终验证）**：Rust **714 passed / 0 failed**，Clippy 0 warnings，测试代码 0 warnings；Flutter analyze 0 issues。
- **Shadow Verification 最终实测**：450 match / 463 用例（97.19%），剩余 3 compile_gap、3 runtime_gap、3 output_gap，另有 2 cide_better 与 2 known_issue。

### 13.3 Shadow Verification 变化

| 指标 | 审阅时 | 修复后（2026-06-14 首轮） | 修复后（2026-06-14 次轮） | 当前完整实测（性能优化后） | 继续推进后（本报告更新） |
|------|--------|--------------------------|--------------------------|--------------|--------------|
| 匹配 | 411 | 412 | 415 | **451** | **450** |
| 编译缺口 | 10 | 5 | 3 | **3** | **3** |
| 运行时缺口 | 27 | 31 | 30 | **2** | **3** |
| 输出差异 | 2 | 2 | 3 | **3** | **3** |
| cide_better | - | - | 0 | **0** | **2**（`keyword_compat`、`merge_default`）|

说明：
- 首轮修复后，编译缺口减少 5 个，其中 `bellmanFord_default`、`spfa_default`、`threadedBinaryTree_default` 原属 `unknown`；`kr_5_10` 原因三目字符串字面量类型不匹配编译失败；`kr_5_16` 原因匿名 enum 不支持编译失败。这 5 个用例中仅 `bellmanFord_default` 实现完整匹配，其余 4 个进入运行阶段后因其他限制仍失败，故运行时缺口增加 4。
- 次轮修复核心变化：
  1. **Batch 输入模式**：Shadow 脚本统一设 Batch 模式，`getchar` 无输入返回 EOF。K&R 中 `kr_1_*`、`kr_4_*`、`kr_5_*`（除 `kr_5_8` 外）、`kr_6_*` 等 31 个运行时缺口全部消除。
  2. **全局/字符串内存重叠修复**：`kr_6_1` 由运行时失败变为 `match`；`kr_6_1` 从 `KNOWN_KR_FAILURES` 移除。
  3. **Clang 头文件精简**：`stdlib.h`/`string.h` 不再由 Shadow 脚本注入，K&R 中用户自定义 `itoa`/`qsort` 不再与标准库声明冲突，4 个 `cide_better` 消除。
- 性能优化轮次（O1/O4）未改变 Shadow Verification 语义，仅提升运行效率；表中"当前完整实测"为执行完整 463 用例 Shadow Verification 的结果。
- 继续推进后（本报告更新）实测：匹配数 450，运行时缺口变为 3 个。新增的运行时缺口 `bTree_default` 为 pre-existing 问题——在 HEAD 提交（f7bab45，不含本次新增改动）下同样失败，根因为访问 NULL 指针区域（地址 0x0010），与本次 B25/B27/B39/B43 改动无关。
- 当前剩余差异：
  - 编译缺口 3 个均为 Cide 教学子集已知不支持特性：`inline_asm`、`static_assert`、`typeof_operator`。
  - 运行时缺口 3 个：`bTree_default`（pre-existing NULL 指针访问）、`infixEvaluation_default`、`spfa_default`（后两者为 Cide 数组越界检查更严格导致）。
  - 输出差异 3 个：`vfs_io_extensions`/`file_fread` 为 VFS 不模拟 Windows CRT 换行转换的已知限制；`kr_5_8` 为函数指针类型转换 UB 导致的 Clang 优化差异。
  - cide_better 2 个：`keyword_compat`（Clang 缺少 `<stdbool.h>` 导致 `bool` 未定义）、`merge_default`（Clang 对隐式函数声明更严格）。
  - known_issue 2 个：`function_pointer_sizeof`、`sizeof_array_param`（Cide 目标为 32 位指针，Clang x64 输出 8，属于已记录的架构差异）。

### 13.4 剩余高价值缺口

1. ~~**`kr_5_14`**：~~ 已确认当前版本可正常编译运行，单独 Shadow 验证为 `match`。原报告将其标为 `unknown` 编译缺口的状态已过时。`kr_5_8` 可正常编译运行，但因函数指针类型转换 UB 导致输出顺序与 Clang 优化结果不同，保留为 `output_gap` 诚实记录。
2. ~~**`argc/argv` 支持**：~~ 已实现：`OpCode::PushArgc`/`PushArgv`、入口包装代码自动推送、`flutter_bridge::set_argv`/`cide_set_argv`、CLI `--` 传参。单独 Shadow 验证 `kr_5_10` 为 `match`。
3. ~~**K&R 运行时缺口（`kr_1_*`、`kr_4_*`、`kr_5_*`、`kr_6_*` 系列）**：~~ K&R 子集 69 用例中 68 个已 `match`，剩余 1 个 `kr_5_8` 为函数指针类型转换 UB 导致的 Clang 优化差异，保留为诚实记录。
4. ~~**Windows 文本换行转换**：~~ 明确列为**已知限制**，不在 VFS 中模拟 Windows CRT 的 `\n` ↔ `\r\n` 转换。`vfs_io_extensions`、`file_fread` 的输出差异保留为诚实记录，已在 `AGENTS.md` 中说明。
5. **Baseline 剩余差异**：
   - `infixEvaluation_default` / `spfa_default`：Cide 数组越界检查更严格，运行时触发 `TrapBounds`，与 Clang 行为不同。属于安全策略差异，保留。
   - `keyword_compat` / `merge_default`：Clang 因缺少 `stdbool.h`（`bool`）/ 函数前向声明隐式声明而编译失败，Cide 接受。属于 Cide 对教学子集的宽松处理，保留为 `cide_better`。
6. **性能优化与架构重构**：O1（mangle_name buffer 复用）、O4（VM snapshot buffer 复用）、O5（JIT trace bulk 执行）、O10（host_qsort 批量写回）已完成；B15/S9（函数指针转 `usize` 比较）已完成；O2-O3、O6-O9、O11-O15、A1-A14 待后续推进。

---

## 14. 附录 A：原报告误判清单

| 编号 | 位置 | 原报告说法 | 实际核对结果 |
|------|------|-----------|-------------|
| B2 | compiler/parser/mod.rs:343-366 | 结构体声明静默丢失 | 会进入 else 分支解析为 class/struct |
| B3 | compiler/parser/mod.rs:496-528 | 空函数体未消费尾部分号 | 第 511 行已 consume(Semicolon) |
| B8 | compiler/codegen/expr.rs:1358 | 继续执行到 _ => 分支导致重复报错 | Identifier 分支结束后函数直接返回 |
| B9 | compiler/codegen/cpp_this_new_delete.rs:191 | 无析构函数时资源泄漏 | 属于 C++ 正常语义 |
| B11 | compiler/lexer.rs:880 | xHH 调用 6 次 advance | 实际 4 次 |
| B12 | compiler/typeck/mod.rs:217 | __ctor__Foo__3 split 取错段 | 普通方法分支不会进入；构造函数有专门分支 |
| B19 | unified/stream.rs:604 | get_sym 越界 panic | 使用 Vec::get，越界返回空字符串 |
| B23 | lib/screens/ide_screen.dart:485 | FloatingActionButton.mini 已废弃 | Flutter 3.29 仍有效且未废弃 |
| B24 | lib/main.dart:15 | destroySession fire-and-forget | 已 await |
| D1 | compiler/ast.rs:247 | function_pointer() 死代码 | typeck/expr.rs:403 有调用 |
| D2 | compiler/ast.rs:359 | array_size() 死代码 | 多处调用 |
| D3 | compiler/ast.rs:844 | impl Default for Expr 死代码 | 被 `std::mem::take` 多处使用 |
| D12 | lib/widgets/concept_graph_view.dart:346 | const y = 12.0 未使用 | 在 _drawLegend 中直接使用 |
| - | 报告 5.3 节 | 无 CI 流水线 | .github/workflows/ci.yml 已存在 |
| - | 报告 5.3 节 | 缺失 flutter_lints/integration_test | pubspec.yaml 已包含 |

---

*报告由 Kimi Code CLI 基于源码逐条验证、运行测试与静态分析后生成。所有行号引用基于审阅时的代码版本。*
