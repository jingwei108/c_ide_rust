# C IDE Phase 2 代码审查与优化计划

> 基于对项目文档（Phase2 完成报告、总体设计、WASM 架构、零侵入可视化、移动端适配、诊断设计等）和全部核心源代码的全面审查，制定本优化计划。

---

## 一、当前状态概述

### 已完成（Phase 2）
- Lexer → Parser → AST → TypeChecker → WasmCodeGen 全链路贯通
- wasm3 沙盒执行 + P/Invoke 前端接入
- 端到端验证通过（递归、循环、条件等基础场景）

### 已知限制
- Source Map：未精确填充指令偏移映射
- 虚拟内存管理：`__cide_malloc`/`__cide_free` 宿主函数已存在但无边界检查
- 指针追踪：`cide_memory_get_pointer_target` 返回 0（stub）
- 单步执行：`cide_step_next` 为 stub
- 输出系统：仅捕获 `main()` 返回值，`__cide_output` 已接入

---

## 二、Bug 修正（高优先级）

### 2.1 Lexer 单字符 Token 列号计算错误
**文件**: `native/src/compiler/Lexer.cpp`
**问题**: `MakeToken(TokenType, std::string)` 使用 `column_`（已递增），导致 `;` `(` `{` 等单字符 Token 的列号比实际位置大 1。
**修复**: 改为 `column_ - text.length()`

### 2.2 WasmCodeGen `PostInc`/`PostDec` 返回值语义错误
**文件**: `native/src/compiler/WasmCodeGen.cpp`
**问题**: 后缀自增 `a++` 返回递增后的新值，但语义应返回旧值。
**修复**: PostInc/PostDec 保存旧值到临时 local，递增后返回旧值。

### 2.3 Parser 不支持全局变量
**文件**: `native/src/compiler/Parser.cpp`
**问题**: `ParseProgram()` 只接受 struct 和函数声明，顶层变量声明被当作函数解析报错。
**修复**: 区分 `Type Name(`（函数）和 `Type Name;`（全局变量）。

### 2.4 C API `malloc` Bump Allocator 无边界检查
**文件**: `native/src/capi/cide_capi.cpp`
**问题**: `heapOffset` 线性递增，不检查是否超过 WASM 内存上限（128KB）。
**修复**: 分配前检查 `newOffset <= memSize`，超出时返回 0。

---

## 三、架构优化（中优先级）

### 3.1 TypeChecker 消除重复 Stmt Dispatch
**文件**: `native/src/compiler/TypeChecker.cpp`
**问题**: `VisitIf`/`VisitWhile`/`VisitFor` 中有完全相同的 7-case switch。
**修复**: 提取 `DispatchStmt(Stmt&)` 通用方法。

### 3.2 编译错误结构化（为 Phase 3 诊断做准备）
**文件**: `native/src/capi/cide_capi.cpp`, `native/src/compiler/*`
**问题**: 错误是纯文本字符串，前端无法解析错误码、分级信息、修复建议。
**修复**:
- 各编译阶段错误增加 `errorCode`
- `cide_compile` 填充 `CideSession::diagnostics` 结构化向量
- 保留 `FormatDiagnostics` 纯文本兼容现有前端

### 3.3 CideSession 职责拆分
**文件**: `native/src/capi/cide_capi.cpp`
**问题**: 单结构体混合编译/运行/内存/诊断/轨迹状态。
**修复**: 内聚分组为 `CompileState` / `RuntimeState` / `MemoryState`。

### 3.4 Source Map 预留接口
**文件**: `native/src/compiler/WasmCodeGen.cpp`, `native/src/capi/cide_capi.cpp`
**问题**: 完全未实现，Phase 3 运行时错误无法精确映射到源码行列。
**修复**:
- `WasmCodeGen` 增加 `sourceMap_`（`wasmOffset → SourceLoc`）
- `CideSession` 存储 Source Map
- C API 增加 `cide_sourcemap_lookup` 预留接口

---

## 四、代码质量改进

### 4.1 Parser `pos_--` Hack 清理
**文件**: `native/src/compiler/Parser.cpp`
**问题**: `ParseProgram` 中 `Match(TokenType::Struct)` 后又 `pos_--` 回退。
**修复**: 改为 `Check(TokenType::Struct)` 避免消费后回退。

### 4.2 C# `PtrToStringUtf8` 简化
**文件**: `Cide.Client/Core/NativeMethods.cs`
**修复**: 使用 `Marshal.PtrToStringUTF8` 替代手动遍历字节。

### 4.3 `strncpy` 重复代码封装
**文件**: `native/src/capi/cide_capi.cpp`
**修复**: 增加 `SafeStrCopy` 辅助函数。

---

## 五、实施顺序与状态

```
Phase A: Bug 修复（已完成 ✅）
  1. Lexer 列号修复 ✅
  2. PostInc/PostDec 语义修复 ✅
  3. malloc 边界检查 ✅
  4. Parser 全局变量支持 ✅

Phase B: 架构优化（已完成 ✅）
  5. TypeChecker DispatchStmt 重构 ✅
  6. CideSession 结构拆分 ✅
  7. 结构化诊断数据填充 ✅

Phase C: 质量改进 + Phase 3 预备（已完成 ✅）
  8. Source Map 预留接口 ✅
  9. Parser pos_-- 清理 ✅
  10. C# NativeMethods 简化 ✅
```

**构建验证**: `build.ps1 -Target Desktop` 通过，0 警告 0 错误。

---

## 六、与后续 Phase 的衔接

| Phase 3 需求 | 本计划预备工作 |
|:---|:---|
| 中文错误分级 (L1/L2/L3) | `CideDiagnostic` 增加 `errorCode` + `messageL1/L2/L3` |
| QuickFix 自动修复 | `CideDiagnostic` 增加 `fixEdit`（替换位置+文本） |
| 零侵入可视化注入 | 预留 `VisualizationInjector` 在 TypeChecker→CodeGen 之间 |
| Source Map 精确映射 | `WasmCodeGen` 记录 `offset → SourceLoc`，C API 暴露查询 |
| 单步调试 | `__cide_step` 已注入，需实现 `cide_step_next` 真实单步 |
| 内存视图/指针追踪 | 建立 `指针地址→目标地址` 映射表 |
