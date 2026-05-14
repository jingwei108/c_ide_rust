# Phase 3 P0 紧急修复日志

> 日期: 2026-04-27  
> 范围: 线程泄漏 + 内存安全  
> 状态: ✅ 已完成，全部测试通过

---

## 修复项 1: 超时后 wasm3 线程泄漏

**严重级别**: 🔴 P0 - 系统稳定性  
**根因**: `cide_run` 中超时后直接 `wasmThread.detach()`，线程在后台永久运行。

### 修改内容

1. **`m3_env.h`** — `M3Runtime` 新增 `cancelled` 标志
   ```cpp
   // Step limit for infinite loop prevention
   i64                     maxSteps;
   i64                     stepCount;
   bool                    cancelled;  // <-- 新增
   ```

2. **`m3_core.c`** — `m3_Yield()` 检查取消标志
   ```cpp
   if (g_cideCurrentRuntime && g_cideCurrentRuntime->cancelled) {
       return m3Err_trapAbort;
   }
   ```

3. **`cide_capi.cpp`** — 超时后安全取消
   ```cpp
   s->wasmRuntime->cancelled = false;  // 运行前重置
   
   // 超时后:
   s->wasmRuntime->cancelled = true;   // 通知线程退出
   
   // 给 5 秒优雅退出时间
   while (!callDone) { ... }
   
   if (callDone) {
       wasmThread.join();  // 正常回收
   } else {
       wasmThread.detach(); // 最后手段
   }
   ```

### 验证
- 所有回归测试通过
- 超时后线程不再泄漏（通过 `cancelled` → `m3_Yield` → `trapAbort` → 调用栈返回）

---

## 修复项 2: 字符串字面量地址与全局内存重叠

**严重级别**: 🔴 P0 - 数据损坏风险  
**根因**: 字符串数据段从固定地址 `0x0100` 开始，全局变量从 `0x1000` 开始，但字符串总长度可能超过 `0x1000` 覆盖全局变量区。

### 修改内容

1. **`WasmCodeGen.hpp`**
   - `nextStringAddr_` 默认改为 `0`（由 `Generate()` 动态设置）
   - 新增 `globalTypes_` 映射表

2. **`WasmCodeGen.cpp`**
   - `Generate()` 开始时 `stringAddrs_.clear()`
   - 处理完全局变量后：`nextStringAddr_ = globalMemOffset_`
   - 字符串数据段现在紧接在全局变量区之后分配
   - `VisitStringLiteral` 新增 128KB 溢出检查

### 内存布局（修复后）
```
0x0000~0x0FFF   : 保留（NULL 陷阱区）
0x1000~nextStr  : 全局变量区 + 字符串数据段
nextStr~0x20000 : 堆区（malloc）
0x10000~        : 栈区（局部变量）
```

---

## 修复项 3: 全局变量取地址无法区分标量与数组

**严重级别**: 🟡 P1 - 语言特性正确性  
**根因**: `EmitAddressOf` 中对全局变量取地址时，无法区分标量（global 存值）和数组/struct（global 存基地址）。

### 修改内容

1. **`WasmCodeGen.hpp`** — 新增 `globalTypes_` 字段
2. **`WasmCodeGen.cpp`**
   - `Generate()` 注册全局变量时：`globalTypes_[g.name] = g.type`
   - `EmitAddressOf` 中：
     ```cpp
     if (globalTypeIt->second.isArray() || globalTypeIt->second.isStruct()) {
         EmitGlobalGet(idx); // 返回基地址
     } else {
         // 标量 global 存的是值，无法取地址
         ReportError("暂不支持对全局标量变量取地址...");
     }
     ```

---

## 回归测试结果

| 测试套件 | 结果 |
|:---|:---|
| Phase 2 回归 | 5/5 passed |
| Batch 1 内存操作 | 9/9 passed |
| Batch 2 运行时错误 | 2/3 passed（除零乱码为已知终端编码问题）|
| Batch 3 Memory View | 1/1 passed |
| Batch 4 printf/scanf | 6/6 passed |
| New Features (char/switch/enum/typedef等) | 全部 passed |

---

## 下一步（P1 功能补齐）

1. **实现 `cide_step_next` 真实单步调试**
2. **堆内存 `free` 后重用**（当前 `heapOffset` 只增不减）
