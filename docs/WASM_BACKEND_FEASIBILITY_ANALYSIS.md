# 后端 WASM 化可行性分析

> 背景：探讨将 `cide_native` 后端编译为 WebAssembly，或将用户 C 代码直接编译为 WASM 的架构可行性。

---

## 当前架构

```
用户 C 代码
    ↓
[Lexer → Parser → TypeChecker → BytecodeGen]  (cide_native)
    ↓
CideVM 字节码（自定义指令集 + 256KB 线性内存）
    ↓
CideVM 解释执行（单步/断点/VisEvent 发射）
    ↓
P/Invoke (cide_capi.h) ←→ .NET 前端
```

**核心优势**：
- 完全受控的 C 子集（去除了指针算术、宏、union 等初学者陷阱）
- CideVM 可注入 `VisEvent`（Compare/Swap/Update）用于零侵入算法可视化
- 中文诊断系统（错误代码 E2xxx/E3xxx + 知识卡片匹配）
- 内存布局完全可控（NullTrap、Global、Heap、Stack 区域）

---

## 方案对比

### 方案 A：整个 `cide_native` 编译为 WASM

使用 Emscripten 将 C++20 后端（编译器 + VM）编译为 WASM 模块，前端通过 JS interop 调用。

```
用户 C 代码
    ↓
[Lexer → Parser → TypeChecker → BytecodeGen → CideVM]  (WASM 模块内部)
    ↓
JS Interop ←→ Blazor ←→ .NET MAUI
```

**优点**：
| 维度 | 说明 |
|---|---|
| 纯客户端 | 无需服务器，浏览器内完整运行 |
| 跨平台一致性 | 桌面/Web/移动端共享同一 WASM 模块 |
| 沙盒安全 | WASM 内存隔离天然防止 VM 越界影响宿主 |

**缺点**：
| 维度 | 说明 |
|---|---|
| 构建链复杂度 | 需引入 Emscripten，与现有 CMake+Clang 构建链并行维护 |
| 双重内存模型 | CideVM 内部已有 256KB 线性内存，WASM 也有 4GB 线性内存，两者嵌套增加心智负担 |
| 调试复杂度 | WASM 内的 CideVM 单步调试需要 source map → WASM → VM 字节码 三层映射 |
| 性能损耗 | VM 解释器在 WASM 中运行 = 双重虚拟化，预计比原生慢 30-50% |
| P/Invoke 重构成本 | 需将所有 `cide_capi.h` 接口改为 JS 导出函数，再封装为 .NET JS interop |
| 体积 | Emscripten 生成的 WASM + JS glue 约 500KB-2MB，加上 CideVM 内存模型 |

**适用场景**：
- 未来推出**纯 Web 版本**（不依赖 .NET MAUI，直接运行在浏览器中）
- 作为**桌面/移动端的替代运行时**，意义不大（已有原生 .dll/.so，性能更好）

---

### 方案 B：用户 C 代码直接编译为 WASM

废弃 CideVM，使用标准工具链（Clang/LLVM 的 `wasm32` 后端）将用户 C 代码编译为 WASM 字节码，在浏览器/宿主 WASM runtime 中直接执行。

```
用户 C 代码
    ↓
Clang -target wasm32  (需要客户端打包 LLVM WASM 版)
    ↓
WASM 字节码（标准 WebAssembly）
    ↓
浏览器 WASM Runtime / Wasmtime
```

**优点**：
| 维度 | 说明 |
|---|---|
| 执行性能 | WASM JIT/AOT 编译后接近原生速度，比 CideVM 解释执行快 5-10 倍 |
| 标准兼容 | 支持更完整的 C 标准（如果这是目标） |
| 无需 VM | 省去 CideVM 的维护成本 |

**缺点**：
| 维度 | 说明 |
|---|---|
| **丧失 VisEvent 能力** | 标准 WASM 没有 Compare/Swap/Update 事件发射机制，算法可视化需要重写为 WASM 插桩或源码级 AST 分析 |
| **丧失中文诊断** | Clang 的诊断是英文，且面向标准 C 专家，不适合教育场景 |
| **丧失受控子集** | 标准 C 的指针算术、未定义行为等初学者陷阱全部暴露 |
| **编译器体积** | 客户端打包 Clang/LLVM WASM 版本约 50MB+，远大于当前 ~300KB 的 `libcide_native.so` |
| **调试困难** | 源码级单步调试需要 DWARF → WASM → Source Map 多层映射，比当前 VM 行号映射复杂得多 |
| **内存可视化** | WASM 的线性内存是单一数组，没有 CideVM 的 Global/Heap/Stack 区域划分，内存可视化需要重新设计 |

**适用场景**：
- 转向**标准 C 编译器**路线，放弃教育定制（与项目目标冲突）
- 追求极致执行性能的在线竞赛/工业场景

---

### 方案 C：保留 CideVM，将 VM 核心编译为 WASM（混合方案）

这是方案 A 的受限版本：只把 CideVM 运行时（不含前端编译器）编译为 WASM，编译器仍然放在 .NET 端或服务器端。

```
用户 C 代码
    ↓
[Lexer → Parser → TypeChecker → BytecodeGen]  (.NET 端或服务器)
    ↓
CideVM 字节码
    ↓
WASM 版 CideVM 解释器（Emscripten）
    ↓
JS Interop ←→ 前端
```

**评价**：
- 拆分了编译器和运行时，增加了序列化/传输字节码的复杂度
- 对当前架构没有明显收益（编译器本身不大，整体 WASM 化更直接）

---

## 综合评估

| 评估维度 | 当前原生架构 | 方案 A（整体 WASM） | 方案 B（C→WASM） |
|---|---|---|---|
| **教育适配性** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐☆ | ⭐⭐☆☆☆ |
| **执行性能** | ⭐⭐⭐☆☆ | ⭐⭐☆☆☆ | ⭐⭐⭐⭐⭐ |
| **调试体验** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐☆☆ | ⭐⭐☆☆☆ |
| **构建复杂度** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐☆☆ | ⭐⭐☆☆☆ |
| **移动端支持** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐☆ | ⭐⭐⭐⭐☆ |
| **纯 Web 支持** | ⭐☆☆☆☆ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **算法可视化** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐☆ | ⭐⭐☆☆☆ |
| **包体积** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐☆☆ | ⭐⭐☆☆☆ |

---

## 结论与建议

### 短期（当前 Stage 4/6）

**不推荐 WASM 化**。

当前原生架构（P/Invoke + `cide_native.so/.dll`）对于教育 IDE 已经足够好：
- Desktop（Avalonia）有原生 .dll，性能优秀
- Mobile（MAUI）有原生 .so，已通过 Android 16KB page-size 验证
- WASM 化带来的复杂度（Emscripten 构建链、JS interop、双重内存模型）远超收益

### 中期（Stage 7+：纯 Web 版本）

**方案 A 有价值**，但应作为**独立的技术路线**平行开发：

```
技术路线 1（当前）：.NET MAUI + P/Invoke + 原生 .so/.dll  →  移动端 + 桌面端
技术路线 2（未来）：纯 Web（React/Vue/Blazor WASM）+ Emscripten cide_native  →  浏览器端
```

两条路线共享同一份 C++ 源码（`native/src`），通过条件编译或 CMake 选项区分：
```cmake
option(CIDE_TARGET_WASM "Build for WebAssembly" OFF)
if(CIDE_TARGET_WASM)
    # Emscripten 工具链
    set(CMAKE_TOOLCHAIN_FILE $ENV{EMSDK}/upstream/emscripten/cmake/Modules/Platform/Emscripten.cmake)
endif()
```

### 长期

如果未来要支持**标准 C 子集以外的代码**（如调用 Web API、DOM 操作），可以考虑在 CideVM 中增加 WASM syscall 代理层，而非替换整个后端。

---

## 附录：WASM 化所需工作量估算

| 任务 | 预估工时 | 说明 |
|---|---|---|
| Emscripten 构建链集成 | 2-3 天 | CMake 工具链文件、CI 流水线、内存模型适配 |
| C API → JS 导出封装 | 1-2 天 | `cide_capi.h` 所有函数改为 `EMSCRIPTEN_KEEPALIVE` |
| .NET JS interop 封装 | 2-3 天 | Blazor 的 `IJSRuntime` 调用 WASM 模块 |
| VisEvent 跨层透传 | 1-2 天 | WASM 内 VM 的 VisEvent 如何回调到前端 |
| 内存可视化适配 | 2-3 天 | WASM 线性内存的只读视图映射到前端 Canvas |
| 调试映射重写 | 3-5 天 | Source map → WASM → VM 字节码 三层映射 |
| 测试与回归 | 3-5 天 | 确保 WASM 版与原生版行为一致 |
| **总计** | **14-23 天** | 相当于 1 个 Sprint 的工作量 |

> 注：以上估算仅针对**纯 Web 版本**的 WASM 化。若目标仅为替换当前原生架构，ROI 为负，不建议投入。
