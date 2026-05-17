# Cide 统一模式设计文档

> **版本**：2026-05-17  
> **状态**：设计阶段  
> **核心原则**：用户不区分"调试模式"和"回放模式"。写代码 → 编译 → 运行 → 自由探索，一个流程走到底。

---

## 目录

- [1. 设计目标](#1-设计目标)
- [2. 核心概念](#2-核心概念)
- [3. 架构总览](#3-架构总览)
- [4. 状态机](#4-状态机)
- [5. Rust 后端](#5-rust-后端)
  - [5.1 VM 快照/恢复](#51-vm-快照恢复)
  - [5.2 检查点管理器](#52-检查点管理器)
  - [5.3 自动执行引擎](#53-自动执行引擎)
  - [5.4 FRB API](#54-frb-api)
- [6. Flutter 前端](#6-flutter-前端)
  - [6.1 状态管理](#61-状态管理)
  - [6.2 执行控制面板](#62-执行控制面板)
  - [6.3 进度条](#63-进度条)
  - [6.4 可视化面板](#64-可视化面板)
  - [6.5 代码编辑器集成](#65-代码编辑器集成)
- [7. 数据流](#7-数据流)
- [8. 边界情况](#8-边界情况)
- [9. 与现有功能集成](#9-与现有功能集成)
- [10. 实施路线图](#10-实施路线图)

---

## 1. 设计目标

### 1.1 解决的问题

传统 IDE 把"调试"和"回放"当成两个独立功能：
- **调试器**：单步执行、看变量、内存、调用栈。没有动画，没有进度条。
- **算法可视化工具**：有动画、有进度条。但预录帧，没有真实变量值，不能修改代码后继续执行。

学生在两个工具之间切换，认知负担大，学习曲线陡峭。

### 1.2 统一模式的定义

**一个模式，四种自由**：

```
写代码 → 编译 → 点击"运行"
              ↓
    ┌─────────┼─────────┐
    ↓         ↓         ↓
  自动播放  单步调试  拖动进度条
    ↓         ↓         ↓
  随时暂停  随时继续  随时恢复执行
    ↓         ↓         ↓
  查看动画  查看变量  修改代码重跑
```

用户**不需要点击任何模式切换按钮**。系统自动判断用户的意图，提供相应的交互能力。

### 1.3 非目标

- ❌ 不是"预录帧后离线播放"（如视频）
- ❌ 不是"纯调试器加个进度条装饰"
- ❌ 不是"牺牲调试能力换取动画"
- ✅ 是"调试能力 + 动画能力 + 进度条能力"的三位一体

---

## 2. 核心概念

### 2.1 三态缓存（Triple-Cache）

统一模式依赖三种不同粒度的缓存，分别服务于不同的交互场景：

| 缓存层 | 存储位置 | 数据内容 | 用途 | 大小（1000步） |
|:---|:---|:---|:---|:---|
| **Frame Cache** | Flutter 前端 | `List<StepPayload>` | 动画渲染、变量面板、进度条拖动 | 2~5MB |
| **Checkpoint** | Rust 后端 | `List<(i32, VMSnapshot)>` | VM 状态恢复（继续执行、查看内存） | 50MB |
| **Active VM** | Rust 后端 | `CideVM` 实例 | 当前可执行的 VM 状态 | 1MB |

**设计原则**：
- 90% 的用户操作（拖动进度条、查看变量）只访问 **Frame Cache**，零延迟
- 10% 的操作（继续执行、查看特定内存地址）需要恢复 **Active VM**，从 Checkpoint 懒加载
- 不需要同时维护多个 Active VM，只有一个"当前 VM"

### 2.2 StepPayload（每步的轻量数据包）

```rust
pub struct StepPayload {
    /// 步骤索引（0-based）
    pub step_index: i32,
    
    /// ① 动画渲染数据
    pub vis_state: VisState,
    
    /// ② 语义元数据（进度条标签）
    pub meta: StepMeta,
    
    /// ③ 调试摘要（悬浮球零延迟）
    pub debug_summary: DebugSummary,
    
    /// ④ 执行热力图数据（该行被执行次数的增量）
    pub heatmap_delta: HeatmapDelta,
}

pub struct VisState {
    pub code_line: i32,
    pub arrays: Vec<VisArray>,
    pub nodes: Vec<VisNode>,
    pub edges: Vec<VisEdge>,
    pub ranges: Vec<VisRange>,
}

pub struct StepMeta {
    pub code_line: i32,
    pub func_name: String,
    pub loop_depth: i32,
    pub loop_iters: Vec<i32>,
    pub is_loop_boundary: bool,
    pub is_func_call: bool,
    pub is_swap: bool,
    pub semantic_label: String,
}

pub struct DebugSummary {
    pub local_vars: Vec<VariableSnapshot>,
    pub call_stack: Vec<FrameInfo>,
    pub memory_summary: MemorySummary,
}

pub struct HeatmapDelta {
    pub line: i32,
    pub count_increment: u64,
}
```

### 2.3 Seek 策略

用户拖动进度条到第 `target` 步时，系统根据 `target` 与当前状态的关系选择策略：

```
if target <= max_collected_step {
    // 目标步已在前端缓存中
    // 策略 A：O(1) 直接切换 Frame
    render_frame(frame_cache[target]);
} else {
    // 目标步尚未收集（用户拖动到了"未来"）
    // 策略 B：从最近 Checkpoint 恢复 VM + 正向执行到 target
    checkpoint = find_nearest_checkpoint(target);
    vm.restore(checkpoint);
    for _ in checkpoint_step..target {
        vm.step_next();
        frame_cache.push(collect_step_payload());
    }
    render_frame(frame_cache[target]);
}
```

**懒加载调试信息**：
- 拖动时：只切换 `Frame Cache` 中的动画和变量摘要（O(1)）
- 停止拖动 500ms 后：如果用户停留在该步，从 Checkpoint 恢复 VM 到该步（用于查看内存/调用栈的完整信息）
- 悬浮球的"局部变量"和"调用栈"面板从 `DebugSummary` 读取，不需要恢复 VM
- 悬浮球的"内存区域"详细查看需要恢复 VM，显示 loading 指示器

---

## 3. 架构总览

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Flutter 前端                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │ CodeEditor   │  │ ExecControl  │  │ AlgoCanvas / VisPanel    │  │
│  │ + Heatmap    │  │ Panel        │  │ + VarPanel + MemoryPanel │  │
│  │              │  │ (Play/Pause/ │  │                          │  │
│  │              │  │  Step/Slider)│  │                          │  │
│  └──────┬───────┘  └──────┬───────┘  └────────────┬─────────────┘  │
│         │                 │                       │                │
│  ┌──────┴─────────────────┴───────────────────────┴────────────┐  │
│  │              ExecutionController (Riverpod)                  │  │
│  │  - FrameCache: List<StepPayload>                             │  │
│  │  - StateMachine: Idle/Collecting/Paused/Playback/Seeking     │  │
│  │  - CurrentStep: int                                          │  │
│  └────────────────────────────┬─────────────────────────────────┘  │
└───────────────────────────────┼─────────────────────────────────────┘
                                │ FRB v2 (SSE Codec)
┌───────────────────────────────┼─────────────────────────────────────┐
│                         Rust 后端                                    │
│  ┌────────────────────────────┴─────────────────────────────────┐  │
│  │              UnifiedModeEngine                                │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐   │  │
│  │  │ AutoExecutor │  │ CheckpointMgr│  │ VM (Active)      │   │  │
│  │  │              │  │              │  │                  │   │  │
│  │  │ - step_loop  │  │ - checkpoints│  │ - memory 1MB     │   │  │
│  │  │ - collect    │  │ - seek()     │  │ - value_stack    │   │  │
│  │  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘   │  │
│  │         │                 │                   │              │  │
│  │  ┌──────┴─────────────────┴───────────────────┴──────────┐   │  │
│  │  │                    CideVM                              │   │  │
│  │  │  - step_next()                                         │   │  │
│  │  │  - snapshot() / restore()                              │   │  │
│  │  │  - read_memory(addr)                                   │   │  │
│  │  └────────────────────────────────────────────────────────┘   │  │
│  └───────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 4. 状态机

统一模式的核心是一个**六状态有限状态机**，所有用户操作都触发状态转换。

```
                    ┌──────────────────────────────────────┐
                    │                                      │
                    ▼                                      │
┌─────┐   编译运行   ┌─────────────┐   执行结束   ┌─────────┐
│Idle │ ──────────► │ Collecting  │ ───────────► │Playback │
└─────┘              └──────┬──────┘              └────┬────┘
   ▲                      │                           │
   │ 用户修改代码          │ 暂停                      │ 拖动进度条
   │                      ▼                           ▼
   │                   ┌─────────┐                ┌─────────┐
   │                   │ Paused  │ ◄───────────── │ Seeking │
   │                   └────┬────┘   用户暂停/拖动  └────┬────┘
   │                      │                           │
   │                      │ 继续                      │ 恢复完成
   │                      ▼                           ▼
   │                   ┌─────────┐                ┌─────────┐
   └────────────────── │StepMode │ ◄───────────── │  (返回  │
      用户点击"重跑"   └────┬────┘   用户点单步    │ Playback)
                           │
                           │ 自动播放
                           ▼
                      ┌─────────┐
                      │Collecting│
                      └─────────┘
```

### 4.1 状态定义

| 状态 | 用户看到的界面 | 允许的交互 |
|:---|:---|:---|
| **Idle** | 编辑器可编辑，"运行"按钮高亮 | 编辑代码、点击运行 |
| **Collecting** | 进度条自动前进，动画播放，代码行高亮跟随 | 暂停、拖动进度条、修改代码（提示将重置） |
| **Paused** | 进度条暂停，动画定格 | 继续、单步下一步、拖动进度条、修改代码 |
| **Playback** | 执行结束，进度条可自由拖动 | 拖动进度条、点击"继续执行"（从当前步恢复 VM）、修改代码 |
| **Seeking** | 进度条拖动中，动画跟随 | 松开进度条后进入 Playback/Paused |
| **StepMode** | 单步状态，类似传统调试器 | 下一步、上一步、自动播放、拖动进度条 |

### 4.2 关键状态转换

**Collecting → Paused**：
- 触发：用户点击暂停按钮
- 动作：停止 `AutoExecutor` 的 step_loop
- 结果：VM 停在当前步，用户可单步或拖动

**Playback → Seeking → Playback**：
- 触发：用户拖动进度条
- 动作：
  1. 立即渲染 `FrameCache[target]` 的动画（O(1)）
  2. 如果 target > max_collected_step，后台线程从 Checkpoint 恢复 VM + 正向执行到 target
  3. 完成后更新 `FrameCache` 和 `Active VM`

**Playback → Collecting（继续执行）**：
- 触发：用户在 Playback 状态点击"继续执行"
- 动作：
  1. 从最近 Checkpoint 恢复 VM
  2. 正向重放到当前步
  3. 继续 `AutoExecutor` 的 step_loop
- 注意：如果用户从第 50 步继续执行，output_lines 需要截断到第 50 步的长度

**StepMode → Collecting（自动播放）**：
- 触发：用户在单步调试时点击"自动播放"
- 动作：从当前步继续自动收集

**任何状态 → Idle（修改代码）**：
- 触发：用户在执行过程中修改了代码
- 动作：提示"代码已修改，是否重新编译？" → 重置所有缓存 → 回到 Idle

---

## 5. Rust 后端

### 5.1 VM 快照/恢复

```rust
/// VM 全量快照（约 1MB + 少量元数据）
#[derive(Clone)]
pub struct VMSnapshot {
    // VM 核心状态
    pub memory: Vec<u8>,                    // 1MB
    pub value_stack: Vec<i64>,
    pub call_stack: Vec<CallFrame>,
    pub pc: u32,
    pub mem_stack_top: u32,
    
    // 运行时状态（包含 rand_seed, input_index, output_lines 等）
    pub runtime: RuntimeSnapshot,
    
    // 内存管理状态
    pub memory_regions: Vec<MemoryRegion>,
    pub free_list: Vec<FreeBlock>,
    pub heap_offset: u32,
}

impl CideVM {
    /// 创建快照（执行前调用，用于异常回退）
    pub fn snapshot(&self, session: &Session) -> VMSnapshot {
        VMSnapshot {
            memory: self.memory.clone(),
            value_stack: self.value_stack.clone(),
            call_stack: self.call_stack.clone(),
            pc: self.pc,
            mem_stack_top: self.mem_stack_top,
            runtime: session.runtime.snapshot(),
            memory_regions: session.memory.regions.clone(),
            free_list: session.memory.free_list.clone(),
            heap_offset: session.memory.heap_offset,
        }
    }
    
    /// 恢复快照
    pub fn restore(&mut self, snap: &VMSnapshot, session: &mut Session) {
        self.memory.copy_from_slice(&snap.memory);
        self.value_stack = snap.value_stack.clone();
        self.call_stack = snap.call_stack.clone();
        self.pc = snap.pc;
        self.mem_stack_top = snap.mem_stack_top;
        session.runtime.restore(&snap.runtime);
        session.memory.regions = snap.memory_regions.clone();
        session.memory.free_list = snap.free_list.clone();
        session.memory.heap_offset = snap.heap_offset;
    }
}
```

**output_lines 截断处理**：

```rust
/// 从检查点恢复时，截断 output_lines 到检查点时的长度
pub fn restore_with_output_truncate(
    &mut self, 
    snap: &VMSnapshot, 
    session: &mut Session
) {
    let target_len = snap.runtime.output_lines_len;
    session.runtime.output_lines.truncate(target_len);
    self.restore(snap, session);
}
```

### 5.2 检查点管理器

```rust
pub struct CheckpointManager {
    /// (step_index, snapshot)
    checkpoints: Vec<(i32, VMSnapshot)>,
    /// 检查点间隔（默认 20 步）
    interval: i32,
    /// 智能检查点：在语义关键点强制保存
    smart_mode: bool,
}

impl CheckpointManager {
    pub fn new(interval: i32) -> Self {
        Self {
            checkpoints: Vec::new(),
            interval,
            smart_mode: true,
        }
    }
    
    /// 判断是否需要保存检查点
    pub fn should_checkpoint(&self, step: i32, meta: &StepMeta) -> bool {
        // 基础间隔
        if step % self.interval == 0 {
            return true;
        }
        
        // 智能模式：语义关键点强制保存
        if self.smart_mode {
            if meta.is_loop_boundary || meta.is_func_call || meta.is_swap {
                return true;
            }
        }
        
        false
    }
    
    /// 保存检查点
    pub fn save(&mut self, step: i32, vm: &CideVM, session: &Session) {
        self.checkpoints.push((step, vm.snapshot(session)));
    }
    
    /// 找到最近的检查点（<= target）
    pub fn nearest(&self, target: i32) -> Option<(i32, &VMSnapshot)> {
        self.checkpoints.iter()
            .rfind(|(step, _)| *step <= target)
            .map(|(step, snap)| (*step, snap))
    }
    
    /// Seek 到目标步：恢复检查点 + 正向重放
    pub fn seek_to(
        &self,
        target: i32,
        vm: &mut CideVM,
        session: &mut Session,
        on_step: &mut dyn FnMut(i32, StepPayload),
    ) -> Result<(), String> {
        let (checkpoint_step, snap) = self.nearest(target)
            .ok_or("No checkpoint available".to_string())?;
        
        vm.restore_with_output_truncate(snap, session);
        
        // 正向重放到目标步
        for step in checkpoint_step..target {
            vm.step_next()?;
            let payload = collect_step_payload(vm, session, step);
            on_step(step, payload);
        }
        
        Ok(())
    }
}
```

### 5.3 自动执行引擎

```rust
pub struct AutoExecutor {
    vm: CideVM,
    checkpoint_mgr: CheckpointManager,
    frame_cache: Vec<StepPayload>,
    max_steps: i32,          // 防止无限循环（默认 10000）
    is_paused: bool,
    is_cancelled: bool,
}

impl AutoExecutor {
    /// 启动自动收集循环
    pub fn run(&mut self, session: &mut Session) -> Result<(), String> {
        while !self.is_cancelled && !self.is_paused && self.vm.can_step() {
            let step = self.vm.get_step_count();
            
            // 检查点保存
            let meta = infer_step_meta(&self.vm);
            if self.checkpoint_mgr.should_checkpoint(step, &meta) {
                self.checkpoint_mgr.save(step, &self.vm, session);
            }
            
            // 执行一步
            self.vm.step_next()?;
            
            // 收集 StepPayload
            let payload = collect_step_payload(&self.vm, session, step);
            self.frame_cache.push(payload);
            
            // 检查最大步数
            if step >= self.max_steps {
                return Err("执行步数超过限制，可能存在无限循环".to_string());
            }
        }
        
        Ok(())
    }
    
    pub fn pause(&mut self) {
        self.is_paused = true;
    }
    
    pub fn resume(&mut self) {
        self.is_paused = false;
    }
    
    pub fn cancel(&mut self) {
        self.is_cancelled = true;
    }
}
```

### 5.4 FRB API

```rust
// ========== 统一模式核心 API ==========

/// 编译并启动自动收集
/// 
/// 返回 RunResult，包含初始状态
pub fn compile_and_run(source: String) -> RunResult {
    let mut session = current_session();
    
    // 编译
    let result = run_compile_pipeline(&mut session, &source);
    if let Err(e) = result {
        return RunResult {
            success: false,
            error: Some(e),
            max_steps: 0,
            initial_payload: None,
        };
    }
    
    // 启动自动执行
    // 注意：这里不阻塞，启动后台线程执行
    start_auto_executor(session);
    
    RunResult {
        success: true,
        error: None,
        max_steps: 0,  // 异步，初始为 0
        initial_payload: None,
    }
}

/// 暂停自动执行
pub fn pause_execution() {
    if let Some(executor) = get_auto_executor() {
        executor.pause();
    }
}

/// 继续自动执行
pub fn resume_execution() {
    if let Some(executor) = get_auto_executor() {
        executor.resume();
    }
}

/// 获取指定步的 StepPayload（前端 Frame Cache 的 fallback）
pub fn get_step_payload(step: i32) -> Option<StepPayload> {
    // 优先从 Frame Cache 读取
    if let Some(payload) = get_frame_cache().get(step as usize) {
        return Some(payload.clone());
    }
    
    // 如果不在缓存中，从 Checkpoint 恢复 + 重放
    // 这是一个耗时操作，建议前端先显示 loading
    seek_and_collect(step).ok()
}

/// Seek 到指定步并恢复 VM 状态
/// 
/// 用于：继续执行、查看内存、查看完整调用栈
pub fn seek_to_step(target: i32) -> Result<SeekResult, String> {
    let mut session = current_session();
    let mut vm = get_active_vm();
    let checkpoint_mgr = get_checkpoint_mgr();
    
    checkpoint_mgr.seek_to(target, &mut vm, &mut session, &mut |step, payload| {
        push_to_frame_cache(step, payload);
    })?;
    
    // 更新 Active VM
    set_active_vm(vm);
    
    Ok(SeekResult {
        target_step: target,
        vm_restored: true,
    })
}

/// 从当前步继续执行（Playback → Collecting）
pub fn continue_from_current() -> Result<RunResult, String> {
    let current_step = get_current_step();
    
    // 确保 VM 已恢复到当前步
    seek_to_step(current_step)?;
    
    // 启动自动执行
    resume_execution();
    
    Ok(RunResult {
        success: true,
        ..Default::default()
    })
}

/// 单步执行（StepMode）
pub fn step_next() -> Result<StepPayload, String> {
    let mut session = current_session();
    let mut vm = get_active_vm();
    
    vm.step_next()?;
    let step = vm.get_step_count();
    let payload = collect_step_payload(&vm, &session, step);
    
    set_active_vm(vm);
    push_to_frame_cache(step, payload.clone());
    
    Ok(payload)
}

/// 获取执行热力图
pub fn get_heatmap() -> HeatmapData {
    let session = current_session();
    HeatmapData {
        line_counts: session.runtime.heatmap.line_counts.clone(),
        max_count: session.runtime.heatmap.max_count(),
    }
}

/// 获取变量历史
pub fn get_var_history(var_name: String) -> Option<VarHistory> {
    get_var_history_cache().get(&var_name).cloned()
}

/// 读取内存（需要 Active VM 已恢复）
pub fn read_memory(addr: u32, count: i32) -> Vec<i32> {
    let vm = get_active_vm();
    vm.read_memory_range(addr, count)
}
```

---

## 6. Flutter 前端

### 6.1 状态管理

使用 `flutter_riverpod` 管理统一模式的复杂状态。

```dart
/// 执行状态枚举
enum ExecutionPhase {
  idle,           // 空闲，可编辑代码
  compiling,      // 编译中
  collecting,     // 自动收集（播放中）
  paused,         // 暂停
  playback,       // 回放（执行结束或用户暂停后）
  seeking,        // 寻址中（拖动进度条）
  stepMode,       // 单步模式
  error,          // 编译错误或运行时错误
}

/// 统一状态对象
@freezed
class ExecutionState with _$ExecutionState {
  const factory ExecutionState({
    required ExecutionPhase phase,
    @Default(0) int currentStep,
    @Default(0) int maxCollectedStep,
    @Default(0) int totalSteps,
    StepPayload? currentPayload,
    @Default([]) List<StepPayload> frameCache,
    @Default(false) bool isPlaying,
    @Default(1.0) double playbackSpeed,
    String? errorMessage,
    @Default(false) bool isVmRestored,  // Active VM 是否已恢复
  }) = _ExecutionState;
}

/// 核心控制器
@riverpod
class ExecutionController extends _$ExecutionController {
  Timer? _playbackTimer;
  
  @override
  ExecutionState build() => const ExecutionState(phase: ExecutionPhase.idle);
  
  /// 编译并运行
  Future<void> compileAndRun(String source) async {
    state = state.copyWith(phase: ExecutionPhase.compiling);
    
    final result = await rust.compileAndRun(source: source);
    
    if (!result.success) {
      state = state.copyWith(
        phase: ExecutionPhase.error,
        errorMessage: result.error,
      );
      return;
    }
    
    state = state.copyWith(phase: ExecutionPhase.collecting);
    _startFrameCollection();
  }
  
  /// 启动帧收集监听（异步接收 FRB 推送的 StepPayload）
  void _startFrameCollection() {
    rust.stepPayloadStream.listen((payload) {
      final newCache = [...state.frameCache, payload];
      state = state.copyWith(
        frameCache: newCache,
        maxCollectedStep: payload.stepIndex,
        currentStep: payload.stepIndex,
        currentPayload: payload,
      );
    });
  }
  
  /// 暂停
  void pause() {
    rust.pauseExecution();
    _playbackTimer?.cancel();
    state = state.copyWith(
      phase: ExecutionPhase.paused,
      isPlaying: false,
    );
  }
  
  /// 继续（从 Collecting 或 StepMode）
  void resume() {
    if (state.phase == ExecutionPhase.playback) {
      // 从 Playback 继续需要先恢复 VM
      _continueFromPlayback();
      return;
    }
    
    rust.resumeExecution();
    state = state.copyWith(
      phase: ExecutionPhase.collecting,
      isPlaying: true,
    );
  }
  
  /// 从 Playback 状态恢复 VM 并继续执行
  Future<void> _continueFromPlayback() async {
    state = state.copyWith(phase: ExecutionPhase.seeking);
    await rust.seekToStep(target: state.currentStep);
    state = state.copyWith(isVmRestored: true);
    resume();
  }
  
  /// 拖动进度条
  Future<void> seekTo(int targetStep) async {
    // 即时响应：如果目标步已在缓存中，O(1) 切换
    if (targetStep <= state.maxCollectedStep && targetStep < state.frameCache.length) {
      state = state.copyWith(
        currentStep: targetStep,
        currentPayload: state.frameCache[targetStep],
        phase: ExecutionPhase.playback,
      );
      return;
    }
    
    // 需要后台恢复 VM
    state = state.copyWith(phase: ExecutionPhase.seeking);
    final result = await rust.seekToStep(target: targetStep);
    
    if (result.vmRestored) {
      state = state.copyWith(
        currentStep: targetStep,
        isVmRestored: true,
        phase: ExecutionPhase.playback,
      );
      // 从 FrameCache 或重新获取 payload
      _updateCurrentPayload(targetStep);
    }
  }
  
  /// 单步下一步
  Future<void> stepNext() async {
    state = state.copyWith(phase: ExecutionPhase.stepMode);
    final payload = await rust.stepNext();
    
    final newCache = [...state.frameCache];
    if (payload.stepIndex < newCache.length) {
      newCache[payload.stepIndex] = payload;
    } else {
      newCache.add(payload);
    }
    
    state = state.copyWith(
      frameCache: newCache,
      currentStep: payload.stepIndex,
      currentPayload: payload,
      maxCollectedStep: math.max(state.maxCollectedStep, payload.stepIndex),
      isVmRestored: true,
    );
  }
  
  /// 自动播放动画（Playback 状态下）
  void startPlaybackAnimation() {
    _playbackTimer?.cancel();
    _playbackTimer = Timer.periodic(
      Duration(milliseconds: (1000 / state.playbackSpeed).round()),
      (_) {
        if (state.currentStep < state.maxCollectedStep) {
          seekTo(state.currentStep + 1);
        } else {
          _playbackTimer?.cancel();
        }
      },
    );
  }
  
  /// 修改代码后重置
  void onCodeChanged() {
    _playbackTimer?.cancel();
    rust.cancelExecution();
    state = const ExecutionState(phase: ExecutionPhase.idle);
  }
}
```

### 6.2 执行控制面板

```dart
class ExecutionControlPanel extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(executionControllerProvider);
    final controller = ref.read(executionControllerProvider.notifier);
    
    return Row(
      children: [
        // 运行/暂停/继续按钮
        _buildPlayPauseButton(state, controller),
        
        // 单步按钮（仅在 Paused 或 StepMode 显示）
        if (state.phase == ExecutionPhase.paused || 
            state.phase == ExecutionPhase.stepMode)
          IconButton(
            icon: const Icon(Icons.skip_next),
            onPressed: controller.stepNext,
          ),
        
        // 进度条
        Expanded(
          child: ExecutionSlider(
            max: state.maxCollectedStep,
            value: state.currentStep.toDouble(),
            onChangeStart: controller.pause,
            onChanged: (v) => controller.seekTo(v.round()),
          ),
        ),
        
        // 播放速度
        PlaybackSpeedButton(
          speed: state.playbackSpeed,
          onChanged: (s) => state = state.copyWith(playbackSpeed: s),
        ),
      ],
    );
  }
  
  Widget _buildPlayPauseButton(ExecutionState state, ExecutionController ctrl) {
    switch (state.phase) {
      case ExecutionPhase.idle:
        return IconButton(
          icon: const Icon(Icons.play_arrow),
          onPressed: () => ctrl.compileAndRun(getCurrentSource()),
        );
      case ExecutionPhase.collecting:
        return IconButton(
          icon: const Icon(Icons.pause),
          onPressed: ctrl.pause,
        );
      case ExecutionPhase.paused:
      case ExecutionPhase.stepMode:
        return IconButton(
          icon: const Icon(Icons.play_arrow),
          onPressed: ctrl.resume,
        );
      case ExecutionPhase.playback:
        return IconButton(
          icon: const Icon(Icons.play_arrow),
          tooltip: '从当前步继续执行',
          onPressed: ctrl.resume,
        );
      default:
        return const SizedBox.shrink();
    }
  }
}
```

### 6.3 进度条

```dart
class ExecutionSlider extends StatelessWidget {
  final int max;
  final double value;
  final VoidCallback onChangeStart;
  final ValueChanged<double> onChanged;
  
  @override
  Widget build(BuildContext context) {
    return Slider(
      min: 0,
      max: max.toDouble(),
      value: value,
      divisions: max > 0 ? max : null,
      label: _buildLabel(value.round()),
      onChangeStart: (_) => onChangeStart(),
      onChanged: onChanged,
    );
  }
  
  String _buildLabel(int step) {
    // 显示语义标签而非步数
    final payload = getFrameCache().getFrame(step);
    if (payload != null) {
      return payload.meta.semanticLabel;
    }
    return '第 $step 步';
  }
}
```

### 6.4 可视化面板

```dart
class UnifiedVisPanel extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(executionControllerProvider);
    final payload = state.currentPayload;
    
    if (payload == null) {
      return const Center(child: Text('点击运行开始'));
    }
    
    return Column(
      children: [
        // 算法动画（排序/链表/树）
        AlgoCanvas(visState: payload.vis_state),
        
        // 变量面板
        VariablePanel(vars: payload.debug_summary.local_vars),
        
        // 调用栈
        CallStackPanel(frames: payload.debug_summary.call_stack),
      ],
    );
  }
}
```

### 6.5 代码编辑器集成

```dart
class UnifiedCodeEditor extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(executionControllerProvider);
    final heatmap = ref.watch(heatmapProvider);
    
    return ReEditor(
      // 当前高亮行
      highlightedLine: state.currentPayload?.code_line,
      
      // 变量级高亮
      variableHighlights: _buildVariableHighlights(state.currentPayload),
      
      // 执行路径热力图（侧边栏）
      gutter: HeatmapGutter(
        lineCounts: heatmap.lineCounts,
        maxCount: heatmap.maxCount,
      ),
      
      // 代码修改监听
      onChanged: (code) {
        if (state.phase != ExecutionPhase.idle) {
          // 提示用户代码已修改，是否重新运行
          showRestartDialog(context, () {
            ref.read(executionControllerProvider.notifier).onCodeChanged();
          });
        }
      },
    );
  }
  
  List<VariableHighlight> _buildVariableHighlights(StepPayload? payload) {
    if (payload == null) return [];
    
    return payload.debug_summary.local_vars.map((var) {
      return VariableHighlight(
        line: var.decl_line,
        column: var.decl_column,
        length: var.name.length,
        color: _highlightColor(var.access_type),
      );
    }).toList();
  }
}
```

---

## 7. 数据流

### 7.1 正常执行流

```
[Flutter] 用户点击"运行"
    ↓ FRB: compileAndRun(source)
[Rust] 编译 → 启动 AutoExecutor 后台线程
    ↓ 每步
[Rust] step_next() → collect StepPayload
    ↓ FRB Stream
[Flutter] 收到 StepPayload → 追加到 FrameCache
    ↓
[Flutter] 更新 ExecutionState → 重绘 UI
    ├── CodeEditor: 高亮当前行
    ├── AlgoCanvas: 渲染 VisState
    ├── VarPanel: 显示局部变量
    └── Slider: 进度条前进
```

### 7.2 拖动进度条流

```
[Flutter] 用户拖动 Slider 到 step 150
    ↓
[Flutter] 立即显示 FrameCache[150] 的动画（O(1)）
    ↓ 500ms 后用户未继续拖动
[Flutter] 调用 rust.seekToStep(target: 150)
    ↓ FRB
[Rust] CheckpointMgr.nearest(150) → 找到检查点 140
    ↓
[Rust] vm.restore(checkpoint_140) → 正向重放 10 步
    ↓
[Rust] 更新 Active VM → 返回 SeekResult
    ↓ FRB
[Flutter] 更新 isVmRestored = true
    ↓
[Flutter] 悬浮球"内存区域"面板现在可以读取真实 VM 内存
```

### 7.3 继续执行流

```
[Flutter] 用户在 Playback step 150 点击"继续执行"
    ↓ FRB: seekToStep(150)
[Rust] 恢复 VM 到 step 150（同上）
    ↓
[Flutter] 调用 rust.resumeExecution()
    ↓
[Rust] AutoExecutor 继续 step_loop
    ↓
[Rust] 从 step 150 继续收集 StepPayload
    ↓ FRB Stream
[Flutter] 追加到 FrameCache，进度条继续前进
```

---

## 8. 边界情况

### 8.1 无限循环

```rust
// AutoExecutor 设置最大步数限制
const MAX_STEPS: i32 = 10_000;

if step >= MAX_STEPS {
    return Err("执行步数超过限制（10,000 步），可能存在无限循环。".to_string());
}
```

前端显示：
```
⚠️ 执行已暂停
程序已执行 10,000 步，可能包含无限循环。

当前代码位置：第 7 行 while (1) {
建议：检查循环终止条件。

[查看当前状态] [强制停止]
```

### 8.2 运行时异常（数组越界、空指针等）

```rust
// VM 的 step_next_safe 包装
try {
    vm.step_next()?;
} catch (trap) {
    // 自动回退到上一步
    vm.restore(&last_snapshot);
    
    // 返回错误信息 + 当前状态
    return Err(format!("运行时错误：{}\n发生在第 {} 行", trap.message, trap.line));
}
```

前端显示异常面板（见 5.5 运行时异常自动回退）。

### 8.3 scanf 等待输入

```rust
// host_scanf 中检测到 input_index >= input_lines.len()
if session.runtime.input_index >= session.runtime.input_lines.len() {
    session.runtime.waiting_input = true;
    // 返回特殊状态，前端弹出输入框
    return StepResult::WaitingInput;
}
```

前端显示：
```
⏸️ 程序等待输入
scanf("%d", &x);

请输入一个整数：
[________] [确认]
```

用户输入后，input_lines 追加新值，继续执行。

### 8.4 代码修改后重置

用户在执行过程中修改代码：
1. 前端检测到代码变化
2. 显示非阻断提示："代码已修改，重新编译以更新执行结果"
3. 用户点击"重新运行" → 取消当前执行 → 清空所有缓存 → 回到 Idle

### 8.5 内存不足（FrameCache 过大）

如果程序执行了 100,000 步：
- FrameCache：100,000 × 2KB = 200MB（可能过大）
- 解决方案：FrameCache 设置上限（如 50MB），超过时丢弃最早的 20% 帧
- 被丢弃的帧可以从 Checkpoint 重新收集（懒加载）

```dart
const MAX_FRAME_CACHE_SIZE = 50 * 1024 * 1024; // 50MB

void _enforceFrameCacheLimit() {
  while (_estimateSize(frameCache) > MAX_FRAME_CACHE_SIZE) {
    // 丢弃最早的 20% 帧
    final discardCount = frameCache.length ~/ 5;
    frameCache.removeRange(0, discardCount);
  }
}
```

---

## 9. 与现有功能集成

### 9.1 与诊断修复系统集成

运行时异常自动回退后，诊断系统可以：
- 根据错误类型（数组越界、空指针、栈溢出）匹配知识卡片
- 提供一键修复建议（如 `i <= n` → `i < n`）
- 记录到学习进度系统（"今天又修复了一个数组越界错误"）

### 9.2 与学习进度系统集成

```dart
// 用户完成一次完整的"运行 → 拖动 → 理解"流程
LearningProgress.recordVisExploration(
  algorithm: detectedAlgorithm,
  stepsExplored: stepsDragged,
  timeSpent: duration,
);
```

### 9.3 与持久化系统集成

```dart
// 自动保存当前代码 + 执行状态
Future<void> autoSave() async {
  await prefs.setString('last_source_code', editor.code);
  await prefs.setInt('last_execution_step', state.currentStep);
  await prefs.setStringList('input_lines', session.runtime.inputLines);
}

// 恢复时
Future<void> restoreSession() async {
  final code = prefs.getString('last_source_code');
  final step = prefs.getInt('last_execution_step') ?? 0;
  // 自动编译 + seek 到上次步骤
}
```

---

## 10. 实施路线图

### Phase 0：VM 快照/恢复（3 天）
- [ ] `VMSnapshot` 数据结构
- [ ] `CideVM::snapshot()` / `restore()`
- [ ] `output_lines` 截断处理
- [ ] 单元测试：快照 → 恢复 → 状态一致

### Phase 1：检查点管理器 + 自动执行引擎（2 天）
- [ ] `CheckpointManager`（基础间隔 + 智能模式）
- [ ] `AutoExecutor`（后台线程 + 步数限制 + 暂停/继续）
- [ ] 集成到 `flutter_bridge.rs`

### Phase 2：FRB API + Flutter 状态管理（3 天）
- [ ] FRB Stream：`StepPayload` 推送
- [ ] `ExecutionController`（Riverpod + 六状态机）
- [ ] 执行控制面板（Play/Pause/Step/Slider）

### Phase 3：执行路径热力图（2 天）
- [ ] VM 层：`heatmap.line_counts` 收集
- [ ] FRB API：`get_heatmap()`
- [ ] Flutter：`HeatmapGutter` 侧边栏渲染

### Phase 4：排序动画 MVP + 语义进度条（3 天）
- [ ] `VisState` 数据结构 + 数组排序反推
- [ ] `StepMeta` 语义标签生成
- [ ] `AlgoCanvas` 柱状图 + 交换动画
- [ ] 进度条语义标签显示

### Phase 5：变量变化历史 + 悬浮球零延迟（2 天）
- [ ] `VarHistory` 收集
- [ ] `DebugSummary` 每步预存
- [ ] 悬浮球变量面板 + 趋势图

### Phase 6：运行时异常自动回退（2 天）
- [ ] `step_next_safe` 包装
- [ ] 异常诊断匹配
- [ ] 自动回退 UI 面板

### Phase 7：变量级高亮（2 天）
- [ ] 编译器符号表 → VM 变量访问映射
- [ ] `VariableHighlight` 数据结构
- [ ] `re_editor` 集成（下划线/边框/底色）

### Phase 8：链表/树可视化增强（1.5 周）
- [ ] `LinkedListVisualizer`
- [ ] `TreeVisualizer`
- [ ] 复用统一模式的所有基础设施

**总计：约 6~7 周**

---

## 附录：与 `VM_EXPERIENCE_ADVANTAGE.md` 的关系

| 文档 | 聚焦点 |
|:---|:---|
| `VM_EXPERIENCE_ADVANTAGE.md` | **为什么**要做这些体验（教学价值、竞品对比） |
| `UNIFIED_MODE_DESIGN.md`（本文） | **怎么做**统一模式（架构、状态机、API、数据流） |

两文档配合使用：先读前者理解"为什么要做"，再读本文理解"怎么落地"。
