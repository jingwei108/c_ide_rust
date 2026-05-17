# 自动数据结构可视化：检测 → 确认 → 动画 详细实施计划

> 目标：学生写完 C 代码后，点击"动画"按钮，程序自动检测代码中的数据结构，弹出确认对话框（学生仅需勾选/点选数据结构名称），确认后自动生成平滑动画。
>
> 约束：学生**不能**配置字段映射、不能选择布局算法、不能修改任何可视化参数。

---

## 1. 系统总览

### 1.1 完整数据流

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  阶段一：编译时静态检测（Rust 后端）                                          │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  C 源码 → Lexer → Parser → AST → TypeChecker                       │    │
│  │                            ↓                                       │    │
│  │  ┌─────────────────────────────────────────────────────────────┐   │    │
│  │  │ DataStructureDetector                                       │   │    │
│  │  │ • 扫描 struct 定义，做类型拓扑分析                           │   │    │
│  │  │ • 扫描全局/局部变量，找根指针候选                            │   │    │
│  │  │ • 扫描函数体，识别操作模式（push/pop/sift/rotate）           │   │    │
│  │  │ • 输出 Vec<DataStructureMatch>（含置信度、字段映射）        │   │    │
│  │  └─────────────────────────────────────────────────────────────┘   │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    ↓                                        │
│  阶段二：用户确认（Flutter 前端）                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  弹出对话框：                                                        │    │
│  │  "程序为你找到了以下数据结构，请选择要可视化的："                     │    │
│  │  ◉ root —— 二叉树 (置信度 96%)                                      │    │
│  │  ○ arr  —— 数组/排序 (置信度 99%)                                   │    │
│  │  ○ s    —— 栈 (置信度 78%)                                          │    │
│  │                                                                     │    │
│  │  [取消]                    [预览]  [确认并开始动画]                 │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    ↓ 用户确认后                              │
│  阶段三：运行时帧捕获（Rust VM）                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  VM 执行时，根据已确认的 DataStructureConfig：                       │    │
│  │  • 在指针存储指令（OP_STORE_PTR）处追踪链接变化                      │    │
│  │  • 在数组索引访问处追踪元素值变化                                    │    │
│  │  • 在特定函数（rotate/sift/push）入口处标记语义事件                  │    │
│  │  • 每帧输出完整的结构状态（节点、边、值、高亮）                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    ↓                                        │
│  阶段四：前端动画播放（Flutter）                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  VisFramePlayer：                                                    │    │
│  │  • 接收 Vec<VisFrame>                                                │    │
│  │  • 用 AnimationController 做 300ms ease-out 帧间插值                 │    │
│  │  • CustomPainter 绘制树/数组/链表/图                                 │    │
│  │  • 播放控制：播放 / 暂停 / 步进 / 拖动进度条                          │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 新增文件清单

**Rust 后端 (`native/src/`)**：
- `compiler/data_structure_detector.rs` — 静态数据结构检测器
- `vis/frame.rs` — 帧数据结构与帧生成器
- `vis/tree_layout.rs` — Reingold-Tilford 树布局（Rust 端预计算坐标）
- `vis/vm_tracer.rs` — VM 运行时结构追踪器

**Flutter 前端 (`CideFlutter/lib/`)**：
- `models/vis_frame.dart` — 帧数据模型
- `models/data_structure_match.dart` — 检测结果模型
- `widgets/vis/vis_frame_player.dart` — 通用帧播放器
- `widgets/vis/tree_animator.dart` — 树动画 CustomPainter
- `widgets/vis/array_animator.dart` — 数组动画 CustomPainter
- `widgets/vis/graph_animator.dart` — 图动画 CustomPainter
- `widgets/vis/confirm_dialog.dart` — 确认对话框
- `providers/vis_notifier.dart` — 可视化状态管理

---

## 2. 核心数据结构定义

### 2.1 Rust 后端：检测结果与配置

```rust
// native/src/session.rs（追加）

/// 检测到的数据结构类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DsKind {
    Array,           // 数组（含排序动画）
    LinkedList,      // 链表
    BinaryTree,      // 普通二叉树
    BinarySearchTree,// 二叉搜索树
    AvlTree,         // AVL 树
    RedBlackTree,    // 红黑树
    Heap,            // 堆
    Stack,           // 栈
    Queue,           // 队列
    GraphMatrix,     // 邻接矩阵图
    GraphAdjList,    // 邻接表图
    UnionFind,       // 并查集
    Trie,            // Trie 树
    BTree,           // B/B+ 树
    GenericGraph,    // 通用指针图（兜底）
}

/// 字段映射信息（程序自动推断，用户不可见不可改）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FieldMapping {
    pub value_field: Option<String>,      // 值字段名（如 "val", "data", "key"）
    pub left_field: Option<String>,       // 左子树/下一个
    pub right_field: Option<String>,     // 右子树
    pub parent_field: Option<String>,    // 父节点（红黑树/AVL）
    pub color_field: Option<String>,     // 颜色字段（红黑树）
    pub height_field: Option<String>,    // 高度字段（AVL）
    pub next_field: Option<String>,      // 链表 next
    pub child_array_field: Option<String>, // Trie/B树的 children[]
    pub child_count_field: Option<String>, // B树的 key_count
    pub top_field: Option<String>,       // 栈 top 变量名
    pub front_field: Option<String>,     // 队列 front
    pub rear_field: Option<String>,      // 队列 rear
    pub size_field: Option<String>,      // 数组有效长度
}

/// 数据结构检测结果
#[frb]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DataStructureMatch {
    pub ds_kind: String,           // DsKind 的字符串表示
    pub display_name: String,      // "二叉树", "链表"
    pub confidence: i32,           // 0-100
    pub struct_name: Option<String>, // 关联的 struct 名（如 "TreeNode"）
    pub root_var_name: String,     // 根指针/数组变量名
    pub root_var_addr: u32,        // 根变量在 VM 内存中的地址（全局区/栈区）
    pub field_mapping: FieldMapping,
    pub suggested_layout: String,  // "hierarchical", "force_directed", "linear", "matrix"
    pub detection_reason: String,  // 检测原因说明（给用户看的简短文本）
}

/// 编译结果追加字段
#[frb]
#[derive(Debug, Clone)]
pub struct CompileResult {
    pub success: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub algorithm_matches: Vec<AlgorithmMatch>,
    pub data_structure_matches: Vec<DataStructureMatch>, // ← 新增
}
```

### 2.2 Rust 后端：可视化帧

```rust
// native/src/vis/frame.rs

/// 单个节点的可视化状态
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VisNode {
    pub id: String,           // 唯一标识（内存地址或数组下标）
    pub label: String,        // 显示文本
    pub x: f32,               // 布局坐标（Rust 端预计算，前端只做插值）
    pub y: f32,
    pub color: String,        // "#RRGGBB"
    pub border_color: Option<String>,
    pub radius: f32,          // 节点大小
    pub opacity: f32,         // 0.0-1.0
    pub meta: HashMap<String, String>, // 额外数据（如红黑树的 color、AVL 的 height）
}

/// 边
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VisEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub color: String,
    pub width: f32,
    pub dashed: bool,
    pub directed: bool,
}

/// 数组元素（特殊节点）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VisArrayElement {
    pub index: i32,
    pub value: i32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: String,
    pub is_highlighted: bool,
    pub label: Option<String>, // 如 "pivot", "i", "j"
}

/// 单帧 = 一个时间 slice 的完整可视化状态
#[frb]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VisFrame {
    pub frame_index: i32,
    pub source_line: i32,              // 对应 C 代码行号
    pub operation_hint: Option<String>, // "swap", "rotate_left", "push", "pop", "sift_up"
    pub nodes: Vec<VisNode>,
    pub edges: Vec<VisEdge>,
    pub array_elements: Vec<VisArrayElement>,
    pub callstack_depth: i32,
}

/// 帧序列（一次执行生成）
#[frb]
#[derive(Debug, Clone)]
pub struct VisFrameSequence {
    pub ds_kind: String,
    pub root_var_name: String,
    pub frames: Vec<VisFrame>,
    pub total_steps: i32,
}
```

### 2.3 Dart 前端：帧数据模型

```dart
// CideFlutter/lib/models/vis_frame.dart

class VisNode {
  final String id;
  final String label;
  final double x;
  final double y;
  final Color color;
  final Color? borderColor;
  final double radius;
  final double opacity;
  final Map<String, String> meta;

  VisNode({...});

  factory VisNode.fromJson(Map<String, dynamic> json) => ...;
}

class VisEdge { ... }

class VisArrayElement { ... }

class VisFrame {
  final int frameIndex;
  final int sourceLine;
  final String? operationHint;
  final List<VisNode> nodes;
  final List<VisEdge> edges;
  final List<VisArrayElement> arrayElements;
  final int callstackDepth;

  VisFrame({...});
  factory VisFrame.fromJson(Map<String, dynamic> json) => ...;
}
```

---

## 3. 静态检测器：`data_structure_detector.rs`

### 3.1 模块定位

文件：`native/src/compiler/data_structure_detector.rs`
入口：`detect_data_structures(program: &ProgramNode) -> Vec<DataStructureMatch>`
调用时机：`run_compile_pipeline` 成功后，与 `detect_algorithms` 并列调用。

### 3.2 检测算法：类型拓扑分析

```rust
use crate::compiler::ast::*;
use crate::session::{DataStructureMatch, DsKind, FieldMapping};

/// 主入口：扫描整个程序，返回所有检测到的数据结构候选
pub fn detect_data_structures(program: &ProgramNode) -> Vec<DataStructureMatch> {
    let mut matches = Vec::new();

    // Step 1: 扫描所有 struct 定义，做类型拓扑分析
    let struct_profiles: Vec<StructProfile> = program.structs.iter()
        .map(|s| analyze_struct_topology(s))
        .collect();

    // Step 2: 对每个 struct profile，匹配可能的数据结构类型
    for profile in &struct_profiles {
        if let Some(ds_kind) = match_struct_to_ds_kind(profile) {
            // Step 3: 找根变量（该类型的全局/局部指针变量）
            let root_vars = find_root_variables(program, &profile.struct_name);

            for (var_name, confidence) in root_vars {
                let field_mapping = infer_field_mapping(profile, &ds_kind);
                let layout = suggest_layout(&ds_kind);
                let reason = build_reason_text(profile, &ds_kind);

                matches.push(DataStructureMatch {
                    ds_kind: format!("{:?}", ds_kind),
                    display_name: ds_display_name(&ds_kind),
                    confidence,
                    struct_name: Some(profile.struct_name.clone()),
                    root_var_name: var_name,
                    root_var_addr: 0, // 运行时由 VM 填充
                    field_mapping,
                    suggested_layout: layout,
                    detection_reason: reason,
                });
            }
        }
    }

    // Step 4: 检测无 struct 的纯数组结构（栈、队列、堆、排序数组）
    matches.extend(detect_array_based_structures(program));

    // Step 5: 检测邻接矩阵图
    matches.extend(detect_graph_matrix(program));

    // 按置信度降序排列
    matches.sort_by(|a, b| b.confidence.cmp(&a.confidence));
    matches
}
```

### 3.3 核心：Struct 类型拓扑分析

```rust
/// struct 的类型拓扑特征
struct StructProfile {
    struct_name: String,
    fields: Vec<FieldProfile>,
    self_pointer_count: usize,        // 指向自身的指针数量
    scalar_count: usize,              // 基本类型字段数
    int_flag_count: usize,            // int 字段中疑似 flag 的（color/height/bf）
    child_array_count: usize,         // 指向自身的数组字段数（Trie/B树）
}

struct FieldProfile {
    name: String,
    ty: Type,
    is_self_pointer: bool,            // 是否指向 struct 自身的指针
    is_self_array: bool,              // 是否是指向自身的指针数组
    is_scalar: bool,
    is_int: bool,
}

fn analyze_struct_topology(decl: &StructDecl) -> StructProfile {
    let struct_name = decl.name.clone();
    let mut fields = Vec::new();
    let mut self_pointer_count = 0;
    let mut scalar_count = 0;
    let mut int_flag_count = 0;
    let mut child_array_count = 0;

    for field in &decl.fields {
        let is_self_ptr = is_pointer_to_struct(&field.ty, &struct_name);
        let is_self_arr = is_array_of_pointers_to_struct(&field.ty, &struct_name);
        let is_scalar = field.ty.is_scalar();
        let is_int = field.ty.kind == TypeKind::Int;

        if is_self_ptr { self_pointer_count += 1; }
        if is_scalar { scalar_count += 1; }
        if is_int && is_likely_flag_field(&field.name) { int_flag_count += 1; }
        if is_self_arr { child_array_count += 1; }

        fields.push(FieldProfile {
            name: field.name.clone(),
            ty: field.ty.clone(),
            is_self_pointer: is_self_ptr,
            is_self_array: is_self_arr,
            is_scalar,
            is_int,
        });
    }

    StructProfile { struct_name, fields, self_pointer_count, scalar_count, int_flag_count, child_array_count }
}

/// 判断一个类型是否是指向指定 struct 的指针
fn is_pointer_to_struct(ty: &Type, struct_name: &str) -> bool {
    ty.kind == TypeKind::Pointer
        && ty.base_kind == TypeKind::Struct
        && ty.name == struct_name
}

/// 判断一个类型是否是指向指定 struct 的指针数组（如 struct Node* children[26]）
fn is_array_of_pointers_to_struct(ty: &Type, struct_name: &str) -> bool {
    ty.kind == TypeKind::Array
        && ty.base_kind == TypeKind::Pointer
        && ty.name == struct_name
}

/// 字段名启发式：判断是否为 flag 字段（color, height, bf, size, count）
fn is_likely_flag_field(name: &str) -> bool {
    let lower = name.to_lowercase();
    matches!(lower.as_str(),
        "color" | "col" | "red" | "black" |
        "height" | "h" | "bf" | "balance" |
        "size" | "count" | "len" | "is_end"
    )
}
```

### 3.4 核心：Struct → 数据结构类型匹配

```rust
fn match_struct_to_ds_kind(profile: &StructProfile) -> Option<DsKind> {
    match (profile.self_pointer_count, profile.child_array_count, profile.int_flag_count) {
        // 0 个自引用指针
        (0, 0, 0) => None, // 普通结构体，不匹配

        // 1 个自引用指针 → 链表 或 线索二叉树
        (1, 0, 0) => Some(DsKind::LinkedList),
        (1, 0, 1) => {
            // 检查 flag 字段名，如果是 is_end → Trie 的简化节点
            if has_field_named(&profile.fields, "is_end") {
                Some(DsKind::Trie)
            } else {
                Some(DsKind::LinkedList) // 带 size/count 的链表节点
            }
        }

        // 2 个自引用指针 → 二叉树家族
        (2, 0, 0) => Some(DsKind::BinaryTree),
        (2, 0, 1) => {
            if has_field_named(&profile.fields, "color")
                || has_field_named(&profile.fields, "red")
                || has_field_named(&profile.fields, "black") {
                Some(DsKind::RedBlackTree)
            } else if has_field_named(&profile.fields, "height")
                || has_field_named(&profile.fields, "bf") {
                Some(DsKind::AvlTree)
            } else {
                Some(DsKind::BinarySearchTree)
            }
        }
        (2, 0, 2) => {
            // 既有 color 又有 parent → 红黑树（带 parent 指针）
            if has_field_named(&profile.fields, "parent") {
                Some(DsKind::RedBlackTree)
            } else {
                Some(DsKind::BinarySearchTree)
            }
        }

        // 3 个自引用指针（left, right, parent）→ 带 parent 的二叉树
        (3, 0, 0) => Some(DsKind::BinarySearchTree),
        (3, 0, 1) => {
            if has_field_named(&profile.fields, "color") {
                Some(DsKind::RedBlackTree)
            } else {
                Some(DsKind::AvlTree)
            }
        }

        // children[] 数组 → Trie 或 B树
        (0..=1, 1.., 0..=1) => {
            if let Some(child_arr) = profile.fields.iter().find(|f| f.is_self_array) {
                let arr_size = child_arr.ty.array_size;
                if arr_size >= 2 && arr_size <= 128 {
                    Some(DsKind::Trie)
                } else if arr_size >= 3 && arr_size <= 1024 {
                    Some(DsKind::BTree)
                } else {
                    None
                }
            } else {
                None
            }
        }

        _ => None,
    }
}

fn has_field_named(fields: &[FieldProfile], name: &str) -> bool {
    fields.iter().any(|f| f.name.eq_ignore_ascii_case(name))
}
```

### 3.5 字段映射自动推断

```rust
fn infer_field_mapping(profile: &StructProfile, ds_kind: &DsKind) -> FieldMapping {
    let mut mapping = FieldMapping::default();

    // 推断值字段：找第一个非指针的基本类型字段（int/char/float）
    if let Some(f) = profile.fields.iter().find(|f| f.is_scalar && !f.is_self_pointer && !f.is_self_array) {
        mapping.value_field = Some(f.name.clone());
    }

    // 推断自引用指针字段
    let self_ptrs: Vec<&FieldProfile> = profile.fields.iter()
        .filter(|f| f.is_self_pointer)
        .collect();

    match ds_kind {
        DsKind::LinkedList => {
            if let Some(f) = self_ptrs.first() {
                mapping.next_field = Some(f.name.clone());
            }
        }
        DsKind::BinaryTree | DsKind::BinarySearchTree | DsKind::AvlTree | DsKind::RedBlackTree => {
            if self_ptrs.len() >= 2 {
                // 按字段名或字段顺序推断 left/right
                let (left, right) = if self_ptrs[0].name.to_lowercase().contains("right")
                    || self_ptrs[1].name.to_lowercase().contains("left") {
                    (self_ptrs[1].name.clone(), self_ptrs[0].name.clone())
                } else {
                    (self_ptrs[0].name.clone(), self_ptrs[1].name.clone())
                };
                mapping.left_field = Some(left);
                mapping.right_field = Some(right);
            }
            if let Some(f) = self_ptrs.iter().find(|f| f.name.to_lowercase().contains("parent")) {
                mapping.parent_field = Some(f.name.clone());
            }
        }
        DsKind::Trie | DsKind::BTree => {
            if let Some(f) = profile.fields.iter().find(|f| f.is_self_array) {
                mapping.child_array_field = Some(f.name.clone());
            }
            if let Some(f) = profile.fields.iter().find(|f| f.is_int && f.name.to_lowercase().contains("count")) {
                mapping.child_count_field = Some(f.name.clone());
            }
        }
        _ => {}
    }

    // 推断特殊字段（color/height）
    if matches!(ds_kind, DsKind::RedBlackTree) {
        if let Some(f) = profile.fields.iter().find(|f| f.is_int && is_likely_flag_field(&f.name)) {
            mapping.color_field = Some(f.name.clone());
        }
    }
    if matches!(ds_kind, DsKind::AvlTree) {
        if let Some(f) = profile.fields.iter().find(|f| f.is_int && (f.name.to_lowercase().contains("height") || f.name.to_lowercase().contains("bf"))) {
            mapping.height_field = Some(f.name.clone());
        }
    }

    mapping
}
```

### 3.6 根变量推断

```rust
/// 在程序中找指定 struct 类型的指针变量，作为根指针候选
/// 返回 (变量名, 置信度) 列表
fn find_root_variables(program: &ProgramNode, struct_name: &str) -> Vec<(String, i32)> {
    let mut candidates = Vec::new();

    // 1. 全局变量中找
    for global in &program.globals {
        if is_pointer_to_struct_type(&global.ty, struct_name) {
            let name = global.name.clone();
            // 全局变量通常就是根
            candidates.push((name, 95));
        }
    }

    // 2. 函数参数中找（递归函数的第一个参数通常是根）
    for func in &program.funcs {
        for (idx, param) in func.params.iter().enumerate() {
            if is_pointer_to_struct_type(&param.ty, struct_name) {
                let mut conf = 70;
                // 如果是第一个参数 → 更像根
                if idx == 0 { conf += 15; }
                // 如果函数名包含 insert/delete/traverse/search → 更像操作函数
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

    // 3. main 函数局部变量中找
    if let Some(main_func) = program.funcs.iter().find(|f| f.name == "main") {
        if let Some(body) = &main_func.body {
            collect_local_ptr_vars(body, struct_name, &mut candidates);
        }
    }

    // 去重：优先保留高置信度
    let mut seen = std::collections::HashSet::new();
    candidates.retain(|(name, _)| seen.insert(name.clone()));

    candidates
}

fn is_pointer_to_struct_type(ty: &Type, struct_name: &str) -> bool {
    ty.kind == TypeKind::Pointer
        && ty.base_kind == TypeKind::Struct
        && ty.name == struct_name
}

fn collect_local_ptr_vars(stmt: &Stmt, struct_name: &str, out: &mut Vec<(String, i32)>) {
    match stmt {
        Stmt::VarDecl { var_type, name, .. } => {
            if is_pointer_to_struct_type(var_type, struct_name) {
                out.push((name.clone(), 80));
            }
        }
        Stmt::Block { stmts, .. } => {
            for s in stmts { collect_local_ptr_vars(s, struct_name, out); }
        }
        Stmt::If { then_stmt, else_stmt, .. } => {
            collect_local_ptr_vars(then_stmt, struct_name, out);
            if let Some(e) = else_stmt { collect_local_ptr_vars(e, struct_name, out); }
        }
        Stmt::While { body, .. } | Stmt::DoWhile { body, .. } => {
            collect_local_ptr_vars(body, struct_name, out);
        }
        Stmt::For { body, init, .. } => {
            if let Some(i) = init { collect_local_ptr_vars(i, struct_name, out); }
            collect_local_ptr_vars(body, struct_name, out);
        }
        _ => {}
    }
}
```

### 3.7 数组型结构检测（栈、队列、堆）

```rust
fn detect_array_based_structures(program: &ProgramNode) -> Vec<DataStructureMatch> {
    let mut matches = Vec::new();

    // 扫描 main 函数（或全局作用域）中的数组+配套变量模式
    // 栈：数组 + top 变量（只 ++/--，用于数组索引）
    // 队列：数组 + front + rear（% 运算）
    // 堆：数组 + 父子索引计算（i/2, 2i+1, 2i+2）

    for func in &program.funcs {
        if let Some(body) = &func.body {
            let array_vars = find_array_variables(body);
            for (arr_name, arr_ty, arr_size) in array_vars {
                // 找配套变量
                let companions = find_companion_variables(body, &arr_name);

                // 栈模式检测
                if let Some(top_var) = detect_stack_pattern(body, &arr_name, &companions) {
                    matches.push(DataStructureMatch {
                        ds_kind: "Stack".to_string(),
                        display_name: "栈".to_string(),
                        confidence: 82,
                        struct_name: None,
                        root_var_name: arr_name.clone(),
                        root_var_addr: 0,
                        field_mapping: FieldMapping {
                            top_field: Some(top_var),
                            ..Default::default()
                        },
                        suggested_layout: "linear".to_string(),
                        detection_reason: format!("检测到数组 {} 与 top 变量 {} 的 push/pop 模式", arr_name, top_var),
                    });
                }

                // 队列模式检测
                if let Some((front, rear)) = detect_queue_pattern(body, &arr_name, &companions) {
                    matches.push(DataStructureMatch {
                        ds_kind: "Queue".to_string(),
                        display_name: "队列".to_string(),
                        confidence: 80,
                        struct_name: None,
                        root_var_name: arr_name.clone(),
                        root_var_addr: 0,
                        field_mapping: FieldMapping {
                            front_field: Some(front),
                            rear_field: Some(rear),
                            ..Default::default()
                        },
                        suggested_layout: "linear".to_string(),
                        detection_reason: format!("检测到数组 {} 与 front/rear 的循环队列模式", arr_name),
                    });
                }

                // 堆模式检测
                if detect_heap_pattern(body, &arr_name, &companions) {
                    let size_var = companions.iter()
                        .find(|n| n.to_lowercase().contains("size") || n.to_lowercase().contains("n") || n.to_lowercase().contains("count"))
                        .cloned();
                    matches.push(DataStructureMatch {
                        ds_kind: "Heap".to_string(),
                        display_name: "堆".to_string(),
                        confidence: 85,
                        struct_name: None,
                        root_var_name: arr_name.clone(),
                        root_var_addr: 0,
                        field_mapping: FieldMapping {
                            size_field: size_var,
                            ..Default::default()
                        },
                        suggested_layout: "hierarchical".to_string(),
                        detection_reason: format!("检测到数组 {} 的父子索引计算（i/2, 2i+1, 2i+2）", arr_name),
                    });
                }
            }
        }
    }

    matches
}

/// 检测堆模式：代码中是否有 arr[i/2], arr[2*i+1], arr[2*i+2] 的索引计算
fn detect_heap_pattern(body: &Stmt, arr_name: &str, _companions: &[String]) -> bool {
    let mut has_parent_idx = false;
    let mut has_left_child_idx = false;
    let mut has_right_child_idx = false;

    walk_expr_in_stmt(body, |expr| {
        if let Expr::Index { array, index, .. } = expr {
            if let Expr::Identifier { name, .. } = array.as_ref() {
                if name == arr_name {
                    let idx_str = expr_to_string(index);
                    let lower = idx_str.to_lowercase();
                    if lower.contains("/2") || lower.contains(">>1") {
                        has_parent_idx = true;
                    }
                    if lower.contains("*2+1") || lower.contains("<<1+1") || lower.contains("*2") {
                        has_left_child_idx = true;
                    }
                    if lower.contains("*2+2") || lower.contains("<<1+2") {
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

## 4. 运行时帧捕获系统

### 4.1 设计原则

学生确认数据结构后，编译管线需要：
1. 将 `DataStructureMatch` + `FieldMapping` 序列化存入 `Session`
2. VM 执行时，根据配置自动追踪内存变化
3. 每执行到关键行或关键操作，生成一帧 `VisFrame`

### 4.2 Session 扩展

```rust
// native/src/session.rs（追加到 Session 结构）

#[derive(Debug, Clone, Default)]
pub struct VisConfig {
    pub enabled: bool,
    pub ds_match: Option<DataStructureMatch>, // 用户确认的数据结构
    pub struct_size: i32,                     // struct 字节大小（用于地址计算）
    pub field_offsets: HashMap<String, i32>,  // 字段名 → 字节偏移
}

pub struct Session {
    pub compile: CompileState,
    pub runtime: RuntimeState,
    pub memory: MemoryState,
    pub vm: Option<CideVM>,
    pub vis_config: VisConfig,        // ← 新增
    pub vis_frames: Vec<VisFrame>,    // ← 新增：执行时填充
}
```

### 4.3 VM 运行时追踪：`vm_tracer.rs`

```rust
// native/src/vis/vm_tracer.rs

use crate::session::{VisConfig, VisFrame, VisNode, VisEdge, VisArrayElement, FieldMapping};
use crate::vm::vm::CideVM;
use std::collections::{HashMap, HashSet};

/// VM 执行单步后调用，检查是否需要生成新帧
pub fn maybe_capture_frame(
    vm: &CideVM,
    config: &VisConfig,
    current_line: i32,
    prev_frames: &[VisFrame],
) -> Option<VisFrame> {
    if !config.enabled { return None; }
    let ds_match = config.ds_match.as_ref()?;

    match ds_match.ds_kind.as_str() {
        "BinaryTree" | "BinarySearchTree" | "AvlTree" | "RedBlackTree" => {
            capture_tree_frame(vm, config, current_line, prev_frames)
        }
        "LinkedList" => {
            capture_linked_list_frame(vm, config, current_line, prev_frames)
        }
        "Array" | "Stack" | "Queue" | "Heap" => {
            capture_array_frame(vm, config, current_line, prev_frames)
        }
        "GraphMatrix" | "GraphAdjList" | "GenericGraph" => {
            capture_graph_frame(vm, config, current_line, prev_frames)
        }
        _ => None,
    }
}

/// 捕获二叉树一帧
fn capture_tree_frame(
    vm: &CideVM,
    config: &VisConfig,
    line: i32,
    _prev: &[VisFrame],
) -> Option<VisFrame> {
    let mapping = &config.ds_match.as_ref()?.field_mapping;
    let root_addr = get_root_address(vm, config)?;

    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = vec![(root_addr, 0i32)]; // (addr, depth)

    let left_off = config.field_offsets.get(mapping.left_field.as_ref()?)?;
    let right_off = config.field_offsets.get(mapping.right_field.as_ref()?)?;
    let val_off = mapping.value_field.as_ref()
        .and_then(|n| config.field_offsets.get(n))
        .copied()
        .unwrap_or(0);

    while let Some((addr, depth)) = queue.pop() {
        if addr == 0 || !visited.insert(addr) { continue; }

        let val = vm.read_i32(addr + val_off as u32).unwrap_or(0);
        let left = vm.read_i32(addr + *left_off as u32).unwrap_or(0) as u32;
        let right = vm.read_i32(addr + *right_off as u32).unwrap_or(0) as u32;

        // color / height 元数据（红黑树/AVL）
        let mut meta = HashMap::new();
        if let Some(color_field) = &mapping.color_field {
            if let Some(off) = config.field_offsets.get(color_field) {
                let color_val = vm.read_i32(addr + *off as u32).unwrap_or(0);
                meta.insert("color".to_string(), color_val.to_string());
            }
        }
        if let Some(h_field) = &mapping.height_field {
            if let Some(off) = config.field_offsets.get(h_field) {
                let h = vm.read_i32(addr + *off as u32).unwrap_or(0);
                meta.insert("height".to_string(), h.to_string());
            }
        }

        nodes.push(VisNode {
            id: format!("{}", addr),
            label: val.to_string(),
            x: 0.0, // 布局在 Rust 端或 Flutter 端计算
            y: depth as f32 * 80.0,
            color: if meta.get("color") == Some(&"1".to_string()) { "#FF453A".to_string() }
                   else if meta.get("color") == Some(&"0".to_string()) { "#000000".to_string() }
                   else { "#0A84FF".to_string() },
            border_color: None,
            radius: 24.0,
            opacity: 1.0,
            meta,
        });

        if left != 0 {
            edges.push(VisEdge {
                from: format!("{}", addr),
                to: format!("{}", left),
                label: None,
                color: "#8E8E93".to_string(),
                width: 2.0,
                dashed: false,
                directed: true,
            });
            queue.push((left, depth + 1));
        }
        if right != 0 {
            edges.push(VisEdge {
                from: format!("{}", addr),
                to: format!("{}", right),
                label: None,
                color: "#8E8E93".to_string(),
                width: 2.0,
                dashed: false,
                directed: true,
            });
            queue.push((right, depth + 1));
        }
    }

    // 用 Reingold-Tilford 算法计算 x 坐标
    layout_tree_reingold_tilford(&mut nodes, &edges);

    Some(VisFrame {
        frame_index: _prev.len() as i32,
        source_line: line,
        operation_hint: infer_tree_operation(vm, line, config),
        nodes,
        edges,
        array_elements: Vec::new(),
        callstack_depth: vm.get_call_stack().len() as i32,
    })
}

/// 获取根变量当前指向的内存地址
fn get_root_address(vm: &CideVM, config: &VisConfig) -> Option<u32> {
    let root_name = &config.ds_match.as_ref()?.root_var_name;
    let symbol = vm.get_symbols().iter().find(|s| s.name == *root_name)?;
    // 符号的 addr 是变量自身的地址，需要读取其值作为指针
    if symbol.ty.is_pointer() {
        Some(vm.read_i32(symbol.addr).unwrap_or(0) as u32)
    } else if symbol.ty.is_array() {
        Some(symbol.addr) // 数组名即首地址
    } else {
        None
    }
}
```

### 4.4 VM 指令级 Hook 点

在 `native/src/vm/vm.rs` 的 `step` 函数中，于关键指令后插入帧捕获调用：

```rust
// 在 vm.rs 的 step() 中，于每步执行后追加：

// 原有代码...
// self.current_line = line;

// 新增：可视化帧捕获
if let Some(ref config) = session.vis_config {
    if config.enabled {
        // 策略：只在以下情况捕获帧，避免帧数爆炸
        // 1. 当前行号变化（学生能看到代码在执行）
        // 2. 或内存状态与上一帧相比有结构变化
        let should_capture = if let Some(last_frame) = session.vis_frames.last() {
            last_frame.source_line != self.current_line
        } else {
            true
        };

        if should_capture {
            if let Some(frame) = crate::vis::vm_tracer::maybe_capture_frame(
                self, config, self.current_line, &session.vis_frames
            ) {
                session.vis_frames.push(frame);
            }
        }
    }
}
```

**性能控制**：
- 设置最大帧数限制（如 500 帧），超过后合并相似帧或停止捕获
- 对于数组排序，只在交换/比较行捕获，连续相同的行跳过

---

## 5. 树布局算法（Rust 端）

### 5.1 Reingold-Tilford 简化版

```rust
// native/src/vis/tree_layout.rs

use crate::vis::frame::{VisNode, VisEdge};
use std::collections::HashMap;

/// 简化版 Reingold-Tilford：中序遍历分配 X，深度分配 Y，最后居中
pub fn layout_tree_reingold_tilford(nodes: &mut [VisNode], edges: &[VisEdge]) {
    if nodes.is_empty() { return; }

    // 构建邻接表
    let mut children: HashMap<String, Vec<String>> = HashMap::new();
    for edge in edges {
        children.entry(edge.from.clone()).or_default().push(edge.to.clone());
    }

    // 找根节点（没有入边的）
    let mut has_parent = std::collections::HashSet::new();
    for edge in edges { has_parent.insert(edge.to.clone()); }
    let root_id = nodes.iter().find(|n| !has_parent.contains(&n.id)).map(|n| n.id.clone());
    let root_id = match root_id {
        Some(r) => r,
        None => return, // 无根，可能是环
    };

    // 中序遍历分配 next_x
    let mut next_x = 0.0;
    let x_spacing = 60.0;
    let mut positions: HashMap<String, f32> = HashMap::new();

    fn inorder(
        id: &str,
        children: &HashMap<String, Vec<String>>,
        positions: &mut HashMap<String, f32>,
        next_x: &mut f32,
        x_spacing: f32,
    ) {
        let kids = children.get(id).cloned().unwrap_or_default();
        if kids.len() >= 2 {
            inorder(&kids[0], children, positions, next_x, x_spacing);
        }
        positions.insert(id.to_string(), *next_x);
        *next_x += x_spacing;
        if kids.len() >= 2 {
            inorder(&kids[1], children, positions, next_x, x_spacing);
        } else if kids.len() == 1 {
            inorder(&kids[0], children, positions, next_x, x_spacing);
        }
    }

    inorder(&root_id, &children, &mut positions, &mut next_x, x_spacing);

    // 居中偏移
    let min_x = positions.values().cloned().fold(f32::INFINITY, f32::min);
    let max_x = positions.values().cloned().fold(f32::NEG_INFINITY, f32::max);
    let center_offset = (min_x + max_x) / 2.0;

    for node in nodes.iter_mut() {
        if let Some(&x) = positions.get(&node.id) {
            node.x = x - center_offset;
        }
    }
}
```

---

## 6. FRB API 扩展

### 6.1 新增 API（`native/src/api/cide.rs`）

```rust
#[frb]
pub fn get_data_structure_matches() -> Vec<DataStructureMatch> {
    let session = crate::flutter_bridge::SESSION.lock().unwrap_or_else(|e| e.into_inner());
    session.compile.data_structure_matches.clone()
}

#[frb]
pub fn set_visualization_target(match_index: i32) -> bool {
    let mut session = crate::flutter_bridge::SESSION.lock().unwrap_or_else(|e| e.into_inner());
    let idx = match_index as usize;
    if idx >= session.compile.data_structure_matches.len() {
        return false;
    }
    let ds_match = session.compile.data_structure_matches[idx].clone();

    // 计算 struct 字段偏移（从编译期的 struct_fields 获取）
    let field_offsets = if let Some(ref sname) = ds_match.struct_name {
        session.compile.struct_fields.get(sname)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .collect::<HashMap<String, i32>>()
    } else {
        HashMap::new()
    };

    // 估算 struct 大小（简单累加字段大小）
    let struct_size = field_offsets.len() as i32 * 4; // 简化：假设都是 4 字节对齐

    session.vis_config = crate::session::VisConfig {
        enabled: true,
        ds_match: Some(ds_match),
        struct_size,
        field_offsets,
    };
    true
}

#[frb]
pub fn get_vis_frames() -> Vec<VisFrame> {
    let session = crate::flutter_bridge::SESSION.lock().unwrap_or_else(|e| e.into_inner());
    session.vis_frames.clone()
}

#[frb]
pub fn clear_vis_frames() {
    let mut session = crate::flutter_bridge::SESSION.lock().unwrap_or_else(|e| e.into_inner());
    session.vis_frames.clear();
}
```

---

## 7. Flutter 前端设计

### 7.1 确认对话框

```dart
// CideFlutter/lib/widgets/vis/confirm_dialog.dart

class DataStructureConfirmDialog extends StatelessWidget {
  final List<DataStructureMatch> matches;

  const DataStructureConfirmDialog({super.key, required this.matches});

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('🎯 选择要可视化的数据结构'),
      content: SizedBox(
        width: 400,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('程序自动检测到以下内容，请选择：',
                style: TextStyle(fontSize: 14, color: Colors.grey)),
            const SizedBox(height: 12),
            ...matches.asMap().entries.map((entry) {
              final i = entry.key;
              final m = entry.value;
              return _MatchCard(
                index: i,
                match: m,
                onTap: () => Navigator.of(context).pop(i),
              );
            }),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(-1),
          child: const Text('取消'),
        ),
      ],
    );
  }
}

class _MatchCard extends StatelessWidget {
  final int index;
  final DataStructureMatch match;
  final VoidCallback onTap;

  const _MatchCard({required this.index, required this.match, required this.onTap});

  @override
  Widget build(BuildContext context) {
    final confidenceColor = match.confidence >= 90
        ? Colors.green
        : match.confidence >= 70
            ? Colors.orange
            : Colors.red;

    return Card(
      margin: const EdgeInsets.only(bottom: 8),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(8),
        child: Padding(
          padding: const EdgeInsets.all(12),
          child: Row(
            children: [
              Icon(
                _iconForDs(match.dsKind),
                color: Theme.of(context).primaryColor,
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      '${match.displayName} — ${match.rootVarName}',
                      style: const TextStyle(fontWeight: FontWeight.bold),
                    ),
                    const SizedBox(height: 2),
                    Text(
                      match.detectionReason,
                      style: const TextStyle(fontSize: 12, color: Colors.grey),
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ],
                ),
              ),
              Chip(
                label: Text('${match.confidence}%'),
                backgroundColor: confidenceColor.withOpacity(0.1),
                labelStyle: TextStyle(color: confidenceColor, fontSize: 12),
              ),
            ],
          ),
        ),
      ),
    );
  }

  IconData _iconForDs(String dsKind) {
    switch (dsKind) {
      case 'BinaryTree':
      case 'BinarySearchTree':
      case 'AvlTree':
      case 'RedBlackTree':
        return Icons.account_tree;
      case 'LinkedList':
        return Icons.linear_scale;
      case 'Array':
      case 'Stack':
      case 'Queue':
        return Icons.view_column;
      case 'Heap':
        return Icons.filter_list;
      case 'GraphMatrix':
      case 'GraphAdjList':
      case 'GenericGraph':
        return Icons.hub;
      default:
        return Icons.memory;
    }
  }
}
```

### 7.2 通用帧播放器

```dart
// CideFlutter/lib/widgets/vis/vis_frame_player.dart

class VisFramePlayer extends StatefulWidget {
  final List<VisFrame> frames;
  final String dsKind;

  const VisFramePlayer({
    super.key,
    required this.frames,
    required this.dsKind,
  });

  @override
  State<VisFramePlayer> createState() => _VisFramePlayerState();
}

class _VisFramePlayerState extends State<VisFramePlayer>
    with TickerProviderStateMixin {
  late AnimationController _animController;
  int _currentFrameIndex = 0;
  bool _isPlaying = false;

  // 当前渲染状态（用于插值）
  List<VisNode> _currentNodes = [];
  List<VisEdge> _currentEdges = [];
  List<VisArrayElement> _currentArray = [];

  @override
  void initState() {
    super.initState();
    _animController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 300),
    );
    _animController.addListener(_onAnimationTick);

    if (widget.frames.isNotEmpty) {
      _applyFrameInstantly(0);
    }
  }

  void _applyFrameInstantly(int index) {
    final frame = widget.frames[index];
    setState(() {
      _currentNodes = frame.nodes;
      _currentEdges = frame.edges;
      _currentArray = frame.arrayElements;
      _currentFrameIndex = index;
    });
  }

  void _onAnimationTick() {
    final t = Curves.easeOutCubic.transform(_animController.value);
    if (_currentFrameIndex + 1 >= widget.frames.length) return;

    final prev = widget.frames[_currentFrameIndex];
    final next = widget.frames[_currentFrameIndex + 1];

    setState(() {
      _currentNodes = _interpolateNodes(prev.nodes, next.nodes, t);
      _currentArray = _interpolateArray(prev.arrayElements, next.arrayElements, t);
      // 边不插值，直接切
      _currentEdges = next.edges;
    });
  }

  void _gotoFrame(int index) {
    if (index < 0 || index >= widget.frames.length) return;

    if (index == _currentFrameIndex + 1) {
      // 相邻帧：启动插值动画
      _animController.forward(from: 0.0).whenComplete(() {
        _currentFrameIndex = index;
        _applyFrameInstantly(index);
      });
    } else {
      // 跳帧：直接切换
      _applyFrameInstantly(index);
      _currentFrameIndex = index;
    }
  }

  void _play() {
    if (_isPlaying) return;
    setState(() => _isPlaying = true);
    _playNext();
  }

  void _playNext() {
    if (!_isPlaying || _currentFrameIndex >= widget.frames.length - 1) {
      setState(() => _isPlaying = false);
      return;
    }
    _gotoFrame(_currentFrameIndex + 1);
    Future.delayed(const Duration(milliseconds: 350), _playNext);
  }

  void _pause() => setState(() => _isPlaying = false);

  void _stepForward() => _gotoFrame(_currentFrameIndex + 1);
  void _stepBackward() => _gotoFrame(_currentFrameIndex - 1);

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        // 工具栏
        _buildToolbar(),
        // 画布
        Expanded(
          child: CustomPaint(
            size: Size.infinite,
            painter: _DsPainter(
              dsKind: widget.dsKind,
              nodes: _currentNodes,
              edges: _currentEdges,
              arrayElements: _currentArray,
              isDark: Theme.of(context).brightness == Brightness.dark,
            ),
          ),
        ),
        // 进度条
        Slider(
          value: _currentFrameIndex.toDouble(),
          max: (widget.frames.length - 1).toDouble(),
          onChanged: (v) => _gotoFrame(v.round()),
        ),
        // 当前操作提示
        if (widget.frames.isNotEmpty && widget.frames[_currentFrameIndex].operationHint != null)
          Text(
            '操作: ${widget.frames[_currentFrameIndex].operationHint}',
            style: const TextStyle(fontSize: 12, color: Colors.grey),
          ),
      ],
    );
  }

  Widget _buildToolbar() {
    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        IconButton(icon: const Icon(Icons.skip_previous), onPressed: _stepBackward),
        IconButton(
          icon: Icon(_isPlaying ? Icons.pause : Icons.play_arrow),
          onPressed: _isPlaying ? _pause : _play,
        ),
        IconButton(icon: const Icon(Icons.skip_next), onPressed: _stepForward),
        Text('${_currentFrameIndex + 1} / ${widget.frames.length}'),
      ],
    );
  }

  // 节点位置插值
  List<VisNode> _interpolateNodes(List<VisNode> prev, List<VisNode> next, double t) {
    final Map<String, VisNode> prevMap = {for (var n in prev) n.id: n};
    return next.map((n) {
      final p = prevMap[n.id];
      if (p == null) {
        // 新节点：从上方飞入
        return n.copyWith(
          y: n.y - 50.0 * (1.0 - t),
          opacity: t,
        );
      }
      return n.copyWith(
        x: p.x + (n.x - p.x) * t,
        y: p.y + (n.y - p.y) * t,
        radius: p.radius + (n.radius - p.radius) * t,
        opacity: p.opacity + (n.opacity - p.opacity) * t,
      );
    }).toList();
  }

  List<VisArrayElement> _interpolateArray(
    List<VisArrayElement> prev,
    List<VisArrayElement> next,
    double t,
  ) {
    final Map<int, VisArrayElement> prevMap = {for (var e in prev) e.index: e};
    return next.map((e) {
      final p = prevMap[e.index];
      if (p == null) return e;
      return e.copyWith(
        x: p.x + (e.x - p.x) * t,
        y: p.y + (e.y - p.y) * t,
        height: p.height + (e.height - p.height) * t,
      );
    }).toList();
  }

  @override
  void dispose() {
    _animController.dispose();
    super.dispose();
  }
}
```

### 7.3 树形绘制器

```dart
// CideFlutter/lib/widgets/vis/tree_animator.dart
// 集成到 _DsPainter 中

class _DsPainter extends CustomPainter {
  final String dsKind;
  final List<VisNode> nodes;
  final List<VisEdge> edges;
  final List<VisArrayElement> arrayElements;
  final bool isDark;

  _DsPainter({...});

  @override
  void paint(Canvas canvas, Size size) {
    final centerX = size.width / 2;
    final centerY = size.height / 2;

    canvas.translate(centerX, 40);

    // 绘制边
    for (final edge in edges) {
      final fromNode = nodes.firstWhereOrNull((n) => n.id == edge.from);
      final toNode = nodes.firstWhereOrNull((n) => n.id == edge.to);
      if (fromNode == null || toNode == null) continue;

      final paint = Paint()
        ..color = _parseColor(edge.color)
        ..strokeWidth = edge.width
        ..style = PaintingStyle.stroke;

      if (edge.dashed) {
        paint.shader = null; // 可用 path effect 实现虚线
      }

      final p1 = Offset(fromNode.x, fromNode.y);
      final p2 = Offset(toNode.x, toNode.y);

      // 贝塞尔曲线边
      final path = Path();
      path.moveTo(p1.dx, p1.dy + fromNode.radius);
      path.cubicTo(
        p1.dx, p1.dy + fromNode.radius + 20,
        p2.dx, p2.dy - toNode.radius - 20,
        p2.dx, p2.dy - toNode.radius,
      );
      canvas.drawPath(path, paint);

      // 箭头
      if (edge.directed) {
        _drawArrow(canvas, p2.dx, p2.dy - toNode.radius, paint);
      }
    }

    // 绘制节点
    for (final node in nodes) {
      final paint = Paint()
        ..color = _parseColor(node.color).withOpacity(node.opacity)
        ..style = PaintingStyle.fill;

      final borderPaint = Paint()
        ..color = node.borderColor != null
            ? _parseColor(node.borderColor!)
            : (isDark ? Colors.white54 : Colors.black54)
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2;

      canvas.drawCircle(Offset(node.x, node.y), node.radius * node.opacity, paint);
      canvas.drawCircle(Offset(node.x, node.y), node.radius * node.opacity, borderPaint);

      // 文字
      final textSpan = TextSpan(
        text: node.label,
        style: TextStyle(
          color: isDark ? Colors.white : Colors.black,
          fontSize: 14,
          fontWeight: FontWeight.bold,
        ),
      );
      final textPainter = TextPainter(
        text: textSpan,
        textDirection: TextDirection.ltr,
      );
      textPainter.layout();
      textPainter.paint(
        canvas,
        Offset(node.x - textPainter.width / 2, node.y - textPainter.height / 2),
      );
    }

    // 绘制数组元素（如果有）
    for (final elem in arrayElements) {
      // ... 柱状图绘制
    }
  }

  void _drawArrow(Canvas canvas, double x, double y, Paint paint) {
    const arrowSize = 8.0;
    final path = Path()
      ..moveTo(x, y)
      ..lineTo(x - arrowSize, y - arrowSize)
      ..lineTo(x + arrowSize, y - arrowSize)
      ..close();
    canvas.drawPath(path, paint..style = PaintingStyle.fill);
  }

  Color _parseColor(String hex) {
    return Color(int.parse(hex.replaceFirst('#', '0xFF')));
  }

  @override
  bool shouldRepaint(covariant _DsPainter oldDelegate) => true;
}
```

---

## 8. 实施路线图

### Phase 1：基础设施（1 周）

1. **创建目录与空文件**
   - `native/src/compiler/data_structure_detector.rs`
   - `native/src/vis/mod.rs`, `frame.rs`, `tree_layout.rs`, `vm_tracer.rs`
   - `CideFlutter/lib/models/vis_frame.dart`, `data_structure_match.dart`
   - `CideFlutter/lib/widgets/vis/confirm_dialog.dart`, `vis_frame_player.dart`, `tree_animator.dart`, `array_animator.dart`
   - `CideFlutter/lib/providers/vis_notifier.dart`

2. **扩展 Session 与 CompileState**
   - 在 `session.rs` 追加 `DsKind`, `FieldMapping`, `DataStructureMatch`, `VisConfig`, `VisFrame` 等类型
   - 在 `CompileState` 追加 `data_structure_matches: Vec<DataStructureMatch>`
   - 在 `Session` 追加 `vis_config: VisConfig`, `vis_frames: Vec<VisFrame>`

3. **扩展 FRB API**
   - 在 `api/cide.rs` 追加 `get_data_structure_matches`, `set_visualization_target`, `get_vis_frames`, `clear_vis_frames`
   - 在 `flutter_bridge.rs` 的 `compile` 中，于 `run_compile_pipeline` 后调用 `detect_data_structures`
   - 在 `flutter_bridge.rs` 的 `run_code` / `step_next` 中，确保 `vis_frames` 随执行填充

### Phase 2：静态检测器（1.5 周）

1. 实现 `analyze_struct_topology` 和 `match_struct_to_ds_kind`
2. 实现 `infer_field_mapping`（类型拓扑自动推断字段映射）
3. 实现 `find_root_variables`（根指针/数组变量推断）
4. 实现 `detect_array_based_structures`（栈、队列、堆的数组模式检测）
5. 实现 `detect_graph_matrix`（邻接矩阵图检测）
6. 编写单元测试：用 10-20 个典型学生代码样例验证检测准确率

### Phase 3：运行时帧捕获（1.5 周）

1. 实现 `vm_tracer.rs` 的 `maybe_capture_frame`
2. 实现 `capture_tree_frame`（二叉树家族）
3. 实现 `capture_linked_list_frame`
4. 实现 `capture_array_frame`（数组/栈/队列/堆）
5. 实现 `capture_graph_frame`
6. 实现 `tree_layout.rs` 的 `layout_tree_reingold_tilford`
7. 在 `vm.rs` 的 `step()` 中插入帧捕获 Hook

### Phase 4：前端确认与动画（1.5 周）

1. 实现 `confirm_dialog.dart`（检测结果卡片列表）
2. 实现 `vis_frame_player.dart`（通用帧播放器 + 播放控制）
3. 实现 `tree_animator.dart`（CustomPainter 树形绘制）
4. 实现 `array_animator.dart`（柱状图 + 交换动画）
5. 实现 `vis_notifier.dart`（Riverpod 状态管理：检测 → 确认 → 帧获取 → 播放）
6. 在 IDE 主界面新增"动画"按钮，绑定确认对话框

### Phase 5：集成与调优（1 周）

1. 端到端测试：学生写二叉树插入 → 点击动画 → 确认"二叉树"→ 播放插入动画
2. 性能调优：帧数控制（最大 500 帧）、内存占用
3. 边界处理：空树、单节点、循环链表（防死循环）、NULL 指针
4. 动画细节：旋转节点的跃起效果、数组交换的平滑滑动、高亮闪烁

---

## 9. 关键设计决策

| 决策点 | 选择 | 理由 |
|--------|------|------|
| 字段映射方式 | **类型拓扑分析**（而非名字匹配） | 学生可能用 `lc`/`rc` 而非 `left`/`right`，类型不会说谎 |
| 根指针推断 | **函数参数位置 + 变量名语义 + 调用模式** | 递归函数第一个参数、被返回赋值的变量最可能是根 |
| 布局计算位置 | **Rust 端预计算坐标**，Flutter 只做插值 | 减少跨语言通信量，布局算法统一在 Rust 端 |
| 帧捕获策略 | **行号变化即捕获**，结构未变时合并 | 保证学生能看到代码执行与动画同步，避免帧数爆炸 |
| 数组结构检测 | **运行时验证索引模式**（i/2, 2i+1） | 纯静态分析无法区分栈/队列/堆，结合 AST 模式 + 运行时索引计算 |
| 失败兜底 | **GenericGraph 模式** | 只要是指针结构，运行时追踪引用关系总能画出图 |

---

## 10. 附录：典型测试用例

### 用例 1：标准二叉搜索树
```c
struct Node { int key; struct Node *left, *right; };
struct Node* insert(struct Node* root, int key) { ... }
int main() { struct Node* root = NULL; root = insert(root, 50); ... }
```
**预期**：检测到 `root —— 二叉树 (置信度 96%)`，字段映射 `left=left, right=right, value=key`

### 用例 2：字段名不标准的红黑树
```c
struct TN { int v; struct TN *lc, *rc, *p; int c; };
```
**预期**：检测到 `root —— 红黑树 (置信度 92%)`，字段映射 `left=lc, right=rc, parent=p, color=c`

### 用例 3：数组模拟的堆
```c
int heap[100], n = 0;
void push(int x) { heap[++n] = x; for (int i = n; i > 1 && heap[i] > heap[i/2]; i /= 2) swap(&heap[i], &heap[i/2]); }
```
**预期**：检测到 `heap —— 堆 (置信度 88%)`，有效长度变量推断为 `n`

### 用例 4：内存池链表（难题）
```c
struct Node { int data, next; } pool[100];
int head = -1, used = 0;
```
**预期**：检测到 `pool —— 链表 (置信度 65%)`，提示"检测到索引式链表，next 为 int 类型"

---

> 本文档为纯设计文档，不修改任何现有代码文件。所有代码片段均为新增模块的参考实现，可直接作为编码起点。
