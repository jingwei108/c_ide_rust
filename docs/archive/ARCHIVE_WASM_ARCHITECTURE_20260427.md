# WASM 架构深度分析

> 核心定位：**WASM 不是前端运行平台，而是后端 C++ 编译器的编译目标格式与执行沙盒容器**
> 运行时：**wasm3**（轻量级 WASM 解释器，嵌入 Native C++ 后端）

---

## 1. 架构重定义

### 1.1 之前的误解 vs 正确理解

| 维度 | 误解（Avalonia.Browser 方案） | 正确理解（wasm3 沙盒方案） |
|:---|:---|:---|
| **前端平台** | Avalonia.Browser（WASM） | Avalonia Android / Desktop（Native） |
| **WASM 角色** | 前端 .NET 运行平台 | 后端用户代码的编译目标 + 执行沙盒 |
| **后端形态** | C++ 编译为 WASM，被浏览器加载 | C++ 编译为 Native DLL/.so，内部嵌入 wasm3 |
| **通信方式** | JSImport/JSExport（复杂且不稳定） | **统一 P/Invoke C API**（与 Android/Desktop 完全一致） |
| **内存安全** | 依赖浏览器沙盒 | wasm3 线性内存隔离 + 越界自动 trap |
| **执行控制** | 困难（浏览器事件循环限制） | **精确控制**（步进、暂停、内存快照） |

### 1.2 正确架构图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     C# Avalonia 前端 (Android / Desktop)                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────────────────┐  │
│  │ CodeEditor  │  │ MemoryView  │  │ KnowledgeCard / QuickFixPanel       │  │
│  │ 代码编辑器   │  │  内存视图    │  │ 知识卡片 / 一键修复面板              │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────────────────┘  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────────────────┐  │
│  │ PointerView │  │ ErrorPanel  │  │ ConsoleOutput                       │  │
│  │  指针视图    │  │  错误面板    │  │ 输出控制台                           │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼ P/Invoke (统一 C API)
┌─────────────────────────────────────────────────────────────────────────────┐
│                     C++ 后端 (Native DLL / .so)                             │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ ① C 子集编译器 (手写)                                                │    │
│  │   用户 C 代码 → Lexer → Parser → AST → TypeChecker → WASM CodeGen   │    │
│  │   输出：WASM 字节码模块（线性内存 + 函数 + import/export）            │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    ↓                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ ② wasm3 运行时 (嵌入)                                                │    │
│  │   ┌─────────────────────────────────────────────────────────────┐   │    │
│  │   │ WASM 字节码模块                                               │   │    │
│  │   │ ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐   │   │    │
│  │   │ │  linear mem │  │   main()    │  │  import __cide_step │   │   │    │
│  │   │ │  (栈+堆+全局)│  │  (用户代码)  │  │  import __cide_out  │   │   │    │
│  │   │ └─────────────┘  └─────────────┘  └─────────────────────┘   │   │    │
│  │   └─────────────────────────────────────────────────────────────┘   │    │
│  │                              ↓                                        │    │
│  │   ┌─────────────────────────────────────────────────────────────┐   │    │
│  │   │ wasm3 解释器 (逐条指令解释执行)                                │   │    │
│  │   │ • m3_Call() 调用入口函数                                       │   │    │
│  │   │ • m3_GetMemory() 读取线性内存                                  │   │    │
│  │   │ • m3_GetErrorInfo() 捕获 trap                                  │   │    │
│  │   │ • 通过 __cide_step() import 实现断点                           │   │    │
│  │   └─────────────────────────────────────────────────────────────┘   │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                    ↓                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │ ③ 诊断与可视化引擎                                                    │    │
│  │   • Source Map（WASM PC → 源码行/列）                                │    │
│  │   • 内存布局元数据（变量名 → 地址/类型）                              │    │
│  │   • 指针追踪表（指针变量 → 目标地址）                                 │    │
│  │   • 错误码映射 → 中文消息生成                                         │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. 为什么选择 wasm3？

### 2.1 wasm3 特性

| 特性 | 说明 | 本项目收益 |
|:---|:---|:---|
| **纯 C 编写** | 无依赖，~50KB 代码 | 轻松嵌入 C++ 后端，无额外依赖 |
| **解释执行** | 逐条 WASM 指令解释 | 教学场景性能足够；可精确控制执行步数 |
| **跨平台** | x86, ARM, RISC-V, WASM itself | 后端 DLL/.so 可在任意平台运行 |
| **C API** | 清晰的 C API（`m3_Call`, `m3_GetMemory` 等） | 与后端 C++ 无缝集成 |
| **Trap 捕获** | 内存越界、除零等自动 trap | **沙盒安全**，错误不会崩溃宿主进程 |
| **Gas 机制** | 可限制执行指令数 | **防止无限循环**，教学场景必需 |
| **Import/Export** | 支持宿主函数导入、WASM 函数导出 | 实现 `__cide_step` 断点、`__cide_output` 输出 |

### 2.2 与其他 WASM 运行时的对比

| 运行时 | 体积 | 执行方式 | 嵌入复杂度 | 执行控制 | 适用性 |
|:---|:---|:---|:---|:---|:---|
| **wasm3** | ~50KB | 解释 | ⭐ 极低 | 精确步进 | ✅ **最佳** |
| Wasmtime (Bytecode Alliance) | ~10MB | JIT | 高 | 一般 | 过重，不适合移动端 |
| V8 (Chrome) | ~10MB+ | TurboFan JIT | 极高 | 困难 | 无法嵌入 Native 后端 |
| WAMR (WebAssembly Micro Runtime) | ~100KB | 解释/AOT | 中 | 较好 | 可选，但 wasm3 更轻 |
| Mono WASM (Blazor) | ~15MB | 解释/AOT | 高 | 受限 | 这是 .NET 运行时，不是用户代码沙盒 |

**结论**：wasm3 是教学 IDE 场景的最优解——足够轻量、足够可控、足够安全。

---

## 3. 核心模块设计

### 3.1 C 子集 → WASM 编译器

#### 3.1.1 WASM 模块结构

编译器生成的 WASM 模块固定包含以下段（Section）：

```wasm
(module
  ;; 1. Type Section: 函数签名类型
  (type $t0 (func (param i32) (result i32)))     ;; __cide_step(i32) -> i32
  (type $t1 (func (param i32 i32)))              ;; __cide_output(ptr, len)
  (type $t2 (func (result i32)))                 ;; main() -> i32
  
  ;; 2. Import Section: 从宿主导入的函数
  (import "env" "__cide_step" (func $step (type $t0)))
  (import "env" "__cide_output" (func $output (type $t1)))
  (import "env" "__cide_malloc" (func $malloc (type $t0)))
  (import "env" "__cide_free" (func $free (type $t0)))
  
  ;; 3. Function Section: 用户函数列表
  (func $main (type $t2) ...)
  (func $bubbleSort (type ...) ...)
  
  ;; 4. Memory Section: 线性内存
  (memory $mem 2)     ;; 初始 2 页 = 128KB
  
  ;; 5. Global Section: 全局变量
  (global $g0 (mut i32) (i32.const 0))   ;; 示例全局变量
  
  ;; 6. Export Section: 导出函数和内存
  (export "memory" (memory $mem))
  (export "main" (func $main))
  
  ;; 7. Code Section: 函数体（由 CodeGen 生成）
  ...
)
```

#### 3.1.2 内存布局设计

WASM 线性内存是一个连续的 `uint8_t` 数组，编译器按以下布局分配：

```
地址空间（WASM linear memory）
├─ 0x00000 ~ 0x00FFF: 保留区域（NULL 指针陷阱区，16KB）
│                     任何解引用 0x0000~0x0FFF 的指针都会触发越界 trap
│
├─ 0x01000 ~ 0x04FFF: 全局变量区（16KB）
│                     编译期为每个全局变量分配固定偏移
│
├─ 0x05000 ~ 0x0FFFF: 堆区（44KB 初始）
│                     malloc 从此区域分配，memory.grow 扩展
│                     宿主通过 __cide_malloc 管理，可追踪泄漏
│
├─ 0x10000 ~ 0x1FFFF: 栈区（64KB）
│                     函数调用时从高地址向低地址增长
│                     栈溢出会触发 trap
│
└─ 0x20000+: memory.grow 动态扩展区域
```

**关键设计**：
- **NULL 指针陷阱区**：0x0000~0x0FFF 保留，任何解引用都会 trap → 映射为"空指针解引用"错误
- **栈底固定**：栈从固定高地址开始，栈溢出可被捕获
- **堆由宿主管理**：`__cide_malloc` 是宿主导入函数，宿主记录分配元数据（变量名、大小、调用栈）

#### 3.1.3 代码生成示例

C 源代码：
```c
int main() {
    int a = 5;
    int* p = &a;
    *p = 10;
    return a;
}
```

生成的 WASM 指令（伪代码）：
```wasm
(func $main (result i32)
  ;; int a = 5;  (a 在栈帧偏移 0)
  i32.const 5
  local.set $l0          ;; l0 = a = 5
  
  ;; call __cide_step(1)  (第 1 行断点)
  i32.const 1
  call $__cide_step
  drop
  
  ;; int* p = &a;  (p 在栈帧偏移 4)
  global.get $__stack_ptr
  i32.const 0
  i32.add                ;; &a = stack_ptr + 0
  local.set $l1          ;; l1 = p = &a
  
  ;; call __cide_step(2)
  i32.const 2
  call $__cide_step
  drop
  
  ;; *p = 10;
  local.get $l1          ;; p
  i32.const 10
  i32.store              ;; *p = 10
  
  ;; call __cide_step(3)
  i32.const 3
  call $__cide_step
  drop
  
  ;; return a;
  local.get $l0          ;; a
)
```

### 3.2 wasm3 宿主集成

#### 3.2.1 宿主函数实现（C++）

```cpp
#include "wasm3.h"
#include "m3_env.h"
#include <string>
#include <vector>

class CideWasmRuntime {
public:
    // wasm3 核心对象
    M3Environment* env = nullptr;
    M3Runtime* runtime = nullptr;
    M3Module* module = nullptr;
    
    // 执行状态
    enum class State { Idle, Running, Paused, StepOver, Finished, Error };
    State state = State::Idle;
    int currentLine = 0;
    std::string lastError;
    std::vector<std::string> consoleOutput;
    
    // Source Map: WASM 指令偏移 → 源码行号
    std::vector<std::pair<uint32_t, int>> sourceMap;
    
    // 内存布局元数据: 变量名 → (地址, 类型, 大小)
    struct VarInfo { uint32_t addr; std::string type; size_t size; };
    std::vector<VarInfo> variables;
    
    // 指针追踪: 指针变量地址 → 目标地址
    std::map<uint32_t, uint32_t> pointerMap;

    // ---- 宿主函数（WASM import）----
    
    // __cide_step(line): 每执行一行调用
    static m3Err_t Host_Step(IM3Runtime rt, int64_t* stack, void* userData) {
        auto* self = static_cast<CideWasmRuntime*>(userData);
        int line = (int)stack[0];
        self->currentLine = line;
        
        if (self->state == State::StepOver) {
            self->state = State::Paused;  // 执行一次后暂停
            return m3Err_trapAbort;        // 通过 trap 暂停执行
        }
        // Running 模式：直接返回，继续执行
        return m3Err_none;
    }
    
    // __cide_output(ptr, len): 输出到控制台
    static m3Err_t Host_Output(IM3Runtime rt, int64_t* stack, void* userData) {
        auto* self = static_cast<CideWasmRuntime*>(userData);
        uint32_t ptr = (uint32_t)stack[0];
        uint32_t len = (uint32_t)stack[1];
        
        uint8_t* mem = m3_GetMemory(self->runtime, nullptr, 0);
        if (mem && ptr + len <= m3_GetMemorySize(self->runtime)) {
            std::string msg((char*)mem + ptr, len);
            self->consoleOutput.push_back(msg);
        }
        return m3Err_none;
    }
    
    // __cide_malloc(size): 宿主管理堆分配
    static m3Err_t Host_Malloc(IM3Runtime rt, int64_t* stack, void* userData) {
        auto* self = static_cast<CideWasmRuntime*>(userData);
        size_t size = (size_t)stack[0];
        
        // 在 WASM 内存中分配，记录元数据
        uint32_t addr = self->allocateHeap(size);
        stack[0] = addr;  // 返回地址
        return m3Err_none;
    }
    
    // __cide_free(addr): 宿主管理释放
    static m3Err_t Host_Free(IM3Runtime rt, int64_t* stack, void* userData) {
        auto* self = static_cast<CideWasmRuntime*>(userData);
        uint32_t addr = (uint32_t)stack[0];
        self->freeHeap(addr);
        return m3Err_none;
    }
};
```

#### 3.2.2 加载与执行流程

```cpp
// 后端 C API 实现
cide_session* cide_session_create() {
    auto* session = new cide_session();
    session->runtime = new CideWasmRuntime();
    
    // 创建 wasm3 环境
    session->runtime->env = m3_NewEnvironment();
    session->runtime->runtime = m3_NewRuntime(session->runtime->env, 64 * 1024, nullptr);
    
    return session;
}

int cide_compile(cide_session* s, const char* source) {
    // 1. Lexer + Parser + TypeChecker
    auto ast = parseSource(source);
    if (!ast.errors.empty()) {
        s->compileErrors = formatErrors(ast.errors);
        return -1;
    }
    
    // 2. WASM CodeGen
    auto wasmBytes = generateWASM(ast);
    s->wasmModule = wasmBytes;
    
    // 3. 记录 Source Map 和内存布局
    s->sourceMap = wasmBytes.sourceMap;
    s->memoryLayout = wasmBytes.memoryLayout;
    
    // 4. 加载到 wasm3
    M3Result result = m3_ParseModule(s->runtime->env, &s->runtime->module, 
                                      wasmBytes.data(), wasmBytes.size());
    if (result) return -1;
    
    result = m3_LoadModule(s->runtime->runtime, s->runtime->module);
    if (result) return -1;
    
    // 5. 链接宿主函数
    m3_LinkRawFunction(s->runtime->module, "env", "__cide_step", "i(i)", 
                       &CideWasmRuntime::Host_Step, s->runtime);
    m3_LinkRawFunction(s->runtime->module, "env", "__cide_output", "v(ii)", 
                       &CideWasmRuntime::Host_Output, s->runtime);
    m3_LinkRawFunction(s->runtime->module, "env", "__cide_malloc", "i(i)", 
                       &CideWasmRuntime::Host_Malloc, s->runtime);
    m3_LinkRawFunction(s->runtime->module, "env", "__cide_free", "v(i)", 
                       &CideWasmRuntime::Host_Free, s->runtime);
    
    return 0;
}

int cide_run(cide_session* s) {
    s->runtime->state = CideWasmRuntime::State::Running;
    s->runtime->consoleOutput.clear();
    
    M3Result result = m3_CallV(s->runtime->module, "main");
    
    if (result == m3Err_none) {
        s->runtime->state = CideWasmRuntime::State::Finished;
        return 0;
    }
    
    // 处理 trap
    if (result == m3Err_trapAbort && s->runtime->state == CideWasmRuntime::State::Paused) {
        // 这是 __cide_step 触发的正常暂停，不是错误
        return 0;
    }
    
    // 真正的运行时错误
    s->runtime->state = CideWasmRuntime::State::Error;
    s->runtime->lastError = mapTrapToChineseError(result, s->runtime->currentLine);
    return -1;
}

int cide_step_next(cide_session* s) {
    s->runtime->state = CideWasmRuntime::State::StepOver;
    M3Result result = m3_CallV(s->runtime->module, "main");
    // ... 类似 cide_run，但只执行一步
}
```

### 3.3 Source Map 与错误映射

#### 3.3.1 编译期 Source Map 生成

编译器在生成 WASM Code Section 时，同时生成 Source Map：

```cpp
struct SourceMapEntry {
    uint32_t wasmOffset;   // WASM Code Section 中的字节偏移
    int sourceLine;        // C 源代码行号
    int sourceColumn;      // C 源代码列号
};

class WASMCodeGen {
    std::vector<SourceMapEntry> sourceMap;
    uint32_t currentOffset = 0;  // 当前 WASM 输出偏移
    
    void emitInstruction(WasmOpcode op, int sourceLine) {
        sourceMap.push_back({currentOffset, sourceLine, 0});
        // 写入 opcode
        currentOffset += getInstructionSize(op);
    }
    
    void emitCall_cide_step(int line) {
        // i32.const line
        emitInstruction(WasmOpcode::I32_CONST, line);
        emitOperand(line);
        // call __cide_step
        emitInstruction(WasmOpcode::CALL, line);
        emitFuncIndex(stepFuncIndex);
        // drop
        emitInstruction(WasmOpcode::DROP, line);
    }
};
```

#### 3.3.2 运行时错误映射

```cpp
std::string mapTrapToChineseError(M3Result trap, int currentLine) {
    if (trap == m3Err_trapOutOfBoundsMemory) {
        // 根据当前 PC 查找 Source Map
        uint32_t pc = m3_GetPC(runtime);
        auto [line, col] = lookupSourceMap(pc);
        return format("第 %d 行，第 %d 列：内存访问越界。可能原因：\n"
                      "• 数组索引超出范围\n"
                      "• 解引用了无效指针\n"
                      "• 使用了已释放的内存（悬垂指针）", line, col);
    }
    if (trap == m3Err_trapDivisionByZero) {
        return format("第 %d 行：除零错误。除数不能为 0。", currentLine);
    }
    if (trap == m3Err_trapStackOverflow) {
        return format("第 %d 行：栈溢出。可能原因：无限递归或局部变量过大。", currentLine);
    }
    // ... 其他 trap 类型
}
```

### 3.4 内存视图实现

#### 3.4.1 通过 wasm3 C API 读取线性内存

```cpp
// C API: 获取内存区域数量
int cide_memory_region_count(cide_session* s) {
    return s->memoryLayout.variables.size() + s->memoryLayout.heapRegions.size();
}

// C API: 获取某个内存区域的值
void cide_memory_region_get(cide_session* s, int index, 
    uint32_t* addr, int* size, char* name, int nameSize,
    char* type, int typeSize, int* isHeap, int* isFreed) {
    
    // 获取 WASM 线性内存指针
    uint8_t* mem = m3_GetMemory(s->runtime->runtime, nullptr, 0);
    size_t memSize = m3_GetMemorySize(s->runtime->runtime);
    
    auto& region = s->memoryLayout.getRegion(index);
    *addr = region.addr;
    *size = region.size;
    strncpy(name, region.name.c_str(), nameSize);
    strncpy(type, region.type.c_str(), typeSize);
    *isHeap = region.isHeap;
    
    // 检查是否已释放（根据宿主的堆分配记录）
    *isFreed = region.isHeap && !s->runtime->isAllocated(region.addr);
}

// C API: 读取某个地址的 int 值
int cide_memory_get_value(cide_session* s, uint32_t addr, int* outVal) {
    uint8_t* mem = m3_GetMemory(s->runtime->runtime, nullptr, 0);
    size_t memSize = m3_GetMemorySize(s->runtime->runtime);
    
    if (addr + 4 > memSize) return -1;  // 越界
    
    // WASM 是小端序
    *outVal = mem[addr] | (mem[addr+1] << 8) | 
              (mem[addr+2] << 16) | (mem[addr+3] << 24);
    return 0;
}
```

#### 3.4.2 前端内存视图渲染

前端通过 P/Invoke 调用上述 C API，获取内存状态后，在 Avalonia Canvas 上绘制：

```csharp
// MemoryCanvas.axaml.cs
public override void Render(DrawingContext context) {
    int count = NativeMethods.cide_memory_region_count(session);
    
    for (int i = 0; i < count; i++) {
        NativeMethods.cide_memory_region_get(session, i, out var addr, 
            out var size, out var name, out var type, out var isHeap, out var isFreed);
        
        // 绘制内存块
        var color = isFreed ? Colors.Red : (isHeap ? Colors.Blue : Colors.Green);
        DrawMemoryBlock(context, addr, size, name, type, color);
        
        // 如果是指针类型，绘制箭头
        if (type == "int*") {
            NativeMethods.cide_memory_get_value(session, addr, out var targetAddr);
            if (targetAddr != 0) {
                DrawPointerArrow(context, addr, targetAddr);
            }
        }
    }
}
```

---

## 4. 单步调试与执行控制

### 4.1 实现原理

wasm3 本身不提供内置的单步调试 API。我们通过 **import hook** 实现：

```
C 源代码中的每条语句
  ↓
编译器在语句前后插入 call __cide_step(line)
  ↓
wasm3 执行到 call __cide_step 时，调用宿主函数
  ↓
宿主函数检查当前执行模式：
  • Running 模式：直接返回，继续执行
  • StepOver 模式：设置状态为 Paused，返回 trapAbort
  ↓
wasm3 捕获 trapAbort，停止执行
  ↓
宿主更新 UI（内存视图、当前行高亮），等待用户输入
```

### 4.2 执行模式状态机

```
          ┌───────────┐
          │   Idle    │
          └─────┬─────┘
                │ 用户点击 "运行"
                ▼
          ┌───────────┐
    ┌──── │  Running  │ ◄─────────────────────────────┐
    │     └─────┬─────┘                               │
    │           │ 遇到 __cide_step，StepOver 模式      │
    │           ▼                                      │
    │     ┌───────────┐     用户点击 "下一步"         │
    │     │  Paused   │ ──────────────────────────────┘
    │     └─────┬─────┘
    │           │ 用户点击 "停止"
    │           ▼
    │     ┌───────────┐
    └────►│  Finished │
    (trap)└───────────┘
                │
                ▼
          ┌───────────┐
          │   Error   │ ◄── trapOutOfBounds / trapDivByZero 等
          └───────────┘
```

### 4.3 与 2048 动画经验的结合

参考 2048 的闪退修复经验（`CancelAllAnimations + SnapTilesToGrid`），在单步调试场景中：

```csharp
// 用户快速点击"下一步"时
public void OnStepNext() {
    // 1. 取消前一步的 UI 动画
    CancelAllAnimations();
    
    // 2. 同步 UI 到最终状态（防止状态混乱）
    SnapMemoryViewToCurrentState();
    SnapPointerViewToCurrentState();
    
    // 3. 调用后端执行下一步
    NativeMethods.cide_step_next(session);
    
    // 4. 获取新状态并启动动画
    var newState = GetExecutionState();
    AnimateMemoryViewTransition(newState);
    AnimatePointerViewTransition(newState);
}
```

---

## 5. 与 VisualBinaryTree & 2048 的对比

| 维度 | VisualBinaryTree | 2048 | 本项目（wasm3 方案） |
|:---|:---|:---|:---|
| **用户代码执行** | C++ 子进程 / C# 解释器 | 无（纯游戏逻辑） | **WASM 沙盒（wasm3）** |
| **执行隔离** | 子进程隔离 / 无隔离 | N/A | **WASM 线性内存隔离** |
| **内存视图** | C# 解释器直接读取 `Value` | 无 | **通过 `m3_GetMemory()` 读取** |
| **错误捕获** | C# try-catch / 子进程退出码 | N/A | **WASM trap + Source Map** |
| **单步调试** | 不支持 | N/A | **`__cide_step` import hook** |
| **跨平台后端** | Windows DLL + 子进程 | DLL / .so | **统一 Native DLL，内部嵌入 wasm3** |
| **通信复杂度** | P/Invoke + 命名管道 | P/Invoke | **统一 P/Invoke（最简单）** |

**核心进步**：
1. 用户代码在 **WASM 沙盒** 中运行，错误不会崩溃宿主进程（比 VisualBinaryTree 的子进程方案更轻量）
2. **统一 P/Invoke 通信**，Android/Desktop 完全一致（比 VisualBinaryTree 的"P/Invoke + 管道"更简单）
3. **精确执行控制**（步进、暂停），VisualBinaryTree 不支持
4. **内存视图直接读取 WASM 线性内存**，无需像 C# 解释器那样维护复杂的 `Value` 对象树

---

## 6. 构建系统

### 6.1 目录结构更新

```
c-ide/
├── build.ps1                          # 一键构建
├── native/                            # C++ 后端
│   ├── CMakeLists.txt
│   ├── include/
│   │   └── cide_capi.h               # C API 头文件
│   ├── third_party/
│   │   └── wasm3/                     # wasm3 源码（git submodule 或拷贝）
│   │       ├── source/
│   │       │   ├── wasm3.h
│   │       │   ├── m3_env.h
│   │       │   ├── m3_compile.c       # wasm3 核心（C 文件）
│   │       │   └── ...
│   │       └── CMakeLists.txt
│   ├── src/
│   │   ├── compiler/                  # C 子集 → WASM 编译器
│   │   │   ├── Lexer.cpp
│   │   │   ├── Parser.cpp
│   │   │   ├── Ast.hpp
│   │   │   ├── TypeChecker.cpp
│   │   │   └── WasmCodeGen.cpp        # WASM 字节码生成
│   │   ├── runtime/                   # wasm3 宿主封装
│   │   │   ├── CideWasmRuntime.cpp    # 核心运行时
│   │   │   ├── SourceMap.cpp          # Source Map 管理
│   │   │   ├── MemoryTracker.cpp      # 堆分配追踪（泄漏检测）
│   │   │   └── PointerTracker.cpp     # 指针追踪
│   │   ├── diagnostics/               # 诊断系统
│   │   │   ├── ErrorCodes.hpp
│   │   │   ├── DiagnosticEngine.cpp
│   │   │   └── QuickFixGenerator.cpp
│   │   └── capi/
│   │       └── cide_capi.cpp
│   └── tests/
│       ├── LexerTest.cpp
│       ├── ParserTest.cpp
│       ├── WasmGenTest.cpp            # 验证生成的 WASM 可加载
│       └── RuntimeTest.cpp            # 验证 wasm3 执行正确
├── Cide.Client/                       # Avalonia 共享库
│   ├── Core/
│   │   ├── CompilerService.cs         # 封装 C API
│   │   ├── NativeMethods.cs           # DllImport
│   │   └── ...
│   └── Views/
│       ├── MemoryCanvas.axaml.cs      # 内存视图
│       ├── PointerCanvas.axaml.cs     # 指针视图
│       └── ...
├── Cide.Client.Android/
└── Cide.Client.Desktop/
```

### 6.2 CMake 配置要点

```cmake
# native/CMakeLists.txt
cmake_minimum_required(VERSION 3.20)
project(cide_native)

set(CMAKE_CXX_STANDARD 20)

# 1. 添加 wasm3 子目录（wasm3 是纯 C，需要用 C 编译器编译）
add_subdirectory(third_party/wasm3/source)

# 2. Cide 编译器与运行时库
file(GLOB_RECURSE CIDE_SOURCES src/*.cpp)
add_library(cide_native SHARED ${CIDE_SOURCES})

# 3. 链接 wasm3
target_link_libraries(cide_native PRIVATE m3)

# 4. 导出 C API
target_compile_definitions(cide_native PRIVATE CIDE_EXPORTS)
set_target_properties(cide_native PROPERTIES
    C_VISIBILITY_PRESET hidden
    CXX_VISIBILITY_PRESET hidden
)
```

**注意**：wasm3 是纯 C 项目，需要在 CMake 中正确处理 C/C++ 混合编译。

---

## 7. 风险与规避

| 风险 | 严重度 | 说明 | 规避方案 |
|:---|:---|:---|:---|
| **手写 WASM CodeGen 工程量大** | 高 | WASM 是栈机模型，CodeGen 比寄存器机复杂 | Phase 1 先实现最简单的子集（变量+算术+if+while）；WASM CodeGen 可借助 `wabt` 库的 `binary-writer`；或先生成 WAT（文本格式）再汇编 |
| **wasm3 不支持 WASM 全部特性** | 低 | wasm3 支持 WASM MVP + 部分扩展 | 教学子集只需要 MVP（i32, memory, call, import/export），完全够用 |
| **Source Map 精度** | 中 | WASM 是栈机，一条 C 语句可能对应多条 WASM 指令 | 只在 `__cide_step` 调用点记录 Source Map，不追踪每条指令 |
| **内存视图性能** | 低 | 每次渲染都 P/Invoke 读取内存 | 前端缓存内存状态，只在步进/执行后刷新；大内存区域虚拟化 |
| **wasm3 的 trapAbort 与正常执行流混淆** | 中 | `__cide_step` 用 trapAbort 暂停，但用户代码也可能触发 trapAbort | 检查 `state` 变量区分：Paused = 正常断点；Error = 真实错误 |
| **C 子集 malloc 实现** | 中 | WASM 没有内置 malloc | 由宿主提供 `__cide_malloc`，简化实现（固定块大小或首次适应算法） |

---

## 8. 关键技术决策总结

1. **WASM 是编译目标，不是前端平台** → 前端统一用 Avalonia Android/Desktop，通信统一用 P/Invoke
2. **wasm3 是执行沙盒** → 用户代码编译为 WASM，在 wasm3 中解释执行，安全隔离
3. **`__cide_step` import hook 实现单步调试** → 编译器在每条语句处插入调用，宿主控制暂停/继续
4. **`m3_GetMemory()` 实现内存视图** → 直接读取 WASM 线性内存，无需额外的序列化
5. **Source Map 实现错误映射** → 编译期记录 WASM 偏移 → 源码位置，运行期根据 PC 反查
6. **宿主管理堆分配** → `__cide_malloc/free` 是 import 函数，宿主记录元数据，支持泄漏检测

---

## 9. 下一步行动建议

1. **验证 wasm3 可嵌入性**：先写一个最小原型（C++ 加载 wasm3，执行一个简单的 `i32.add` WASM 模块，通过 P/Invoke 从 C# 调用）
2. **定义最小 C 子集**：确定 Phase 1 支持的语法（建议：变量声明、赋值、算术、if/else、while、函数调用）
3. **选择 WASM 生成策略**：
   - 方案 A：手写 WASM 二进制生成器（工作量大，但无依赖）
   - 方案 B：生成 WAT（文本格式），用 `wat2wasm` 工具转换（需要引入 wabt 工具链）
   - **推荐 Phase 1 用方案 B**，快速验证；后续再迁移到方案 A
4. **搭建最小端到端原型**：C# 编辑器 → P/Invoke → C++ 编译器（WAT）→ wat2wasm → wasm3 执行 → 返回结果 → C# 显示
