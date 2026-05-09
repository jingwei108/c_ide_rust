# Clang 编译器迁移记录

> 迁移日期：2026-04-27  
> 目标：将 Native 后端从默认 MSVC 切换到 Clang 22.1.4，并清理全部编译警告。

---

## 一、迁移背景

项目 Native 后端（C++20）此前默认使用 MSVC 编译器。经过评估，切换至 Clang 能带来以下收益：

| 维度 | Clang 优势 |
|------|-----------|
| **C++20 标准一致性** | 更严格的标准符合性，减少 MSVC 特有"宽松"行为 |
| **诊断能力** | 默认警告级别更高，` -Wall -Wextra` 能发现 MSVC 静默的代码质量问题 |
| **VM 解释器性能** | `CideVM::Step()` 的大型 `switch-case` 更易被优化为跳转表 |
| **工具链生态** | 可使用 `clang-tidy`、AddressSanitizer、UBSan 等工具 |
| **跨平台一致性** | 若后续需移植 Android/Linux 后端，代码已在 Clang 验证 |
| **ABI 兼容** | 使用 `x86_64-pc-windows-msvc` 版本，与 .NET P/Invoke 完全兼容 |

---

## 二、环境信息

- **Clang 路径**：`C:\Clang\clang+llvm-22.1.4-x86_64-pc-windows-msvc`
- **CMake**：4.2
- **生成器**：Ninja（随 Visual Studio 2022 附带）
- **构建脚本**：`build.ps1`

---

## 三、修改内容

### 3.1 `native/CMakeLists.txt`

**1. 增加 Clang 的 UTF-8 编译选项**

项目大量使用中文诊断信息（如 `Trap("LoadLocal: 无调用帧")`），需确保 Clang 下字符编码正确：

```cmake
# Before (仅 MSVC)
if(MSVC)
    target_compile_options(cide_native PRIVATE /utf-8)
endif()

# After (MSVC + Clang)
if(MSVC)
    target_compile_options(cide_native PRIVATE /utf-8)
elseif(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
    target_compile_options(cide_native PRIVATE -Wall -Wextra -fexec-charset=UTF-8 -finput-charset=UTF-8)
endif()
```

> 测试目标同步修改。

### 3.2 `build.ps1`

**1. 新增 `-Compiler` 参数**

```powershell
[ValidateSet("Default", "Clang", "ClangCL", "MSVC", "MinGW")]
[string]$Compiler = "Default"
```

**2. Native Backend 构建逻辑增加编译器选择**

```powershell
$clangRoot = "C:/Clang/clang+llvm-22.1.4-x86_64-pc-windows-msvc"
$clangBin = "$clangRoot/bin"

switch ($Compiler) {
    "Clang" {
        $cmakeArgs += @("-G", "Ninja")
        $cmakeArgs += "-DCMAKE_C_COMPILER=$clangBin/clang.exe"
        $cmakeArgs += "-DCMAKE_CXX_COMPILER=$clangBin/clang++.exe"
        $cmakeArgs += "-DCMAKE_RC_COMPILER=$clangBin/llvm-rc.exe"
    }
    "ClangCL" { ... }
    "MSVC" { ... }
    "MinGW" { ... }
    default { ... }
}
```

> `CMAKE_RC_COMPILER` 是解决 Windows-Clang (GNU-like) 模式下 CMake 找不到资源编译器的关键配置。

---

## 四、构建过程问题与解决

### 4.1 `CMAKE_RC_COMPILER` 缺失

**现象**：
```
CMake Error: No CMAKE_RC_COMPILER could be found.
```

**根因**：Windows 上使用 Clang (GNU-like command-line) 时，CMake 默认需要 `rc.exe`（MSVC 资源编译器）。当显式指定 Clang 编译器路径后，CMake 无法自动定位。

**解决**：显式指定 `CMAKE_RC_COMPILER=$clangBin/llvm-rc.exe`，使用 LLVM 自带的资源编译器。

---

## 五、Warning 清理（`-Wall -Wextra`）

开启 `-Wall -Wextra` 后，Clang 共发现约 **60 个 warning**（MSVC 默认全部静默）。已全部分类修复：

### 5.1 `missing-field-initializers`（~40 个）

**根因**：`Ast.hpp` 中 `Type` 结构体的 `std::string name` 没有默认成员初始化器：
```cpp
struct Type {
    TypeKind kind = TypeKind::Void;
    std::string name;   // 无默认初始化
    // ...
};
```

代码中 40 多处使用 `Type{TypeKind::Int}` 聚合初始化，Clang 报 `name` 未初始化。

**修复**：一行改动：
```cpp
std::string name{};   // 添加默认空初始化
```

### 5.2 `unused-parameter`（~6 个）

**位置**：
- `TypeChecker::VisitProgram`
- `BytecodeGen::VisitProgram` / `VisitFuncDecl` / `VisitCase`
- `WasmCodeGen::VisitFuncDecl` / `VisitInitList` / `VisitProgram`

**修复**：将未使用的参数名注释掉：
```cpp
void BytecodeGen::VisitProgram(ProgramNode& /*node*/) {}
```

### 5.3 `unused-variable` / `unused-but-set-variable`（2 个）

- `BytecodeGen.cpp:191`：`size_t mainIP = 0;` 赋值后从未读取 → **删除**
- `Phase3StepTest.cpp:89`：`int ret = -1;` 从未使用 → **删除**

### 5.4 `unused-const-variable`（1 个）

- `WasmCodeGen.cpp:56`：`constexpr uint8_t OP_SELECT = 0x1b;` 定义后未使用 → 加 `[[maybe_unused]]`

### 5.5 `-Wswitch`（1 个）

- `WasmCodeGen.cpp:1062`：`GenExpr` 的 `switch` 缺少 `ExprKind::InitList` 分支 → **补全分支**

### 5.6 `strncpy` deprecated（5 个）

**根因**：Windows CRT 将 `strncpy` 标记为不安全，建议使用 `strncpy_s`。

**位置**：`cide_capi.cpp` 中 5 处字符串复制。

**修复**：统一替换为 `std::string::copy`（标准 C++，无平台依赖）：
```cpp
// Before
std::strncpy(name, r.name.c_str(), name_size - 1);
name[name_size - 1] = '\0';

// After
size_t copied = r.name.copy(name, name_size - 1);
name[copied] = '\0';
```

---

## 六、验证结果

### 构建
```
[9/9] 构建成功
零 warning
```

### 测试
```
100% tests passed, 0 tests failed out of 6
Total Test time (real) = 0.09 sec
```

### DLL 验证
- `llvm-objdump` 确认链接器版本 `MajorLinkerVersion 14`（LLVM lld-link）
- DLL 正常复制到 `dist/desktop/`，前端无感兼容

---

## 七、后续使用方式

```powershell
# 使用 Clang（推荐日常开发）
.\build.ps1 -Compiler Clang -Configuration Debug

# 使用 ClangCL（MSVC 兼容模式）
.\build.ps1 -Compiler ClangCL -Configuration Debug

# 切回 MSVC
.\build.ps1 -Compiler MSVC -Configuration Debug

# 保持旧行为（自动探测）
.\build.ps1
```

---

## 八、结论

迁移完成。项目现在处于 **Clang 22.1.4 + Ninja + 零警告** 的干净状态。ABI 完全兼容，前端无需任何改动。Clang 的严格诊断已经帮助发现和修复了 60 个潜在代码质量问题。
