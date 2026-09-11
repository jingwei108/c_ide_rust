# Cide 数据结构可视化计划文档

> 版本：2026-05-17（设计稿）  
> 状态：数组排序/链表/二叉树可视化 MVP ✅ 已在**当时的前端**实现；图/哈希表等复杂结构 ⏳ 待实现。**2026-09-11 前端切割后**：渲染与交互层（`*_visualizer.dart` 等）已随 `CideFlutter/` 移出仓库，本仓库只承载**数据结构检测/运行时反推的数据载荷**，经三出口（C ABI / wasm32 / `cide_cli serve`）交付给消费方。  
> 设计原则：**零侵入（不写 vis_*()） + 人机协作（分层确认兜底） + 按需反推（不预录帧）**（设计原则不因切割改变）  
> **最后核对日期**：2026-09-11（修订：Dart widget 引用改为三出口数据载荷；如实补记 §4 的 `data_structure_detector.rs` 在仓库中不存在、Phase 0/1 检测器未实施这一缺口）

---

## 目录

- [1. 设计哲学](#1-设计哲学)
  - [1.1 零侵入 ≠ 零交互](#11-零侵入--零交互)
  - [1.2 分层交互模型](#12-分层交互模型)
  - [1.3 按需反推](#13-按需反推)
- [2. 架构总览](#2-架构总览)
- [3. 核心数据模型](#3-核心数据模型)
- [4. 后端：数据结构检测器](#4-后端数据结构检测器)
  - [4.1 类型拓扑分析](#41-类型拓扑分析)
  - [4.2 根变量推断](#42-根变量推断)
  - [4.3 字段偏移推断](#43-字段偏移推断)
  - [4.4 数组型结构检测](#44-数组型结构检测)
- [5. 后端：VM 运行时反推引擎](#5-后端vm-运行时反推引擎)
- [6. 交互层：分层确认与兜底](#6-交互层分层确认与兜底)
- [7. 前端：帧缓存与渲染](#7-前端帧缓存与渲染)
- [8. 各数据结构可视化方案](#8-各数据结构可视化方案)
- [9. 失败兜底与用户体验](#9-失败兜底与用户体验)
- [10. 跨平台移植策略](#10-跨平台移植策略)
- [11. 实施路线图](#11-实施路线图)

---

## 1. 设计哲学

### 1.1 零侵入 ≠ 零交互

**零侵入**的边界是：**不修改用户源代码，不增加学习成本**。

```c
// 用户写的代码（纯净的 C，无任何额外函数）
void bubbleSort(int arr[], int n) {
    for (int i = 0; i < n - 1; i++) {
        for (int j = 0; j < n - i - 1; j++) {
            if (arr[j] > arr[j + 1]) {
                int temp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
        }
    }
}
```

用户不需要写 `vis_array()`，不需要学 `// @vis:` 注释，不需要改任何一行代码。

但**编译成功后**，IDE 可以问用户一句"我识别出你在写冒泡排序，对吗？"——这不是侵入，是协作。就像编译器报完错后给出修复建议一样，用户可以选择接受或忽略。

### 1.2 分层交互模型

面对学生**不规范、有错漏、命名随意**的代码，自动检测不可能 100% 准确。系统提供三层保障：

```
编译成功
    ↓
Rust 后端检测
    ↓
【置信度分层】
    │
    ├── 高置信度（>=90%）+ 单一数据结构
    │       → 全自动出动画
    │       → 动画面板顶部显示："已自动识别为链表（置信度 96%）[识别有误？]"
    │
    ├── 中置信度（70%-90%）或 多数据结构并存
    │       → 底部非阻断选择条：
    │         "检测到：① 数组排序（95%） ② 链表（82%），点击切换查看"
    │
    └── 低置信度（<70%）或检测失败
            → 动画面板显示友好提示：
              "未能自动识别数据结构"
              [手动选择类型]  [查看代码建议]
```

**关键设计**：
- 不是阻断式弹窗（AlertDialog），而是面板内嵌提示（SnackBar / 顶部条）
- 用户随时可点击"识别有误"重新选择或手动配置
- 手动配置后实时刷新，无需重新编译

### 1.3 按需反推

**拒绝预录帧**。Cide 是交互式 VM（单步/暂停/继续），不需要像视频一样提前录好所有帧。

```
用户点击"下一步"
    ↓
VM 执行到下一个 StepEvent（第 42 行）
    ↓
出口消费方请求当前可视化状态（capi / serve JSON-lines / wasm）
    ↓
Rust 读取 VM 内存 → 构造 VisState 载荷 → 返回
    ↓
消费方本地缓存 → 插值动画 → 渲染
```

**优势**：无内存爆炸、实时交互、核心与出口解耦。

---

## 2. 架构总览

```
┌─────────────────────────────────────────────────────────────┐
│        出口消费方（表现+交互层：社区前端 / Web / headless）     │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  交互层（消费方自行实现）                              │   │
│  │  - 分层提示条（全自动 / 多选切换 / 手动配置）          │   │
│  │  - 手动配置面板：类型选择 + 字段 offset 映射           │   │
│  └─────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  帧缓存与动画层（消费方自行实现）                      │   │
│  │  - 帧缓存：按 StepPayload 窗口语义（schema §4）        │   │
│  │  - 由消费方的动画/插值机制驱动帧间过渡                 │   │
│  └─────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  布局引擎（各消费方独立实现）                          │   │
│  │  - 数组：柱状图坐标分配                                │   │
│  │  - 树：递归宽度计算 / Reingold-Tilford                 │   │
│  │  - 链表：水平线性布局                                  │   │
│  └─────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  渲染组件（消费方自行实现）                            │   │
│  │  - 数组排序动画 / 链表图 / 二叉树图 / 变量值面板        │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              ↑
        三出口：C ABI（capi）/ wasm32 / cide_cli serve（JSON-lines）
                              ↓
┌─────────────────────────────────────────────────────────────┐
│                Rust 后端（数据+执行层）                        │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  ① 数据结构检测器（编译时）                            │   │
│  │     输入：ProgramNode（AST）                          │   │
│  │     输出：Vec<DataStructureMatch>（含置信度、字段偏移） │   │
│  │     - 类型拓扑分析（自引用指针、标量、flag 字段）        │   │
│  │     - 根变量推断（全局/参数/局部变量）                  │   │
│  │     - 数组型结构检测（栈/队列/堆索引模式）              │   │
│  └─────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  ② CideVM 教学虚拟机                                   │   │
│  │     - 字节码执行、StepEvent、内存隔离                  │   │
│  │     - 单步暂停后，外部可读内存、符号表                  │   │
│  └─────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  ③ 运行时反推引擎（VM 暂停时触发）                     │   │
│  │     输入：当前行号 + DataStructureMatch + VM 状态      │   │
│  │     输出：VisState（数组值、结构状态、语义高亮）        │   │
│  │     - 从符号表查地址 → 从 memory_ 读取值              │   │
│  │     - 遍历指针扫描链表/树节点                          │   │
│  │     - 语义事件推断（compare/swap/recurse）            │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. 核心数据模型

### 3.1 Rust 后端：检测与运行时

> **现状核实（2026-09-11）**：本节是 2026-05-17 的**类型草案**，其中的 `#[frb]` 标注属 FRB 桥接时代（该桥接与 `native/src/api/` 已随前端切割移出，历史资产，已迁出）。经 `grep` 核实（`native/**/*.rs`）：`DsKind` / `FieldOffsets` / `DataStructureMatch` / `SemanticEvent` / `VisArrayState` / `VisNodeState` / `VisStructureState` / `VisVariableState` / `VisState` / `VisStatus` / `VisResult` **在 Rust 源码中均不存在**——它们只出现在本文档与 `docs/archive/AUTO_VISUALIZATION_DETECTION_PLAN.md` 两份设计稿里，即本节的检测器与可视化载荷**尚未落地**（未完成缺口，见 §11 Phase 0/1）。
>
> **当前实际交付的可视化载荷**是 [`docs/spec/STEP_PAYLOAD_SCHEMA_V0_1.md`](../spec/STEP_PAYLOAD_SCHEMA_V0_1.md)：`StepPayload`（含 `semantic_label` / `heatmap_line` / `heatmap_count`）、`ApiVariableSnapshot`、`ApiFrameInfo`、`AccessedVar`、`ArraySnapshot`、`PointerSnapshot`（四状态）、`AlgorithmStepSnapshot`、`VisEvent`、`RootCauseHint`。本文档的数据结构检测扩展（链表/树/栈/队列/堆）若落地，应作为**该 schema 的增量字段**通过三出口交付，而不是复活 FRB 专用类型（FRB 桥接为历史资产，已迁出）。

```rust
// 设计稿（文件路径为草案；native/src/session.rs 现仍存在，但以下类型未在其中定义）

/// 数据结构种类
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DsKind {
    Array,
    LinkedList,
    BinaryTree,
    Stack,
    Queue,
    Heap,
}

/// 字段偏移信息（后端只输出 offset，不输出语义名）
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FieldOffsets {
    pub value_offset: Option<i32>,
    pub left_offset: Option<i32>,
    pub right_offset: Option<i32>,
    pub next_offset: Option<i32>,
    pub color_offset: Option<i32>,
    pub height_offset: Option<i32>,
    pub parent_offset: Option<i32>,
}

/// 数据结构检测结果
// (历史) #[frb]  —— FRB 桥接已迁出，未来经三出口以 JSON 载荷交付
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DataStructureMatch {
    pub ds_kind: String,
    pub display_name: String,
    pub confidence: i32,
    pub struct_name: Option<String>,
    pub root_var_name: String,
    pub root_var_addr: u32,
    pub field_offsets: FieldOffsets,
    pub struct_size: i32,
    pub detection_reason: String,
}

/// 语义事件：描述当前行在算法层面的含义
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SemanticEvent {
    pub kind: String,
    pub description: String,
}

/// 数组运行时状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VisArrayState {
    pub name: String,
    pub values: Vec<i32>,
    pub highlights: Vec<(i32, String)>,
    pub active_range: Option<(i32, i32)>,
}

/// 节点运行时状态（链表/树）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VisNodeState {
    pub addr: u32,
    pub value: i32,
    pub left_addr: Option<u32>,
    pub right_addr: Option<u32>,
    pub next_addr: Option<u32>,
}

/// 结构运行时状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VisStructureState {
    pub kind: String,
    pub name: String,
    pub root_addr: u32,
    pub nodes: Vec<VisNodeState>,
    pub highlighted_addrs: Vec<u32>,
}

/// 变量状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VisVariableState {
    pub name: String,
    pub value: String,
    pub ty: String,
}

/// 单次 StepEvent 对应的可视化状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VisState {
    pub line: i32,
    pub arrays: Vec<VisArrayState>,
    pub structures: Vec<VisStructureState>,
    pub variables: Vec<VisVariableState>,
    pub semantic_event: Option<SemanticEvent>,
}

/// 可视化结果状态（用于消费方判断交互层级）
// (历史) #[frb]  —— FRB 桥接已迁出
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum VisStatus {
    AutoDetected,
    UserConfirmed,
    ManuallyConfigured,
    Failed(String),
}

// (历史) #[frb]  —— FRB 桥接已迁出
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VisResult {
    pub status: VisStatus,
    pub state: Option<VisState>,
    pub suggestion: Option<String>,
}
```

### 3.2 消费方：配置持久化（职责移交社区前端）

> 下例为 2026-05-17 设计稿中的 Dart 实现，**已随 `CideFlutter/` 迁出**（历史资产，已迁出）。配置持久化（如浏览器 localStorage / 桌面端偏好存储）属**出口消费方职责**；后端只负责在载荷中返回检测结果（`root_var_name`、`field_offsets`、`confidence` 等，若按 §3.1 落地）。

```dart
// 历史设计稿（已迁出）：lib/models/vis_config.dart

class VisConfig {
  final String dsKind;
  final String rootVarName;
  final FieldOffsets fieldOffsets;
  final int structSize;
  
  // 配置来源标记
  final ConfigSource source; // autoDetected / userConfirmed / manual
}

enum ConfigSource { autoDetected, userConfirmed, manual }
```

配置持久化到消费方本地存储，用户下次打开同类型代码时直接复用。

---

## 4. 后端：数据结构检测器

> **⚠️ 未完成缺口（2026-09-11 核实）**：文件 `native/src/compiler/data_structure_detector.rs` **在仓库中不存在**（`native/src/compiler/` 现有：`algorithm_detector/`、`ast.rs`、`cfg.rs`、`data_flow.rs`、`intent.rs`、`mod.rs`），入口函数 `detect_data_structures` 亦不存在。本章与 §11 的 Phase 0/1 是**尚未实施的计划项**，如实保留，不视为已完成。
>
> 未来的落点应遵循架构纪律：检测逻辑落语言中立 Rust 层（与 `algorithm_detector` 同级），经三出口以 JSON 载荷交付（见 §3.1 的现状说明）。

文件（计划）：`native/src/compiler/data_structure_detector.rs`
入口（计划）：`detect_data_structures(program: &ProgramNode) -> Vec<DataStructureMatch>`
调用时机（计划）：`run_compile_pipeline` 成功后，与 `detect_algorithms` 并列调用。

### 4.1 类型拓扑分析

```rust
struct StructProfile {
    struct_name: String,
    fields: Vec<FieldProfile>,
    self_pointer_count: usize,
    scalar_count: usize,
    int_flag_count: usize,
    child_array_count: usize,
}

struct FieldProfile {
    name: String,
    ty: Type,
    is_self_pointer: bool,
    is_self_array: bool,
    is_scalar: bool,
    is_int: bool,
}

fn analyze_struct_topology(decl: &StructDecl) -> StructProfile {
    // 扫描字段：自引用指针数、标量数、疑似 flag 字段数
}

fn match_struct_to_ds_kind(profile: &StructProfile) -> Option<DsKind> {
    match (profile.self_pointer_count, profile.child_array_count, profile.int_flag_count) {
        (1, 0, 0) => Some(DsKind::LinkedList),
        (2, 0, 0) => Some(DsKind::BinaryTree),
        (2, 0, 1) => {
            if has_field_named(profile, "color") || has_field_named(profile, "red") {
                Some(DsKind::BinaryTree) // 前端根据 color 字段显示红黑样式
            } else if has_field_named(profile, "height") || has_field_named(profile, "bf") {
                Some(DsKind::BinaryTree) // 前端根据 height 显示 AVL 样式
            } else {
                Some(DsKind::BinaryTree)
            }
        }
        _ => None,
    }
}
```

**设计约束**：后端只区分到 `BinaryTree` 这一层。是否显示 BST/AVL/红黑树的特殊样式，由前端根据 `FieldOffsets` 中是否存在 `color_offset` / `height_offset` 决定。

### 4.2 根变量推断

```rust
fn find_root_variables(program: &ProgramNode, struct_name: &str) -> Vec<(String, i32)> {
    let mut candidates = Vec::new();

    // 1. 全局变量（置信度 95）
    for global in &program.globals {
        if is_pointer_to_struct_type(&global.ty, struct_name) {
            candidates.push((global.name.clone(), 95));
        }
    }

    // 2. 函数参数（第一个参数 + 函数名语义）
    for func in &program.funcs {
        for (idx, param) in func.params.iter().enumerate() {
            if is_pointer_to_struct_type(&param.ty, struct_name) {
                let mut conf = 70;
                if idx == 0 { conf += 15; }
                let name_lower = func.name.to_lowercase();
                if name_lower.contains("insert") || name_lower.contains("delete")
                    || name_lower.contains("traverse") || name_lower.contains("search")
                    || name_lower.contains("build") || name_lower.contains("create") {
                    conf += 10;
                }
                candidates.push((param.name.clone(), conf.min(98)));
            }
        }
    }

    // 3. main 函数局部变量（置信度 80）
    if let Some(main_func) = program.funcs.iter().find(|f| f.name == "main") {
        if let Some(body) = &main_func.body {
            collect_local_ptr_vars(body, struct_name, &mut candidates);
        }
    }

    // 去重：保留最高置信度
    let mut seen = HashSet::new();
    candidates.retain(|(name, _)| seen.insert(name.clone()));
    candidates
}
```

### 4.3 字段偏移推断

```rust
fn infer_field_offsets(profile: &StructProfile, ds_kind: &DsKind) -> FieldOffsets {
    let mut offsets = FieldOffsets::default();
    let mut current_offset = 0i32;

    for field in &profile.fields {
        let field_size = type_size(&field.ty);
        
        // 值字段：第一个非指针标量
        if field.is_scalar && !field.is_self_pointer && !field.is_self_array && offsets.value_offset.is_none() {
            offsets.value_offset = Some(current_offset);
        }
        
        // 自引用指针字段
        if field.is_self_pointer {
            match ds_kind {
                DsKind::LinkedList => {
                    if offsets.next_offset.is_none() {
                        offsets.next_offset = Some(current_offset);
                    }
                }
                DsKind::BinaryTree => {
                    let name_lower = field.name.to_lowercase();
                    if name_lower.contains("right") || name_lower.contains("rc") {
                        offsets.right_offset = Some(current_offset);
                    } else if name_lower.contains("left") || name_lower.contains("lc") {
                        offsets.left_offset = Some(current_offset);
                    } else if name_lower.contains("parent") || name_lower.contains("p") {
                        offsets.parent_offset = Some(current_offset);
                    } else {
                        // 按顺序推断：第一个=left，第二个=right
                        if offsets.left_offset.is_none() { offsets.left_offset = Some(current_offset); }
                        else if offsets.right_offset.is_none() { offsets.right_offset = Some(current_offset); }
                    }
                }
                _ => {}
            }
        }
        
        // 特殊字段（color/height）
        let name_lower = field.name.to_lowercase();
        if field.is_int {
            if name_lower.contains("color") || name_lower.contains("red") || name_lower.contains("black") {
                offsets.color_offset = Some(current_offset);
            }
            if name_lower.contains("height") || name_lower.contains("bf") || name_lower.contains("balance") {
                offsets.height_offset = Some(current_offset);
            }
        }
        
        current_offset += field_size;
    }

    offsets
}
```

### 4.4 数组型结构检测

```rust
fn detect_array_based_structures(program: &ProgramNode) -> Vec<DataStructureMatch> {
    // 扫描数组 + 配套变量模式
    // 栈：数组 + top 变量（只 ++/--）
    // 队列：数组 + front + rear（% 运算）
    // 堆：数组 + 父子索引计算（i/2, 2i+1, 2i+2）
}

fn detect_heap_pattern(body: &Stmt, arr_name: &str) -> bool {
    let mut has_parent_idx = false;
    let mut has_left_child_idx = false;
    let mut has_right_child_idx = false;
    
    walk_expr_in_stmt(body, |expr| {
        if let Expr::Index { array, index, .. } = expr {
            if let Expr::Identifier { name, .. } = array.as_ref() {
                if name == arr_name {
                    let idx_str = expr_to_string(index).to_lowercase();
                    if idx_str.contains("/2") || idx_str.contains(">>1") {
                        has_parent_idx = true;
                    }
                    if idx_str.contains("*2+1") || idx_str.contains("<<1+1") {
                        has_left_child_idx = true;
                    }
                    if idx_str.contains("*2+2") || idx_str.contains("<<1+2") {
                        has_right_child_idx = true;
                    }
                }
            }
        }
    });
    
    has_parent_idx && (has_left_child_idx || has_right_child_idx)
}
```

---

## 5. 后端：VM 运行时反推引擎

当 VM 触发 `StepEvent` 时，后端**不预录帧**，而是**按需反推**。

> **现状（2026-09-11）**：下例中的 `native/src/api/cide.rs`（FRB 桥接层）**已随前端切割移出仓库**，`#[frb]` 标注与 `cide_get_vis_state()` 均不存在。当前已落地的"按需反推"能力走三出口：`cide_step_next_json` / `cide_get_step_payloads_json`（capi，见 [`docs/spec/STEP_PAYLOAD_SCHEMA_V0_1.md`](../spec/STEP_PAYLOAD_SCHEMA_V0_1.md) §6）与 `cide_cli serve` 的同构帧。本节余下代码为设计稿（历史资产，已迁出）。

```rust
// 历史设计稿（已迁出）：native/src/api/cide.rs

// (历史) #[frb]  —— FRB 桥接已迁出
pub fn cide_get_vis_state(session: &mut Session) -> VisResult {
    let compile = &session.compile;
    let runtime = &session.runtime;
    let vm = session.vm.as_ref()?;

    // 获取当前激活的可视化配置
    let config = match &session.vis_config {
        Some(c) if c.enabled => c,
        _ => {
            // 无配置：尝试自动匹配当前函数的 AlgorithmMatch
            return try_auto_detect(session);
        }
    };

    let ds_match = config.ds_match.as_ref()?;

    match ds_match.ds_kind.as_str() {
        "Array" | "Stack" | "Queue" | "Heap" => {
            build_array_vis_result(session, ds_match)
        }
        "LinkedList" => {
            build_linked_list_vis_result(session, ds_match)
        }
        "BinaryTree" => {
            build_tree_vis_result(session, ds_match)
        }
        _ => VisResult {
            status: VisStatus::Failed("不支持的数据结构类型".to_string()),
            state: None,
            suggestion: Some("请尝试手动选择其他类型".to_string()),
        },
    }
}

fn build_array_vis_result(session: &Session, ds_match: &DataStructureMatch) -> VisResult {
    let vm = session.vm.as_ref().unwrap();
    let values = read_array_from_vm(vm, ds_match.root_var_addr, ds_match.array_len());
    let highlights = infer_array_highlights(session, ds_match);
    
    VisResult {
        status: VisStatus::AutoDetected,
        state: Some(VisState {
            line: session.runtime.current_line,
            arrays: vec![VisArrayState {
                name: ds_match.root_var_name.clone(),
                values,
                highlights,
                active_range: infer_active_range(session, ds_match),
            }],
            structures: Vec::new(),
            variables: capture_local_variables(vm, &session.compile.symbols),
            semantic_event: infer_semantic_event(session, ds_match),
        }),
        suggestion: None,
    }
}
```

---

## 6. 交互层：分层确认与兜底（消费方职责）

> **现状（2026-09-11）**：本节描述的分层交互（自动 / 多选 / 手动兜底）是**出口消费方**要实现的行为契约，随 `CideFlutter/` 迁出后不再由本仓库实现。后端需保证的是：检测结果带**置信度**与**多候选**，让消费方能自行分层。下例 Dart 为历史设计稿（历史资产，已迁出）。

### 6.1 编译后：检测结果到交互的映射

```dart
// 历史设计稿（已迁出）
class VisNotifier extends StateNotifier<VisUiState> {
  Future<void> onCompileSuccess(CompileResult result) async {
    final matches = result.dataStructureMatches;
    
    if (matches.isEmpty) {
      state = VisUiState.failed(
        message: "未能识别出数据结构",
        suggestion: "尝试手动选择数据结构类型",
      );
      return;
    }
    
    matches.sort((a, b) => b.confidence.compareTo(a.confidence));
    final top = matches.first;
    
    if (matches.length == 1 && top.confidence >= 90) {
      state = VisUiState.autoDetected(
        match: top,
        message: "已自动识别为${top.displayName}（置信度 ${top.confidence}%）",
      );
      await _activateVisualization(top);
    } else if (top.confidence >= 70) {
      state = VisUiState.selectionRequired(
        matches: matches,
        message: "检测到 ${matches.length} 个数据结构，请选择查看",
      );
    } else {
      state = VisUiState.failed(
        message: "自动识别置信度较低",
        suggestion: "请手动选择数据结构类型并配置字段",
        manualConfigAvailable: true,
      );
    }
  }
}
```

### 6.2 手动配置面板

手动配置面板不是"惩罚"，而是**教学工具本身**。学生在配置时会理解："原来链表的 data 和 next 是这样被程序理解的。"

### 6.3 非阻断提示条 UI

| 状态 | UI |
|:---|:---|
| AutoDetected | 绿色提示条："已自动识别为链表（96%）[识别有误？]" |
| SelectionRequired | 橙色选择条：ChoiceChip 列表切换多个数据结构 |
| Failed | 红色提示条："未能识别" + [手动配置] 按钮 |

---

## 7. 消费方：帧缓存与渲染（职责移交社区前端）

> **现状（2026-09-11）**：帧缓存与渲染实现已随 `CideFlutter/` 迁出（历史资产，已迁出）。**后端已交付的对应契约**是 [`docs/spec/STEP_PAYLOAD_SCHEMA_V0_1.md`](../spec/STEP_PAYLOAD_SCHEMA_V0_1.md) §4 的 **frameCache 窗口语义**：`payload.get(start, end)`、越窗 seek 的懒重算、以及"seek 到第 N 步后各视图均以第 N 步快照为准"的一致性要求——消费方据此重绘，不需要自行回退状态。下例 Dart 为历史设计稿：

```dart
// 历史设计稿（已迁出）
class VisFrameBuffer {
  final List<VisState> _history = [];
  static const int maxHistory = 5;
  
  void push(VisState state) {
    _history.add(state);
    if (_history.length > maxHistory) _history.removeAt(0);
  }
  
  VisState? get previous => _history.length >= 2 ? _history[_history.length - 2] : null;
  VisState? get current => _history.isNotEmpty ? _history.last : null;
}
```

布局算法由消费方实现。⚠️ 设计稿中提到的 `VIS_LAYOUT_STANDARD.md` **在本仓库中不存在**（全仓无此文件），跨平台布局标准尚无正式文档——如实记为缺口。

---

## 8. 各数据结构可视化方案（渲染属消费方；后端提供数据）

| 结构 | 渲染（消费方） | 后端需提供的载荷 |
|:---|:---|:---|
| **数组排序** | 柱状图 + 比较/交换动画 | `ArraySnapshot`（值序列 + 高亮）、`VisEvent`（compare/swap）、`AlgorithmStepSnapshot` |
| **链表** | 水平节点 + 箭头动画 | 节点遍历结果（地址/值/next）、`PointerSnapshot` 四状态（Valid/Freed/Null/Dangling） |
| **二叉树** | 递归布局 + 颜色/高度元数据 | 节点遍历结果（地址/值/left/right）+ 颜色/高度字段偏移 |

> 上表"后端需提供的载荷"中，`ArraySnapshot` / `VisEvent` / `PointerSnapshot` **已落地**（见 `docs/spec/STEP_PAYLOAD_SCHEMA_V0_1.md`）；链表/树节点遍历结果依赖 §4 的检测器，**尚未落地**（缺口）。

---

## 9. 失败兜底与用户体验

| 场景 | 用户看到的提示 |
|:---|:---|
| 无 struct 定义 | "未能识别。尝试手动选择类型。" |
| 字段命名非标准 | "置信度较低。请手动配置字段映射。" |
| 指针混乱/循环引用 | "遍历异常：检测到循环引用。请检查指针赋值。" |
| 单步无变化 | "当前步骤无结构变化。尝试继续执行。" |
| 多数据结构并存 | "检测到多个数据结构，请选择查看对象。" |

---

## 10. 跨平台移植策略

| 组件 | 复用策略 |
|:---|:---|
| CideVM + 编译器 | **直接复用**（Rust core，经三出口暴露） |
| 数据结构检测器 | **直接复用**（⚠️ 尚未实现，见 §4） |
| 运行时反推引擎 | **直接复用**（已落地部分：StepPayload 反推，见 `docs/spec/STEP_PAYLOAD_SCHEMA_V0_1.md`） |
| VisResult/VisState | **载荷对齐**（以 StepPayload schema 为准，而非 FRB Dart 类型——FRB 桥接为历史资产，已迁出） |
| 布局算法 | **消费方独立实现**（⚠️ `VIS_LAYOUT_STANDARD.md` 不存在，暂无标准文档） |
| 渲染管线 | **消费方独立实现** |
| 交互组件 | **消费方独立实现** |

---

## 11. 实施路线图

> **现状（2026-09-11 核实）**：Phase 0/1 **未实施**（`data_structure_detector.rs` 不存在）；Phase 2 中仅"按需反推"的**载荷侧**部分落地（StepPayload 系列），其 FRB 与 Dart 部分已随切割作废；Phase 3/5 的前端实现已迁出（历史资产，已迁出）。下列清单保留原始规划项，未完成项一律保持未勾选，不以"前端已实现"冒充后端已完成。

### Phase 0：数据模型（1 周 · ⏳ 未实施）
- [ ] `DataStructureMatch` / `FieldOffsets` / `VisResult` -> 语言中立 Rust 层（原计划落 `session.rs`；`native/src/session.rs` 仍存在，但这些类型**尚未定义**）
- [ ] ~~FRB 重新生成 Dart 类型~~（作废：FRB 已随前端切割迁出；改为经三出口交付 JSON 载荷）
- [ ] 创建 `data_structure_detector.rs`（**不存在**）

### Phase 1：检测器（1.5 周 · ⏳ 未实施）
- [ ] `StructProfile` / `FieldProfile` 类型拓扑分析
- [ ] `match_struct_to_ds_kind`（链表/二叉树）
- [ ] `infer_field_offsets`（字段名启发式 + 顺序兜底）
- [ ] `find_root_variables`（全局/参数/局部变量）
- [ ] `detect_array_based_structures`（栈/队列/堆）
- [ ] 单元测试：10-20 个典型样例

### Phase 2：反推与交互（1.5 周）
- [ ] `cide_get_vis_state()` 按需反推 API（未实现；现有等价入口为 capi 的 `cide_step_next_json` / `cide_get_step_payloads_json` + `cide_cli serve` 同构帧）
- [ ] 链表/树 VM 内存扫描器（未实现）
- [ ] `VisResult` 状态分层返回（未实现；分层所需的置信度/多候选字段未落地）
- [ ] ~~Flutter：`DetectionHintBar` / `ManualConfigPanel`~~（**移交社区前端**；后端只提供载荷）
- [ ] ~~配置持久化到 `SharedPreferences`~~（**移交社区前端**）

### Phase 3：数组排序动画 MVP（✅ 当时前端已实现 · 现已迁出）
- [x] `ArrayVisualizer` 柱状图 + 比较/交换动画（`widgets/array_visualizer.dart`，历史资产，已迁出）
- [x] 冒泡/选择/插入/快排/归并/二分排序动画 + 算法检测信息条（历史资产，已迁出）
- [x] VisEvent 比较事件高亮对应条形（琥珀色 + 发光阴影）（历史资产，已迁出）
- 后端侧对应能力（`ArraySnapshot` / `VisEvent`）✅ 已落地，见 `docs/spec/STEP_PAYLOAD_SCHEMA_V0_1.md`

### Phase 4：链表与二叉树（2 周 · 未实施）
- [ ] `LinkedListVisualizer` 创建/删除/重连动画（渲染移交社区前端；后端节点遍历载荷未落地）
- [ ] `TreeVisualizer` 递归布局 + 遍历高亮（同上）

### Phase 5：快排/归并/二分（✅ 已合并至 Phase 3 · 前端实现已迁出）
- [x] 快排（pivot 高亮 + partition 区间）（历史资产，已迁出）
- [x] 归并（双数组 + merge 区间）（历史资产，已迁出）
- [x] 二分查找（搜索区间收缩）（历史资产，已迁出）

### Phase 6：体验打磨（1 周 · 移交社区前端）
- [ ] 动画速度调节（0ms ~ 500ms）
- [ ] 失败场景提示语全覆盖
- [ ] 暗色/亮色主题适配
- [ ] 端到端测试

**总计：约 9 周，可分阶段交付。**（原估算不含 2026-09-11 前端切割带来的"后端检测器从零补齐 + 载荷扩展"工作量）
