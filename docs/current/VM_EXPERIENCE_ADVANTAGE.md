# Cide 自研 VM 体验优势设计文档

> **核心理念**：自研 VM 的优势不是省内存，而是做通用 IDE/可视化工具做不出来的教学体验。  
> **性能原则**：中端手机 50MB 内存换零延迟交互，完全可接受。拒绝为省 47.5MB 做过度工程化。

---

## 目录

- [1. 执行路径热力图（Execution Heatmap）](#1-执行路径热力图)
- [2. 语义进度条（Semantic Timeline）](#2-语义进度条)
- [3. 变量变化历史（Variable History）](#3-变量变化历史)
- [4. 智能检查点密度（Smart Checkpoints）](#4-智能检查点密度)
- [5. 运行时异常自动回退（Auto-Rollback on Trap）](#5-运行时异常自动回退)
- [6. 变量级高亮（Variable-Level Highlighting）](#6-变量级高亮)
- [7. 技术底座：全量快照 + 三件套元数据](#7-技术底座)
- [8. 与统一模式的协作关系](#8-与统一模式的协作关系)
- [9. 实施优先级](#9-实施优先级)

---

## 1. 执行路径热力图

### 1.1 用户看到什么

代码编辑器左侧行号旁边，出现一列彩色条带，颜色深浅表示该行被执行的次数：

```
  1  ░░░░░░░░░░  #include <stdio.h>          (0 次)
  2  ░░░░░░░░░░  void bubbleSort(int arr[], int n) {
  3  ▓▓▓▓▓▓▓▓▓▓  for (int i = 0; i < n - 1; i++) {      (45 次)
  4  ██████████    for (int j = 0; j < n - i - 1; j++) {   (100 次) 🔥
  5  ████████        if (arr[j] > arr[j + 1]) {            (78 次)
  6  ▓▓▓▓              int temp = arr[j];                   (34 次)
  7  ▓▓▓▓              arr[j] = arr[j + 1];                 (34 次)
  8  ▓▓▓▓              arr[j + 1] = temp;                   (34 次)
  9  ░░░░░░░░░░      }
 10  ░░░░░░░░░░    }
 11  ░░░░░░░░░░  }
```

- **灰色 `░░░░`**：未执行的代码（如注释、未触发的 else 分支）
- **浅蓝 `▓▓▓▓`**：执行次数中等
- **深红 `████`**：执行次数最高的热点代码

鼠标悬停在彩色条带上，弹出 Tooltip：
> `第 4 行：执行 100 次（占总执行步数的 67%）`

### 1.2 教学价值

| 场景 | 学生看到的洞察 |
|:---|:---|
| **冒泡排序** | 内层循环执行了 `n*(n-1)/2` 次，直观理解 O(n²) |
| **二分查找** | 循环体只执行了 `log₂(n)` 次，理解 O(log n) |
| **未覆盖分支** | `else` 分支是灰色的，学生意识到"我的测试用例没覆盖到这里" |
| **死代码** | 某行始终是灰色，学生发现"这行代码永远跑不到" |

### 1.3 技术实现

```rust
// VM 层：step_next 中收集
pub struct ExecutionHeatmap {
    pub line_counts: HashMap<i32, u64>,      // 行号 → 执行次数
    pub line_total_ms: HashMap<i32, u64>,    // 行号 → 总耗时（可选扩展）
}

impl CideVM {
    pub fn step_next(&mut self) {
        let instr = &self.instructions[self.pc];
        let line = instr.loc.line;
        *self.heatmap.line_counts.entry(line).or_insert(0) += 1;
        // ... 执行指令 ...
    }
}
```

```dart
// Flutter 层：re_editor 侧边栏渲染
class HeatmapGutter extends LeafRenderObjectWidget {
  final Map<int, int> lineCounts;   // 来自 Rust
  final int maxCount;               // 用于归一化颜色
  
  @override
  void paint(PaintingContext context, Offset offset) {
    for (final entry in lineCounts.entries) {
      final intensity = entry.value / maxCount;
      final color = Color.lerp(Colors.grey[300], Colors.red[700], intensity)!;
      // 在对应行号位置绘制彩色条带
      context.canvas.drawRect(rect, Paint()..color = color);
      // 绘制执行次数
      context.canvas.drawText('${entry.value}', textOffset, textStyle);
    }
  }
}
```

### 1.4 与统一模式的结合

用户拖动进度条到第 X 步时，热力图可以**动态变化**：
- 只显示"执行到第 X 步为止"的累计次数
- 学生可以看到"随着执行推进，哪些行逐渐变热"

---

## 2. 语义进度条

### 2.1 用户看到什么

不是冷冰冰的"第 247 / 500 步"，而是：

```
[初始化]═══[外层循环 i=0]═══[i=1]═══[i=2]═══[i=3]═══[i=4]═══[完成]
               ↑
            当前位置
```

拖动时，进度条吸附到语义边界，不会停在无聊的 `i++` 上。

### 2.2 技术实现

```rust
pub struct StepMeta {
    pub step_index: i32,
    pub code_line: i32,
    pub func_name: String,
    pub loop_depth: i32,
    pub loop_iters: Vec<i32>,       // [外层第几次, 内层第几次]
    pub is_loop_boundary: bool,     // 是否循环边界（语义检查点）
    pub is_func_call: bool,         // 是否函数调用
    pub is_swap: bool,              // 是否数组交换
}
```

每步自动推断语义标签：
```rust
fn infer_semantic_label(meta: &StepMeta) -> String {
    if meta.is_swap {
        return format!("交换 arr[{}] ↔ arr[{}]", meta.loop_iters[0], meta.loop_iters[0]+1);
    }
    if meta.loop_depth > 0 {
        return format!("循环 {}", meta.loop_iters.iter().map(|i| format!("i{}={}", i, i)).collect::<Vec<_>>().join(", "));
    }
    if meta.is_func_call {
        return format!("调用 {}", meta.func_name);
    }
    format!("第 {} 行", meta.code_line)
}
```

---

## 3. 变量变化历史

### 3.1 用户看到什么

拖动进度条时，悬浮球的"局部变量"面板零延迟更新：

```
┌─ 局部变量 ─────────────┐
│ i      = 3     ▓▓▓░░░ │  ← 变化趋势条
│ j      = 5     ▓▓▓▓▓░ │
│ temp   = 8     ░░░▓░░ │  ← 刚刚被赋值
│ arr[0] = 1     ░░░░░░ │
│ arr[1] = 3     ▓▓░░░░ │  ← 之前交换过
└────────────────────────┘
```

每个变量旁边有一个**迷你趋势图**，显示该变量在执行过程中的变化轨迹。

### 3.2 技术实现

```rust
pub struct VarHistory {
    pub name: String,
    pub ty: Type,
    pub changes: Vec<(i32, String)>,   // (step_index, display_value)
}

// 执行过程中自动收集
fn collect_var_history(vm: &CideVM, step: i32) -> Vec<VarHistory> {
    vm.symbols.iter()
        .filter(|s| s.scope == Scope::Local)
        .map(|s| VarHistory {
            name: s.name.clone(),
            ty: s.ty.clone(),
            changes: vec![(step, format_value(vm.read_var(s)))],
        })
        .collect()
}
```

1000 步 × 20 个变量 × 每个值 20 字节 = 400KB。忽略不计。

---

## 4. 智能检查点密度

### 4.1 原则

不是机械地"每 20 步一个检查点"，而是**代码结构感知**：

| 代码特征 | 检查点密度 | 理由 |
|:---|:---|:---|
| 循环边界（for/while 迭代开始） | **必存** | 学生关心"第几轮循环" |
| 数组元素交换 | **必存** | 排序算法的关键帧 |
| 函数调用/返回 | **必存** | 调用栈变化的关键点 |
| 条件分支（if/else）首次进入 | **存** | 覆盖路径变化 |
| 普通语句（`i++`、`a = b + c`） | **不存** |  boring，重放 5 步内无感知 |

100 步的冒泡排序可能只有 **15 个检查点**，但覆盖了所有"关键时刻"。

### 4.2 用户感知

拖动进度条时，磁铁吸附到最近的语义检查点。学生感觉自己在"关键帧之间跳跃"，不会卡在无聊的赋值语句上。

---

## 5. 运行时异常自动回退

### 5.1 用户看到什么

学生代码写错了：

```c
for (int i = 0; i <= n; i++) {   // 应该是 i < n
    arr[i] = i;                    // 第 7 行：数组越界！
}
```

**通用 IDE**：程序崩溃，终端输出 `Segmentation fault`，学生一脸懵逼。

**Cide**：

```
⚠️ 运行时错误：数组越界
   第 7 行：arr[i] = i;
   
   🔙 自动回退到上一步：
   第 6 行：for (int i = 0; i <= n; i++)
   
   📊 变量状态：
   i = 10, n = 10, arr = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
   
   🔧 智能诊断：
   循环条件 i <= n 导致 i 取到 10，而 arr 大小为 10（有效索引 0~9）。
   建议改为 i < n。
   
   [一键修复]  [查看知识卡片：数组越界]
```

### 5.2 技术实现

```rust
impl CideVM {
    pub fn step_next_safe(&mut self) -> Result<(), TrapInfo> {
        let checkpoint = self.snapshot();  // 执行前保存
        match self.step_next() {
            Ok(_) => Ok(()),
            Err(trap) => {
                self.restore(&checkpoint);  // 自动回退
                Err(trap)
            }
        }
    }
}
```

配合诊断系统：
- 如果 trap 是数组越界 → 检查循环条件是否有 `<=` 应为 `<`
- 如果 trap 是空指针解引用 → 检查 `malloc` 返回值是否被检查
- 如果 trap 是栈溢出 → 检查递归终止条件

---

## 6. 变量级高亮

### 6.1 用户看到什么

回放时，代码编辑器不仅高亮当前行，还**高亮当前正在读写的变量**：

```c
for (int i = 0; i < n; i++) {           // i 边框橙色（正在自增）
    for (int j = 0; j < n - i; j++) {   // j 边框橙色
        if (arr[j] > arr[j + 1]) {      // arr[j] 和 arr[j+1] 底色淡红（参与比较）
            int temp = arr[j];           // temp 边框绿色（新声明）
            arr[j] = arr[j + 1];         // arr[j] 闪烁黄色（被写入）
            arr[j + 1] = temp;           // arr[j+1] 闪烁黄色
        }
    }
}
```

### 6.2 技术实现

编译器已经知道每个标识符的 `SourceLoc`（行、列、长度）。VM 的 `step_next` 可以通过符号表反查当前指令访问了哪些变量。

```rust
pub struct VariableHighlight {
    pub name: String,
    pub line: i32,
    pub column: i32,
    pub length: i32,
    pub highlight_type: HighlightType,  // Read / Write / Declare / Compare
}

// VM 每步输出
pub fn get_variable_highlights(&self) -> Vec<VariableHighlight> {
    // 通过当前指令的符号引用反推高亮信息
}
```

Flutter 侧 `re_editor` 支持在特定文本范围绘制下划线/边框/底色。

---

## 7. 技术底座：全量快照 + 三件套元数据

### 7.1 为什么选择全量快照？

| 方案 | 1000 步内存 | 恢复耗时 | 复杂度 | 结论 |
|:---|:---|:---|:---|:---|
| 全量快照（1MB/20步） | 50MB | 2~3ms | 极低 | ✅ **采用** |
| 差分编码（50KB/检查点） | 2.5MB | 1~2ms | 中 | 省 47.5MB，但用户感知为 0 |
| COW 页表（4KB/步） | 4MB | <1ms | 高 | 过度工程化 |

**50MB 在中端手机上连一个微信小程序都不如。为了省这点内存引入复杂度，是过度工程化。**

### 7.2 三件套元数据

除了全量 VM 快照，每步保存轻量级元数据：

```rust
pub struct StepPayload {
    // ① 动画数据（用于可视化面板）
    pub vis_state: VisState,
    
    // ② 语义元数据（用于进度条）
    pub meta: StepMeta,
    
    // ③ 调试摘要（用于悬浮球零延迟）
    pub debug_summary: DebugSummary,
}

pub struct DebugSummary {
    pub local_vars: Vec<(String, String, Type)>,  // 名、值、类型
    pub call_stack: Vec<FrameInfo>,               // 函数名、返回地址
    pub memory_summary: MemorySummary,            // 栈顶、堆顶、全局区
}
```

每步 `StepPayload` 大小：
- `vis_state`：几百字节到几 KB（取决于数据结构复杂度）
- `meta`：~50 字节
- `debug_summary`：~1KB（20 个局部变量 + 5 层调用栈）

1000 步 ≈ 2~5MB。前端内存完全无压力。

### 7.3 检查点管理

```rust
pub struct CheckpointManager {
    pub checkpoints: Vec<(i32, VMSnapshot)>,  // (step_index, snapshot)
    pub interval: i32,                        // 20 步
}

impl CheckpointManager {
    pub fn maybe_save(&mut self, step: i32, vm: &CideVM) {
        if step % self.interval == 0 {
            self.checkpoints.push((step, vm.snapshot()));
        }
    }
    
    pub fn seek_to(&self, target: i32, vm: &mut CideVM) {
        // 找到最近检查点
        let (idx, snap) = self.checkpoints.iter().rfind(|(s, _)| *s <= target).unwrap();
        vm.restore(snap);
        // 正向重放到目标步
        for _ in *idx..target {
            vm.step_next();
        }
    }
}
```

---

## 8. 与统一模式的协作关系

```
用户点击"运行"
    ↓
[自动执行模式] VM 连续执行
    ├── 每步：构造 VisState → FRB 推送 → 前端缓存
    ├── 每步：更新 Heatmap（行号侧边栏颜色实时变化）
    ├── 每步：收集 StepMeta（语义标签）
    ├── 每步：收集 VarHistory（变量变化历史）
    ├── 每 20 步：保存 VM 全量快照（检查点）
    └── 遇 Trap：自动回退到检查点 + 显示诊断
    ↓
执行结束 / 用户暂停
    ↓
[统一模式] 用户自由探索
    ├── 拖动进度条 → O(1) 切换 VisState（动画零延迟）
    ├── 悬浮球 → O(1) 读取 VarHistory（变量零延迟）
    ├── 代码编辑器 → 实时显示 VariableHighlight（变量级高亮）
    ├── 侧边栏 → Heatmap 显示累计执行次数
    └── 点击"继续执行" → 从当前步恢复 VM 并继续
```

**用户完全不需要区分"调试模式"和"回放模式"。只有一个模式：写代码 → 运行 → 自由探索。**

---

## 9. 实施优先级

| 优先级 | 功能 | 状态 | 实际文件 |
|:---|:---|:---|:---|
| P0 | VM 全量快照/恢复 + 检查点管理 | ✅ 已实现 | `vm/snapshot.rs` + `unified/checkpoint.rs` |
| P0 | 自动执行模式（收集 StepPayload + StepMeta） | ✅ 已实现 | `unified/engine.rs` `run_batch()` |
| P1 | 执行路径热力图（Heatmap） | ✅ 已实现 | `session.rs` heatmap + Flutter 覆盖率显示 |
| P1 | 排序动画 MVP + 语义进度条 | ✅ 已实现 | `widgets/array_vis_tab.dart` + `ExecutionControlPanel` |
| P1 | 变量变化历史（悬浮球零延迟） | ✅ 已实现 | `widgets/var_history_tab.dart` 迷你趋势图 |
| P2 | 运行时异常自动回退 | ✅ 已实现 | `unified/engine.rs` `pre_step_snap` + Trap 回退 |
| P2 | 变量级高亮 | 🔄 部分实现 | `StepPayload.accessed_vars` 已收集；`re_editor` 集成待实现 |
| P3 | 链表/树可视化增强 | ⏳ 待实现 | `LinkedListVisualizer` / `TreeVisualizer` |

**实际用时：约 2 周（后端 5 天 + 前端 5 天 + 联调 4 天）。**

---

## 附录：竞品对比

| 功能 | VisuAlgo | Python Tutor | GDB/LLDB | VisualBinaryTree | **Cide（本方案）** |
|:---|:---|:---|:---|:---|:---|
| 算法动画 | ✅ | ❌ | ❌ | ✅ | ✅ |
| 进度条拖动 | ❌ | ❌ | ❌ | ✅ | ✅ |
| 真实代码执行 | ❌ | ✅ | ✅ | ✅ | ✅ |
| 局部变量查看 | ❌ | ✅ | ✅ | ❌ | ✅ |
| 内存查看 | ❌ | ❌ | ✅ | ❌ | ✅ |
| 调用栈查看 | ❌ | ❌ | ✅ | ❌ | ✅ |
| 执行热力图 | ❌ | ❌ | ❌（Profiler 有，但非教学向） | ❌ | ✅ |
| 异常自动回退 | ❌ | ❌ | ❌ | ❌ | ✅ |
| 变量级高亮 | ❌ | ❌ | ❌ | ❌ | ✅ |
| 语义进度条 | ❌ | ❌ | ❌ | ❌ | ✅ |
| **统一模式（无需区分调试/回放）** | ❌ | ❌ | ❌ | ❌ | ✅ |
