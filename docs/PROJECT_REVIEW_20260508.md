# C IDE 项目审查报告

> 审查日期：2026-05-08
> 范围：Cide.Client / Cide.Client.Desktop / Cide.Client.Maui / Cide.Client.Shared / native / 构建脚本

---

## 一、勘误（Bug / 缺陷）

### 🔴 P0 - 严重

#### 1. `CideVM::Step()` 错误延迟报告
- **文件**：`native/src/vm/CideVM.cpp`
- **问题**：`Step()` 内部触发 `Trap()` 后执行 `break`，函数返回 `StepResult::OK`（而非 `Trap`）。错误只在**下一次** `Step()` 调用时才被函数开头的 `if (!error_.empty())` 捕获。
- **影响**：单步调试时，触发 Trap 的当前步返回 `OK`，错误延迟一步暴露，VM 状态（栈、内存）在错误指令后仍被读取，可能导致 UI 显示不一致。
- **修复**：在 `switch (inst.op)` 结束后增加：
  ```cpp
  if (!error_.empty()) return StepResult::Trap;
  ```

#### 2. `cide_compile_all` 多文件诊断使用错误源代码
- **文件**：`native/src/capi/cide_capi.cpp:438`
- **问题**：TypeChecker 的诊断全部硬编码使用 `s->compile.compileUnits[0].source` 生成结构化修复（`PopulateStructuredFix`）。
- **影响**：多文件编译时，若非首个文件出错，自动修复会基于第一个文件的源代码计算替换位置，导致代码被错误修改。
- **修复**：为每个编译单元记录源文件索引，诊断时传入对应源代码。

#### 3. `cide_set_input` 未处理 Windows 换行符 `\r\n`
- **文件**：`native/src/capi/cide_capi.cpp:930-945`
- **问题**：按 `\n` 分割后，每行末尾可能残留 `\r`，导致 `scanf` 解析异常。
- **修复**：分割后去除每行末尾的 `\r`。

---

### 🟠 P1 - 重要

#### 4. `CompilerService` 线程安全可见性问题
- **文件**：`Cide.Client.Shared/Core/CompilerService.cs`
- **问题**：`_disposed` 使用 `Interlocked.Exchange` 写入，但 `EnsureNotDisposed()` 直接读取非 `volatile` 字段。
- **影响**：在 ARM 等弱内存序架构下，可能读取到陈旧值，导致已释放对象被继续使用。
- **修复**：`private volatile int _disposed;`

#### 5. `MainViewModel` 并发属性可见性
- **文件**：`Cide.Client/ViewModels/MainViewModel.cs:351`
- **问题**：`_isRunInProgressFlag` 被 `Interlocked.CompareExchange` 修改，但 `CanRunCodeAsync` 属性直接读取。
- **修复**：`public bool CanRunCodeAsync => Interlocked.CompareExchange(ref _isRunInProgressFlag, 0, 0) == 0;`

#### 6. 未对齐内存读取（Native C API）
- **文件**：`native/src/capi/cide_capi.cpp:788-789, 802-803`
- **问题**：`mem[addr] | (mem[addr+1] << 8) | ...` 在 ARMv7 等旧架构可能导致未对齐访问错误。
- **修复**：使用 `std::memcpy` 安全读取。

#### 7. `StoreMemByte` 重复边界检查逻辑
- **文件**：`native/src/vm/CideVM.cpp:509-515`
- **问题**：`StoreMemByte` 内联了与 `StoreI8` 完全相同的边界检查逻辑，导致代码重复。
- **修复**：复用 `StoreI8(addr, val, inst.loc)`。

---

### 🟡 P2 - 轻微

#### 8. `TakeVisEvents` 不必要的 `clear()`
- **文件**：`native/src/vm/CideVM.cpp:323-327`
- `std::move` 后的 `vector` 已为空，`clear()` 冗余但无害。

#### 9. `Cide.Client.Maui` 仅引用 `arm64-v8a` 的 `.so`
- **文件**：`Cide.Client.Maui.csproj:83-84`
- `build.ps1` 构建两个 ABI，但 csproj 只引用 `arm64-v8a`，32 位设备无法运行。

#### 10. 构建脚本 `catch` 块吞异常
- **文件**：`build.ps1`, `build-release.ps1`, `test-mobile.ps1`
- `catch { Write-Error ... }` 不会终止脚本，`$LASTEXITCODE` 在 `catch` 后被重置为 `0`，导致构建继续。
- `test-mobile.ps1` 第 229 行手动设置 `$LASTEXITCODE = 0`，会掩盖前面的 adb 错误。

#### 11. `build.ps1` 清理目录重复
- 第 45-46 行 `Cide.Client.Maui/bin/obj` 重复列出。

#### 12. `build.ps1` Android 构建标题与内容不符
- 第 233 行标题为 "Building Avalonia Android Frontend (Legacy)"，实际构建的是 MAUI 项目。

---

## 二、优化建议

### 架构与性能

| # | 项目 | 建议 | 收益 |
|---|------|------|------|
| 13 | Avalonia Compiled Bindings | 统一 `AvaloniaUseCompiledBindingsByDefault=true` | 提升启动性能，编译期捕获绑定错误 |
| 14 | `ObservableCollection` 批量更新 | .NET 10 已支持 `AddRange` / `Reset`，替换逐条 `Add` | 大幅减少 `CollectionChanged` 事件，提升 UI 响应 |
| 15 | Native 字符串拼接 | C++20 使用 `std::format` 替代 `std::string` 多次拼接 | 减少内存分配 |
| 16 | `CompilerSessionService` 缓存 | 考虑对象池复用 `CideSession`，减少 `new/delete` | 降低 GC 和 Native 内存分配压力 |

### 代码质量

| # | 项目 | 建议 |
|---|------|------|
| 17 | `NativeMethods.cs` | 将 `IntPtr session` 封装为 `SafeHandle` 派生类，避免裸指针传递 |
| 18 | 重复 ViewModel | `GraphNodeViewModel.cs`、`CodeTemplate.cs`、`ViewModelBase.cs` 在 Client/Shared 中重复，确认已通过 `TypeForwards` 解决 |

---

## 三、框架更迭建议

| 组件 | 当前 | 建议 |
|------|------|------|
| `LangVersion` | `12` | **升级到 `14` 或 `latest`**（.NET 10 原生支持 C# 14） |
| `Avalonia` | `11.3.0` | 升级到 `11.3.x` 最新补丁 |
| `Avalonia.AvaloniaEdit` | `11.4.1` | ⚠️ Minor 高于 Avalonia，确认兼容性或统一升级 |
| `GaelJ.BlazorCodeMirror6` | `0.10.0` | 升级到最新稳定版 |
| `CMAKE_C_STANDARD` | `11` | 升级到 `17`（兼容且为缺陷修复版） |
| `ANDROID_PLATFORM` | `android-21` | 与 MAUI 的 `SupportedOSPlatformVersion=24.0` 对齐为 `android-24` |
| `MauiVersion` | SDK 隐式 | 建议显式声明在 `Directory.Packages.props` 中 |

---

## 四、修复任务清单

- [x] **P0-1** 修复 `CideVM::Step()` 错误延迟（`native/src/vm/CideVM.cpp`）
- [ ] **P0-2** 修复多文件诊断源代码关联（需架构改动，暂未实施）
- [x] **P0-3** 修复 `cide_set_input` `\r` 处理（`native/src/capi/cide_capi.cpp`）
- [x] **P1-4** `CompilerService._disposed` 添加 `volatile`（`Cide.Client.Shared/Core/CompilerService.cs`）
- [x] **P1-5** `MainViewModel.CanRunCodeAsync` 使用 `Interlocked` 读取（Desktop + Maui）
- [x] **P1-6** Native C API 未对齐读取改用 `memcpy`（`native/src/capi/cide_capi.cpp`）
- [x] **P1-7** `StoreMemByte` 复用 `StoreI8`（`native/src/vm/CideVM.cpp`）
- [ ] **P2-8** 删除 `TakeVisEvents` 冗余 `clear()`（低风险，保留）
- [x] **P2-9** 补充 `armeabi-v7a` `.so` 引用（`Cide.Client.Maui.csproj`）
- [x] **P2-10** 修复构建脚本异常处理（`build.ps1`, `test-mobile.ps1`）
- [x] **P2-11** 删除 `build.ps1` 重复清理行
- [x] **P2-12** 修正 `build.ps1` Android 构建标题
- [x] **OPT-13** 统一启用 Avalonia Compiled Bindings（`Cide.Client.csproj`）
- [x] **OPT-14** 升级 `LangVersion` 到 14（全部 4 个 `.csproj`）
- [x] **新增** `CodeFixService` `\r\n` 处理（`Cide.Client.Shared/Core/CodeFixService.cs`）
- [x] **新增** `KnowledgeCardLoader` 线程安全（`Cide.Client.Shared/Core/KnowledgeCardLoader.cs`）
- [x] **新增** `DiagnosticService` HashSet 去重优化（`Cide.Client.Shared/Core/DiagnosticService.cs`）
- [x] **新增** `GraphCanvas` 重复地址过滤（`Cide.Client/Views/GraphCanvas.axaml.cs`）
- [x] **新增** `MainWindow` 移除构造时无效 `UpdateLayout`（`Cide.Client/Views/MainWindow.axaml.cs`）
