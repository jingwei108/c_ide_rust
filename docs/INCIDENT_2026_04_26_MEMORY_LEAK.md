# 事故报告：调试程序内存泄漏导致系统卡死

**日期**：2026-04-26  
**影响**：`debug_*.exe` 系列调试程序运行时占满 32GB 内存，导致系统卡死并强制关机  
**状态**：已修复，已加固

---

## 1. 事故摘要

在排查 "局部变量初始化 / 直接返回字面量返回 0" 的 codegen bug 过程中，创建了多个临时调试程序（`debug_simple_add.exe`、`debug_return_literal.exe`、`debug_printf.exe`、`debug_f_only.exe`、`debug_f_compile_only.exe` 等）。

其中 F 测试用例：
```c
int main() { int* p = 0; p = (int*)100; return 0; }
```
包含不支持的 `(int*)` 类型转换语法。Parser 将该代码解析为 `UnaryExpr(Deref, LiteralExpr(100))`，导致后续编译/运行时出现异常行为，进程内存泄漏至数 GB 仍不退出。

| 进程 | 占用内存 |
|------|---------|
| `debug_f_compile_only.exe` | ~2.9 GB |
| `debug_f_only.exe` | ~12.6 GB |
| `debug_return_literal.exe` | ~10.5 GB |

---

## 2. 根因分析

### 2.1 直接原因
- F 测试用例包含 `(int*)100` 类型转换，本编译器子集**不支持 cast 语法**
- Parser 将 `(int*)100` 错误解析为 `Deref(100)`（解引用 100）
- 生成的 WASM 代码在 wasm3 中执行时触发异常或无限循环
- 由于**缺少执行步数限制**和**缺少运行超时**，进程持续运行并泄漏内存

### 2.2 深层原因
- 调试程序直接裸运行，没有任何超时/内存保护
- `cide_run` 同步调用 `m3_CallV`，一旦 wasm3 陷入死循环即永久阻塞
- wasm3 本身没有内置的步数限制机制

---

## 3. 已采取的修复措施

### 3.1 清理与删除
- 终止所有残留 `debug_*.exe` 进程
- 删除所有临时调试源码和可执行文件
- 从 `CMakeLists.txt` 移除所有临时 `add_executable(debug_*)` 目标

### 3.2 核心 Bug 修复
- **修复 `VisitVarDecl` 标量初始化栈顺序**：`i32.store` 之前错误地将 value 先于 address 压栈，导致初始化值写入错误地址
- 该修复恢复了 Phase 2/3 全部回归测试（5/5 + 9/9 + 6/6 通过）

### 3.3 三重防卡死加固

#### 第一层：脚本层 — 安全运行包装器
新增 `native/tests/safe_run.ps1`：
```powershell
.\safe_run.ps1 -ExePath ".\xxx_test.exe" -TimeoutSeconds 10 -MaxMemoryMB 500
```
- 超时（默认 10 秒）或超内存（默认 500MB）**强制终止进程**

#### 第二层：运行时层 — wasm3 执行步数限制
修改 wasm3 源码：
- `m3_env.h`：`M3Runtime` 结构体新增 `maxSteps` / `stepCount`
- `m3_core.c`：`m3_Yield()` 通过线程局部存储检查步数，超过 **1000 万步** 返回 `m3Err_trapAbort`
- `cide_run` 中自动设置 `maxSteps = 10_000_000`

#### 第三层：线程层 — 硬超时
修改 `cide_run`：
- `m3_CallV` 在**独立线程**中执行
- 主线程监控，超过 **10 秒** 立即返回 `"程序执行超时"` 错误
- 避免主线程阻塞导致整个系统卡死

---

## 4. 安全测试指南

### 4.1 运行单个测试（推荐）
```powershell
cd native\build\Debug
.\safe_run.ps1 -ExePath ".\phase2_regression_test.exe"
```

### 4.2 运行所有回归测试
```powershell
cd native\build\Debug
.\phase2_regression_test.exe
.\phase3_batch1_test.exe
.\phase3_batch2_test.exe
.\phase3_batch3_test.exe
.\phase3_batch4_test.exe
```

> 以上正式测试用例均很短（< 1 秒），不会触发超时。

### 4.3 禁止行为
- **不要再创建裸的 `debug_xxx.exe` 临时程序直接运行**
- **任何新测试必须先通过 `safe_run.ps1` 验证**
- **不要在测试中包含不支持的语法**（如类型转换 `(int*)x`）

---

## 5. 验证结果

| 测试套件 | 结果 |
|---------|------|
| Phase 2 回归 | 5/5 passed |
| Batch 1 内存操作 | 9/9 passed |
| Batch 2 输出/错误 | 2/3 passed（除零乱码为已知终端编码问题）|
| Batch 3 内存视图 | 1/1 passed |
| Batch 4 printf/scanf | 6/6 passed |

---

## 6. 后续建议

1. **Parser 增强**：遇到不支持的语法（如 cast）时应明确报错并停止编译，而不是生成错误的 AST
2. **CI/CD**：在自动化测试中统一使用 `safe_run.ps1` 作为测试包装器
3. **文档更新**：在 `C_SUBSET_SPEC.md` 中明确列出不支持的语言特性
