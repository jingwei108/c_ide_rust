# Cide 项目 Agent 指南

## 项目概览

Cide 是一个跨平台 C 语言 IDE，包含：

- **前端**：Flutter (Android + Desktop Windows) — 使用 `re_editor` 编辑器 + `flutter_riverpod` 状态管理
- **后端**：共享 Rust native 编译器/VM (`cide_native`)
- **编译管线**：Lexer → Parser → TypeChecker → BytecodeGen → CideVM
- **桥接**：flutter_rust_bridge v2 (`native/src/api/cide.rs` → `CideFlutter/lib/src/rust`)

## 技术栈

| 层级 | 技术 |
|------|------|
| Android | Flutter + `re_editor` + CustomPainter 可视化 |
| Desktop | Flutter + `re_editor` + CustomPainter 可视化 |
| Native | **Rust 1.95.0**, Cargo, cdylib/staticlib/rlib |
| VM | 自定义字节码解释器，1MB 线性内存 |
| Bridge | flutter_rust_bridge v2.12.0 (SSE codec) |

## 关键目录

```
native/src/compiler/    Lexer, Parser, TypeChecker, BytecodeGen, AST (Rust)
native/src/vm/          CideVM 字节码解释器 (Rust)
native/src/unified/     统一模式 / 时间旅行引擎 (Rust)
native/src/engine/      编译管线与工具 (Rust)
native/src/capi/        C API (MAUI 兼容层) (Rust)
native/src/api/         FRB API (flutter_rust_bridge) (Rust)
native/src/diagnostics/ 结构化诊断、自动修复建议 (Rust)
CideFlutter/            Flutter 跨平台前端 (Android + Desktop Windows)
docs/                   设计文档、事故报告
```

## Rust 迁移进度（已完成 ✅）

| 阶段 | 模块 | 状态 |
|------|------|------|
| Phase 0 | Rust 骨架 + C API 桩 + Session 类型 | ✅ 完成 |
| Phase 1 | VM 迁移 (CideVM + host funcs) | ✅ 完成 |
| Phase 2a | Lexer | ✅ 完成 |
| Phase 2b | AST | ✅ 完成 |
| Phase 2c | Parser | ✅ 完成 |
| Phase 2d | TypeChecker | ✅ 完成 |
| Phase 2e | BytecodeGen | ✅ 完成 |
| Phase 2f | C API `cide_compile_all` 接线 | ✅ 完成 |
| Phase 3 | ~~C# 前端~~ → Flutter 前端端到端测试 | ✅ 完成 |
| Phase 4 | Android 目标构建（cargo-ndk） | ✅ 完成 |
| Phase 5 | 清理遗留 C++ / CMake 文件 | ✅ 完成 |
| Phase 6 | 全面审查：编译警告清理 + 安全加固 + 测试覆盖拓展 | ✅ 完成 |
| Phase 7 | Desktop 内存泄漏修复 + sizeof/scanf 子集拓展 | ✅ 完成 |
| Phase 8 | `float` 类型全管线支持（Lexer→Parser→TypeChecker→BytecodeGen→VM）+ 诊断系统拓展 | ✅ 完成 |
| Phase 9 | Flutter 前端从零搭建：IDE 界面 + 编辑器 + 调试面板 + 算法可视化 | ✅ 完成 |
| Phase 10 | 内存映射 Canvas + 算法可视化事件 FRB 集成 + 交互增强 | ✅ 完成 |
| Phase 11 | 代码审查修复 + 工程规范（`rustfmt.toml`/`CHANGELOG.md`）+ 240 个单元测试 + Flutter 前端全面模块化拆分 | ✅ 完成 |
| Phase 12 | `union` 类型全管线支持（Lexer→Parser→TypeChecker→BytecodeGen→VM）+ `sizeof(union U)` | ✅ 完成 |
| Phase 13 | **统一模式 / 时间旅行**：VM 快照/恢复 + 检查点管理器 + 批量自动执行 + Seek 进度条 + 异常自动回退 + 语义标签 + 变量历史趋势图 | ✅ 完成 |
| Phase 14 | **堆内存可视化增强**：malloc 分配行号追踪 + 外部碎片（free_list）可视化 + 程序结束泄漏检测报告 | ✅ 完成 |
| Phase 15 | **指针追踪动画**：统一模式每步收集 `PointerSnapshot`，前端 `PointerArrowWidget` 实时绘制指针箭头；支持 Valid/Freed/Null/Dangling 四种状态可视化 | ✅ 完成 |
| Phase 16 | **算法步骤语义标注**：为 6 种检测到的算法预定义步骤模板，运行时结合源码行特征 + 变量值生成教学描述（如冒泡排序"第 {i} 趟：将第 {n-i} 大的元素放到正确位置"）；前端 `ExecutionControlPanel` 实时展示步骤横幅 + `AlgorithmTab` 静态流程预览 + `ArrayVisTab` 已排序边界高亮 | ✅ 完成 |
| Phase 17 | **代码模板参数化 + 交互式教程**：模板支持 `{{key:default}}` 占位符（数组长度、查找目标等）；选择模板后弹出参数对话框；填入参数后启动 `TemplateTutorialPanel` 逐行引导理解；关键行带 💡 可展开解释；教程完成自动编译运行并启动统一模式 | ✅ 完成 |
| Phase 18 | **6-04 地毯式审阅修复**：P0 soundness 修复 + VM 热点 O(1) 优化 + Call/CallPtr 去重 + algorithm_detector AST 精确匹配 + 格式解析 DRY + type_size 统一提取 + check_assignable 拆分 + 隐式转换映射表 + Session 预设文件序列化 + 边界检查统一 + ptr_step_size 数组指针支持 + clippy 0 警告 | ✅ 完成 |

## 编码约定

### Rust (native)
- AST 使用 enum 替代 C++ 多态类层次：`Expr` / `Stmt` 枚举 + `Box<Expr>` / `Vec<Box<Expr>>`
- `SourceLoc` 已添加 `Copy` derive（两个 `i32`，值传递无开销）
- Parser 零进度保护：`if pos_ == checkpoint { self.advance(); }`
- 错误处理：不 panic，收集到 `Vec<Error>` 后统一返回
- Borrow checker 冲突解决模式：先 clone 数据再调用需要 `&mut self` 的方法

### Dart / Flutter (frontend)
- 状态管理：`flutter_riverpod` (`StateNotifier` + `StateNotifierProvider`)
- 编辑器：`re_editor`（CustomPainter 实现），非 CodeMirror
- Rust 调用通过 `flutter_rust_bridge`：`rust.compile()` / `rust.stepNext()` 等
- UI 线程：`Future.delayed` / `async-await`，无需显式主线程切换
- 自定义组件：算法验证、内存映射、链表可视化、教程引导等均为 CustomPainter / Widget 实现



## 已知限制

### 当前不支持
- **匿名结构体变量声明** — `struct { int x; } v;`（直接以匿名 struct 类型声明变量暂不支持；仅支持通过 `typedef struct { ... } Name;` 间接使用）
- **`double`** — ✅ **已完整支持**（64 位 f64，字节偏移架构，含 `sizeof(double)=8`、`printf("%f")` 读取 f64）
- **函数调用参数的隐式转换提示** — 当前对 `printf` 格式字符串 `%f` 的参数不做类型检查，传入 int 不会自动转换（已知限制）

### 已支持的关键特性
- **逗号分隔的多变量声明** — `int a = 1, b = 2;`
- **多维数组**（`int arr[3][3]`）— 声明、嵌套初始化列表 `{ {1,2}, {3,4} }`、索引访问 `arr[i][j]`、函数参数传递 `void f(int[][3])`
- **`#define` 宏** — 简单常量替换（如 `#define N 100`）
- **printf 可变参数** — 支持任意数量参数（如 `printf("%d %d %d", a, b, c)`）
- **局部 `char` 数组字符串初始化** — `char s[6] = "hello"; printf("%s", s);`
- **`enum` 局部/全局变量声明** — `enum Color c = GREEN;`（需先声明 enum 类型）
- **`typedef`** — `typedef int Integer; Integer a = 42;`
- **`typedef struct`** — `typedef struct { int x; } Point; Point p;`（匿名结构体 + typedef 别名）以及 `typedef struct Vec { int x; } VecAlias;`（命名结构体 + typedef 别名）
- **`sizeof` 运算符** — `sizeof(int)`、`sizeof(char)`、`sizeof(struct S)`、`sizeof(union U)`、`sizeof(arr)`、`sizeof(ptr)`
- **`scanf` 多参数** — `scanf("%d %d %d", &a, &b, &c)`
- **指针算术** — `p++` / `p--` / `p + i` / `p - i` / `p - q`，自动按 pointee 类型大小缩放（`int*` 步长 4，`char*` 步长 1，`struct*` 步长为结构体大小）
- **函数前向声明** — `int foo(int);` 原型声明，函数定义可放在调用者之后
- **字符串库函数** — `strlen(s)`、`strcpy(dest, src)`、`strcmp(a, b)`（宿主导入函数）
- **显式类型转换（Cast）** — `(int*)p`、`(char*)arr`、`(float)a`、`(int)b` 等标量/指针间转换
- **`fprintf`** — `fprintf(stdout, "format", ...)` / `fprintf(stderr, "format", ...)`，stream 参数被忽略，输出行为与 `printf` 相同
- **`realloc`** — `realloc(ptr, new_size)`，支持扩容/缩容、NULL ptr（等价 malloc）、size 0（等价 free）
- **`qsort`** — `qsort(base, nmemb, size, compar)`，支持用户自定义比较函数（通过 VM 调用用户函数）
- **`union` 类型** — `union U { int i; double d; }; union U u; u.i = 1; u.d = 3.14; printf("%.2f", u.d);`，内存布局为所有字段 offset=0、size=max(fields)，支持成员访问、指针访问（`p->i`）、`sizeof(union U)`
- **统一模式 / 时间旅行** — 点击"运行"后自动逐语句执行并收集每步状态快照；可随时暂停、单步前进、拖动进度条回退到任意历史步；系统从最近检查点（每 20 步）恢复 VM 状态并正向重放；运行时异常自动回退到上一步并弹出知识卡片诊断
  - `VMSnapshot` 全量快照（`vm/snapshot.rs`）：1MB 内存 + 运行时状态 + 内存管理状态
  - `CheckpointManager` 检查点管理器（`unified/checkpoint.rs`）
  - `UnifiedEngine` 批量自动执行 + Seek + Trap 回退（`unified/engine.rs`）
  - `StepCollector` 每步数据收集：变量快照、调用栈、可视化事件、语义标签、热力图（`unified/collector.rs`）
  - Flutter 前端：`UnifiedNotifier` 状态机 + `ExecutionControlPanel` 控制面板 + `VarHistoryTab` 变量历史趋势图
- **指针追踪动画** — `PointerVisTab` + `PointerArrowWidget` 实时绘制指针箭头；统一模式每步自动收集 `PointerSnapshot`（名称/类型/自身地址/目标地址/目标变量名/状态），支持时间旅行回溯查看任意历史时刻的指针状态
  - `PointerStatus` 四种状态：Valid（蓝色实线箭头）/ Freed（灰色虚线箭头）/ Null（接地符号空箭头）/ Dangling（红色虚线箭头）
  - 后端：`StepCollector::collect_pointer_snapshots` 遍历变量快照，解析指针值，结合 `session.memory.regions` 判断是否为已释放堆内存
  - 前端：`PointerArrowWidget` 使用 `CustomPainter` 绘制箭头，左右卡片布局，状态色编码
- **数组排序动画增强** — `ArrayVisualizer` 高亮脉冲（缩放+发光）、交换金色光晕、值变化弹性弹跳；`ArrayVisTab` 解析 Swap 语义标签驱动交换动画
- **算法步骤语义标注** — 为 6 种算法（冒泡/选择/插入/快速/归并/二分）预定义步骤模板，运行时根据源码行特征和变量值推断当前阶段并生成中文教学描述
  - 后端：`unified/algorithm_steps.rs` 推断引擎，每种算法独立推断逻辑；`StepPayload` 新增 `algorithm_step` 字段
  - 前端：`ExecutionControlPanel` 步骤横幅（按 phase 着色：outer_loop 蓝、swap 琥珀、compare 紫、finish 绿等）；`AlgorithmTab` 静态步骤流程预览（带运行时高亮）；`ArrayVisTab` 已排序边界绿色高亮（冒泡右侧/选择左侧/插入左侧）
- **链表可视化** — `LinkedListVisualizer` CustomPainter 绘制节点+箭头，支持 NodeCreate/Access/Delete 闪色；渐进式入场动画；`LinkedListVisTab` 集成统一模式，从 `StepPayload.localVars` 读取头指针驱动时间旅行
- **二叉树可视化** — `TreeVisualizer` 满二叉树位置层级布局，节点滑入+连线渐进动画，最大深度 6 限制；`TreeVisTab` 集成统一模式
- **变量级高亮** — `re_editor` `spanBuilder` 集成：当前执行行的被读变量名显示淡蓝底色、被写变量名显示淡橙底色，保留语法高亮；`VariablesTab` 值变化背景闪烁动画
- **代码模板扩展** — 新增选择排序、插入排序、归并排序、线性查找、链表头插法/遍历、二叉树节点/先序遍历、栈（数组实现）等 8 个模板，总计 16 个
- **代码模板参数化 + 交互式教程** — 核心算法模板（冒泡/选择/插入/快速/归并/二分/线性查找）支持参数占位符（如 `{{n:5}}`、`{{target:3}}`）；`TemplateParamDialog` 底部弹窗收集参数；`TemplateTutorialPanel` 逐步骤高亮代码行并展示教学描述；每步骤的关键行带 💡 `ExpansionTile` 可展开查看详细解释；教程最后一步点击"运行代码"自动插入生成代码、编译并启动统一模式；`LearningProgress` 记录 `completedTutorials`

### 已修复的关键 Bug
- **Parser 死循环（2026-04-27）**：`struct*` 返回类型误识别为 struct 声明 → `ParseStructDecl` 零进度保护
- **Parser 死循环（2026-05-09）**：`ParseBlock()` 遇到无法解析的 token 时不前进 → 添加 `pos_ == checkpoint` 保护
- **Parser 死循环（2026-05-10）**：`parse_case_stmt` 的 while 循环缺少零进度保护；`advance()` 空 token 列表 usize 下溢 panic；`synchronize()` 从未被调用 → 全面修复
- **VM 安全加固（2026-05-10）**：`addr+4` u32 溢出、`step_count` i32 溢出、`host_malloc` u32 溢出、Jump 目标越界、值栈无上限 → 全部修复
- **TypeChecker 警告代码勘误（2026-05-10）**：`W3050`/`W3051` 被滥用于不相关场景 → 新增 `W3052`~`W3055`
- **BytecodeGen char 数组初始化（2026-05-10）**：`char s[] = "hello"` 使用 `StoreLocal`（i32）导致字符间隔 3 字节零 → 改用 `StoreMemByte` 连续存储
- **移动端内存泄漏**：JS interop 监听器未清理、CTS 未 Dispose、ConsoleOutput 无上限
- **clippy 警告清零（2026-05-18）**：`Type::to_string` 改为 `Display`、`SourceLoc` clone 清理、`if_same_then_else`、`module_inception` 等 → `cargo clippy -- -D warnings` 0 警告（含本次审查修复的 `needless_return`/`needless_borrow`）
- **unsigned 类型提示（2026-05-10）**：Parser 保留 `is_unsigned` 标记；TypeChecker 遇到 `unsigned int x;` 时报告 `W3056` 提示"被映射为 int，暂不支持无符号语义"
- **`float` 类型支持** — `float x = 3.14;`、`float a = 5;`（隐式 int→float 转换）、算术/比较/复合赋值、强制转换 `(float)`/`(int)`、`printf("%f")` / `scanf("%f")`
- **函数调用参数隐式转换** — `void foo(float x) {} foo(5);` 自动插入 `(float)` cast；`bar(3.7f)` 传入 int 形参自动截断为 int，并发出 `W3053` 精度丢失警告
- **C 子集 P0 拓展（2026-05-10）**：字符字面量 `'a'`、块注释 `/* */`、十六进制 `0xFF`、八进制 `077`、类型修饰符 `long/short/signed/const`、更多转义序列 `\r\a\b\f\v\xHH` → Lexer + Parser 全部支持，新增 5 个 E2E 测试
- **影子验证发现 bug #4（2026-05-17）**：八进制字面量 `077` 被误解析为十进制 77 → Lexer `number()` 新增八进制分支
- **影子验证发现 bug #5（2026-05-17）**：`&&` / `||` 无短路求值，右侧表达式总是被求值 → BytecodeGen 新增 `Dup` + `JumpIfZero` / `JumpIfNotZero` 短路逻辑
- **已知问题（2026-05-17）**：`for (int i = 0; ...)` 循环变量作用域未隔离外部同名变量；字符串字面量 `strlen` 手动计算长度与 Clang 不一致（Cide 输出 10 vs Clang 5）
- **C 子集 P1 拓展（2026-05-10）**：复合赋值扩展到数组索引/指针解引用/结构体成员（`a[i]+=1`、`*p+=1`、`s.mem+=1`）、取地址扩展到复杂左值（`&a[i]`、`&s.mem`）、全局结构体变量成员访问、自增/自减扩展到复杂左值（`a[i]++`、`*p++`、`s.mem++`）→ BytecodeGen 全部支持，新增 7 个 E2E 测试
- **C 子集 P2 拓展（2026-05-10）**：位运算符 `& | ^ ~ << >>` 全管线支持（Lexer→Parser→TypeChecker→BytecodeGen→VM），新增 2 个 E2E 测试；三目运算符 `? :` 全管线支持，新增 1 个 E2E 测试
- **BytecodeGen 指针步长修复（2026-05-10）**：`BinaryOp::Add` 指针+整数时硬编码 `PushConst 4` → 改用 `ptr_step_size()`，正确支持 `char*`（步长 1）和 `struct*`（步长为结构体大小）
- **VM 栈-堆碰撞保护修复（2026-05-10）**：`heap_limit` 闭包在 `setup_vm` 时按值捕获初始 `heap_offset`，后续 `malloc` 修改不反映 → 删除闭包机制，`Call` 指令处直接读取 `session.memory.heap_offset`
- **TypeChecker 警告透传修复（2026-05-10）**：`W3050`~`W3056` 被 `_type_warnings` 丢弃，前端完全看不到 → 新增 `push_warnings()`，severity 设为 1，Flutter 前端正确渲染为 warning 行
- **VM 移位指令越界保护（2026-05-10）**：`Shl`/`Shr` 直接执行 `a << b` 不检查边界 → 添加 `!(0..32).contains(&b)` 检查，越界时 `trap` 报告未定义行为
- **位运算错误码勘误（2026-05-10）**：位运算报错借用 `E3019_LogicTypeError` → 新增 `E3048_BitOpTypeError` 专用错误码
- **Session Default 冲突修复（2026-05-10）**：`session.rs` 同时存在 `#[derive(Default)]` 和手动 `impl Default` → 删除派生宏，保留手动实现（`vm: Some(CideVM::default())`）
- **BytecodeGen 缺失 main 保护（2026-05-10）**：`self.func_index["main"]` 在空源码时 panic → 改为 `get("main")` 安全查找，缺失时返回错误"缺少 main 函数入口"
- **TypeChecker 赋值警告降噪（2026-05-10）**：`W3053_ImplicitScalarConversion` 对 `char->int` 安全提升也报警告 → 只保留 `int->char`（可能截断）的警告；`W3055_VoidPointerCast` 对 `malloc` 返回的 `void*->int*` 也报警告 → 删除（C 标准允许）
- **隐式转换提示系统（2026-05-10）**：TypeChecker 新增 `hints` 集合（severity=2），对所有被允许的隐式转换分类提示：
  - **Warning (severity=1)**：危险转换（`int→char`、`float→int`、`array→pointer`、`int→pointer`）
  - **Hint (severity=2)**：安全提升（`char→int`、`int→float`、`char→float`、`void*→具体指针`）
  - C API 新增 `push_hints`，编译成功后按 error → warning → hint 顺序推送诊断
  - 前端 `EnsureCompiled()` 编译成功后也调用 `LoadDiagnostics()`，确保 warnings/hints 被加载
  - `RunCodeAsync()` 在 `ConsoleOutput` 开头追加提示汇总（如"发现 X 处隐式类型转换"），优先级在错误之后
- **新增宿主函数 `fprintf`/`realloc`/`qsort`**：
  - `fprintf(stream, format, ...)`：忽略 stream 参数，输出行为与 `printf` 相同；Lexer 预定义 `stdout=1`、`stderr=2` 宏
  - `realloc(ptr, new_size)`：完整支持扩容/缩容、NULL ptr（等价 malloc）、size 0（等价 free）
  - `qsort(base, nmemb, size, compar)`：支持用户自定义比较函数，通过 `vm.call_user_function` 在 host 上下文中调用用户函数
- **函数指针基础支持**：TypeChecker 将函数名识别为 `int`（函数索引）；BytecodeGen 生成 `PushConst func_idx`；支持将函数名作为参数传递（如 `qsort(..., cmp)`）
- **函数指针高级语法（2026-05-18）**：
  - 多级函数指针：`int (**pp)(int) = &fp;`（指向函数指针的指针）
  - 返回指针的函数指针：`int *(*fp)(int) = greet;`
  - `sizeof` 函数指针类型：`sizeof(int (*)(int))`、`sizeof(int (**)(int))`
  - `typedef` 函数指针（全局/局部均支持）：`typedef int (*Op)(int, int); Op op = mul; Op ops[2] = {mul, divi};`
  - `static` 局部变量：`static int arr[3] = {1, 2, 3};`（函数体内静态存储期）
- **算法可视化事件 FRB 集成**：`VisEvent` 扩展 `context` 字段保留比较上下文（如 `arr[i]:arr[i+1]`）；Flutter 算法面板支持展开查看关键比较事件列表
- **内存映射 Canvas 组件**：1MB 内存以 256×4KB 网格可视化，彩色编码（栈/堆/全局/代码/NULL陷阱/已释放），点击块显示详细 BottomSheet
- **堆内存可视化增强**：
  - `malloc` / `realloc` / `fopen` 分配时记录源码行号（`MemoryRegion.alloc_line` / `alloc_by`）
  - 外部碎片可视化：`free_list` 中的空闲块以金色高亮显示在内存网格中，BottomSheet 中可查看碎片地址和大小
  - 程序结束时自动泄漏检测：遍历未释放的堆区域，输出 "第 X 行的 malloc 分配了 Y 字节，未被 free" 报告，并统计泄漏总字节数
  - 堆内存统计面板：实时显示总堆空间、已分配、碎片字节数及碎片率（0~100%），并以彩色进度条可视化占比
- **VS-style Enter 格式化**：`re_editor` 拦截 Enter 键，自动补充分号、大括号配对、智能缩进
- **教程引导 overlay**：`IntroOverlay` 组件支持多步骤引导，带跳过/下一步按钮
- **Touch swipe tabs**：底部和悬浮面板支持水平滑动手势（60px 阈值）切换 Tab
- **Execution speed slider**：单步模式下支持 0–500ms 执行速度调节
- **学习进度追踪系统**：
  - `LearningProgress` 数据模型：编译次数、成功/失败率、错误码统计、修复统计、知识卡片阅读、算法验证通过率、连续活跃天数 streak
  - `SharedPreferences` 本地持久化
  - Flutter「学习进度」面板：5 个维度卡片（连续活跃、编译统计、错误修复、知识卡片、算法验证）+ 线性进度条 + 重置按钮
  - 自动追踪：编译后更新错误统计、修复后记录、算法验证后记录、查看知识卡片后记录
- **多文件/项目模式**：
  - 前端文件标签栏：新建/删除/切换文件
  - 后端多文件 AST 合并编译：`main.c + utils.c + sort.c → merge → compile`
  - `static` 函数作用域隔离：跨文件访问 static 函数报 `E3058`
  - 诊断信息携带 `filename` 字段，前端正确渲染多文件错误位置
  - 算法验证、统一模式、单步执行全部支持多文件
- **TypeChecker 指针关系运算（2026-05-10）**：`< <= > >=` 拒绝指针比较 → 允许同类型指针（含数组退化）间比较
- **Lexer UTF-8 安全加固（2026-05-10）**：`peek()`/`advance()` 使用 `as_bytes()[i] as char` → 改用 `source[pos..].chars().nth()` 和 `char.len_utf8()`，正确跳过多字节 UTF-8 字符（如中文注释）
- **BytecodeGen 错误消息勘误（2026-05-10）**：`gen_member_addr` 中"全局结构体暂不支持" → 改为"未声明的结构体变量"（该分支实际处理的是变量未找到）
- **字符字面量类型精度（2026-05-10）**：Lexer 返回 `TokenType::Number`，Parser 无法区分 `'a'` 和 `97` → 新增 `TokenType::CharLiteral`，Parser 生成 `Type::char()` 的 `Expr::Literal`
- **VM StepEvent 逻辑集中（2026-05-10）**：StepEvent 的断点检查分散在 `match` 之前和 `match` 分支中 → 全部合并到 `match` 分支，消除状态不一致风险
- **host_strcpy 安全加固（2026-05-10）**：不检查目标缓冲区大小，空间不足时可能不写终止符 → 始终确保在边界内写入 null 终止符
- **host_malloc u32 溢出保护（2026-05-10）**：`new_offset as u32` 在极端大值时截断 → 添加 `new_offset > u32::MAX` 检查
- **NULL 指针内存视图（2026-05-10）**：`cide_memory_get_pointer_target` 用 `target > 0` 排除 NULL → 改为 `target >= 0`，内存视图中可显示指向 0x0000 的指针
- **Parser 重复代码消除（2026-05-10）**：`parse_program()` 中 enum/struct/普通类型三个分支各含 ~25 行重复的变量声明/初始化逻辑 → 提取 `parse_global_var_or_func()` 公共方法
- **类型转换回滚安全（2026-05-10）**：`parse_unary()` 用 checkpoint + rollback 检测 `(type)expr`，`parse_type_only()` 中解析 `enum Name` 会副作用插入 `typedef_names` → rollback 时同步恢复 `typedef_names` 快照
- **C API 裸指针文档（2026-05-10）**：`cide_get_compile_errors` 返回 `String` 内部裸指针，无生命周期文档 → 添加 `///` 安全注释，明确指针仅在下次编译前有效
- **会话保存/加载（2026-05-10）**：`cide_session_save/load` 为桩函数 → 引入 `serde` + `serde_json`，实现 `SessionSnapshot` 序列化/反序列化，保存 compile/runtime/memory 状态
- **文档同步（2026-05-10）**：`DESIGN.md` / `ROADMAP.md` 仍描述 C++ 后端（CMake/Clang/WasmCodeGen）→ 全面更新为 Rust 后端（Cargo/自定义字节码）
- **C 头文件同步（2026-05-10）**：`cide_capi.h` 缺失 `E2007`/`E2008`/`E3048`/`W3051`~`W3056` → 补全；注释"分号分隔"改为"换行分隔"
- **CI/CD 初始化（2026-05-10）**：零 CI/CD → 新增 `.github/workflows/ci.yml`，覆盖 Rust 编译/测试/clippy + Flutter 构建
- **审阅报告修复（2026-05-18）——P0 严重 Bug（5 个）**：
  - `call_user_function` 循环次数错误：`exit_function()` 将 `arg_count` 覆盖为总 word 数，`call_user_function` 误将其当作参数个数 → `FuncMeta` 拆分 `param_count`（参数个数）与 `arg_count`（总 word 数）
  - `restore()` 快照大小不匹配 panic：`copy_from_slice` 要求长度严格相等 → 改为 `min` + 切片安全拷贝
  - 复编译时 `f64_constants` 残留：`run_compile_pipeline` 清空 `i64_constants` 但遗漏 `f64_constants` → 添加 `f64_constants.clear()`
  - 常量索引越界静默返回 0：`PushConstD` / `PushConstQ` 使用 `.unwrap_or(0)` → 改为 `trap` 报告越界错误
  - `PushConstF` 符号扩展导致负 float 损坏：`operand as u64` 对负 i32 做符号扩展 → 改为 `operand as u32 as u64`
- **审阅报告修复（2026-05-18）——VM/安全/代码质量**：
  - `TrapBounds` 栈为空时静默返回 0 → `trap` 报告"值栈为空"
  - C API `cide_get_call_frame`：`session.vm.as_ref().unwrap()` → 安全匹配，VM 未初始化时优雅返回
  - `write_cstring`：`#[allow(clippy::int_plus_one)]` 移除，边界条件改写为 `a + bytes.len() < self.memory.len()`
  - 统一宿主函数名→ID 映射：`host_func_id.rs` 新增 `by_user_name()` / `is_builtin()`，消除 `bytecode_gen.rs` 与 `type_checker.rs` 的 3 处重复
  - 检查点内存无限增长：`CheckpointManager` 新增 `max_checkpoints = 50`，超过时移除最旧检查点
  - `Session.errors_buffer` 冗余字段：与 `errors` 完全重复 → 删除 `errors_buffer`，C API 直接使用 `errors`
  - 字符串字面量上限：`0x8000` (32KB) → `MEM_SIZE / 16` (64KB)
  - `gen_struct_copy` / `gen_struct_copy_to_local` 重复 → 提取 `gen_struct_copy_common` 闭包机制
  - `parse_abstract_declarator` / `parse_declarator_node` ~90% 重复 → `parse_declarator_node` 新增 `is_abstract` 标志，抽象声明符复用同一函数
  - `insert_implicit_cast` 中间 `Box` 分配：`std::mem::replace` + dummy `Expr::Literal` → `Expr` 实现 `Default`，改用 `std::mem::take`
  - 删除未使用的 `parse_call_expr`，消除 clippy `dead_code` 警告
  - `cargo clippy -- -D warnings` 完全通过（含 `needless_return`、`needless_borrow` 修复）
- **审阅报告修复（2026-05-18）——工程化/文档/Flutter**：
  - Android `applicationId`：`com.example.cide` → `com.cide.app`，Release 签名添加警告注释
  - `re_editor` 锁定确切版本 `0.8.0`，添加私有 API 依赖注释
  - NDK 配置添加环境变量说明注释
  - CI 增强：新增 Release 构建验证 + Flutter 测试
  - `DESIGN.md`：指令集 `~30 条` → `106 条`，C++ 伪代码 → Rust 风格
  - `AGENTS.md` / `CHANGELOG.md`：测试数量 `44` → `240`
  - `ROADMAP.md`：知识图谱标记为未启动，函数指针标记为已完成
  - `CideFlutter/README.md`：重写为项目说明
  - `LinkedListVisualizer` / `TreeVisualizer`：异步 `setState()` 前添加 `mounted` 检查
  - `LinkedListVisualizer`：内存上限硬编码 256KB → `rust.getMemorySize()` 动态获取
  - `MemoryTab`：`StatelessWidget` → `StatefulWidget`，`initState` 缓存 Future 避免重复 FFI
  - `IdeScreen`：键盘状态同步从 `build()` 移至 `didChangeDependencies`，消除潜在循环重建
- **Parser 匿名 struct typedef 支持（2026-05-10）**：`typedef struct { ... } Name;` 和 `typedef struct Name { ... } Alias;` 原报"预期结构体名称"级联错误 → 全面支持，新增 `E1006_UnsupportedFeature` 错误码用于友好提示其他暂不支持语法
- **诊断与修复系统全面拓展（2026-05-10）**：
  - 新增 `native/src/diagnostics/error_catalog.rs`：为全部 56+ 个错误/警告码提供中文标题、emoji、通俗解释、常见原因
  - `push_diagnostics`/`push_warnings` 统一调用 `error_catalog::generate_fix`，自动生成结构化修复坐标
  - 新增可自动修复场景：缺少 `"`（E1002）、缺少 `}`/`)/`]`（E2006/E2007/E2008）、`|`→`||` / `&`→`&&`（E1004）、`<=`→`<`（W3051）、条件内 `=`→`==`（W3050）等
  - 前端 `CodeFixService` 增加 `InsertText` 支持及更多 fallback 修复模式（`->`→`.`、补 `return 0;` 等）
  - 新增 11 张知识卡片 JSON（Flutter 端资源）：覆盖缺少分号/括号/引号、变量未声明、scanf 取地址、结构体成员访问、右值赋值、缺少返回值等高频错误
- **代码审查修复（2026-05-14）**：
  - `cide_session_load` 丢失 VM 状态：`setup_vm()` 恢复 bytecode/函数表/断点，会话保存→加载→运行链路可用
  - `call_user_function` Trap 时错误取栈顶值：拆分 `Finished`/`Trap` match 分支，Trap 返回 `None`
  - Hex 字面量 `0x80000000` 被误判溢出：阈值从 `i32::MAX` 放宽为 `u32::MAX`
  - 算法检测仅返回首个匹配：`detect_in_func` 改为返回 `Vec<AlgorithmMatch>`
  - `call_user_function` 内部断点干扰 `run()`：保存/清空/恢复 breakpoints，host 回调不受用户断点影响
  - `Type::is_scalar()` 不含 `Float`：与 `TypeChecker` 版本对齐，加入 `Float`
  - If 语句跳转标记 `end_jump` 命名混淆：重命名为 `skip_else_jump`
  - `malloc(0)` 无教学提示：向 `output_lines` 推送 `W3057` 警告，说明实现定义行为
  - 编译管线 DRY 重构：提取 `run_compile_pipeline()` 消除 `flutter_bridge.rs` 与 `capi/mod.rs` 的 ~100 行重复
  - Host Function ID 统一常量：新建 `vm/host_func_id.rs`，防止编译期与运行期 ID 不匹配
- **C 子集 P0 拓展（2026-05-10）**：
  - `NULL` 关键字：`int *p = NULL;` 现在编译通过，`NULL` 被解析为 `(void*)0`
  - 新增 8 个宿主函数：`getchar`/`putchar`/`rand`/`srand`/`memset`/`exit`/`strcat`/`atoi`
  - `const` 语义：`const int MAX = 100;` 现在会阻止后续赋值和自增/自减，新增错误码 `E3049_AssignToConst`
  - VM 新增 `finished`/`exit_code` 机制，支持 `exit(code)` 提前终止并记录返回值
  - 新增 10 个端到端测试覆盖上述全部特性
- **多文件/项目模式（2026-05-18）**：
  - 后端：`FuncDecl`/`GlobalDecl` 新增 `is_static` + `source_file` 字段
  - Parser：全局级别保留 `static` 标记（函数/全局变量）
  - `run_multi_file_pipeline`：合并多文件源码、独立 Lexer→Parser、AST 合并、行号→文件名映射
  - TypeChecker：`static_funcs` 按文件隔离，跨文件访问报 `E3058_StaticFuncAccess`
  - `Diagnostic` 新增 `filename` 字段，FRB 两端同步
  - FRB API：`compileMulti` / `compileAndRunMulti` + `CodeFile` 类型
  - Flutter：文件标签栏 `FileTabBar`、多文件状态管理、编译/运行/修复全适配
  - 新增 5 个 E2E 测试覆盖多文件编译与 static 隔离
- **审阅报告修复（2026-05-18）**：
  - Parser `LongLiteral` 误用作类型关键字：4 处 `TokenType::LongLiteral` → `TokenType::Long`
  - VFS `fwrite` unwrap 风险：`files.get_mut()` 改为安全匹配，缺失时返回 `0`
  - Flutter Bridge `expect` panic：`current_session()` / `current_unified_engine()` 增加安全 fallback，找不到时自动创建默认 session，永不 panic
  - C API `vm.take().unwrap()` 边缘 panic：`cide_step_next` 改为 `unwrap_or_default()`
  - BytecodeGen `LongLiteral` 静默截断：`flatten_init_list` 中超出 `i32` 范围时推入编译错误，而非静默截断
  - Parser 字面量解析失败静默返回 0：数组维度及数字/字符/浮点字面量 `parse()` 失败时记录具体错误信息
  - VM `step()` 超巨型 match（~720 行）：拆分为 12 个指令类别处理器（`execute_stack/local/global/memory/arithmetic/comparison/bitwise/float/double/longlong/control_flow/debug`），`step()` 缩减为 ~90 行分发逻辑
  - Host `printf` 严重重复：`host_printf_1/2` 复用已有的 `format_printf_string()`，消除重复格式解析逻辑
  - Flutter Bridge session 销毁不完整：`destroy_session` 同步清理 `UNIFIED_ENGINES`；`create_session` 与引擎管理对齐

## 构建命令

```bash
# 日常构建（桌面端 Debug）
python scripts/build_flutter.py

# 构建并运行桌面端 Release
python scripts/build_flutter.py -c Release --run

# Android 完整构建（.so + APK）
python scripts/build_flutter.py -t Android

# 构建 + 安装 + 启动 + 日志（移动端完整流水线）
python scripts/test_mobile.py --install --run --logcat

# Release 发布构建
python scripts/build_release.py

# 构建前运行测试和 lint
python scripts/build_flutter.py --test

# Flutter 离线构建（无网络环境）
python scripts/build_flutter.py --offline

# Flutter 清理构建产物
python scripts/build_flutter.py --clean

# --- 手动命令（脚本不可用时的备选） ---

# 构建 native DLL (Release Desktop)
cd native && cargo build --release
# 输出: native/target/release/cide_native.dll

# 构建 Android .so (arm64-v8a + armeabi-v7a)
cd native
cargo ndk -t aarch64-linux-android --platform 21 build --release
cargo ndk -t armv7-linux-androideabi --platform 21 build --release

# 构建并运行 Flutter 桌面端（手动命令）
cd CideFlutter
flutter pub get --offline
flutter build windows --debug
flutter run -d windows

# 构建 Flutter Android APK（手动命令）
cd CideFlutter
flutter build apk --release

# 安装并启动（手动命令）
adb install -r "build/app/outputs/flutter-apk/app-release.apk"
adb shell monkey -p com.example.cide -c android.intent.category.LAUNCHER 1
```

## 调试技巧

### Native 层调试 (Rust)
1. 项目属性 → 调试 → **启用本机代码调试**
2. 在 `native/src/capi/mod.rs` 的 `cide_compile_all` / `cide_run` 打断点
3. PDB 警告（`apphost.pdb` 缺失）可以安全忽略

### 内存泄漏定位
- 托管 vs 本机：VS 内存分析器看"托管内存"，如果增长很小但任务管理器内存很大 → 泄漏在 native heap
- Parser 死循环特征：内存缓慢持续增长（~100MB/秒），AST 节点或错误消息不断累积
