# Phase 2 完成报告：C 子集编译器

## 概述

Phase 2 的核心目标——**手写 C 子集编译器（Lexer → Parser → TypeChecker → WasmCodeGen）并集成到 C API**——已全面完成并通过端到端验证。

---

## 已完成工作清单

### 1. 编译器前端

| 组件 | 文件 | 行数 | 状态 |
|------|------|------|------|
| **Lexer** | `native/src/compiler/Lexer.cpp/hpp` | ~150 | ✅ 关键词、标识符、数字、运算符、字符串 |
| **Parser** | `native/src/compiler/Parser.cpp/hpp` | ~400 | ✅ 递归下降，表达式优先级，语句，函数定义，struct |
| **AST** | `native/src/compiler/Ast.hpp` | ~344 | ✅ 20+ 节点类型，Visitor 模式 |
| **TypeChecker** | `native/src/compiler/TypeChecker.cpp/hpp` | ~561 | ✅ 类型推导、符号表、作用域、错误收集 |
| **WasmCodeGen** | `native/src/compiler/WasmCodeGen.cpp/hpp` | ~830 | ✅ WASM MVP 二进制格式生成 |

### 2. C API 集成

`native/src/capi/cide_capi.cpp` 已完成真实编译器链路替换：

```cpp
int cide_compile(CideSession* s, const char* source) {
    // 1. Lexer::Tokenize()
    // 2. Parser::Parse()
    // 3. TypeChecker::Check()
    // 4. WasmCodeGen::Generate()
    // 5. 存储 WASM 字节码到 session
}

int cide_run(CideSession* s) {
    // 1. m3_ParseModule() + m3_LoadModule()
    // 2. m3_FindFunction("main")
    // 3. m3_CallV() 执行
    // 4. 捕获返回值，输出结果
}
```

### 3. 前端 P/Invoke 接入

`Cide.Client/ViewModels/MainViewModel.cs` 中 `RunCode` 命令已接入 `CompilerService`：

```csharp
using var compiler = new CompilerService();
bool ok = compiler.Compile(SourceCode);
ok = compiler.Run();
ConsoleOutput = compiler.GetOutput();
```

### 4. 关键 Bug 修复

#### Bug 1：递归调用失败
**现象**：`factorial(5)` 编译报错 "未定义的函数 'factorial'"
**根因**：`Generate()` 在生成代码的同时逐个 `AddFunc`，导致函数调用自身时 `funcIndices_` 尚未注册当前函数
**修复**：预注册所有函数索引（`baseFuncIdx + i`），再生成代码

```cpp
// 预注册函数索引，使前向/自引用调用正常工作
uint32_t baseFuncIdx = builder_.FuncCount();
for (size_t i = 0; i < program.funcs.size(); i++) {
    funcIndices_[program.funcs[i].name] = baseFuncIdx + static_cast<uint32_t>(i);
}
```

#### Bug 2：If/Else 双分支 Return 导致 wasm3 栈错误
**现象**：`if (cond) { return a; } else { return b; }` 报 "incorrect value count on stack"
**根因**：wasm3 编译器在处理 if/else 两端都 terminal-return 时，外层 block 的 stack 状态未正确标记为 polymorphic
**修复**：检测 then/else 都 terminal-return 时，在 if 块结束后 emit `unreachable` (0x00)

```cpp
static bool IsTerminalReturn(const Stmt& stmt) { /* ... */ }

void WasmCodeGen::VisitIf(IfStmt& node) {
    // ... 生成 if/else ...
    if (IsTerminalReturn(*node.thenStmt) &&
        node.elseStmt && IsTerminalReturn(*node.elseStmt)) {
        EmitOp(0x00); // unreachable
    }
}
```

#### Bug 3：cide_session_destroy 崩溃
**现象**：Exit code -1073741819 (0xC0000005)
**根因**：`m3_LoadModule()` 后运行时拥有模块所有权，`m3_FreeModule()` 导致 double-free
**修复**：销毁时只释放 runtime（自动释放模块），每次 run 重建 fresh runtime

---

## 端到端验证结果

| 测试用例 | 源码特征 | 返回值 | 状态 |
|---------|---------|--------|------|
| Simple Add | 变量、算术 | 8 | ✅ |
| Loop Sum | `for` 循环 1..5 | 15 | ✅ |
| If/Else Function | 多函数、`if/else` | 7 | ✅ |
| While Count | `while` 循环 | 5 | ✅ |
| Recursive Factorial | 递归调用 5! | 120 | ✅ |
| Error Program | 语法错误 | — | ✅ 正确报错 |

---

## 已知限制（Phase 2 剩余）

1. **Source Map**：WASM 偏移 → 源码行/列 的映射表已预留结构，但未精确填充每个指令的偏移
2. **虚拟内存管理**：`__cide_malloc`/`__cide_free` 宿主函数未实现，当前 WASM 线性内存仅由 wasm3 管理
3. **指针追踪**：`cide_memory_get_pointer_target` 返回 0（stub）
4. **单步执行**：`__cide_step` hook 未注入到生成的 WASM 中，`cide_step_next` 为 stub
5. **输出系统**：当前仅捕获 `main()` 的返回值，未实现 `__cide_output` 宿主函数用于 `printf` 式输出

---

## 架构确认

```
源代码字符串
    |
    v
+-----------------------------------+
| Lexer::Tokenize()                 |
+-----------------------------------+
    | vector<Token>
    v
+-----------------------------------+
| Parser::Parse()                   |
+-----------------------------------+
    | unique_ptr<ProgramNode>
    v
+-----------------------------------+
| TypeChecker::Check()              |
+-----------------------------------+
    | 类型推导 + 错误收集
    v
+-----------------------------------+
| WasmCodeGen::Generate()           |
+-----------------------------------+
    | vector<uint8_t> (WASM 模块)
    v
+-----------------------------------+
| wasm3: m3_ParseModule / LoadModule|
|       m3_Call("main")             |
+-----------------------------------+
    | 返回值 / trap 错误
    v
CideSession::outputLines
```

---

## 下一步

进入 **Phase 3：诊断与可视化**

优先级：
1. `__cide_output` 宿主函数 + 前端 Console 输出
2. `__cide_step` 单步 hook 注入
3. Source Map 精确映射（用于错误定位）
4. 内存视图基础（`m3_GetMemory` + 区域划分）
