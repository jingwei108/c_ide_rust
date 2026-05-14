# Cide 前端迁移计划：Tauri + TypeScript

> 状态：草案  
> 日期：2026-05-12  
> 目标：将前端从 .NET MAUI BlazorWebView 迁移至 Tauri Mobile + React/TypeScript，保留 Rust 后端 (`cide_native`) 完全不动。

---

## 目录

1. [执行摘要](#1-执行摘要)
2. [为什么选 Tauri](#2-为什么选-tauri)
3. [架构总览](#3-架构总览)
4. [项目结构重组](#4-项目结构重组)
5. [Rust 层设计](#5-rust-层设计-src-tauri)
6. [前端设计](#6-前端设计-frontend)
7. [虚拟键盘与 SymbolBar](#7-虚拟键盘与-symbolbar)
8. [移动端适配](#8-移动端适配)
9. [构建与 CI/CD](#9-构建与-cicd)
10. [迁移路线图](#10-迁移路线图-10-周)
11. [风险清单](#11-风险清单)
12. [附录：决策对比表](#12-附录决策对比表)

---

## 1. 执行摘要

### 1.1 背景

Cide 当前前端为 .NET 10 MAUI BlazorWebView，后端为 Rust (`cide_native`)。项目处于上升期，后续有密集的 UI 交互需求（底部框拖拽拉伸、9 元素统合、跨容器拖拽交换、智能补全等）。这些需求在 Blazor 中实现成本极高，在纯 TS 前端中实现成本极低。

### 1.2 核心发现

`native/Cargo.toml` 已配置 `crate-type = ["cdylib", "staticlib", "rlib"]`，意味着 **Tauri 可以直接将 `cide_native` 作为 Rust crate 引用**，完全绕过现有的 C API 层 (`capi/`)。

### 1.3 目标

- 删除 C API 胶水层和 C# 前端胶水层（约 3700 行代码）
- 前端完全使用 React + TypeScript + CodeMirror 6
- APK 体积从 ~25-35 MB 降至 ~10-15 MB
- 10 周内完成迁移 + 新功能开发

### 1.4 保留与删除

| 保留（不动） | 删除 | 新建 |
|-------------|------|------|
| `native/` 全部 Rust 后端 | `Cide.Client.Maui/` MAUI 项目 | `src-tauri/` Tauri 壳子 |
| `native/tests/` | `Cide.Client.Shared/` C# 共享库 | `frontend/` React + TS 前端 |
| `docs/` 文档 | `cide_native` C API 层（逐步） | `.github/workflows/ci.yml` 新 CI |

---

## 2. 为什么选 Tauri

### 2.1 与 MAUI WebView + TS 的关键差异

| 维度 | Tauri + TS | MAUI WebView + TS |
|------|-----------|-------------------|
| **Session 管理** | `State<Mutex<Session>>`，零 unsafe | `IntPtr` + 手动 `Dispose()` |
| **后端桥接代码量** | ~400 行（Rust command） | ~700 行（C# WebBridge + NativeMethods） |
| **是否需要 C API** | ❌ 完全不需要 | ✅ 必须保留 |
| **产物体积 (APK)** | ~10-15 MB | ~25-35 MB |
| **启动速度** | 快（无 .NET JIT） | 慢（.NET MAUI 初始化） |
| **调试 Native 层** | 差（rust-lldb） | 极好（VS F5） |
| **构建速度** | 慢（Rust 全量编译） | 快（.NET 编译快） |
| **CI 改造工作量** | 2 周 | 0.5 周 |
| **总迁移工期** | 10 周 | 6-8 周 |

### 2.2 不可替代的优势

1. **直接引用 rlib**：`cide_native` 已是 rlib，Tauri 可直接 `use cide_native::session::*`，无需 `#[no_mangle] extern "C"`。
2. **内存安全**：`State<Mutex<Session>>` 由 Rust 所有权系统自动管理，从根本上消灭 `double-free` 和 `use-after-free`。
3. **字符串传递**：Rust `String` ↔ JS `string` 自动序列化，无需 `MarshalAs(LPUTF8Str)` 和 `byte[]` 缓冲区管理。
4. **产物体积**：无 .NET runtime，APK 小一半。

### 2.3 决策建议

选 Tauri，如果：
- 把这个项目看作**长期产品**（2 年以上）
- APK 体积对目标用户（学生低端机）很重要
- 愿意多花 2-4 周换长期架构简洁

选 MAUI WebView + TS，如果：
- 需要**最快时间**上线新功能（2 个月内）
- 无法承受任何构建链风险

**本计划基于 Tauri 方案编写。**

---

## 3. 架构总览

### 3.1 迁移前架构

```
前端 (Blazor Razor)
  ↓ BlazorWebView
C# (Cide.Client.Maui + Shared)
  ↓ P/Invoke (Cdecl, Marshal, IntPtr)
C API (cide_native capi/)
  ↓ unsafe 指针
Rust 后端 (cide_native compiler/vm)
```

### 3.2 迁移后架构

```
前端 (React + TS + CodeMirror 6)
  ↓ Tauri IPC（自动生成，类型安全）
Tauri Core (Rust, ~400 行 command handler)
  ↓ 直接函数调用（零开销，同进程）
Rust 后端 (cide_native rlib)
```

### 3.3 通信模型

Tauri 的 JS ↔ Rust 通信通过 **Command 模式**实现：

1. Rust 侧定义 `#[tauri::command] fn compile(...) -> Result<T, String>`
2. Tauri 自动生成 JS binding
3. 前端调用 `invoke('compile', { source })`，参数和返回值自动 JSON 序列化
4. 单次往返延迟 < 1ms（本地 IPC），远快于 WebView JS Bridge

---

## 4. 项目结构重组

```
c_ide_rust/
├── native/                          # 现有后端，100% 保留
│   ├── Cargo.toml                   # crate-type 已有 "rlib" ✅
│   ├── src/
│   │   ├── lib.rs
│   │   ├── session.rs               # Session + 所有数据结构（已有 serde）
│   │   ├── compiler/
│   │   ├── vm/
│   │   ├── diagnostics/
│   │   └── capi/                    # 保留，但 Tauri 不经过这里
│   └── tests/
│
├── src-tauri/                       # 新：Tauri Rust 壳子（~400 行）
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   └── src/
│       ├── main.rs                  # 入口
│       ├── lib.rs                   # Tauri command handler + AppState
│       └── compiler_bridge.rs       # 编译/执行逻辑封装（从 capi 提取）
│
├── frontend/                        # 新：纯 TS 前端
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── index.html
│   └── src/
│       ├── main.tsx
│       ├── App.tsx                  # 根布局
│       ├── bridge.ts                # Tauri invoke 封装（~150 行）
│       ├── stores/
│       │   └── appStore.ts          # Zustand 全局状态
│       ├── components/
│       │   ├── Editor/
│       │   │   └── CodeMirror.tsx   # CM6 直接引入
│       │   ├── Layout/
│       │   │   ├── AppLayout.tsx
│       │   │   ├── BottomPanel.tsx  # 底部面板（可拖拽拉伸）
│       │   │   ├── FloatingBall.tsx # 悬浮球
│       │   │   └── DragProvider.tsx # @dnd-kit context
│       │   ├── Panels/              # 9 个元素的内容面板
│       │   │   ├── OutputPanel.tsx
│       │   │   ├── DiagnosticsPanel.tsx
│       │   │   ├── AlgorithmPanel.tsx
│       │   │   ├── KnowledgeCardPanel.tsx
│       │   │   ├── PointerViewPanel.tsx
│       │   │   ├── ArrayVisualizationPanel.tsx
│       │   │   ├── MemoryRegionPanel.tsx
│       │   │   ├── LocalVariablesPanel.tsx
│       │   │   ├── WatchVariablesPanel.tsx
│       │   │   └── CallStackPanel.tsx
│       │   └── Toolbar/
│       │       ├── MainToolbar.tsx
│       │       ├── SymbolBar.tsx    # 虚拟键盘上方符号栏
│       │       └── TemplateBar.tsx
│       └── hooks/
│           ├── useCompile.ts
│           ├── useExecution.ts
│           ├── useDragAndDrop.ts
│           └── useKeyboard.ts       # 虚拟键盘高度监听
│
├── .github/workflows/
│   └── ci.yml                       # 改造后
│
└── scripts/
    └── build_tauri.py               # 新构建脚本
```

---

## 5. Rust 层设计 (src-tauri)

### 5.1 AppState：替代 IntPtr 的 Session 管理

```rust
// src-tauri/src/lib.rs
use std::sync::Mutex;
use tauri::State;
use cide_native::session::{
    Session, CompileState, RuntimeState, MemoryState, 
    Diagnostic, MemoryRegion, VariableSnapshot, AlgorithmMatch, VisEvent
};

pub struct AppState {
    session: Mutex<Session>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            session: Mutex::new(Session::default()),
        }
    }
}

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            compile,
            run_code,
            step_next,
            get_output,
            get_diagnostics,
            get_variables,
            get_memory_regions,
            get_callstack,
            get_vis_events,
            get_algorithm_matches,
            add_breakpoint,
            remove_breakpoint,
            clear_breakpoints,
            set_input,
            provide_input_line,
            is_waiting_input,
            get_current_line,
            save_session,
            load_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 5.2 Command 映射表

Tauri command 不需要 1:1 映射 C API。很多 "count + get-by-index" 的 C API 模式合并为 "返回整个数组"。

| C API 模式 | Tauri Command | 优化 |
|-----------|--------------|------|
| `cide_compile` + `cide_compile_all` | `compile(source: String) -> CompileResult` | 合并 |
| `cide_run` | `run_code() -> RunResult` | 直接返回 |
| `cide_step_next` | `step_next() -> StepResult` | 直接返回 |
| `cide_diagnostic_count` + `cide_diagnostic_get` × N | `get_diagnostics() -> Vec<Diagnostic>` | 批量返回 |
| `cide_variable_count` + `cide_variable_get` × N | `get_variables() -> Vec<VariableSnapshot>` | 批量返回 |
| `cide_memory_region_count` + `cide_memory_region_get` × N | `get_memory_regions() -> Vec<MemoryRegion>` | 批量返回 |
| `cide_vis_event_count` + `cide_vis_event_get_ex` × N | `get_vis_events() -> Vec<VisEvent>` | 批量返回 |
| `cide_callstack_count` + `cide_callstack_get` × N | `get_callstack() -> Vec<CallStackFrame>` | 批量返回 |
| `cide_algorithm_match_count` + `cide_algorithm_match_get` × N | `get_algorithm_matches() -> Vec<AlgorithmMatch>` | 批量返回 |

### 5.3 关键 Command 实现

#### compile

```rust
#[derive(serde::Serialize)]
struct CompileResult {
    success: bool,
    diagnostics: Vec<Diagnostic>,
    algorithm_matches: Vec<AlgorithmMatch>,
}

#[tauri::command]
fn compile(source: String, state: State<'_, AppState>) -> Result<CompileResult, String> {
    let mut guard = state.session.lock().map_err(|e| e.to_string())?;
    
    guard.compile.compile_units.clear();
    guard.compile.compile_units.push(cide_native::session::CompileUnit {
        filename: "main.c".to_string(),
        source,
    });
    
    let success = compiler_bridge::compile_all(&mut guard)?;
    
    Ok(CompileResult {
        success,
        diagnostics: guard.compile.diagnostics.clone(),
        algorithm_matches: guard.compile.algorithm_matches.clone(),
    })
}
```

#### run_code

```rust
#[derive(serde::Serialize)]
struct RunResult {
    success: bool,
    output: String,
    waiting_input: bool,
    error: Option<String>,
}

#[tauri::command]
fn run_code(state: State<'_, AppState>) -> Result<RunResult, String> {
    let mut guard = state.session.lock().map_err(|e| e.to_string())?;
    let result = compiler_bridge::run(&mut guard);
    
    Ok(RunResult {
        success: result >= 0,
        output: guard.runtime.output_lines.join("\n"),
        waiting_input: guard.runtime.waiting_input,
        error: if guard.runtime.error.is_empty() {
            None
        } else {
            Some(guard.runtime.error.clone())
        },
    })
}
```

#### step_next

```rust
#[derive(serde::Serialize)]
struct StepResult {
    status: &'static str,
    current_line: i32,
    output: String,
    waiting_input: bool,
}

#[tauri::command]
fn step_next(state: State<'_, AppState>) -> Result<StepResult, String> {
    let mut guard = state.session.lock().map_err(|e| e.to_string())?;
    let result = compiler_bridge::step_next(&mut guard)?;
    
    Ok(StepResult {
        status: match result {
            0 => "paused",
            2 => "waiting_input",
            -1 => "finished",
            _ => "trap",
        },
        current_line: guard.runtime.current_line,
        output: guard.runtime.output_lines.join("\n"),
        waiting_input: guard.runtime.waiting_input,
    })
}
```

#### 批量数据获取

```rust
#[tauri::command]
fn get_variables(state: State<'_, AppState>) -> Result<Vec<VariableSnapshot>, String> {
    let guard = state.session.lock().map_err(|e| e.to_string())?;
    Ok(guard.runtime.variable_snapshot.clone())
}

#[tauri::command]
fn get_memory_regions(state: State<'_, AppState>) -> Result<Vec<MemoryRegion>, String> {
    let guard = state.session.lock().map_err(|e| e.to_string())?;
    Ok(guard.memory.regions.clone())
}

#[tauri::command]
fn get_diagnostics(state: State<'_, AppState>) -> Result<Vec<Diagnostic>, String> {
    let guard = state.session.lock().map_err(|e| e.to_string())?;
    Ok(guard.compile.diagnostics.clone())
}

#[tauri::command]
fn get_vis_events(state: State<'_, AppState>) -> Result<Vec<VisEvent>, String> {
    let guard = state.session.lock().map_err(|e| e.to_string())?;
    Ok(guard.runtime.vis_event_cache.clone())
}
```

#### 断点管理

```rust
#[tauri::command]
fn add_breakpoint(line: i32, state: State<'_, AppState>) -> Result<(), String> {
    let mut guard = state.session.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut vm) = guard.vm {
        vm.add_breakpoint(line as usize);
    }
    Ok(())
}

#[tauri::command]
fn clear_breakpoints(state: State<'_, AppState>) -> Result<(), String> {
    let mut guard = state.session.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut vm) = guard.vm {
        vm.clear_breakpoints();
    }
    Ok(())
}
```

#### 输入管理

```rust
#[tauri::command]
fn set_input(input: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut guard = state.session.lock().map_err(|e| e.to_string())?;
    guard.runtime.input_lines = input.lines()
        .map(|l| l.trim_end_matches('\r').to_string())
        .collect();
    guard.runtime.input_index = 0;
    guard.runtime.input_char_offset = 0;
    Ok(())
}

#[tauri::command]
fn provide_input_line(line: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut guard = state.session.lock().map_err(|e| e.to_string())?;
    guard.runtime.input_lines.push(line);
    guard.runtime.waiting_input = false;
    if let Some(ref mut vm) = guard.vm {
        vm.resume();
    }
    Ok(())
}

#[tauri::command]
fn is_waiting_input(state: State<'_, AppState>) -> Result<bool, String> {
    let guard = state.session.lock().map_err(|e| e.to_string())?;
    Ok(guard.runtime.waiting_input)
}
```

### 5.4 compiler_bridge.rs

`compiler_bridge.rs` 存放从 `capi/mod.rs` 提取的编译/执行核心逻辑。

**放置策略**：
- **Phase 1**（快速验证）：放在 `src-tauri/src/compiler_bridge.rs`，不修改 `native/` 任何代码
- **Phase 2**（重构）：提取到 `native/src/compiler_driver.rs`，供 `capi/` 和 `src-tauri/` 共享

```rust
// src-tauri/src/compiler_bridge.rs
use cide_native::session::Session;
use cide_native::compiler::{Lexer, Parser, TypeChecker, BytecodeGen};
use cide_native::vm::vm::CideVM;

pub fn compile_all(session: &mut Session) -> Result<bool, String> {
    // 清空编译状态
    session.compile.bytecode.clear();
    session.compile.globals_init.clear();
    session.compile.diagnostics.clear();
    session.compile.source_map.clear();
    session.compile.func_table.clear();
    session.compile.func_index.clear();
    session.compile.string_data.clear();
    session.compile.symbols.clear();
    session.compile.algorithm_matches.clear();
    session.compile.struct_fields.clear();
    session.compile.errors.clear();
    session.compile.errors_buffer.clear();
    session.compile.compiled = false;

    // 拼接所有编译单元
    let mut full_source = String::new();
    for unit in &session.compile.compile_units {
        full_source.push_str(&unit.source);
        if !unit.source.ends_with('\n') {
            full_source.push('\n');
        }
    }

    // 1. Lexer
    let (tokens, lex_errors) = Lexer::new(full_source.clone()).tokenize();
    if !lex_errors.is_empty() {
        // push_diagnostics...
        return Ok(false);
    }

    // 2. Parser
    let (maybe_program, parse_errors) = Parser::new(tokens).parse();
    if !parse_errors.is_empty() {
        // push_diagnostics...
        return Ok(false);
    }

    // 3. TypeChecker
    // 4. BytecodeGen
    // ...（与 capi/mod.rs 中 cide_compile_all 完全一致）

    session.compile.compiled = true;
    Ok(true)
}

pub fn run(session: &mut Session) -> i32 {
    // 与 capi/mod.rs 中 cide_run 一致
}

pub fn step_next(session: &mut Session) -> Result<i32, String> {
    // 与 capi/mod.rs 中 cide_step_next 一致
}
```

---

## 6. 前端设计 (frontend)

### 6.1 技术栈

| 层级 | 选型 | 理由 |
|------|------|------|
| 框架 | React 18 + TypeScript | 团队好招人，拖拽生态最成熟 |
| 构建 | Vite | 秒级冷启动，Tauri 原生支持 |
| 编辑器 | CodeMirror 6 (JS 直接引入) | 去掉 Blazor 封装，直接掌控所有配置 |
| 拖拽 | @dnd-kit/core + sortable | 支持触摸、多容器、排序、碰撞检测 |
| 状态 | Zustand | 比 Redux 轻，比 Context 性能好 |
| 动画 | Framer Motion (可选) | 悬浮球展开、面板切换弹簧动画 |
| 手势 | 自定义 Touch Events | 底部框拉伸等需求太定制化 |

### 6.2 bridge.ts：Tauri API 封装

```typescript
import { invoke } from '@tauri-apps/api/core';

export interface CompileResult {
  success: boolean;
  diagnostics: Diagnostic[];
  algorithm_matches: AlgorithmMatch[];
}

export interface RunResult {
  success: boolean;
  output: string;
  waiting_input: boolean;
  error?: string;
}

export interface StepResult {
  status: 'paused' | 'waiting_input' | 'finished' | 'trap';
  current_line: number;
  output: string;
  waiting_input: boolean;
}

export const bridge = {
  compile: (source: string) => 
    invoke<CompileResult>('compile', { source }),
    
  run: () => 
    invoke<RunResult>('run_code'),
    
  stepNext: () => 
    invoke<StepResult>('step_next'),
    
  getVariables: () => 
    invoke<VariableSnapshot[]>('get_variables'),
    
  getMemoryRegions: () => 
    invoke<MemoryRegion[]>('get_memory_regions'),
    
  getDiagnostics: () => 
    invoke<Diagnostic[]>('get_diagnostics'),
    
  getVisEvents: () => 
    invoke<VisEvent[]>('get_vis_events'),
    
  getAlgorithmMatches: () => 
    invoke<AlgorithmMatch[]>('get_algorithm_matches'),
    
  getCallStack: () => 
    invoke<CallStackFrame[]>('get_callstack'),
    
  addBreakpoint: (line: number) => 
    invoke<void>('add_breakpoint', { line }),
    
  clearBreakpoints: () => 
    invoke<void>('clear_breakpoints'),
    
  setInput: (input: string) => 
    invoke<void>('set_input', { input }),
    
  provideInputLine: (line: string) => 
    invoke<void>('provide_input_line', { line }),
    
  isWaitingInput: () => 
    invoke<boolean>('is_waiting_input'),
    
  getCurrentLine: () => 
    invoke<number>('get_current_line'),
    
  saveSession: (filepath: string) => 
    invoke<void>('save_session', { filepath }),
    
  loadSession: (filepath: string) => 
    invoke<void>('load_session', { filepath }),
};
```

### 6.3 状态管理：Zustand

```typescript
import { create } from 'zustand';
import { bridge } from '../bridge';

interface PanelElement {
  id: string;
  container: 'bottom-panel' | 'floating-ball';
  order: number;
}

interface AppState {
  // 编辑器
  sourceCode: string;
  setSourceCode: (code: string) => void;
  
  // 编译状态
  isCompiled: boolean;
  diagnostics: Diagnostic[];
  algorithmMatches: AlgorithmMatch[];
  
  // 执行状态
  isRunning: boolean;
  isStepping: boolean;
  isWaitingInput: boolean;
  currentLine: number;
  consoleOutput: string;
  executionSpeed: number;
  
  // 调试数据
  variables: VariableSnapshot[];
  memoryRegions: MemoryRegion[];
  callStack: CallStackFrame[];
  visEvents: VisEvent[];
  
  // 面板状态（9 元素统合）
  bottomPanel: {
    height: number;
    isOpen: boolean;
    elements: PanelElement[];
    activeId: string | null;
  };
  floatingBall: {
    position: { x: number; y: number };
    isExpanded: boolean;
    expandDirection: 'up' | 'down';
    elements: PanelElement[];
  };
  
  // Actions
  compile: () => Promise<void>;
  run: () => Promise<void>;
  stepNext: () => Promise<void>;
  stop: () => void;
  submitInput: (line: string) => Promise<void>;
  
  // Editor actions
  insertTemplate: (text: string) => void;
  insertPair: (open: string, close: string) => void;
  moveCursor: (delta: number) => void;
  undo: () => void;
  redo: () => void;
}

export const useAppStore = create<AppState>((set, get) => ({
  sourceCode: '// 输入 C 代码...\n',
  isCompiled: false,
  diagnostics: [],
  algorithmMatches: [],
  isRunning: false,
  isStepping: false,
  isWaitingInput: false,
  currentLine: 0,
  consoleOutput: '',
  executionSpeed: 0,
  variables: [],
  memoryRegions: [],
  callStack: [],
  visEvents: [],
  
  bottomPanel: {
    height: 200,
    isOpen: true,
    elements: [
      { id: 'output', container: 'bottom-panel', order: 0 },
      { id: 'diagnostics', container: 'bottom-panel', order: 1 },
      { id: 'algorithm', container: 'bottom-panel', order: 2 },
    ],
    activeId: 'output',
  },
  
  floatingBall: {
    position: { x: 20, y: 100 },
    isExpanded: false,
    expandDirection: 'up',
    elements: [
      { id: 'knowledge-card', container: 'floating-ball', order: 0 },
      { id: 'pointer-view', container: 'floating-ball', order: 1 },
      { id: 'array-viz', container: 'floating-ball', order: 2 },
      { id: 'memory-region', container: 'floating-ball', order: 3 },
      { id: 'local-vars', container: 'floating-ball', order: 4 },
      { id: 'watch-vars', container: 'floating-ball', order: 5 },
      { id: 'call-stack', container: 'floating-ball', order: 6 },
    ],
  },
  
  compile: async () => {
    const result = await bridge.compile(get().sourceCode);
    set({ 
      isCompiled: result.success, 
      diagnostics: result.diagnostics,
      algorithmMatches: result.algorithm_matches,
    });
  },
  
  run: async () => {
    set({ isRunning: true, consoleOutput: '' });
    const result = await bridge.run();
    set({ 
      isRunning: false, 
      consoleOutput: result.output,
      isWaitingInput: result.waiting_input,
    });
    if (result.error) {
      set(state => ({ 
        consoleOutput: state.consoleOutput + '\n[错误] ' + result.error 
      }));
    }
  },
  
  stepNext: async () => {
    const result = await bridge.stepNext();
    set({ 
      currentLine: result.current_line,
      consoleOutput: result.output,
      isWaitingInput: result.waiting_input,
    });
    const [vars, mem, stack, events] = await Promise.all([
      bridge.getVariables(),
      bridge.getMemoryRegions(),
      bridge.getCallStack(),
      bridge.getVisEvents(),
    ]);
    set({ variables: vars, memoryRegions: mem, callStack: stack, visEvents: events });
  },
  
  stop: () => {
    // 停止执行逻辑
    set({ isRunning: false, isStepping: false });
  },
  
  submitInput: async (line: string) => {
    await bridge.provideInputLine(line);
    // 继续执行
    const result = await bridge.run();
    set({ 
      consoleOutput: result.output,
      isWaitingInput: result.waiting_input,
    });
  },
  
  insertTemplate: (text: string) => {
    // CodeMirror API 插入文本
  },
  
  insertPair: (open: string, close: string) => {
    // CodeMirror API 插入成对符号
  },
  
  moveCursor: (delta: number) => {
    // CodeMirror API 移动光标
  },
  
  undo: () => {
    // CodeMirror API
  },
  
  redo: () => {
    // CodeMirror API
  },
}));
```

### 6.4 CodeMirror 6 直接引入

```tsx
// components/Editor/CodeMirror.tsx
import { useEffect, useRef } from 'react';
import { EditorView, basicSetup } from 'codemirror';
import { cpp } from '@codemirror/lang-cpp';
import { oneDark } from '@codemirror/theme-one-dark';
import { autocompletion } from '@codemirror/autocomplete';
import { useAppStore } from '../../stores/appStore';

export function CodeMirrorEditor() {
  const ref = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView>();
  const { sourceCode, setSourceCode, isDarkMode } = useAppStore();
  
  useEffect(() => {
    if (!ref.current) return;
    
    const view = new EditorView({
      doc: sourceCode,
      extensions: [
        basicSetup,
        cpp(),
        isDarkMode ? oneDark : [],
        autocompletion({ override: [cideCompletions] }),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            setSourceCode(update.state.doc.toString());
          }
        }),
      ],
      parent: ref.current,
    });
    
    viewRef.current = view;
    return () => view.destroy();
  }, []);
  
  return <div ref={ref} className="codemirror-wrapper" />;
}
```

### 6.5 9 元素统合与拖拽

```typescript
// 统一的数据模型
interface DockElement {
  id: ElementId;
  container: 'bottom-panel' | 'floating-ball';
  order: number;
}

type ElementId = 
  | 'output' | 'diagnostics' | 'algorithm'
  | 'knowledge-card' | 'pointer-view' | 'array-viz'
  | 'memory-region' | 'local-vars' | 'watch-vars' | 'call-stack';
```

拖拽使用 @dnd-kit：

```tsx
<DndContext onDragEnd={handleDragEnd}>
  <BottomPanel>
    <SortableContext items={bottomElements}>
      {bottomElements.map(el => <SortablePanel key={el.id} id={el.id} />)}
    </SortableContext>
  </BottomPanel>
  
  <FloatingBall>
    {isExpanded && (
      <SortableContext items={ballElements}>
        {ballElements.map(el => <SortablePanel key={el.id} id={el.id} />)}
      </SortableContext>
    )}
  </FloatingBall>
</DndContext>
```

---

## 7. 虚拟键盘与 SymbolBar

### 7.1 方案概述

将现有的特殊符号快捷栏（`symbol-bar`）粘在虚拟键盘上方，采用 **纯前端自适应 + Tauri Android 原生优化** 的组合方案。

### 7.2 核心思路

不试图"追到键盘上方"，而是让**编辑器区域高度 = 可视区域高度 - 工具栏高度 - SymbolBar 高度**。键盘弹出时 `visualViewport` 缩小，编辑器收缩，SymbolBar 始终贴在编辑器底部——视觉上就是键盘上方。

```
┌─────────────────┐  ← Toolbar (固定 48px)
│  ▶ ■ ⏭  [===]  │
├─────────────────┤
│                 │
│   CodeMirror    │  ← 弹性高度：占满剩余空间
│   Editor        │
│                 │
├─────────────────┤
│ { } ( ) [ ] ;   │  ← SymbolBar (固定 44px)
│ -> & * = == ... │     键盘弹出时，它就在这里
└─────────────────┘      ↓
                    ┌──────────┐
                    │  软键盘   │  ← 系统键盘
                    └──────────┘
```

### 7.3 键盘高度 Hook

```typescript
// hooks/useKeyboard.ts
import { useState, useEffect } from 'react';

interface KeyboardInfo {
  height: number;
  visible: boolean;
  viewportHeight: number;
}

export function useKeyboard(): KeyboardInfo {
  const [info, setInfo] = useState<KeyboardInfo>({
    height: 0,
    visible: false,
    viewportHeight: window.innerHeight,
  });

  useEffect(() => {
    if (!('visualViewport' in window)) {
      const onResize = () => {
        const height = window.innerHeight;
        const fullHeight = window.screen.height;
        setInfo(prev => ({
          ...prev,
          height: Math.max(0, fullHeight - height),
          visible: fullHeight - height > 150,
          viewportHeight: height,
        }));
      };
      window.addEventListener('resize', onResize);
      return () => window.removeEventListener('resize', onResize);
    }

    const vv = window.visualViewport!;
    
    const update = () => {
      const keyboardHeight = Math.max(0, window.innerHeight - vv.height);
      setInfo({
        height: keyboardHeight,
        visible: keyboardHeight > 100,
        viewportHeight: vv.height,
      });
    };

    vv.addEventListener('resize', update);
    vv.addEventListener('scroll', update);
    window.addEventListener('scroll', update);
    update();

    return () => {
      vv.removeEventListener('resize', update);
      vv.removeEventListener('scroll', update);
      window.removeEventListener('scroll', update);
    };
  }, []);

  return info;
}
```

### 7.4 根布局组件

```tsx
// components/Layout/AppLayout.tsx
import { useKeyboard } from '../../hooks/useKeyboard';
import { Toolbar } from '../Toolbar/Toolbar';
import { CodeMirrorEditor } from '../Editor/CodeMirror';
import { SymbolBar } from '../Toolbar/SymbolBar';
import { BottomPanel } from './BottomPanel';
import { FloatingBall } from './FloatingBall';
import { useAppStore } from '../../stores/appStore';

export function AppLayout() {
  const { viewportHeight, visible: keyboardVisible } = useKeyboard();
  const { bottomPanel, isWaitingInput } = useAppStore();

  const toolbarHeight = 48;
  const symbolBarHeight = 44;
  const bottomPanelHeight = bottomPanel.isOpen ? bottomPanel.height : 0;
  
  const editorHeight = isWaitingInput && keyboardVisible
    ? viewportHeight - toolbarHeight - symbolBarHeight
    : viewportHeight - toolbarHeight - symbolBarHeight - bottomPanelHeight;

  return (
    <div className="app-layout">
      <Toolbar />
      
      <div 
        className="editor-section"
        style={{ height: editorHeight }}
      >
        <CodeMirrorEditor />
        <SymbolBar />
      </div>

      {!keyboardVisible && <BottomPanel />}
      <FloatingBall />
    </div>
  );
}
```

### 7.5 CSS 布局（关键）

```css
/* styles/layout.css */
html, body, #root {
  margin: 0;
  padding: 0;
  height: 100%;
  overflow: hidden;
}

.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  height: 100dvh;       /* 动态视口，排除地址栏/导航栏 */
  overflow: hidden;
}

.toolbar {
  flex-shrink: 0;
  height: 48px;
}

.editor-section {
  display: flex;
  flex-direction: column;
  min-height: 0;        /* 关键：允许 flex 子项在容器缩小时收缩 */
  overflow: hidden;
}

.cm-wrapper {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.cm-editor {
  height: 100%;
}

.symbol-bar {
  flex-shrink: 0;
  height: 44px;
  display: flex;
  align-items: center;
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
  scrollbar-width: none;
}

.symbol-bar::-webkit-scrollbar {
  display: none;
}

.symbol-btn {
  flex-shrink: 0;
  padding: 0 12px;
  height: 32px;
  border: none;
  background: var(--symbol-btn-bg);
  color: var(--symbol-btn-text);
  border-radius: 6px;
  margin: 0 2px;
  font-family: monospace;
  font-size: 14px;
}

.symbol-btn:active {
  background: var(--symbol-btn-active);
}

.symbol-divider {
  width: 1px;
  height: 24px;
  background: var(--border-color);
  margin: 0 6px;
  flex-shrink: 0;
}
```

### 7.6 SymbolBar 组件

```tsx
// components/Toolbar/SymbolBar.tsx
import { useAppStore } from '../../stores/appStore';

const SYMBOLS = [
  { label: '{ }', insert: '{', pair: '}' },
  { label: '( )', insert: '(', pair: ')' },
  { label: '[ ]', insert: '[', pair: ']' },
  { label: '" "', insert: '"', pair: '"' },
  { label: "' '", insert: "'", pair: "'" },
  { label: ';', insert: ';' },
  { label: '#', insert: '#' },
  { label: '->', insert: '->' },
  { label: '&', insert: '&' },
  { label: '*', insert: '*' },
  { label: '=', insert: '=' },
  { label: '==', insert: '==' },
  { label: '!=', insert: '!=' },
  { label: '<', insert: '<' },
  { label: '>', insert: '>' },
  { label: '+', insert: '+' },
  { label: '-', insert: '-' },
  { label: '/', insert: '/' },
  { label: '%', insert: '%' },
  { label: '&&', insert: '&&' },
  { label: '||', insert: '||' },
  { label: '!', insert: '!' },
  { label: '|', insert: '|' },
  { label: '^', insert: '^' },
  { label: '~', insert: '~' },
  { label: ',', insert: ',' },
  { label: '.', insert: '.' },
];

const ACTIONS = [
  { label: '←', action: 'moveLeft' as const },
  { label: '→', action: 'moveRight' as const },
  { label: 'Tab', action: 'tab' as const },
  { label: '↩', action: 'undo' as const },
  { label: '↪', action: 'redo' as const },
];

export function SymbolBar() {
  const { insertTemplate, insertPair, moveCursor, undo, redo } = useAppStore();

  const handleSymbol = (sym: typeof SYMBOLS[0]) => {
    if (sym.pair) {
      insertPair(sym.insert, sym.pair);
    } else {
      insertTemplate(sym.insert);
    }
  };

  return (
    <div className="symbol-bar" id="symbol-bar">
      <div className="symbol-scroll">
        {SYMBOLS.map((sym) => (
          <button
            key={sym.label}
            className="symbol-btn"
            tabIndex={-1}
            onMouseDown={(e) => {
              e.preventDefault();
              handleSymbol(sym);
            }}
          >
            {sym.label}
          </button>
        ))}
        <div className="symbol-divider" />
        {ACTIONS.map((act) => (
          <button
            key={act.label}
            className="symbol-btn symbol-action"
            tabIndex={-1}
            onMouseDown={(e) => {
              e.preventDefault();
              switch (act.action) {
                case 'moveLeft': moveCursor(-1); break;
                case 'moveRight': moveCursor(1); break;
                case 'tab': insertTemplate('    '); break;
                case 'undo': undo(); break;
                case 'redo': redo(); break;
              }
            }}
          >
            {act.label}
          </button>
        ))}
      </div>
    </div>
  );
}
```

**关键点**：`onMouseDown` + `e.preventDefault()` 确保点击符号按钮时 CodeMirror 不会失焦。

### 7.7 Tauri Android 原生优化（备用）

如果纯前端方案在某些机型（华为、小米旧版 WebView）上不准确，启用原生优化。

修改 `src-tauri/gen/android/app/src/main/AndroidManifest.xml`：

```xml
<activity
    android:name=".MainActivity"
    android:windowSoftInputMode="adjustResize"
    android:exported="true">
    ...
</activity>
```

`adjustResize` 确保键盘弹出时 WebView 会 resize 而不是整个页面上推。

---

## 8. 移动端适配

### 8.1 Tauri Android 配置

`src-tauri/tauri.conf.json`：

```json
{
  "identifier": "com.cide.app",
  "build": {
    "frontendDist": "../../frontend/dist",
    "devUrl": "http://localhost:5173"
  },
  "app": {
    "windows": [],
    "security": {
      "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline';"
    }
  },
  "bundle": {
    "android": {
      "minSdkVersion": 24,
      "targetSdkVersion": 34
    }
  }
}
```

### 8.2 触摸优化

```css
/* styles/touch.css */
.cm-editor {
  touch-action: pan-x pan-y;
  -webkit-overflow-scrolling: touch;
}

.panel-resize-handle {
  height: 12px;
  touch-action: none;
  cursor: ns-resize;
}

.floating-ball {
  touch-action: none;
  user-select: none;
  -webkit-user-select: none;
}
```

### 8.3 响应式布局（手机/平板/桌面）

```typescript
// hooks/useDeviceForm.ts
import { useState, useEffect } from 'react';

export type DeviceForm = 'phone' | 'tablet' | 'desktop';

export function useDeviceForm(): DeviceForm {
  const [form, setForm] = useState<DeviceForm>('phone');
  
  useEffect(() => {
    const check = () => {
      const width = window.innerWidth;
      const height = window.innerHeight;
      
      if (width >= 1024 && height >= 700) {
        setForm('desktop');
      } else if (width >= 768 && height >= 600) {
        setForm('tablet');
      } else {
        setForm('phone');
      }
    };
    
    window.addEventListener('resize', check);
    check();
    return () => window.removeEventListener('resize', check);
  }, []);
  
  return form;
}
```

| 形态 | 布局特征 |
|------|---------|
| 小手机 | 单栏，底部面板固定高度，悬浮球默认收起 |
| 大手机/小平板 | 底部面板可拉伸到半屏，悬浮球可 dock 到右侧 |
| 平板横屏 | 左侧编辑器 55%，右侧底部面板固定（变成侧边面板），悬浮球隐藏或变成右侧工具栏 |
| 桌面 | 三栏或编辑器 + 侧边调试面板 |

---

## 9. 构建与 CI/CD

### 9.1 本地开发流程

```bash
# 1. 启动前端 dev server
$ cd frontend && npm run dev

# 2. 新终端：启动 Tauri Desktop（自动连接前端 dev server）
$ cd src-tauri && cargo tauri dev

# 3. Android 真机/模拟器（前端热重载正常工作）
$ cd src-tauri && cargo tauri android dev
```

### 9.2 生产构建

```bash
# Desktop (Windows)
$ cd src-tauri && cargo tauri build

# Android APK
$ cd src-tauri && cargo tauri android build --apk
# 输出: src-tauri/gen/android/app/build/outputs/apk/release/app-release.apk
```

### 9.3 CI/CD 改造

`.github/workflows/ci.yml`：

```yaml
name: CI

on:
  push:
    branches: [main, master]
  pull_request:
    branches: [main, master]

jobs:
  rust:
    name: Rust Build & Test
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: aarch64-linux-android,armv7-linux-androideabi
      
      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: |
            native
            src-tauri
      
      - name: Build native
        run: cargo build --release
        working-directory: native
      
      - name: Test native
        run: cargo test
        working-directory: native
      
      - name: Clippy native
        run: cargo clippy -- -D warnings
        working-directory: native

  frontend:
    name: Frontend Lint & Build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      
      - name: Install deps
        run: npm ci
        working-directory: frontend
      
      - name: Lint
        run: npm run lint
        working-directory: frontend
      
      - name: Type check
        run: npm run typecheck
        working-directory: frontend
      
      - name: Build
        run: npm run build
        working-directory: frontend

  tauri-desktop:
    name: Tauri Desktop Build
    needs: [rust, frontend]
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Build Tauri
        uses: tauri-apps/tauri-action@v0
        with:
          projectPath: ./src-tauri
          args: --target x86_64-pc-windows-msvc

  tauri-android:
    name: Tauri Android Build
    needs: [rust, frontend]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup JDK
        uses: actions/setup-java@v4
        with:
          distribution: 'temurin'
          java-version: '17'
      
      - name: Setup Android SDK
        uses: android-actions/setup-android@v3
      
      - name: Install NDK
        run: sdkmanager "ndk;25.2.9519653"
      
      - name: Build Android
        uses: tauri-apps/tauri-action@v0
        with:
          projectPath: ./src-tauri
          args: --target aarch64-linux-android
```

---

## 10. 迁移路线图（10 周）

### Phase 0：环境搭建（Week 1）

| 任务 | 产出 | 验收标准 |
|------|------|---------|
| 安装 Tauri CLI | `cargo tauri --version` 有输出 | ✅ |
| 初始化 `src-tauri/` | `cargo tauri init` 完成 | ✅ |
| 配置 `src-tauri/Cargo.toml` 引用 `cide_native` | `cargo check` 通过 | ✅ |
| 初始化 `frontend/`（Vite + React + TS） | `npm run dev` 启动 | ✅ |
| 验证 Tauri 加载前端 dev server | 空窗口显示 "Hello Tauri" | ✅ |
| 验证 `cargo tauri android dev` | Android 模拟器启动 | ✅ |

### Phase 1：Rust 桥接层（Week 2）

| 任务 | 产出 | 验收标准 |
|------|------|---------|
| 创建 `compiler_bridge.rs` | 提取 compile/run/step 逻辑 | 与 capi 行为一致 |
| 实现 20 个 Tauri command | `src-tauri/src/lib.rs` | 全部可调用 |
| 编写简单 JS 测试页 | 验证 compile / run / step | C 程序编译运行通过 |
| **里程碑** | Tauri 能完整编译运行 C 程序 | ✅ |

### Phase 2：前端骨架（Week 3-4）

| 任务 | 产出 | 验收标准 |
|------|------|---------|
| 搭建 `frontend/src/` 目录结构 | 组件目录就位 | ✅ |
| 实现 `bridge.ts` 封装 | 所有 API 有 TS 类型 | ✅ |
| 实现 Zustand store | 编译、执行、调试状态 | ✅ |
| 集成 CodeMirror 6 | 基本编辑 + 主题切换 | ✅ |
| 实现 Toolbar | 运行/停止/单步/速度滑块 | ✅ |
| 实现 SymbolBar / TemplateBar | 符号按钮可点击 | ✅ |
| 实现虚拟键盘适配 | SymbolBar 粘在键盘上方 | ✅ |
| **里程碑** | 前端功能对齐现有 Blazor 50% | ✅ |

### Phase 3：功能完全对齐（Week 5-6）

| 任务 | 产出 | 验收标准 |
|------|------|---------|
| BottomPanel（Output / Diagnostics / Algorithm） | 三标签面板 | 与现有行为一致 |
| FloatingBall（radial menu + 拖拽） | 悬浮球可拖拽 | ✅ |
| Debug Modal / 所有调试视图 | 内存、变量、调用栈等 | 数据正确显示 |
| Canvas 可视化 | linked-list, memory-map | 图形渲染正常 |
| 主题切换、知识卡片、算法验证 | 暗色/亮色切换 | ✅ |
| 断点管理 | CM6 装饰 + 桥接 | 断点命中正确 |
| **里程碑** | 功能 100% 对齐现有版本 | ✅ |

### Phase 4：9 元素统合 + 拖拽交互（Week 7-8）

| 任务 | 产出 | 验收标准 |
|------|------|---------|
| 统合 9 个元素的数据模型 | Zustand 状态更新 | ✅ |
| 底部面板拖拽拉伸 | 标题栏拖拽调整高度 | 流畅 60fps |
| @dnd-kit 跨容器拖拽 | 底部 ↔ 悬浮球拖拽 | 触摸 + 鼠标 |
| 双击删除/转移逻辑 | 底部→悬浮球，反之亦然 | 悬浮球上限提示 |
| 元素左右拖拽交换 | 同容器内排序 | ✅ |
| **里程碑** | 7 个新交互需求全部实现 | ✅ |

### Phase 5：平板适配 + 智能补全（Week 9）

| 任务 | 产出 | 验收标准 |
|------|------|---------|
| 响应式布局 | 手机/平板形态切换 | 布局正确 |
| CodeMirror autocomplete | 上下文感知补全 | 基于编译符号 |
| 符号表从编译结果提取 | 变量/函数名补全 | ✅ |
| **里程碑** | 平板体验流畅 | ✅ |

### Phase 6：打磨 + 发布（Week 10）

| 任务 | 产出 | 验收标准 |
|------|------|---------|
| Android 真机测试 | 多厂商测试通过 | 华为/小米/OPPO/vivo |
| 性能优化 | 虚拟列表、Canvas 按需渲染 | 低端机流畅 |
| 删除旧 MAUI 代码 | Cide.Client.Maui 移除 | 构建通过 |
| CI/CD 切换 | GitHub Actions 新流程 | 全部绿 |
| **里程碑** | Tauri 版本上线 | ✅ |

---

## 11. 风险清单

| 风险 | 概率 | 影响 | 应对措施 |
|------|------|------|---------|
| Tauri Android 构建链踩坑 | 中 | 高 | Week 1 专门验证 `cargo tauri android dev`，出问题立即回退到 MAUI WebView |
| `compiler_bridge.rs` 与 `capi` 行为不一致 | 中 | 高 | Phase 1 用同一套测试用例两边对比验证 |
| CodeMirror 6 在 Android WebView 触摸异常 | 低 | 中 | 当前 MAUI 已有 workaround，直接迁移到 Tauri |
| 低端机性能（Tauri IPC 频率） | 低 | 中 | 单步执行时批量获取数据（一次 invoke 返回所有调试信息） |
| 团队 Rust 能力不足 | 低 | 中 | `src-tauri/src/lib.rs` 只有 400 行，语法简单，有 Rust 后端团队支撑 |
| Tauri Mobile 长期维护 | 低 | 中 | Tauri v2 已 GA，社区活跃；最坏情况可切回 MAUI WebView（前端代码复用 95%） |
| 虚拟键盘在特定机型错位 | 低 | 中 | 备用方案 B（Kotlin WindowInsets 桥接）随时启用 |

---

## 12. 附录：决策对比表

### 12.1 三种前端方案终极对比

| 维度 | 保留 MAUI + Blazor | MAUI WebView + TS | Tauri + TS |
|------|-------------------|-------------------|-----------|
| 前端自由度 | 低 | 高 | 高 |
| 后端桥接复杂度 | 低 | 中 | 低 |
| 打包复杂度 | 低 | 低 | 中 |
| APK 体积 | ~25-35 MB | ~25-35 MB | ~10-15 MB |
| 启动速度 | 中 | 中 | 快 |
| 真机调试 | 极好 | 极好 | 差 |
| 构建速度 | 快 | 快 | 慢 |
| CI 改造 | 0.5 周 | 0.5 周 | 2 周 |
| 团队切换成本 | 低 | 低 | 中 |
| 长期架构健康度 | 差 | 中 | 极好 |
| **推荐度** | ❌ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |

### 12.2 前端技术选型对比

| 需求 | 选型 | 备选 | 不选 |
|------|------|------|------|
| UI 框架 | React 18 + TS | Vue 3 + TS | Svelte（团队学习成本） |
| 构建工具 | Vite | Rollup | Webpack（慢） |
| 状态管理 | Zustand | Jotai | Redux（重） |
| 拖拽 | @dnd-kit/core | react-beautiful-dnd | 手写（成本高） |
| 动画 | Framer Motion | CSS transitions | GSAP（重） |
| 编辑器 | CodeMirror 6 | Monaco Editor | 其他（CM6 移动支持最好） |

---

## 13. 立即开始

如果决定推进，第一步：

```bash
# 1. 安装 Tauri CLI
$ cargo install tauri-cli

# 2. 初始化 src-tauri（在项目根目录）
$ cargo tauri init
# 回答提示：
# - app name: cide
# - window title: Cide
# - frontend dist: ../frontend/dist
# - dev server url: http://localhost:5173

# 3. 配置 src-tauri/Cargo.toml 添加 cide_native 依赖
$ cat >> src-tauri/Cargo.toml << 'EOF'
[dependencies]
cide_native = { path = "../native" }
EOF

# 4. 验证编译
$ cd src-tauri && cargo check
```

**下一步行动**：由 Rust 后端团队负责 Phase 0-1（Tauri 骨架 + compiler_bridge），前端团队并行准备 Phase 2（React 项目初始化）。
