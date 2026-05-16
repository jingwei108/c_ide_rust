# Rust 迁移后构建脚本更新、前端联编验证与测试扩展（2026-05-09）

## 概述

本记录汇总了 2026-05-09 对 Cide 项目 Rust 后端的维护工作，包括：
1. CI/CD 构建脚本从 CMake 全面迁移到 `cargo` / `cargo-ndk`
2. C# 前端（Desktop + Maui Android）联编验证
3. 端到端测试扩展（新增 6 个场景）
4. 复合赋值运算符（`-=`、`/=`、`%=`）操作数顺序 Bug 修复

---

## 1. CI/CD 脚本更新

### 背景
项目已从 C++ / CMake 完全迁移到 Rust / Cargo。原有的 `build.ps1`、`build-release.ps1`、`test-mobile.ps1` 仍包含大量 CMake / Ninja 逻辑，需要同步更新。

### 修改内容

#### `build.ps1`
- 更新文件头注释："C++ native backend" → "Rust native backend"
- `Clean` 阶段：移除 `native/build`、`native/build-android-*` 等 CMake 遗留目录，保留 `native/target`
- Android `cargo-ndk` 构建后：
  - 将 `.so` 复制到 `native/target/android/<abi>/`（与 `Cide.Client.Maui.csproj` 中 `AndroidNativeLibrary` 的引用路径对齐）
  - 同时保留向 `Cide.Client.Maui/lib/<abi>/` 的复制（兼容旧路径）

#### `build-release.ps1`
- Desktop Native 构建：从 `cmake -G Ninja + cmake --build` 替换为 `cargo build --release`
- DLL 复制源路径：从 `native/build/bin/...` 更新为 `native/target/release/cide_native.dll`
- Android Native 构建：从 CMake Android toolchain 替换为 `cargo ndk --target <triple> --platform 21 build --release`
- `.so` 复制目标路径同步更新为 `native/target/android/<abi>/`

#### `test-mobile.ps1`
- Native `.so` 构建段落：完全移除 CMake 配置/编译逻辑，替换为 `cargo ndk`
- 保留 APK 打包、安装、运行、logcat 捕获等 MAUI 前端逻辑不变

#### `scripts/check-memory-safety.ps1`
- 移除 C++ 扫描逻辑（`.cpp` / `.hpp` 文件已不存在）
- 新增 Rust 扫描规则：
  - `std::mem::transmute` 使用检测
  - 裸指针 `.offset()` 无边界检查检测
  - `CStr::from_ptr` 生命周期风险提示
  - 手动 allocator（`alloc`、`Layout::new`）检测
- C# 扫描逻辑（事件订阅、async void、IntPtr IDisposable 等）保持不变

---

## 2. C# 前端联编验证

验证 P/Invoke 接口（`NativeMethods.cs`）与 Rust DLL / `.so` 的 ABI 兼容性。

### Desktop
| 配置 | 命令 | 结果 |
|------|------|------|
| Debug | `dotnet build Cide.Client.Desktop.csproj -c Debug` | ✅ 成功 |
| Release (Native AOT) | `dotnet publish ... -r win-x64 --self-contained` | ✅ 成功 |

Release 构建产生少量 IL trim 警告（Avalonia ReflectionBinding、GraphCanvas），不影响构建结果。

### Maui Android
| 配置 | 命令 | 结果 |
|------|------|------|
| Debug | `dotnet build Cide.Client.Maui.csproj -f net10.0-android -c Debug` | ✅ 成功 |

`Cide.Client.Maui.csproj` 通过 `<AndroidNativeLibrary>` 引用 `native/target/android/<abi>/libcide_native.so`，构建时正确打包到 APK。

---

## 3. 端到端测试扩展

在 `native/tests/end_to_end_test.rs` 新增 6 个测试，覆盖此前未验证的场景。

| 测试名 | 场景 | 预期结果 |
|--------|------|----------|
| `test_e2e_pointer_deref` | `int x = 42; int *p = &x; printf("%d", *p);` | 输出 `42` |
| `test_e2e_compound_assign` | `a += 5; a -= 3; a *= 2; a /= 4;` | 输出 `6` |
| `test_e2e_array_bounds_trap` | `int arr[3]; arr[5] = 99;` | 运行时 `TrapBounds` 错误 |
| `test_e2e_while_loop` | `while (i < 5) { sum += i; i++; }` | 输出 `10` |
| `test_e2e_function_call` | `int add(int a, int b) { return a+b; }` | 输出 `7` |
| `test_e2e_recursive_factorial` | `int fact(int n) { ... }` 递归 | 输出 `120` |

### 测试总数
- Compile Pipeline Tests：**12** 个
- End-to-End Tests：**17** 个（原 11 + 新增 6）
- **合计：29 个，全部通过**

---

## 4. 复合赋值操作数顺序 Bug 修复

### 现象
端到端测试 `test_e2e_compound_assign` 最初输出 `0` 而非预期的 `6`。逐步排查发现：
- `+=`、`*=` 测试通过（交换律掩盖了问题）
- `-=` 实际输出 `-7`（被 `contains("7")` 误判定为通过）
- `/=` 实际输出 `0`（`4 / 24 = 0`）

### 根因
`bytecode_gen.rs` 的 `gen_assign` 中 compound assign 生成顺序为：

```rust
self.gen_expr(right);        // push right
self.emit(LoadLocal, ...);   // push left
emit_compound(self, loc);    // op: pop b=left, pop a=right → right op left
```

对于非交换律运算符（`-`、`/`、`%`），计算的是 `right op left` 而非 `left op right`。

### 修复
将 compound assign 分支的生成顺序调整为：先加载左操作数，再求值右操作数。

```rust
if *op != AssignOp::Assign {
    self.emit(OpCode::LoadLocal, local_idx, loc);  // push left
    self.gen_expr(right);                            // push right
    emit_compound(self, loc);                        // op: pop b=right, pop a=left → left op right
} else {
    self.gen_expr(right);
}
```

同时修复了 `global_idx` 分支的同样问题。

### 验证
修复后全部 17 个端到端测试通过，包括包含 `-=`、`/=` 的链式复合赋值场景。

---

## 相关文件变更

| 文件 | 变更类型 |
|------|----------|
| `build.ps1` | 修改 |
| `build-release.ps1` | 修改 |
| `test-mobile.ps1` | 修改 |
| `scripts/check-memory-safety.ps1` | 修改 |
| `native/src/compiler/bytecode_gen.rs` | 修改（compound assign 顺序 + `gen_index` result_ty 参数） |
| `native/tests/end_to_end_test.rs` | 修改（新增 6 个测试） |

---

## 备注

- `non_camel_case_types` 警告（66 个）被有意保留，以匹配 C API 错误码命名风格
- 少量 dead code 警告（`synchronize`、`is_global`、`ret` 赋值未读）未处理，属于已知遗留
