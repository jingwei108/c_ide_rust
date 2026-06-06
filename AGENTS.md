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
native/src/compiler/    Lexer, Parser, TypeChecker, BytecodeGen, AST, CFG, DataFlow, IntentInference (Rust)
native/src/vm/          CideVM 字节码解释器 (Rust)
native/src/unified/     统一模式 / 时间旅行引擎 (Rust)
native/src/engine/      编译管线与工具 (Rust)
native/src/capi/        C API (MAUI 兼容层) (Rust)
native/src/api/         FRB API (flutter_rust_bridge) (Rust)
native/src/diagnostics/ 结构化诊断、自动修复建议、知识图谱、教学推理 (Rust)
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
| Phase 11 | 代码审查修复 + 工程规范（`rustfmt.toml`/`CHANGELOG.md`）+ 331 个测试 + Flutter 前端全面模块化拆分 | ✅ 完成 |
| Phase 12 | `union` 类型全管线支持（Lexer→Parser→TypeChecker→BytecodeGen→VM）+ `sizeof(union U)` | ✅ 完成 |
| Phase 13 | **统一模式 / 时间旅行**：VM 快照/恢复 + 检查点管理器 + 批量自动执行 + Seek 进度条 + 异常自动回退 + 语义标签 + 变量历史趋势图 | ✅ 完成 |
| Phase 14 | **堆内存可视化增强**：malloc 分配行号追踪 + 外部碎片（free_list）可视化 + 程序结束泄漏检测报告 | ✅ 完成 |
| Phase 15 | **指针追踪动画**：统一模式每步收集 `PointerSnapshot`，前端 `PointerArrowWidget` 实时绘制指针箭头；支持 Valid/Freed/Null/Dangling 四种状态可视化 | ✅ 完成 |
| Phase 16 | **算法步骤语义标注**：为 6 种检测到的算法预定义步骤模板，运行时结合源码行特征 + 变量值生成教学描述（如冒泡排序"第 {i} 趟：将第 {n-i} 大的元素放到正确位置"）；前端 `ExecutionControlPanel` 实时展示步骤横幅 + `AlgorithmTab` 静态流程预览 + `ArrayVisTab` 已排序边界高亮 | ✅ 完成 |
| Phase 17 | **代码模板参数化 + 交互式教程**：模板支持 `{{key:default}}` 占位符（数组长度、查找目标等）；选择模板后弹出参数对话框；填入参数后启动 `TemplateTutorialPanel` 逐行引导理解；关键行带 💡 可展开解释；教程完成自动编译运行并启动统一模式 | ✅ 完成 |
| Phase 18 | **6-04 地毯式审阅修复**：P0 soundness 修复 + VM 热点 O(1) 优化 + Call/CallPtr 去重 + algorithm_detector AST 精确匹配 + 格式解析 DRY + type_size 统一提取 + check_assignable 拆分 + 隐式转换映射表 + Session 预设文件序列化 + 边界检查统一 + ptr_step_size 数组指针支持 + clippy 0 警告 | ✅ 完成 |
| Phase 19 | **Use-After-Free / Double-Free 运行时检测**：VM `execute_memory` 指令层添加 `freed_logs` 检查，访问已释放堆内存时立即 trap 并报告分配/释放位置；`host_free` 检测重复释放；统一模式自动回退 + 知识卡片（E3060/E3061）| ✅ 完成 |
| Phase 20 | **认知推理 P0（运行时根因分析）**：`TraceAnalyzer` 执行轨迹切片 + 根因推断引擎；支持数组越界（OffByOne/未初始化/索引错误）、Use-After-Free、Double-Free、除零、NULL 指针 5 类 Trap 的根因推断；`RootCauseHint` 结构化数据经 FRB 传输到 Flutter；前端 `RootCauseBanner` 组件实时展示根因提示与相关行号跳转 | ✅ 完成 |
| Phase 21 | **认知推理 P1（教学推理层）**：`MisconceptionPattern` 6 种认知误解模式定义 + `detect_misconceptions` 检测引擎；`LearningPath` 推荐引擎为每种模式组装知识卡片→模板高亮→练习路径；Flutter `LearningProgress` 新增 `recentCompileRecords`（保留最近 20 次编译记录）；`LearningPathPanel` BottomSheet 面板实时展示诊断结果与可点击学习步骤 | ✅ 完成 |
| Phase 22 | **认知推理 P2（知识图谱层）**：`KnowledgeGraph` 概念图谱核心，定义 24 个 C 语言核心概念节点（编译/内存/控制流三大域）+ 30+ 条关系边（Prerequisite/LeadsTo/CommonMistake/UsedTogether/Contradicts）；错误码→概念动态激活 + AST 特征激活 + 前置依赖路径查找；Flutter `ConceptGraphView` CustomPainter 三列布局绘制概念网络，激活节点高亮发光，点击弹出概念解释 BottomSheet | ✅ 完成 |
| Phase 23 | **认知推理 P3（代码理解层）**：`ControlFlowGraph` 从 AST 构建基本块+终结符+边，支持 dominator 计算、不可达块检测、循环检测；`DataFlow` 活跃变量分析 + 常量条件求值；`IntentInference` 基于函数名+变量名+CFG 结构+递归特征推断代码意图（Sort/Search/Traverse/Compute/Transform）；`algorithm_detector` 集成 CFG 特征（回边/提前返回/不可达块）修复 `has_min_max_track` 检测；FRB 导出 `infer_intent_from_source`；Flutter `IntentInferencePanel` 底部 Tab 实时展示意图推断结果与置信度 | ✅ 完成 |
| Phase 24 | **语义智能补全 v2**：`CompletionEngine` 轻量级补全引擎（`native/src/engine/completion.rs`），基于编译快照 + 增量 Token 扫描；支持成员访问（`expr.` / `expr->`）、类型上下文、表达式上下文、格式字符串（`printf`/`scanf`）、预处理指令五种上下文感知补全；编译时从 AST 提取函数/变量/结构体/联合体/typedef 快照持久化到 `Session.compile.completion_snapshot`；增量扫描提取光标所在作用域局部变量；FRB 导出 `get_completion_candidates(source, line, column, prefix)`；Flutter `AutocompleteController` 增强为静态关键词 + 语义候选混合模式，150ms 防抖异步获取；`AutocompleteOverlay` 增强类型图标（Variable/Function/Struct/Union/Enum/Field/Format 等）与签名详情；函数补全自动插入括号并将光标定位到括号内 | ✅ 完成 |
| Phase 25 | **模板 JIT（Trace-based Loop Accelerator）**：在 free-run 模式（`CideVM::run()`）中检测热点循环 trace（backward jump 目标命中超 100 次触发录制），将 trace 编译为预优化的函数指针序列（`jit_templates.rs`），跳过两层 match 分支；`StepEvent` 作为透明指令跳过录制；支持 side-exit（`JumpIfZero`/`JumpIfNotZero` 跳转被 taken 时退出 trace）；批量执行期间通过 `bulk_step_check` 保持 `step_count` 和 `max_steps` 的无限循环检测；运行完成后输出 `[JIT] 已编译 X 条 trace，加速执行 Y 步` 统计信息；统一模式保持逐指令执行不变 | ✅ 完成 |
| Phase 26 | **Flutter Bridge 通信优化**：FRB `Stream` 模式替代 Timer 轮询（`run_auto_steps_stream`），Rust 后台线程批量执行 100 步后主动推送；差分编码（`StepPayloadDelta`）仅传输变化的变量值；符号表 dedup（`StepStreamBatch.symbol_table`）全局去重变量名/类型名/函数名等字符串；前端 `UnifiedNotifier` 订阅 Stream 并实时解码恢复完整 `StepPayload` | ✅ 完成 |

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
- **`double`** — ✅ **已完整支持**（64 位 f64，字节偏移架构，含 `sizeof(double)=8`、`printf("%f")` 读取 f64）
- **~~函数调用参数的隐式转换提示~~** — ✅ **已解决（2026-06-05）**：`printf`/`scanf`/`fprintf` 格式字符串与参数类型静态匹配检查已落地。传入 int 给 `%f`、传入 float* 给 `%d` 等不匹配场景会在编译期报错（E3062/E3063）
- **参数化宏调用后带分号** — 形如 `SWAP(int,x,y);` 的参数化宏调用，若宏体本身已包含大括号 `{ ... }`，展开后形成 `{ ... };`（复合语句 + 空语句），当前 Parser 无法正确解析。 workaround：宏调用后不加额外分号（如 `SWAP(int,x,y)`），或使用 `do { ... } while(0)` 模式

### 已支持的关键特性
- **逗号分隔的多变量声明** — `int a = 1, b = 2;`
- **多维数组**（`int arr[3][3]`）— 声明、嵌套初始化列表 `{ {1,2}, {3,4} }`、索引访问 `arr[i][j]`、函数参数传递 `void f(int[][3])`
- **`#define` 宏** — 简单常量替换（如 `#define N 100`）
- **参数化宏** — `#define MAX(a,b) ((a)>(b)?(a):(b))`，支持多参数、嵌套括号实参、嵌套宏调用（如 `MAX(1, MIN(2,3))`）
- **for 循环变量作用域隔离** — `for (int i = 0; i < 3; i++)` 中的 `i` 只在循环体内可见，不覆盖外部同名变量；Block 语句块同样支持作用域隔离
- **printf 可变参数** — 支持任意数量参数（如 `printf("%d %d %d", a, b, c)`）
- **`printf`/`scanf` 格式字符串参数类型检查** — `printf("%f", 5)` 编译期报错（int 不匹配 `%f`）；`scanf("%d", &f)` 编译期报错（float* 不匹配 `%d`）。支持 `%d/%f/%s/%p/%c/%ld/%lf/%x/%o` 等常见格式说明符与参数类型的静态匹配
- **局部 `char` 数组字符串初始化** — `char s[6] = "hello"; printf("%s", s);`
- **全局 `char` 数组字符串初始化** — `char s[6] = "hello";`（全局作用域字符串初始化，正确写入连续字节）
- **`sizeof(字符串字面量)`** — `sizeof("hello")` 返回 6（含 `\0`），类型识别为 `char[N]` 数组而非 `char*` 指针
- **`enum` 局部/全局变量声明** — `enum Color c = GREEN;`（需先声明 enum 类型）
- **`typedef`** — `typedef int Integer; Integer a = 42;`
- **匿名结构体变量声明** — `struct { int x; } v;`（直接以匿名 struct 类型声明变量，局部/全局均支持，含初始化列表）
- **`typedef struct`** — `typedef struct { int x; } Point; Point p;`（匿名结构体 + typedef 别名）以及 `typedef struct Vec { int x; } VecAlias;`（命名结构体 + typedef 别名）
- **`sizeof` 运算符** — `sizeof(int)`、`sizeof(char)`、`sizeof(struct S)`、`sizeof(union U)`、`sizeof(arr)`、`sizeof(ptr)`
- **`scanf` 多参数** — `scanf("%d %d %d", &a, &b, &c)`
- **指针算术** — `p++` / `p--` / `p + i` / `p - i` / `p - q`，自动按 pointee 类型大小缩放（`int*` 步长 4，`char*` 步长 1，`struct*` 步长为结构体大小）
- **函数前向声明** — `int foo(int);` 原型声明，函数定义可放在调用者之后
- **字符串库函数** — `strlen(s)`、`strcpy(dest, src)`、`strcmp(a, b)`（宿主导入函数）
- **显式类型转换（Cast）** — `(int*)p`、`(char*)arr`、`(float)a`、`(int)b` 等标量/指针间转换
- **`fprintf`** — `fprintf(stdout, "format", ...)` / `fprintf(stderr, "format", ...)`，stream 参数被忽略，输出行为与 `printf` 相同
- **`fgets`/`fputs`** — 文件逐行读写；`fgets(buf, n, fp)` 读取最多 `n-1` 字节或到换行，`fputs(s, fp)` 写入字符串；配合 `fopen`/`fclose` 实现完整文件 I/O 流程
- **`realloc`** — `realloc(ptr, new_size)`，支持扩容/缩容、NULL ptr（等价 malloc）、size 0（等价 free）
- **`qsort`** — `qsort(base, nmemb, size, compar)`，支持用户自定义比较函数（通过 VM 调用用户函数）
- **`union` 类型** — `union U { int i; double d; }; union U u; u.i = 1; u.d = 3.14; printf("%.2f", u.d);`，内存布局为所有字段 offset=0、size=max(fields)，支持成员访问、指针访问（`p->i`）、`sizeof(union U)`
- **`static` 局部变量** — `static int count = 0;` 分配在全局内存区，跨函数调用持久化；支持读写、自增自减、取地址、数组状态保持；初始化只在首次进入函数时执行一次
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
- **算法步骤语义标注** — 为 27 种算法/数据结构操作（冒泡/选择/插入/快速/归并/堆排序/希尔排序/计数排序/二分/BFS/DFS/DP/链表删除与尾插/BST插入与查找/字符串反转/GCD/素数判断/汉诺塔/顺序表/循环队列/链栈/链队列/层序遍历/哈希表/约瑟夫环）预定义步骤模板，运行时根据源码行特征和变量值推断当前阶段并生成中文教学描述
  - 后端：`unified/algorithm_steps.rs` 推断引擎，每种算法独立推断逻辑；`StepPayload` 新增 `algorithm_step` 字段
  - 前端：`ExecutionControlPanel` 步骤横幅（按 phase 着色：outer_loop 蓝、swap 琥珀、compare 紫、finish 绿等）；`AlgorithmTab` 静态步骤流程预览（带运行时高亮）；`ArrayVisTab` 已排序边界绿色高亮（冒泡右侧/选择左侧/插入左侧）
- **链表可视化** — `LinkedListVisualizer` CustomPainter 绘制节点+箭头，支持 NodeCreate/Access/Delete 闪色；渐进式入场动画；`LinkedListVisTab` 集成统一模式，从 `StepPayload.localVars` 读取头指针驱动时间旅行
- **二叉树可视化** — `TreeVisualizer` 满二叉树位置层级布局，节点滑入+连线渐进动画，最大深度 6 限制；`TreeVisTab` 集成统一模式
- **变量级高亮** — `re_editor` `spanBuilder` 集成：当前执行行的被读变量名显示淡蓝底色、被写变量名显示淡橙底色，保留语法高亮；`VariablesTab` 值变化背景闪烁动画
- **代码模板扩展** — 总计 **43 个模板**，覆盖排序（冒泡/选择/插入/快速/归并/堆排序/希尔排序/计数排序）、查找（线性/二分）、图算法（BFS/DFS）、动态规划（斐波那契/01背包）、数据结构（顺序表、链表节点/头插/尾插/遍历/删除/双向链表、二叉树节点/先序/中序/后序/层序遍历/BST插入与查找、栈/链栈/队列/链队列/循环队列、哈希表）、字符串（反转）、基础（数组遍历、指针交换、GCD、素数判断、约瑟夫环）、递归（阶乘/斐波那契/汉诺塔）
- **代码模板参数化 + 交互式教程** — 核心算法模板（冒泡/选择/插入/快速/归并/二分/线性查找）支持参数占位符（如 `{{n:5}}`、`{{target:3}}`）；`TemplateParamDialog` 底部弹窗收集参数；`TemplateTutorialPanel` 逐步骤高亮代码行并展示教学描述；每步骤的关键行带 💡 `ExpansionTile` 可展开查看详细解释；教程最后一步点击"运行代码"自动插入生成代码、编译并启动统一模式；`LearningProgress` 记录 `completedTutorials`
- **Use-After-Free / Double-Free 运行时检测** — VM `execute_memory` 指令层在每次堆内存解引用前检查 `freed_logs`；访问已释放但尚未重用的堆内存时立即 trap 并弹出知识卡片（E3060），报告分配行号和释放行号；`host_free` 检测到对同一块内存重复释放时 trap（E3061）；`malloc`/`realloc` 重用内存时自动清理对应 `freed_logs`；统一模式 Trap 自动回退到上一步；新增 3 个 E2E 测试
- **认知推理 P0（运行时根因分析）** — `TraceAnalyzer` 基于执行历史切片推断 Trap 根因：数组越界时识别 OffByOne（`<=` 条件）、索引变量未初始化、循环起始错误；Use-After-Free / Double-Free 时提取分配/释放时间线；除零时定位值为 0 的除数变量；NULL 指针时追踪指针历史变化。`RootCauseHint` 结构化数据（category/one_liner/related_lines/fix_kind）经 FRB 传输；前端 `RootCauseBanner` 以琥珀/深橙/紫/青等颜色编码展示根因，附带可点击行号跳转和修复建议标签
- **认知推理 P1（教学推理层）** — `MisconceptionPattern` 定义 6 种认知误解模式（边界混淆 M01、指针生命周期混淆 M02、赋值与比较混淆 M03、数组指针退化误解 M04、递归边界遗漏 M05、格式化字符串误用 M06）；`detect_misconceptions` 基于最近 20 次编译记录滑动窗口检测稳定犯错模式；`recommend_learning_paths` 为每种检测到的模式组装知识卡片→模板高亮→练习的最小有效学习路径；Flutter `LearningProgress` 新增 `recentCompileRecords` 字段（SharedPreferences 持久化）；`LearningPathPanel` BottomSheet 面板展示诊断结果与彩色路径卡片，点击步骤可直接加载模板/启动教程
- **认知推理 P2（知识图谱层）** — `KnowledgeGraph` 定义 24 个概念节点（编译域 8 个：变量声明/类型系统/隐式转换/指针类型/算术/逻辑/位运算符/作用域；内存域 8 个：栈/堆/指针/取地址/解引用/指针算术/数组/数组退化/结构体布局；控制流域 6 个：条件分支/for/while/边界条件/函数调用/参数传递/返回值/递归）和 30+ 条关系边；`activate_from_error` 错误码映射激活（如 3051 越界激活 Array + BoundaryCondition）、`activate_from_ast` AST 特征激活（如 malloc/free 激活 HeapMemory + Pointer）、`find_prerequisite_path` 前置依赖路径查找；Flutter `ConceptGraphView` CustomPainter 三列域布局，激活节点发光高亮+邻居半透明，边按关系类型着色（Prerequisite 橙/LeadsTo 蓝/CommonMistake 红虚线），点击节点弹出概念解释 BottomSheet
- **认知推理 P3（代码理解层）** — `ControlFlowGraph` 从 AST `FuncDecl` 提取基本块（`BasicBlock { id, stmts, terminator }`），支持 `Return`/`Goto`/`Branch`/`Switch`/`FallThrough` 五种终结符，自动构建边表；迭代算法计算 dominator 树；检测不可达块和循环（通过回边识别）；`DataFlow` 提供活跃变量分析（LiveVarResult，反向迭代数据流固定点）和常量条件求值（`evaluate_constant_condition`）；`IntentInference` 综合函数名启发式（sort/search/find/traverse/calc）、变量名模式（swap/temp/mid/left/right/target/next/curr/sum/result）、CFG 结构特征（循环嵌套/回边/提前返回/不可达块）、递归调用检测，输出 5 种意图（Sort/Search/Traverse/Compute/Transform）的置信度评分与推理原因；`algorithm_detector` 集成 CFG 特征并修复 `has_min_max_track` 检测缺失；FRB 导出 `infer_intent_from_source` 接受源码字符串返回 `Vec<IntentScore>`；Flutter `IntentInferencePanel` 作为底部 Tab 实时展示意图排名卡片（按置信度分高/中/低三级徽章），附带 💡 推理原因列表
- **Flutter Bridge 通信优化** — `run_auto_steps_stream` Stream 模式替代 `Timer.periodic` 轮询：Rust 后台线程每 batch 执行 100 步后立即通过 `StreamSink` 推送 `StepStreamBatch`；差分编码 `StepPayloadDelta` 仅传输值变化的变量（`var_deltas: Vec<VarDelta>`），不变变量不重复传输；符号表 dedup `symbol_table: Vec<String>` 全局去重变量名、类型名、函数名等字符串，用 `SymIdx` 索引替代；Flutter `UnifiedNotifier` 订阅 Stream 并调用 `_decodeBatch` 恢复完整 `StepPayload` 列表，后续业务逻辑完全不变
- **语义智能补全 v2** — `CompletionEngine`（`native/src/engine/completion.rs`）基于编译快照 + 轻量级 Token 增量扫描，提供五种上下文感知补全：
  - **成员访问**：`expr.` / `expr->` 自动列出 struct/union 字段（如 `p.x`, `n->next`）
  - **类型上下文**：`int `、`struct ` 后自动提示类型名、结构体/联合体/typedef 别名
  - **表达式上下文**：当前作用域局部变量优先，全局变量、函数签名、类型名混合排序；函数候选自动插入 `()` 并将光标定位到括号内
  - **格式字符串**：`printf("` / `scanf("` 内自动提示 `%d`、`%f`、`%s`、`%p` 等占位符
  - **预处理指令**：`#` 后提示 `include`、`define`、`ifdef` 等，并提供标准头文件列表
  - 后端：每次成功编译时从 AST 提取符号快照（`CompletionSnapshot`）持久化到 `Session.compile.completion_snapshot`；增量扫描只解析光标前源码，提取局部变量和参数
  - FRB API：`get_completion_candidates(source, line, column, prefix)` 返回语义候选列表
  - Flutter：`AutocompleteController` 150ms 防抖异步获取语义候选，与静态关键词列表合并去重；`AutocompleteOverlay` 增强类型图标（Variable/Function/Struct/Union/Enum/Field/Format 等）与详情签名展示

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
- **影子验证新增（2026-06-06）**：为数据结构教材语法拓展补充 8 个影子测试用例（参数化宏 `MAX`/`SWAP`/`SQUARE`/`MIN` 嵌套、static 局部变量计数器/数组持久化、`fgets`/`fputs` 文件读写），全部与 Clang 输出匹配；`shadow_verify.py` 新增内存泄漏检测报告过滤
- ~~**已知问题（2026-05-17）**：`for (int i = 0; ...)` 循环变量作用域未隔离外部同名变量~~ → ✅ **已解决（2026-06-05）**：BytecodeGen 新增 `local_scope_stack` 作用域栈，Block 和 For 语句进入/退出时正确保存/恢复局部变量映射，循环变量不再泄漏到外部作用域
- **全局数组变量声明名字错误（2026-06-05）**：`char s[6];` 等全局数组声明中，`parse_global_var_or_func` 使用 `self.previous()` 获取变量名，在数组后缀 `]` consume 后导致变量名被错误设为 `]` → 改用 `parse_declarator` 返回的 `name`
- **字符串字面量类型精度（2026-06-05）**：Parser 将字符串字面量类型设为 `char*`（指针），导致 `sizeof("hello")` 返回指针大小 4 而非数组大小 6 → 改为 `char[N]` 数组类型，TypeChecker 同步更新；数组到指针退化由现有 `check_array_pointer_assignable` 处理
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
- **函数指针完整支持（2026-06-05）**：
  - 局部/全局函数指针变量声明与初始化：`int (*fp)(int) = add;`、`int (*global_fp)(int, int) = add;`
  - 取函数地址：`int (*fp)(int) = &add;`
  - 函数指针作为参数传递：`apply(int (*op)(int), x)`
  - 函数指针调用：`fp(3, 4)`、`(*pp)(5)`、`ops[0](3, 4)`
  - 结构体成员函数指针：`struct Ops { int (*op)(int, int); }; ops.op = add; ops.op(3, 4);`
  - 多级函数指针：`int (**pp)(int) = &fp;`
  - 返回指针的函数指针：`int *(*fp)(int) = greet;`
  - `sizeof` 函数指针类型：`sizeof(int (*)(int))`
  - `typedef` 函数指针：`typedef int (*Op)(int, int); Op op = mul; Op ops[2] = {mul, divi};`
  - 支持 `double`/`long long` 参数的函数指针间接调用（`SplitD`/`SplitQ` 正确处理）
  - `static` 局部变量：`static int arr[3] = {1, 2, 3};`
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
  - `AGENTS.md` / `CHANGELOG.md`：测试数量 `44` → `331`
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
- **Phase 27 — 增量快照 + 智能检查点（2026-06-06）**：
  - **F-02 增量快照**：VM 引入 4KB 页级脏页追踪（`dirty_pages: [u64; 4]` bitmap，覆盖 256 页）。所有内存写入操作（`store_i32/i64/i8`、`write_cstring`、`write_memory`、`copy_memory`、`local zero-fill`）自动标记脏页。`CheckpointManager` 每 `full_every=5` 个检查点保存一个全量基准，其余保存 `MemoryImage::Delta`（仅脏页）。`nearest()` 自动重建为完整 `MemoryImage::Full` 供 `restore()` 使用。检查点内存占用从 50MB（50×1MB）降至典型 5-10MB。
  - **F-01 智能检查点**：`should_checkpoint` 从纯固定间隔升级为"固定间隔保底 + 智能边界触发"。基于源码行轻量级推断：`for`/`while` → 循环边界、`return` → 返回、`malloc`/`free` → 内存操作、含 `temp`+`arr[`+`=` → 数组交换、其他函数调用 → 调用。带 `min_gap = interval/4` 密集保护，避免检查点过于密集。
  - **`seek_to` 重放动态检查点**：正向重放过程中每隔 `interval` 步自动保存新检查点，后续 seek 无需从旧检查点重新重放，显著加速长程序时间旅行。
  - 新增 3 个专项测试：`test_incremental_snapshot_size`、`test_checkpoint_manager_incremental_chain`、`test_smart_checkpoint_triggers`。

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
