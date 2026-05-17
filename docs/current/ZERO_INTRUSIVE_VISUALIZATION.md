# Cide 数据结构可视化计划文档

> 版本：2026-05-17  
> 设计原则：**零侵入（不写 vis_*()） + 人机协作（分层确认兜底） + 按需反推（不预录帧）**

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
Flutter 请求当前可视化状态
    ↓
Rust 读取 VM 内存 → 构造 VisState → 返回
    ↓
Flutter 本地缓存 → 插值动画 → 渲染
```

**优势**：无内存爆炸、实时交互、前后端解耦。

---

## 2. 架构总览

```
┌─────────────────────────────────────────────────────────────┐
│                    Flutter 前端（表现+交互层）                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  交互层（Dart）                                      │   │
│  │  - 分层提示条（全自动 / 多选切换 / 手动配置）          │   │
│  │  - 手动配置面板：类型选择 + 字段 offset 映射           │   │
│  └─────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  帧缓存与动画层（Dart）                               │   │
│  │  - VisFrameBuffer：缓存最近 N 个 VisState              │   │
│  │  - AnimationController 驱动帧间插值                   │   │
│  └─────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  布局引擎（Dart）—— 各平台独立实现                     │   │
│  │  - 数组：柱状图坐标分配                                │   │
│  │  - 树：递归宽度计算 / Reingold-Tilford                 │   │
│  │  - 链表：水平线性布局                                  │   │
│  └─────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  渲染组件（Flutter CustomPainter）                    │   │
│  │  - SortingVisualizer（数组排序动画）                   │   │
│  │  - LinkedListVisualizer（链表图）                      │   │
│  │  - TreeVisualizer（二叉树图）                          │   │
│  │  - VariablePanel（变量值面板）                         │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              ↑
                    flutter_rust_bridge v2
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

```rust
// native/src/session.rs

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
#[frb]
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

/// 可视化结果状态（新增：用于前端判断交互层级）
#[frb]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum VisStatus {
    AutoDetected,
    UserConfirmed,
    ManuallyConfigured,
    Failed(String),
}

#[frb]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VisResult {
    pub status: VisStatus,
    pub state: Option<VisState>,
    pub suggestion: Option<String>,
}
```

### 3.2 前端：配置持久化

```dart
// lib/models/vis_config.dart

class VisConfig {
  final String dsKind;
  final String rootVarName;
  final FieldOffsets fieldOffsets;
  final int structSize;
  
  // 配置来源标记
  final ConfigSource source; // autoDetected / userConfirmed / manual
}

enum ConfigSource { autoDetected, userConfirmed, ma
nual }
```

配置持久化到 `SharedPreferences`，用户下次打开同类型代码时直接复用。

---

## 4. 后端：数据结构检测器

文件：`native/src/compiler/data_structure_detector.rs`
入口：`detect_data_structures(program: &ProgramNode) -> Vec<DataStructureMatch>`
调用时机：`run_compile_pipeline` 成功后，与 `detect_algorithms` 并列调用。

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

当 VM 触发 `StepEvent` 时，后端**不预录帧**，而是**按需构造 VisState**。

```rust
// native/src/api/cide.rs

#[frb]
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

## 6. 交互层：分层确认与兜底

### 6.1 编译后：检测结果到交互的映射

```dart
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

## 7. 前端：帧缓存与渲染

```dart
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

布局算法由前端实现，遵循 `VIS_LAYOUT_STANDARD.md`（跨平台标准文档）。

---

## 8. 各数据结构可视化方案

- **数组排序**：`SortingVisualizer` 柱状图 + 比较/交换动画
- **链表**：`LinkedListVisualizer` 水平节点 + 箭头动画
- **二叉树**：`TreeVisualizer` 递归布局 + 颜色/高度元数据

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
| CideVM + 编译器 | **直接复用**（Rust .so） |
| 数据结构检测器 | **直接复用** |
| 运行时反推引擎 | **直接复用** |
| VisResult/VisState | **类型对齐** |
| 布局算法 | **按标准文档重写** |
| 渲染管线 | **独立实现** |
| 交互组件 | **独立实现** |

---

## 11. 实施路线图

### Phase 0：数据模型（1 周）
- [ ] `DataStructureMatch` / `FieldOffsets` / `VisResult` -> `session.rs`
- [ ] FRB 重新生成 Dart 类型
- [ ] 创建 `data_structure_detector.rs`

### Phase 1：检测器（1.5 周）
- [ ] `StructProfile` / `FieldProfile` 类型拓扑分析
- [ ] `match_struct_to_ds_kind`（链表/二叉树）
- [ ] `infer_field_offsets`（字段名启发式 + 顺序兜底）
- [ ] `find_root_variables`（全局/参数/局部变量）
- [ ] `detect_array_based_structures`（栈/队列/堆）
- [ ] 单元测试：10-20 个典型样例

### Phase 2：反推与交互（1.5 周）
- [ ] `cide_get_vis_state()` 按需反推 API
- [ ] 链表/树 VM 内存扫描器
- [ ] `VisResult` 状态分层返回
- [ ] Flutter：`DetectionHintBar` / `ManualConfigPanel`
- [ ] 配置持久化到 `SharedPreferences`

### Phase 3：数组排序动画 MVP（1 周）
- [ ] `SortingVisualizer` 柱状图 + 比较/交换动画
- [ ] 冒泡/选择/插入排序完整动画

### Phase 4：链表与二叉树（2 周）
- [ ] `LinkedListVisualizer` 创建/删除/重连动画
- [ ] `TreeVisualizer` 递归布局 + 遍历高亮

### Phase 5：快排/归并/二分（1 周）
- [ ] 快排（pivot 高亮 + partition 区间）
- [ ] 归并（双数组 + merge 区间）
- [ ] 二分查找（搜索区间收缩）

### Phase 6：体验打磨（1 周）
- [ ] 动画速度调节（0ms ~ 500ms）
- [ ] 失败场景提示语全覆盖
- [ ] 暗色/亮色主题适配
- [ ] 端到端测试

**总计：约 9 周，可分阶段交付。**
