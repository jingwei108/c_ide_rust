# Cide 内置容器布局解耦计划：从 Rust 硬编码到 .cpp 真相来源

**版本**: 1.0  
**日期**: 2026-06-10  
**状态**: 已完成 ✅  
**依赖**: C++ 编译器已支持 `.cpp` 扩展名自动检测（`compile_pipeline.rs` 540 行）

---

## 一、问题陈述

当前 `native/src/compiler/cpp_frontend/` 中存在**三重冗余**：

| 信息 | 真相来源 A | 真相来源 B | 真相来源 C |
|------|-----------|-----------|-----------|
| `vector<int>` 字段布局 | `runtime_libc/cide/vec_int.c` | `builtin_layout.rs` 硬编码 | `layouts.toml`（未使用） |
| `push_back` 方法签名 | `runtime_libc/cide/vec_int.c` | `builtin_layout.rs` 硬编码 | `layouts.toml`（未使用） |
| `vector<int>` → `cide_vec_int` 映射 | — | `type_map.rs` 硬编码 | — |
| `push_back` → `cide_vec_push_int` 映射 | — | `type_map.rs` 硬编码 | — |

**后果**：
- 新增一个容器类型需要同时修改 `.c` + Rust 代码 3 个文件，极易遗漏
- `builtin_layout.rs` 和 `type_map.rs` 中的 `Type::int()` 等调用是**不可被外部工具验证**的 Rust 代码
- `layouts.toml` 已经存在但与代码不同步（文档里写了，实际代码没加载）
- `.c` 文件虽可被 Clang 编译，但**按 C 模式编译**，不验证 C++ 语法（如 class/struct 差异、方法重载等）

**根本矛盾**：Cide 已经能编译 `.cpp`（`is_cpp_mode` 自动检测），但内置容器的元数据仍被困在 Rust 硬编码和 `.c` 文件里，没有利用 `.cpp` 的语义能力。

---

## 二、目标

1. **唯一真相来源**：容器布局与方法签名仅存于 `.cpp` 文件中
2. **零 Rust 硬编码**：`builtin_layout.rs` 和 `type_map.rs` 变为纯 JSON 加载器，新增容器无需改 Rust 代码
3. **可被 Clang++ 验证**：`.cpp` 文件必须能通过 `clang++ -fsyntax-only` 编译检查
4. **可被 Cide 编译**：`.cpp` 文件在 Cide C++ 模式下能正确编译（短期至少能解析提取，长期能生成字节码）
5. **向后兼容**：Stage 0 的 `.c` 字节码实现短期共存，逐步过渡到 `.cpp` 直接生成字节码

---

## 三、架构设计

```
runtime_libc/cide/*.cpp     <- 唯一真相来源（C++ 语法）
  |- vector_int.cpp: class vector<int> { ... }
  |- vector_float.cpp: class vector<float> { ... }
  |- vector_char.cpp: class vector<char> { ... }
  |- string.cpp: class string { ... }
  |- list_int.cpp: class list<int> { ... }
         |
         v
scripts/extract_cpp_builtin_layout.py
  轻量解析（正则 + 简单状态机），提取：
  - class/struct 字段名与类型
  - public 方法签名（参数、返回类型）
  - template 实例化类型推导
  输出：builtin_layout_data.json
         |
         v
native/src/compiler/cpp_frontend/builtin_layout_data.json
  构建期生成，版本控制，编译时通过 include_str! 嵌入
         |
         v
Rust 运行时加载（零硬编码）
  |- builtin_layout.rs: HashMap<String, ClassLayout> 加载器
  |- type_map.rs: HashMap 加载器（自动生成映射）
```

---

## 四、`.cpp` 文件规范

### 4.1 设计原则

- **纯接口声明**：当前阶段 `.cpp` 文件只包含 `class` / `struct` 定义 + 方法声明，**不包含实现**
- 实现仍由 `.c` 文件提供（`cide_vec_push_int` 等全局函数）
- 未来 Stage 2 直接在 `.cpp` 中写方法体，替换 `.c`
- **文件扩展名 `.cpp`**：确保 `compile_pipeline.rs` 的扩展名检测生效

### 4.2 文件模板

```cpp
// runtime_libc/cide/vector_int.cpp
// Cide 内置容器 vector<int> 的 C++ 接口声明
// 当前实现由 cide_vec_*.c 提供，未来替换为纯 C++ 实现

#ifndef CIDE_BUILTIN_CONTAINER
#define CIDE_BUILTIN_CONTAINER

template<class T>
class vector {
    int n;
    int m;
    T* a;
public:
    vector();
    void push_back(T x);
    T pop_back();
    int size();
    int capacity();
    T front();
    T back();
    void pop_front();
    T get(int i);
    void clear();
    ~vector();
};

// 显式实例化，提取脚本据此推导 cide_vec_int
template class vector<int>;

#endif
```

### 4.3 命名约定（提取脚本自动推导）

| C++ 语法 | 推导规则 | 示例 |
|----------|---------|------|
| `template class vector<int>;` | 类型名 = `cide_vec_int` | `vector` + `int` → `cide_vec_int` |
| `template class list<int>;` | 类型名 = `cide_list_int` | `list` + `int` → `cide_list_int` |
| `class string` | 类型名 = `cide_string` | `string` → `cide_string` |
| `void push_back(T x)` | 方法名保留 | `push_back` |
| `T get(int i)` | 方法名保留，返回类型 `T` 按实例化替换 | `get` → 返回 `int` |

**C 函数名映射规则**（提取脚本内置小表）：
- `cide_vec_` + `{method}` + `_` + `{type}`
- 例：`vector<int>::push_back` → `cide_vec_push_int`

### 4.4 Clang++ 验证

每个 `.cpp` 文件必须能通过：

```bash
clang++ -fsyntax-only -std=c++14 runtime_libc/cide/vector_int.cpp
```

CI 中增加此检查，确保 `.cpp` 语法合法。

---

## 五、提取脚本设计

### 5.1 `scripts/extract_cpp_builtin_layout.py`

输入：`runtime_libc/cide/*.cpp`
输出：`native/src/compiler/cpp_frontend/builtin_layout_data.json`

解析策略（轻量，不引入完整 C++ parser）：

```python
# 1. 提取显式实例化
#    template class vector<int>;
#    -> cpp_name="vector<int>", cide_name="cide_vec_int"

# 2. 提取 class 定义
#    class vector { int n; int m; T* a; public: ... };
#    -> fields=[("n","int"),("m","int"),("a","T*")]

# 3. 提取 public 方法声明
#    void push_back(T x);
#    -> MethodSig(name="push_back", params=["T"], ret="void")

# 4. 类型替换：将模板参数 T 替换为实例化类型
#    T -> int, T* -> int*

# 5. 计算 size（简单累加：int=4, int*=4, float=4, char=1）
#    未来可调用 clang 获取准确 sizeof
```

### 5.2 JSON 输出格式

```json
{
  "version": 1,
  "generated_at": "2026-06-10T11:26:03",
  "classes": {
    "cide_vec_int": {
      "cpp_name": "vector<int>",
      "source_file": "runtime_libc/cide/vector_int.cpp",
      "size": 12,
      "fields": [
        {"name": "n", "type": "int"},
        {"name": "m", "type": "int"},
        {"name": "a", "type": "int*"}
      ],
      "methods": [
        {"name": "push_back", "params": ["int"], "ret": "void", "is_virtual": false},
        {"name": "pop_back", "params": [], "ret": "int", "is_virtual": false},
        {"name": "size", "params": [], "ret": "int", "is_virtual": false},
        {"name": "capacity", "params": [], "ret": "int", "is_virtual": false},
        {"name": "front", "params": [], "ret": "int", "is_virtual": false},
        {"name": "back", "params": [], "ret": "int", "is_virtual": false},
        {"name": "pop_front", "params": [], "ret": "void", "is_virtual": false},
        {"name": "get", "params": ["int"], "ret": "int", "is_virtual": false},
        {"name": "clear", "params": [], "ret": "void", "is_virtual": false},
        {"name": "destroy", "params": [], "ret": "void", "is_virtual": false}
      ]
    }
  }
}
```

### 5.3 方法映射自动生成

提取脚本同时生成 `method_map` 段，替换 `type_map.rs` 的硬编码：

```json
{
  "method_map": {
    "cide_vec_int": {
      "push_back": "cide_vec_push_int",
      "pop_back": "cide_vec_pop_int",
      "size": "cide_vec_size_int"
    }
  }
}
```

生成规则：`cide_{class_prefix}_{method}_{type_suffix}`
- `vector<int>` → prefix=`vec`, type_suffix=`int`
- `string` → prefix=`string`, type_suffix=""

---

## 六、Rust 加载器重构

### 6.1 `builtin_layout.rs` 重写目标

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClassLayout {
    pub size: i32,
    pub fields: Vec<(String, String)>, // 类型存字符串，解析时转换为 Type
    pub methods: Vec<MethodSig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MethodSig {
    pub name: String,
    pub params: Vec<String>,
    pub ret: String,
    pub is_virtual: bool,
}

pub fn load_builtin_layouts() -> HashMap<String, ClassLayout> {
    let json = include_str!("builtin_layout_data.json");
    let data: LayoutData = serde_json::from_str(json)
        .expect("Failed to parse builtin_layout_data.json");
    data.classes
}
```

**注意**：字段类型和方法签名的类型从 `String` 转为 `Type` 枚举时，增加一个 `parse_type_str()` 函数（解析 `"int"`, `"int*"`, `"void"` 等），替代当前硬编码的 `Type::int()`。

### 6.2 `type_map.rs` 重写目标

```rust
pub fn cpp_type_to_cide(name: &str) -> Option<&str> {
    let json = include_str!("builtin_layout_data.json");
    // 从 json.classes[*].cpp_name 反向查找 cide_name
}

pub fn map_container_method(class_name: &str, method: &str) -> Option<&str> {
    let json = include_str!("builtin_layout_data.json");
    // 从 json.method_map 查找
}
```

**优化**：为避免每次调用都解析 JSON，使用 `std::sync::LazyLock` 一次性加载并缓存：

```rust
static LAYOUT_DATA: LazyLock<LayoutData> = LazyLock::new(|| {
    let json = include_str!("builtin_layout_data.json");
    serde_json::from_str(json).unwrap()
});
```

---

## 七、预编译脚本扩展

### 7.1 `scripts/precompile_bytecode_libc.py`

当前第 69 行只收集 `.c`：

```python
if f.endswith(".c")
```

修改为同时收集 `.c` 和 `.cpp`：

```python
if f.endswith(".c") or f.endswith(".cpp")
```

**短期行为**：
- `.c` 文件：正常预编译为字节码（实现代码）
- `.cpp` 文件：不预编译为字节码（当前只有声明），仅用于提取布局

**长期行为（Stage 2）**：
- 删除 `.c` 文件
- `.cpp` 文件包含完整实现，直接预编译为字节码
- `cide_cli export` 已经支持 `.cpp`（`compile_pipeline.rs` 自动检测扩展名）

### 7.2 CI 集成

在 `.github/workflows/ci.yml` 中增加：

```yaml
- name: Verify C++ builtin layout syntax
  run: |
    for f in native/runtime_libc/cide/*.cpp; do
      clang++ -fsyntax-only -std=c++14 "$f"
    done

- name: Regenerate builtin layout data
  run: python scripts/extract_cpp_builtin_layout.py

- name: Check builtin layout data is up-to-date
  run: |
    git diff --exit-code native/src/compiler/cpp_frontend/builtin_layout_data.json
```

---

## 八、实施步骤

### Step 1: 创建 `.cpp` 接口文件 ✅

| 文件 | 说明 |
|------|------|
| `native/runtime_libc/cide/vector_int.cpp` | `template class vector<int>` 声明 |
| `native/runtime_libc/cide/vector_float.cpp` | `template class vector<float>` 声明 |
| `native/runtime_libc/cide/vector_char.cpp` | `template class vector<char>` 声明 |
| `native/runtime_libc/cide/string.cpp` | `class string` 声明 |
| `native/runtime_libc/cide/list_int.cpp` | `template class list<int>` 声明 |

验证：`clang++ -fsyntax-only` 全部通过。

### Step 2: 写提取脚本 `scripts/extract_cpp_builtin_layout.py` ✅

- 解析 `.cpp` 文件
- 生成 `builtin_layout_data.json`
- 同时生成 `method_map` 段
- 验证：JSON 与 Rust 硬编码完全等价

### Step 3: 重写 Rust 加载器 ✅

- `builtin_layout.rs`：改为 JSON 加载 + `parse_type_str()`
- `type_map.rs`：改为 JSON 加载
- `codegen/mod.rs` & `typeck/cpp_class_layout.rs`：硬编码 `container_mappings` 改为动态遍历 `builtin_class_mappings()`
- 删除 `native/runtime_libc/cide/layouts.toml`

### Step 4: 修改预编译脚本 ✅

- `precompile_bytecode_libc.py`：扩展 glob 支持 `.cpp`
- `.cpp` 当前阶段不生成字节码（只有声明），仅 `.c` 传入 `export`

### Step 5: 测试与验证 ✅

- `cargo test` 全部通过（44 lib + 600+ 集成测试）
- Dogfooding 测试 25 个全部通过
- C++ 相关测试（bytecode_gen_cpp / typeck_cpp / parser_cpp）全部通过
- Host Contract / Bytecode Libc Consistency / Differential Stress 全部通过

### Step 6: 文档更新 ✅

- 更新 `CHANGELOG.md`
- 更新本文档状态为「已完成」

**总计：约 5 个工作日**

---

## 九、验收标准

| # | 验收项 | 验证方式 |
|---|--------|---------|
| 1 | 新增容器无需修改 Rust 代码 | 仅添加 `foo.cpp` + 运行提取脚本，编译通过 |
| 2 | `builtin_layout.rs` 无硬编码容器信息 | `grep -n "cide_vec_int\|cide_string" native/src/compiler/cpp_frontend/builtin_layout.rs` 无匹配 |
| 3 | `type_map.rs` 无硬编码映射 | `grep -n "cide_vec_push" native/src/compiler/cpp_frontend/type_map.rs` 无匹配 |
| 4 | 全部测试通过 | `cargo test` 全绿 |
| 5 | Clang++ 语法验证 | `for f in *.cpp; do clang++ -fsyntax-only "$f"; done` 零报错 |
| 6 | JSON 与 `.cpp` 同步检查 | CI 中 `--check` 模式通过 |

---

## 十、风险与缓解

| 风险 | 可能性 | 影响 | 缓解 |
|------|--------|------|------|
| 轻量解析器无法处理复杂 C++ 语法 | 低 | 提取失败 | 当前只解析简单 class 模板，语法可控；若未来需要复杂语法，再引入 libclang 提取 |
| `parse_type_str()` 与 Cide `Type` 枚举不同步 | 中 | 运行时类型错误 | 单元测试覆盖所有内置类型字符串解析；CI 检查 |
| `.cpp` 与 `.c` 实现不同步 | 中 | 布局一致但行为不一致 | Dogfooding 测试保证 stdout 一致；短期 `.cpp` 只有声明，不同步风险极低 |
| 删除 `layouts.toml` 导致其他工具依赖断裂 | 低 | 构建失败 | 先 grep 全仓库确认无引用，再删除 |

---

## 十一、长期愿景（Stage 2）

当 Cide C++ 编译器成熟后：

1. `.cpp` 文件写入完整方法实现（`vector<int>::push_back(int x) { ... }`）
2. 删除所有 `.c` 文件
3. `precompile_bytecode_libc.py` 直接从 `.cpp` 编译字节码
4. `cide_cli export` 处理 `.cpp` 生成 `bytecode_libc_data.json`
5. 提取脚本与预编译脚本合并：同一套 `.cpp` 既提取布局元数据，又生成字节码

此时，内置容器的**全部信息**（布局 + 实现 + 字节码）都来自 `.cpp`，Rust 中零硬编码，完全符合 C++ 子集拓展的 Dogfooding 哲学。

---

**计划制定**: Kimi Code CLI  
**状态**: 待审批  
**完成日期**: 2026-06-10
