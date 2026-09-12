# Stage 2b 内置 C++ 容器模板化迁移笔记

> **已归档（2026-09-13）**：迁移已完成（Phase 34 容器收口 + Phase 41 布局解耦）；文中登记的两条活约束已于 2026-09-12（G13）回填 `docs/current/C++子集规范.md` §4.4。
> 本文件仅作历史追溯保留，内容不再维护；当前有效文档见 `docs/current/`。


**日期**: 2026-06-13  
**状态**: ✅ **迁移已完成**（Phase 34 容器收口 + Phase 41 容器布局解耦），本文档转为**参考笔记**保留  
**范围**: `native/runtime_libc/cide/*.cpp` 内置容器实现，从 `class cide_vec_int { ... }` 改回标准模板写法 `template <class T> class cide_vec { ... };`，配合 `template class cide_vec<int>;` 显式实例化。  
**目标**: 学生代码和标准库实现都写标准 C++，所有妥协通过修复工具链解决，不再用 force-instantiate 桩来绕过编译器。  
**最后核对日期**: 2026-09-11（修订：标注迁移完成与涉及路径现状；把两条活约束（同模板类不可跨文件重复定义、`T()` 值初始化不支持）显式关联到 `CPP_SUBSET_SPEC.md`，并如实记录该规范中检索不到这两条约束的缺口；`native/runtime_libc/cide/` 现为 4 个文件：`vector.cpp` / `list.cpp` / `string.cpp` / `sort_int.cpp`）

---

## 1. 最终文件结构

```
native/runtime_libc/cide/
├── vector.cpp   # template <class T> class cide_vec<T>；显式实例化 int/float/char
├── list.cpp     # template <class T> class cide_list<T> / cide_list_node<T>；显式实例化 int
├── string.cpp   # template <class T> class cide_string<T>；显式实例化 char
└── sort_int.cpp # 自由函数模板 cide_sort_int<T>（函数模板仍用调用桩触发实例化）
```

已删除的文件：

- `vector_int.cpp` / `vector_float.cpp` / `vector_char.cpp`
- `list_int.cpp`
- 旧 `.c` 实现：`vec_int.c` / `vec_float.c` / `vec_char.c` / `list_int.c` / `string.c` / `sort_int.c`

---

## 2. 关键注意事项

### 2.1 同一模板类不能跨文件重复定义

Cide 的 Bytecode Libc 预编译把所有 `runtime_libc/cide/*.cpp` 作为一个多文件工程一起编译，因此：

- **不能把 `template <class T> class cide_vec { ... };` 写在三个文件里**。
- 方案：把 `vector<int/float/char>` 合并到单个 `vector.cpp`，通过显式实例化导出三个特化版本。
- `list<int>` 同理合并到 `list.cpp`。
- `string` 虽然只有 `char` 一种特化，也统一写成 `template <class T> class cide_string<T>`，保持与其他容器风格一致。

> **⚠️ 约束归属缺口（2026-09-11 核实）**：上述约束是**用户可见的语言/工具链约束**（学生自己写模板类时也可能撞上），理应写入面向学生的 [`CPP_SUBSET_SPEC.md`](CPP_SUBSET_SPEC.md)。但实测在该规范中**检索不到**"同一模板类不可跨文件重复定义"这一条（`grep 跨文件` / `重复定义` 均无命中；该文档 §2.7 只覆盖模板支持范围与显式实例化）。**如实记为缺口，本文件不越权修改 `CPP_SUBSET_SPEC.md`**（该文件由他人负责同步）。

### 2.2 编译器暂不支持 `T()` 值初始化

当前 Cide C++ 前端会把 `T()` 解析为"对非函数指针的调用"，导致类型错误。

> **⚠️ 约束归属缺口（2026-09-11 核实）**：与 §2.1 同理，`T()` 值初始化不支持属**面向学生的边界**，但实测在 [`CPP_SUBSET_SPEC.md`](CPP_SUBSET_SPEC.md) 中**检索不到**该条（该文档 §2.7 未列此项）。**如实记为缺口**，不在本文件中越权修改那份规范。

**推荐写法**：

```cpp
T pop_back() {
    if (n == 0) {
        return (T)0;   // ✅ 零初始化，兼容 POD 类型
    }
    return a[--n];
}
```

**避免写法**：

```cpp
return T();  // ❌ 当前编译器报错：无法对非函数指针类型进行调用
```

### 2.3 嵌套模板类需要显式实例化

`list.cpp` 中 `cide_list<T>` 依赖 `cide_list_node<T>`。仅实例化 `cide_list<int>` 时，节点类虽然会被隐式实例化，但当前 TypeChecker 不会把它的字段/方法注册到符号表，导致后续访问 `node->data`、`node->next` 报错。

**解决方案**：在 `list.cpp` 末尾显式实例化节点类：

```cpp
template class cide_list_node<int>;
template class cide_list<int>;
```

### 2.4 模板类名与 mangled 名的对应关系

编译器内置了容器的短名规则（与 `scripts/extract_cpp_builtin_layout.py` 保持一致）：

| 模板基名 | 类型参数 | mangled 类名 | 用户可见名 |
|---|---|---|---|
| `cide_vec` | `int` | `cide_vec_int` | `vector<int>` |
| `cide_vec` | `float` | `cide_vec_float` | `vector<float>` |
| `cide_vec` | `char` | `cide_vec_char` | `vector<char>` |
| `cide_list` | `int` | `cide_list_int` | `list<int>` |
| `cide_string` | `char` | `cide_string` | `string` |

普通模板类仍使用 `Base__T1_T2` 的通用 mangling，不在上表范围内。

### 2.5 删除 `__cide_force_instantiate_*` 容器桩

旧实现为了强制导出容器方法，在每个 `.cpp` 末尾写一个空函数调用所有方法：

```cpp
void __cide_force_instantiate_cide_vec_int() {
    cide_vec_int v;
    v.push_back(0);
    // ...
}
```

现在改为显式实例化 `template class cide_vec<int>;` 后，编译器会自动导出所有 inline 方法，不再需要这类桩函数。

> **例外**：`sort_int.cpp` 是函数模板，当前 Parser 只支持类模板的显式实例化，因此仍保留 `__cide_force_instantiate_cide_sort_int()` 调用桩。待函数模板显式实例化语法支持后可删除。

### 2.6 布局提取脚本需要同步更新

`scripts/extract_cpp_builtin_layout.py` 不再按文件名直接找 `class cide_vec_int`，而是：

1. 按 `FILE_RULES` 找到模板基名（如 `cide_vec`）。
2. 提取 `template <class T> class cide_vec { ... };` 类体。
3. 对每个 `type_arg` 替换类体中的 `T`，再解析字段和方法。
4. 按 mangling 规则生成 `cide_name` 和 `method_map`。

字段正则也需要支持模板指针类型，如 `cide_list_node<int>* head;`。

---

## 3. 测试框架

### 3.1 修改容器源码后的标准流程

```bash
# 1. 提取类布局（生成 builtin_layout_data.json）
python scripts/extract_cpp_builtin_layout.py

# 2. 重新预编译 Bytecode Libc（生成 bytecode_libc_data.json / bytecode_libc_index.rs）
python scripts/precompile_bytecode_libc.py

# 3. 仅检查预编译产物是否与当前源码一致（不重新编译）
python scripts/precompile_bytecode_libc.py --check

# 4. 跑 C++ 容器 Dogfooding 测试
 cd native && cargo test --release --test cpp_dogfooding_test

# 5. 跑 C++ 字节码生成单元测试
cargo test --release --test bytecode_gen_cpp_unit_test

# 6. 跑全量 Rust 测试
cargo test --release
```

### 3.2 关键测试说明

| 测试 | 作用 |
|---|---|
| `cpp_dogfooding_test` | 验证 C++ 容器实现的运行时 stdout 与 Clang++ 一致；验证 `get`/`size` 等方法字节码逐指令等价 |
| `bytecode_gen_cpp_unit_test` | 验证 Parser/TypeChecker/BytecodeGen 对类模板、显式实例化、方法调用生成正确 |
| `bytecode_libc_consistency` | 验证 Bytecode Libc 自身实现（ctype/stdlib/string）编译运行一致 |
| `differential_stress` | 对比 Host 实现与 Bytecode 实现结果是否永远一致 |
| `cide_e2e` | C/C++ 端到端用例回归 |

### 3.3 本次验证结果

```text
cargo test --release --test cpp_dogfooding_test        28 passed
cargo test --release --test bytecode_gen_cpp_unit_test 40 passed
cargo test --release                                    全绿
```

---

## 4. 相关文件

- 容器源码（现状：**4 个文件**）：`native/runtime_libc/cide/vector.cpp`、`list.cpp`、`string.cpp`、`sort_int.cpp`
- 布局脚本：`scripts/extract_cpp_builtin_layout.py`
- 预编译脚本：`scripts/precompile_bytecode_libc.py`
- 产物：`native/crates/cide_cpp_frontend/src/builtin_layout_data.json`（原路径 `native/src/compiler/cpp_frontend/builtin_layout_data.json`，已随 crate 化迁移）
- 产物：`native/crates/cide_vm/src/bytecode_libc_data.json`（原路径 `native/src/vm/bytecode_libc_data.json`）
- 产物：`native/crates/cide_runtime/src/bytecode_libc_index.rs`（原路径 `native/src/vm/bytecode_libc_index.rs`）
- 相关阶段：Phase 34（容器收口：`list<int>` / `vector<char>` / `sort_int`；C++ 三 tier 纳入 CI）、Phase 41（容器布局解耦：`.cpp` 接口声明作为唯一真相来源 + JSON 加载器，零 Rust 硬编码）——见 `AGENTS.md` 的迁移进度表
