# Cide C++14 教学子集拓展实施计划

**版本**: 2.6  
**日期**: 2026-06-10  
**状态**: Stage 1 Dogfooding 验证推进完成：`vector<int/float/char>`、`string`、`list<int>` 运行时 stdout 与 C 基线完全一致；`get`/`size` 等简单方法字节码逐指令等价验证通过（StepEvent 归一化 + 启动代码排除）；`sort_int` C++ 全局函数实现运行时一致性验证通过；**M5 隐式移动构造函数自动生成已实现**：类定义含指针/资源字段时，编译器自动生成 `__ctor__{Class}__move`，`std::move` 初始化对象时调用，并将源对象指针字段置空防止双重释放；新增 17 个 Dogfooding 测试（总计 28 个，全绿）；全部 600+ Rust 单元测试保持全绿，C++ 三 tier 已纳入 CI  
**前置依赖**: `C_SUBSET_SPEC.md` P0/P1 阶段完成、Phase 31~33 C++ Parser/TypeChecker/BytecodeGen 完成

---

## 变更摘要（v2.0 → v2.1）

| 问题 | v2.0 | v2.1 |
|------|------|------|
| 前置条件状态 | 将 static、goto、#ifdef、volatile、qsort 等标记为"待实现" | **已修正为"已实现"**（代码事实） |
| 错误码范围 | 声称保护 E1001~E3061 | **已修正为 E1001~E3071**，C++ 预留 E4001~E4999 |
| 容器库策略 | 直接复制 klib 头文件 | **Stage 0：手写 C 容器（临时）→ Stage 1：Dogfooding 用 Cide C++ 编译器编译 C++ 容器源码 → Stage 2：替换为 C++ 实现** |
| 容器库时机 | 全部前置 | **类型布局前置（硬编码），算法实现后置** |
| 依赖问题 | 示例使用 `lazy_static!`（不在 Cargo.toml） | **改为 `std::sync::LazyLock`（Rust 1.95 已支持）或普通函数** |
| 内部矛盾 | Dogfooding 示例出现被排除的 `operator[]` | **已删除运算符重载示例** |
| 预编译兼容 | 假设 klib `.h` 可直接预编译 | **明确预编译脚本仅支持 `.c` 文件** |

---

## 一、愿景与目标

### 1.1 为什么需要 C++

Cide 的 C 子集已覆盖 95% 教学场景，但国内考研/竞赛/工程教学的绝对主流语言是 **C++14**。不支持 Lambda、移动语义、`unique_ptr`、`auto`、`范围 for` 的子集没有竞争力。

### 1.2 核心约束（铁律）

| 约束 | 说明 |
|------|------|
| **零改动 VM** | CideVM（1MB 线性内存、栈式字节码）不做任何修改 |
| **不复用 Clang** | 不引入 libclang 或任何外部 C++ 编译器，保持自研可控 |
| **BytecodeGen 线性扩展** | TypeChecker 直接处理 C++ 语义，BytecodeGen 新增 C++ 节点分支 |
| **诊断体系保护** | E1001~E3071 错误码、知识图谱、UAF/DF 检测、堆内存可视化不得破坏 |
| **测试哲学延续** | All in. Record don't hide. Fix real bugs, not test cases. |
| **狗吃自己狗粮** | 无论是开始时的c写cpp容器，还是后期的cpp写cpp容器等类似场景，都以尽可能以工业的标准去写，如文档提到的以klib为参照标准。不因本项目的缺陷而扭曲代码，如发现标准代码不支持，小问题就修复，大问题记录下来。
### 1.3 C++14 边界

**支持（P1~P5）**:
- Lambda（含泛型，带捕获）
- 右值引用 / 移动语义 / `std::move`
- `unique_ptr` / `shared_ptr`（简化版，无自定义 deleter）
- `auto` 类型推导
- 范围 `for`
- 模板单态化（仅类型参数，无 SFINAE/特化/偏特化）
- 单继承 + 虚函数多态
- `class` / `public` / `private` / `this`
- `nullptr` / `bool` / `true` / `false`
- `new` / `delete`（映射为 malloc/free Host Func）

**排除**:
- 异常（try/catch/throw）
- 多重继承
- RTTI（typeid/dynamic_cast）
- **运算符重载**（教学价值低，复杂度极高，明确排除）
- SFINAE / 模板特化 / 偏特化
- 标准 STL（依赖异常/allocator/复杂元编程，VM 不支持）

### 1.4 编写层面的原生 C++ 兼容性（Honest Subset）

**目标**：让学生用"看起来像原生 C++"的代码编写，但明确知道边界在哪里。

#### 可以做到 80% 语法兼容（编译期映射）

```cpp
// ===== 学生写的（标准 C++ 语法）=====
#include <vector>
#include <algorithm>
#include <memory>
using namespace std;

int main() {
    vector<int> v = {3, 1, 4};
    sort(v.begin(), v.end());
    auto f = [](int x) { return x * 2; };
    unique_ptr<int> p(new int(42));
    for (auto x : v) { printf("%d\n", x); }
    return 0;
}
```

```cpp
// ===== Cide 编译器内部处理（直接生成）=====
// 1. 头文件映射: <vector> → 编译器内置类型
// 2. 命名空间擦除: using namespace std; → 删除, std::vector → vector
// 3. 语法处理:
//    vector<int> v = {...}  → 类布局已知，生成初始化字节码
//    auto f = [](int x)       → struct __lambda_0 { ... };
//    unique_ptr<int> p        → struct unique_ptr_int p; ...
//    for (auto x : v)         → 范围 for AST 节点，BytecodeGen 展开为索引循环
```

#### 做不到的是语义兼容（架构限制）

```cpp
// 以下代码无法兼容，VM 不支持，会报明确错误
try { throw runtime_error("err"); } catch (...) {}  // [E4001] 不支持异常
class A { A operator+(const A& o); };                // [E4002] 不支持运算符重载
template<> struct max<int> { ... };                   // [E4003] 不支持模板特化
std::thread t([]{ ... });                             // [E4004] 不支持多线程
```

| 维度 | 能做到 | 做不到 |
|------|--------|--------|
| `std::` 前缀 | ✅ 自动擦除 | |
| 标准头文件名 | ✅ 映射到 cide 头文件 | |
| Lambda / auto / 范围 for | ✅ 完全一致 | |
| 模板基本语法 | ✅ 一致（受限） | ❌ SFINAE / 特化 / 元编程 |
| class 语法 | ✅ 基本一致 | ❌ 运算符重载 |
| 异常 / 多线程 | | ❌ 不支持 |
| 标准库完整语义 | | ❌ allocator / 异常安全 |

#### Honest Subset 原则

> **不是"假装是 C++"，而是"诚实标注这是教学子集"。**

- 文件扩展名：`.cidecpp`（可选）或 `.cpp`，IDE 标题栏显示 "Cide C++ 子集"
- 每个特性文档标注 `[标准 C++ 差异]`
- `--show-lowered` 让学生看到 `vector<int>` 底层是什么
- 教材最后一章："从 Cide C++ 到标准 C++"

---

## 二、前置条件修正（基于代码事实）

C++ 拓展**不可**在以下工作完成前启动：

| 前置任务 | v2.0 状态 | **v2.1 修正（代码事实）** | 说明 |
|----------|-----------|--------------------------|------|
| `static` 完整语义 | ⏳ 待实现 | **✅ 已实现** | `BytecodeGen` 已有 `static_local_indices`/`static_local_types`（`codegen/mod.rs` 行 51-52、638-657） |
| `goto` | ⏳ 待实现 | **✅ 已实现** | `TypeChecker` 已有 `func_labels`/`pending_gotos`（`typeck/mod.rs` 行 50-51、693-711），错误码 `E3071_UndefinedLabel` 已存在 |
| `volatile` | ⏳ 待实现 | **✅ 已实现** | `TokenType::Volatile` 已存在，`Type` 系统已支持修饰符 |
| `#ifdef` / `#ifndef` / `#endif` | ⏳ 待实现 | **✅ 已实现** | `Lexer` 已有 `conditional_stack` 和条件编译状态机 |
| `<limits.h>` / `<stdbool.h>` | ⏳ 待实现 | **✅ 已实现** | `runtime_libc/include/` 下已存在 |
| `qsort` | ⏳ 待实现 | **✅ 已实现** | `C_SUBSET_SPEC.md` 及 `TypeChecker::visit_call()` 已支持 |
| `puts` / `sprintf` / `calloc` / `bsearch` | ⏳ 待实现 | **✅ 已实现** | `stdlib.h` 已声明，`TypeChecker` 已注册内建函数检查器 |

**事实结论**：C 子集 P0/P1 的绝大部分工作已经完成，前置条件不再是 C++ 拓展的阻塞项。

---

### `volatile` C++ 语义补充

`volatile` 在 C 模式下已实现（`TokenType::Volatile`、`Type` 修饰符支持），C++ 模式下的语义定义如下：

| 场景 | C 模式行为 | C++ 模式行为 | 说明 |
|------|-----------|-------------|------|
| `volatile int x` | 修饰符标记，代码生成与普通 `int` 相同 | 同 C 模式 | 教学子集不模拟内存屏障 |
| `volatile` 成员变量 | struct/union 字段 | class 字段，语义同 C | 布局计算中 `volatile` 不影响 `sizeof` |
| `const volatile` 组合 | 支持 | 支持 | 顺序无关，`is_const && is_volatile` 同时标记 |
| `volatile` 成员函数 | N/A（C 无成员函数） | **不支持** | 明确排除，遇 `volatile` 成员函数报 `E4006_VolatileMemberNotSupported` |
| 模板参数中的 `volatile` | N/A | **不支持** | `volatile T` 作为模板参数实例化后保留修饰符，但无特殊代码生成 |

**结论**：`volatile` 在 Cide C++ 子集中仅作为**类型修饰符标记**存在，不生成特殊内存屏障指令，与 C 模式保持完全一致。

---

## 三、架构概览：混合架构（TypeChecker + BytecodeGen 直接扩展）

```
┌─────────────────────────────────────────────────────────────┐
│                      学生代码输入层                          │
│         C++14 语法（vector<int>、Lambda、auto...）           │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Parser（扩展）                            │
│       C++14 语法 → C++ AST（单一 AST，混合 C/C++ 节点）       │
│              class / Lambda / template / 范围 for             │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                 TypeChecker（扩展）                          │
│     • auto 类型推导（替换为具体类型）                         │
│     • 模板单态化（生成 FuncDecl 插入 Program）               │
│     • 类/继承/vtable 布局分析                                │
│     • 引用 & / && 语义标记                                   │
│     • 重载决议（移动构造/拷贝构造优先级）                     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                轻量降解层（仅容器 + 语法糖）                  │
│     • vector.push_back → __cide_vec_push_int               │
│     • 范围 for → 索引循环 AST                                │
│     • Lambda → 闭包 struct + 函数（AST 变换）                │
│     • unique_ptr/shared_ptr → struct + 手动 destroy          │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                 BytecodeGen（扩展）                          │
│     • This / MemberCall（非虚函数）                          │
│     • MemberCall（虚函数 = vtable 间接调用）                  │
│     • New/Delete（HostCall malloc/free）                     │
│     • 其他节点复用现有 C 分支                                │
│     • SourceLoc 天然保留（原始 C++ 代码位置）                │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                现有 VM（零改动）                             │
│              CideVM（1MB 线性内存，栈式字节码）               │
└─────────────────────────────────────────────────────────────┘
```

### 3.1 容器库基座：参照 klib 手写 C 实现

**不是直接引入 klib 头文件，而是参照 klib 的算法设计，手写 Cide-C 子集兼容的 `.c` 文件**。

| 误区 | 正解 |
|------|------|
| "直接复制 klib `.h` 文件" | klib 是头文件宏泛型（`kvec_t(int)`），Cide 预编译脚本只编译 `.c` 文件 |
| "klib 零修改即可使用" | klib 宏泛型与 Cide `Type` 枚举没有"宏展开"概念，TypeChecker 无法直接理解 |
| "用 C++ 写容器给 Cide 用" | Cide 编译器目前不支持 C++ 语法，`cide_cli` 只能编译 `.c` 文件 |

**核心洞察**：参照 klib 的极简算法（kvec 核心仅 3 个字段 + realloc 扩容），手写每种容器×每种类型的 `.c` 文件，直接放入 `runtime_libc/cide/` 预编译。

### 3.2 编译器双模式

| 模式 | 名称 | 行为 | 适用场景 |
|------|------|------|----------|
| **Mode A** | `CppExplicit` | 学生显式写 `__init` / `__destroy` | 进阶教学，理解 RAII 本质 |
| **Mode B** | `CppImplicit` | 编译器自动插入作用域守卫析构 | 入门教学，降低心智负担 |

---

## 四、H1 详细方案：Cide 容器库（参照 klib 自研）

### 4.1 Dogfooding 路线：C 基底 → C++ 替换

> **手写 C 容器是临时方案（Stage 0），最终目标是用 Cide C++ 编译器编译 C++ 容器源码（Stage 1 Dogfooding），验证通过后替换 C 实现（Stage 2）。**

**注意**：Cide 编译器本身用 Rust 编写，不存在"自举"（编译器不能用自身编译）。此处是用 Cide C++ 编译器去编译 C++ 容器库，属于 **Dogfooding**（开发团队用自己的产品解决自己的问题），是验证编译器正确性的终极手段。

**时序约束（关键）**：Stage 1 必须在 C++ 编译器核心（Parser→TypeChecker→BytecodeGen）全部完成后才能启动，不存在循环依赖：
1. **Stage 0** 现在就能做：手写 C 容器，预编译为字节码，学生代码立即调用
2. **Phase 1-3** 实现 C++ 编译器核心（约 4.5 个月）
3. **Stage 1** 编译器完成后：用 C++ 编译器编译 C++ 容器源码，做 Dogfooding 验证
4. **Stage 2** 验证通过后：删除 C 容器，完全替换为 C++ 实现

```
Stage 0（现在 ──→ Phase 1-3 ──→ Stage 1 ──→ Stage 2）
  C 容器（手写）    编译器核心      C++ 容器源码      C++ 容器（运行时）
  预编译为 BC Libc  （Parser/       用 Cide C++       完全替换 C 实现
  学生代码调用      TypeChecker/    编译器编译
                    BytecodeGen）   字节码对比验证
```

| 阶段 | 容器实现语言 | 编译方式 | 用途 | 时机 |
|------|-------------|----------|------|------|
| **Stage 0** | C（手写极简） | `precompile_bytecode_libc.py` | 学生代码运行时调用 | **现在** |
| **Stage 1** | Cide C++ 子集 | Cide C++ 编译器 | Dogfooding + 教学源码展示 | **C++ 编译器核心完成后** |
| **Stage 2** | Cide C++ 子集 | Cide C++ 编译器 | **完全替换 Stage 0** | **Dogfooding 验证通过后** |

**Stage 1 的验证方法**：
1. 用 Cide C++ 子集写 `cide::vector<int>`（class template + 构造函数 + push_back）
2. 用 Cide C++ 编译器编译 → 生成字节码 A
3. 对比 Stage 0 的 C 版本预编译字节码 B
4. 如果 A ≡ B（逐指令一致）→ 编译器正确，可进入 Stage 2（Dogfooding 通过）

### 4.2 设计原则

- **Stage 0 参照 klib 算法**：数据结构设计和扩容策略与 klib 保持一致（已验证的工业级算法）
- **Cide-C 子集编写**：使用项目已支持的 C 语法（`runtime_libc/src/string.c` 和 `stdlib.c` 的风格）
- **预编译友好**：每个容器类型是独立的 `.c` 文件，直接由 `scripts/precompile_bytecode_libc.py` 编译
- **标准库后置**：容器算法实现（`.c` 文件）在编译器核心完成后补充，但类型布局信息前置硬编码
- **为未来替换留接口**：C 容器的函数签名和内存布局与目标 C++ 容器一致，确保 Stage 1/2 平滑替换

### 4.2 容器组件

| 组件 | 参照来源 | 对应 C++ STL | Cide 优先级 | 说明 |
|------|----------|-------------|------------|------|
| `cide_vec` | kvec.h | `vector` | ✅ P0 | 动态数组，核心结构 `{ int n, m; T *a; }` |
| `cide_list` | klist.h | `list` / `forward_list` | ✅ P0 | 单向链表，简化版（不做内存池） |
| `cide_string` | kstring.h | `string` | ✅ P0 | 动态字符串，简化版（不做 printf/vsprintf） |
| `cide_deque` | kdq.h | `deque` | ⚠️ P2 | 双端队列 |
| `cide_hash` | khash.h | `unordered_map` / `unordered_set` | ⚠️ P3 | 开放寻址哈希表 |
| `cide_sort` | ksort.h | `algorithm::sort` | ✅ P1 | 内省/归并/堆排序 |

### 4.3 参照实现示例（vector<int>）

```c
// native/runtime_libc/cide/vec_int.c
// 参照 kvec.h 算法，用 Cide-C 子集重写

typedef struct {
    int n;      /* 当前元素数 */
    int m;      /* 容量 */
    int *a;     /* 数据指针 */
} cide_vec_int;

void cide_vec_init_int(cide_vec_int *v) {
    v->n = 0;
    v->m = 0;
    v->a = 0;
}

void cide_vec_push_int(cide_vec_int *v, int x) {
    if (v->n == v->m) {
        v->m = v->m ? v->m << 1 : 2;
        v->a = (int *)realloc(v->a, sizeof(int) * v->m);
    }
    v->a[v->n++] = x;
}

int cide_vec_pop_int(cide_vec_int *v) {
    return v->a[--v->n];
}

int cide_vec_size_int(cide_vec_int *v) {
    return v->n;
}

int cide_vec_get_int(cide_vec_int *v, int i) {
    return v->a[i];
}

void cide_vec_clear_int(cide_vec_int *v) {
    v->n = 0;
}

void cide_vec_destroy_int(cide_vec_int *v) {
    free(v->a);
}
```

**与 klib 的对应关系**：

| klib 宏 | 参照实现函数 |
|---|---|
| `kvec_t(int)` | `cide_vec_int` |
| `kv_init(v)` | `cide_vec_init_int(&v)` |
| `kv_push(int, v, x)` | `cide_vec_push_int(&v, x)` |
| `kv_pop(v)` | `cide_vec_pop_int(&v)` |
| `kv_size(v)` | `cide_vec_size_int(&v)` |
| `kv_A(v, i)` | `cide_vec_get_int(&v, i)` |
| `kv_destroy(v)` | `cide_vec_destroy_int(&v)` |

### 4.4 集成目录结构

```
native/runtime_libc/
├── include/                    # 已有：stdio.h / stdlib.h / ctype.h / math.h / string.h
├── src/                        # 已有：ctype.c / stdlib.c / string.c
└── cide/                       # 新增：Cide 容器库（.c 文件，直接编译）
    ├── vec_int.c               # 动态数组 int 型
    ├── vec_float.c             # 动态数组 float 型
    ├── vec_char.c              # 动态数组 char 型
    ├── list_int.c              # 单向链表 int 型
    ├── string.c                # 动态字符串
    └── sort_int.c              # int 数组排序
```

### 4.5 预编译流程

复用现有 `scripts/precompile_bytecode_libc.py`：**零改动**。脚本已支持编译 `runtime_libc/src/*.c`，只需将 glob 模式扩展为包含 `runtime_libc/cide/*.c`。

```python
# scripts/precompile_bytecode_libc.py（无需修改逻辑，只需扩展路径）
# 现有：编译 runtime_libc/src/*.c
# 新增：编译 runtime_libc/cide/*.c
# 统一生成 bytecode_libc_data.json
# 索引 key 格式: "cide_vec_init_int", "cide_vec_push_int", ...
```

**事实依据**：现有脚本第 62-63 行遍历 `.c` 文件，第 71 行调用 `cide_cli export`。`.cide/*.c` 文件与现有 `.src/*.c` 文件在编译流程上无差异。

### 4.6 类型映射表（编译器内置）

```rust
// native/src/compiler/cpp_frontend/type_map.rs
// Rust 1.95 支持 std::sync::LazyLock，无需引入 lazy_static crate

use std::collections::HashMap;
use std::sync::LazyLock;

static KLIB_TYPE_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("vector<int>", "cide_vec_int");
    m.insert("vector<float>", "cide_vec_float");
    m.insert("vector<char>", "cide_vec_char");
    m.insert("list<int>", "cide_list_int");
    m.insert("string", "cide_string");
    m
});
```

### 4.7 方法映射表

| Cide C++ 语法 | 映射为容器库调用 | 说明 |
|--------------|----------------|------|
| `vector<int> v;` | `cide_vec_int v; cide_vec_init_int(&v);` | 构造 = struct 定义 + init 调用 |
| `v.push_back(x);` | `cide_vec_push_int(&v, x);` | 插入，自动扩容 |
| `v[i]` | `cide_vec_get_int(&v, i)` | 索引访问（无边界检查） |
| `v.size()` | `cide_vec_size_int(&v)` | 元素个数 |
| `v.pop_back()` | `cide_vec_pop_int(&v)` | 尾部弹出 |
| `v.clear()` | `cide_vec_clear_int(&v)` | 逻辑清空 |
| `list<int> l;` | `cide_list_int l;` | 链表 |
| `l.push_back(x);` | `cide_list_push_back_int(&l, x);` | 尾部插入 |
| `l.push_front(x);` | `cide_list_push_front_int(&l, x);` | 头部插入 |

### 4.8 内存适配

容器库使用 `malloc`/`realloc`/`free`。Cide 已将这些映射为 VM Host Call（`HostMalloc`/`HostFree`），**无需修改容器库源码**。

**VM 内存模型事实**：
- `size_t` 在 Cide 中为 `unsigned int`（32 位，`runtime_libc/include/stdlib.h` 行 1）
- `malloc` 参数类型为 `int`（`stdlib.h` 行 3：`void* malloc(int size)`）
- 容器库中统一使用 `int` 表示大小和容量，与 VM 字长一致

---

## 五、容器类型布局前置方案

### 5.1 为什么类型布局必须前置

编译器核心开发阶段（Parser/TypeChecker/BytecodeGen）需要测试 `vector<int>` 的语义分析和代码生成。如果容器库尚未完成，编译器无法知道 `vector<int>` 占多少字节、`push_back` 的签名是什么。

**解决方案**：容器**类型布局信息**作为编译器内置知识硬编码，**算法实现**后置补充。

### 5.2 前置内容（TOML 配置文件）

类型布局从外部 TOML 配置文件加载，避免在 Rust 中硬编码 16+ 条目：

```toml
# runtime_libc/cide/layouts.toml

[vector_int]
size = 12
[[vector_int.fields]]
name = "n"
type = "int"
[[vector_int.fields]]
name = "m"
type = "int"
[[vector_int.fields]]
name = "a"
type = "int*"

[[vector_int.methods]]
name = "push_back"
params = ["int"]
ret = "void"
is_virtual = false

# ... 其他类型参见 layouts.toml 完整文件
```

编译器启动时读取并缓存：

```rust
// native/src/compiler/cpp_frontend/builtin_layout.rs

use std::collections::HashMap;
use std::sync::LazyLock;

pub struct ClassLayout {
    pub size: i32,
    pub fields: Vec<(String, Type)>,
    pub methods: Vec<MethodSig>,
}

pub struct MethodSig {
    pub name: String,
    pub params: Vec<Type>,
    pub ret: Type,
    pub is_virtual: bool,
}

static BUILTIN_LAYOUTS: LazyLock<HashMap<String, ClassLayout>> = LazyLock::new(|| {
    let toml_str = include_str!("../../../runtime_libc/cide/layouts.toml");
    parse_layouts_toml(toml_str)
});

pub fn builtin_class_layout(name: &str) -> Option<ClassLayout> {
    BUILTIN_LAYOUTS.get(name).cloned()
}
```

### 5.3 后置内容（容器算法实现）

容器库的 `.c` 文件在编译器核心稳定后补充。补充流程：
1. 手写 `runtime_libc/cide/vec_int.c`
2. 运行 `python scripts/precompile_bytecode_libc.py`
3. 将预编译函数名从 `__cide_vec_push_int_stub` 替换为真实的 `cide_vec_push_int`

**注意**：内置布局表中的方法签名必须与后置的 `.c` 实现严格一致。

---

## 六、H2 详细方案：混合架构（TypeChecker + BytecodeGen 直接扩展）

### 6.1 设计原则

**核心变更**：不再将 C++ AST 完全降解为 C AST，而是让 TypeChecker 和 BytecodeGen 直接理解 C++ 语义节点。

| 维度 | 原方案（v1.0 完全降解） | 新方案（v2.1 混合） |
|------|----------------------|-------------------|
| **AST** | C++ AST → C AST（两层） | 单一 AST（混合 C/C++ 节点） |
| **TypeChecker** | 只处理 C | 处理 C + auto 推导 + 模板单态化 + 类布局 |
| **BytecodeGen** | 只处理 C | 处理 C + 虚函数/this/new/delete |
| **SourceLoc** | 需要 Source Map 回查 | **天然保留**（直接指向 C++ 源码） |
| **编译速度** | 慢（完整 AST→AST 降解） | 快（线性扩展） |
| **测试验证** | 可对比降解产物（differential） | 直接验证字节码 + 轻量降解产物审计 |

**复杂度转移**：将"降解器复杂度"转移到"BytecodeGen 分支复杂度"，换取 SourceLoc 天然保留和编译速度提升。

### 6.2 AST 扩展

```rust
// native/src/compiler/ast.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum TypeKind {
    // === 现有 C 类型 ===
    #[default]
    Void,
    Int,
    Char,
    Float,
    Double,
    LongLong,
    Pointer,
    Array,
    Struct,
    Union,
    Function,

    // === C++ 新增 ===
    Class,       // P1
    Reference,   // P2: & / const&
    RValueRef,   // P5: &&
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Type {
    // === 现有 C 类型 ===
    Void { is_const: bool },
    Int { is_unsigned: bool, is_const: bool },
    Char { is_unsigned: bool, is_const: bool },
    Float { is_const: bool },
    Double { is_const: bool },
    LongLong { is_unsigned: bool, is_const: bool },
    Pointer { pointee: Box<Type>, is_const: bool },
    Array { element: Box<Type>, array_size: i32, dims: Vec<i32>, is_const: bool, is_vla: bool, vla_dims: Vec<Box<Expr>> },
    Function { return_type: Box<Type>, param_types: Vec<Type>, is_const: bool },
    Struct { name: String, is_const: bool },
    Union { name: String, is_const: bool },

    // === C++ 新增 ===
    Class { name: String, is_const: bool },         // P1
    Reference { base: Box<Type>, is_const: bool },  // P2
    Auto,                                            // P2: 占位类型，TypeChecker 推导后替换
    RValueRef { base: Box<Type> },                  // P5
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Expr {
    // === 现有 C 表达式（Binary, Unary, Literal, Identifier, Call, CallPtr, Index, Member, Assign, Ternary, Sizeof, Cast, InitList, Offsetof）===
    // ... 省略已有变体

    // === C++ 新增 ===
    This { loc: SourceLoc },                             // P1: this 指针
    MemberCall {                                         // P1: obj.method()
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
        is_virtual: bool,
        loc: SourceLoc,
        ty: Type,
    },
    Lambda {                                             // P2
        capture: Vec<CaptureMode>,
        params: Vec<Param>,
        body: Box<Stmt>,
        unique_id: u64,
        loc: SourceLoc,
    },
    New { elem_type: Type, size_expr: Option<Box<Expr>>, init: Option<Box<Expr>>, loc: SourceLoc },      // P2
    Delete { expr: Box<Expr>, is_array: bool, loc: SourceLoc },                                          // P2
    Move { expr: Box<Expr>, loc: SourceLoc },            // P5: std::move(x)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Stmt {
    // === 现有 C 语句（Block, VarDecl, Expr, If, While, DoWhile, For, Return, Break, Continue, Switch, Case, Goto, Label）===
    // ... 省略已有变体

    // === C++ 新增 ===
    RangeFor { var: String, var_type: Type, iter: Box<Expr>, body: Box<Stmt>, loc: SourceLoc },  // P2
    Try { body: Box<Stmt>, catches: Vec<CatchClause>, loc: SourceLoc },                         // P5: 语法占位，TypeChecker 报错
}

// === 新增节点 ===
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClassDecl {
    pub name: String,
    pub base: Option<String>,    // 单继承，最多一个基类
    pub members: Vec<ClassMember>,
    pub vtable: Option<VTable>,  // 虚函数表（TypeChecker 生成）
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ClassMember {
    Field { name: String, type_: Type, access: AccessSpec },
    Method { name: String, ret: Type, params: Vec<Param>, body: Stmt, is_virtual: bool, access: AccessSpec },
    Constructor { params: Vec<Param>, body: Stmt, is_default: bool },
    Destructor { body: Stmt },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TemplateDecl {
    pub params: Vec<TemplateParam>,   // 仅类型参数
    pub decl: Box<Templateable>,      // 函数模板或类模板
}
```

**序列化兼容性风险**：新增 `Type`/`Expr`/`Stmt` 变体会改变 serde 序列化格式。如果 Flutter Bridge 或 VM 快照依赖旧格式，需要验证兼容性。

### 6.3 Lexer 扩展

```rust
// native/src/compiler/lexer.rs

pub enum TokenType {
    // === 现有 C 关键字 ===
    Int, Void, Char, If, Else, While, Do, For, Return, Break, Continue,
    Struct, Union, Sizeof, Offsetof, Switch, Case, Default, Typedef, Enum,
    Unsigned, Long, Short, Signed, Const, Extern, Float, Double,
    Volatile, Inline, Restrict, Register, Auto, Bool, Goto, Null,

    // === C++ 新增关键字 ===
    Class, Public, Private, Protected, This,          // P1
    Using, Namespace,                                 // P2（Namespace 仅用于报错"不支持"）
    Virtual, Override, Friend,                        // P4
    Template, Typename,                               // P5
    Static_cast, Const_cast, Reinterpret_cast,        // P5
    New, Delete,                                      // P6

    // === C++ 新增运算符/标点 ===
    ColonColon,       // ::
    ArrowStar,        // ->*
    DotStar,          // .*
    LineComment,      // // 行注释

    // ... 其他现有 Token 不变
}
```

**注意**：现有 Lexer 已有 `Auto`（C 存储类）和 `Bool`/`Null` Token。C++ 的 `auto` 语义（类型推导）与 C 的 `auto`（存储类）完全不同，Parser 需要根据上下文区分。

### 6.4 Parser 扩展

基于现有递归下降架构扩展：

```rust
// parser/mod.rs

impl Parser {
    fn parse_class_decl(&mut self) -> Result<ClassDecl, ParseError> {
        // class Name [: public Base] { public: ... private: ... }
        // 降级为 ClassDecl AST 节点
    }

    fn parse_range_for(&mut self) -> Result<Stmt, ParseError> {
        // for (auto x : expr) { body }
        // 生成 Stmt::RangeFor
    }

    fn parse_lambda(&mut self) -> Result<Expr, ParseError> {
        // [capture](params) { body }
        // 生成 Expr::Lambda
    }

    fn parse_member_call(&mut self, object: Expr) -> Result<Expr, ParseError> {
        // obj.method(args)
        // 生成 Expr::MemberCall
    }
}
```

### 6.5 TypeChecker 扩展

#### 6.5.1 auto 类型推导

```rust
// typeck/cpp_auto.rs

impl TypeChecker {
    fn deduce_auto_type(&mut self, init: &Expr) -> Type {
        match init {
            Expr::Literal(_) => Type::Int,
            Expr::FloatLiteral(_) => Type::Float,
            Expr::Identifier(name) => self.lookup_var_type(name).clone(),
            Expr::Call { ty, .. } | Expr::MemberCall { ty, .. } => ty.clone(),
            // ... 其他表达式类型推断
            _ => Type::Int, // fallback
        }
    }
}
```

#### 6.5.2 模板受限单态化

```rust
// typeck/cpp_monomorph.rs

impl TypeChecker {
    fn monomorphize_template(&mut self, name: &str, type_args: &[Type]) -> String {
        let mangle_name = format!("{}__{}", name, type_args.iter().map(|t| t.mangle()).collect::<String>());
        if !self.has_instance(&mangle_name) {
            let template = self.find_template(name);
            let func_decl = self.instantiate(&template, type_args);
            self.program.funcs.push(func_decl);
        }
        mangle_name
    }
}
```

**约束**：仅类型参数，无 SFINAE/特化/偏特化，递归深度 ≤ 8。

#### 6.5.3 类/vtable 布局分析

```rust
// typeck/cpp_class_layout.rs

impl TypeChecker {
    fn analyze_class(&mut self, class: &mut ClassDecl) {
        let mut offset = 0;
        if let Some(base) = class.base.as_ref() {
            offset = self.get_class_layout(base).size;
        }
        let mut vtable = VTable::new();
        for method in &class.members {
            if let ClassMember::Method { is_virtual: true, .. } = method {
                vtable.add(method.name.clone(), method.ty());
            }
        }
        class.vtable = Some(vtable);
    }
}
```

### 6.6 BytecodeGen 扩展

#### 6.6.1 虚函数调用

```rust
// codegen/cpp_member_call.rs

Expr::MemberCall { object, method, args, is_virtual: true, loc, .. } => {
    self.gen_expr(object);
    self.emit(OpCode::Dup, 0, &loc);        // this 指针
    self.emit(OpCode::LoadMem, 0, &loc);    // vptr
    let vtable_offset = self.get_vtable_offset(&object.ty(), method);
    self.emit(OpCode::PushConst, vtable_offset, &loc);
    self.emit(OpCode::Add, 0, &loc);        // &vptr[method]
    self.emit(OpCode::LoadMem, 0, &loc);    // 函数指针
    for arg in args { self.gen_expr(arg); }
    self.emit(OpCode::CallPtr, args.len() as i32 + 1, &loc);
}
```

**VM 事实依据**：`OpCode::CallPtr = 111` 已存在（`opcode.rs`），支持函数指针间接调用。

#### 6.6.2 非虚成员调用

```rust
Expr::MemberCall { object, method, args, is_virtual: false, loc, .. } => {
    let func_name = self.mangle_method(&object.ty(), method);
    self.gen_expr(object);  // this
    for arg in args { self.gen_expr(arg); }
    self.emit(OpCode::Call, self.func_index(&func_name), &loc);
}
```

#### 6.6.3 this 指针

```rust
Expr::This { loc } => {
    self.emit(OpCode::LoadLocal, 0, &loc);  // this = 第 0 个参数
}
```

#### 6.6.4 new / delete

```rust
Expr::New { elem_type, init, loc, .. } => {
    let size = self.type_size(&elem_type);
    self.emit(OpCode::PushConst, size as i32, &loc);
    self.emit(OpCode::CallHost, HOST_MALLOC_ID, &loc);
    if let Some(init_expr) = init {
        self.emit(OpCode::Dup, 0, &loc);
        self.gen_expr(init_expr);
        self.emit(OpCode::Call, self.ctor_index(&elem_type), &loc);
    }
}

Expr::Delete { expr, loc, .. } => {
    self.gen_expr(expr);
    self.emit(OpCode::Call, self.dtor_index(&expr.ty()), &loc);
    self.emit(OpCode::CallHost, HOST_FREE_ID, &loc);
}
```

### 6.7 轻量降解层（仅容器 + 语法糖）

#### 6.7.1 容器方法映射（TypeChecker 阶段）

```rust
// typeck/cpp_container.rs

impl TypeChecker {
    fn check_member_call(&mut self, expr: &mut Expr) {
        if let Expr::MemberCall { object, method, args, .. } = expr {
            if is_builtin_vector(&object.ty()) && method == "push_back" {
                let elem_ty = object.ty().type_param(0);
                let host_func = format!("cide_vec_push_{}", elem_ty.mangle());
                *expr = Expr::Call {
                    func: Box::new(Expr::Identifier(host_func)),
                    args: vec![
                        Expr::Unary { op: UnaryOp::Addr, expr: object.clone(), loc, ty: Type::pointer_to(object.ty()) },
                        args[0].clone(),
                    ],
                    loc,
                    ty: Type::void(),
                };
            }
        }
    }
}
```

#### 6.7.2 范围 for（AST 轻量变换）

```cpp
// 学生代码
for (auto x : v) { print(x); }
```

```rust
// 轻量降解：RangeFor → For 循环 AST
Stmt::RangeFor { var, var_type, iter, body, loc } => {
    let index_var = gen_unique_name("__i");
    Stmt::For {
        init: Some(Box::new(Stmt::VarDecl {
            name: index_var.clone(),
            var_type: Type::Int,
            init: Some(Expr::Literal(0)),
            extra_vars: vec![],
            is_static: false,
            loc,
        })),
        cond: Some(Expr::Binary {
            op: BinaryOp::Lt,
            left: Box::new(Expr::Identifier(index_var.clone())),
            right: Box::new(Expr::Call {
                func: Box::new(Expr::Identifier("cide_vec_size_int".to_string())),
                args: vec![iter.clone()],
                loc,
                ty: Type::Int,
            }),
        }),
        step: Some(Expr::Unary {
            op: UnaryOp::PostInc,
            expr: Box::new(Expr::Identifier(index_var.clone())),
        }),
        body: Box::new(Stmt::Block(vec![
            Stmt::VarDecl {
                name: var,
                var_type: var_type,
                init: Some(Expr::Call {
                    func: Box::new(Expr::Identifier("cide_vec_get_int".to_string())),
                    args: vec![iter.clone(), Expr::Identifier(index_var)],
                    loc,
                    ty: var_type,
                }),
                extra_vars: vec![],
                is_static: false,
                loc,
            },
            *body,
        ])),
        loc,
    }
}
```

#### 6.7.3 Lambda（AST 轻量变换）

```cpp
// 学生代码
auto f = [x](int y) { return x + y; };
int z = f(5);
```

```rust
// 轻量降解：Lambda → 闭包 struct + 函数
// 1. 生成闭包 struct
struct __lambda_0 {
    int x;  // 捕获变量
};

// 2. 生成调用函数
fn __lambda_0__call(this: &__lambda_0, y: int) -> int {
    this.x + y
}

// 3. 替换原 AST
// auto f = [x](int y) { ... };
// →
// struct __lambda_0 f;
// f.x = x;  // 捕获初始化
// int z = __lambda_0__call(&f, 5);
```

**注意**：Lambda 的轻量降解保留 SourceLoc 映射（`__lambda_0__call` 函数体的每个语句的 loc 指向原始 Lambda 体对应行）。

### 6.8 模块结构

```
native/src/compiler/
├── typeck/
│   ├── mod.rs                       # 现有入口
│   ├── cpp_auto.rs                  # auto 类型推导
│   ├── cpp_monomorph.rs             # 模板单态化
│   ├── cpp_class_layout.rs          # 类/vtable 布局分析
│   ├── cpp_container.rs             # 容器方法映射（轻量降解）
│   └── cpp_overload.rs              # 重载决议（移动/拷贝构造）
├── codegen/
│   ├── mod.rs                       # 现有入口
│   ├── cpp_member_call.rs           # 虚函数/非虚成员调用生成
│   ├── cpp_this_new_delete.rs       # this/new/delete 生成
│   └── cpp_lambda.rs                # Lambda 闭包生成
├── cpp_frontend/
│   ├── mod.rs
│   ├── type_map.rs                  # C++ 类型名映射
│   └── builtin_layout.rs            # 内置容器类型布局
├── ast.rs                           # 扩展：新增 C++ 节点
└── parser.rs                        # 扩展：新增 C++ 语法
```

---

## 七、编译器双模式详解

### 7.1 CppExplicit 模式

学生显式管理资源生命周期，与 C 模式一致：

```cpp
vector<int> v;
v.push_back(1);
v.push_back(2);
// ...
v.__destroy();   // 显式调用
```

### 7.2 CppImplicit 模式

编译器在每个作用域退出点自动插入析构调用：

```cpp
void foo() {
    vector<int> v;      // __init 自动插入
    v.push_back(1);
    if (cond) {
        vector<int> w;  // __init 自动插入
        w.push_back(2);
        return;         // w.__destroy 自动插入，v.__destroy 自动插入
    }
    v.push_back(3);
}                       // v.__destroy 自动插入
```

### 7.3 作用域守卫生成算法

```rust
fn generate_implicit_dtors(stmt: &Stmt) -> Stmt {
    match stmt {
        Stmt::Block(stmts) => {
            let mut dtors = Vec::new();
            let mut new_stmts = Vec::new();
            for s in stmts {
                if is_early_exit(s) {
                    new_stmts.extend(dtors.iter().cloned());
                }
                new_stmts.push(generate_implicit_dtors(s));
                if let Stmt::VarDecl { name, var_type, .. } = s {
                    if has_dtor(var_type) {
                        dtors.push(generate_dtor_call(name, var_type));
                    }
                }
            }
            for dtor in dtors.iter().rev() {
                new_stmts.push(dtor.clone());
            }
            Stmt::Block(new_stmts)
        }
        _ => stmt.clone(),
    }
}
```

---

## 八、实施阶段

### 8.1 前置条件（已满足）

| 前置任务 | 状态 | 代码事实 |
|----------|------|----------|
| `static` 完整语义 | ✅ 已实现 | `codegen/mod.rs` 行 638-657 |
| `goto` | ✅ 已实现 | `typeck/mod.rs` 行 837-847 |
| `volatile` | ✅ 已实现 | `lexer.rs` 行 8 |
| `#ifdef` / `#ifndef` / `#endif` | ✅ 已实现 | `lexer.rs` 条件编译状态机 |
| `<limits.h>` / `<stdbool.h>` | ✅ 已实现 | `runtime_libc/include/` |
| `qsort` | ✅ 已实现 | `stdlib.h` 行 11 |
| `puts` / `sprintf` / `calloc` / `bsearch` | ✅ 已实现 | `stdlib.h` 行 4-5、12 |

**预估时间**：0 周（已满足）。

### 8.2 Phase 1：编译器核心 + 内置类型布局（4 周）

| 周 | 任务 | 产出 |
|----|------|------|
| W1 | AST 扩展（Type/Expr/Stmt 新增节点） | `ast.rs` 含 C++ 节点；新增 `Class`/`Reference`/`Auto`/`RValueRef` TypeKind |
| W2 | Lexer 扩展（C++ 关键字 + // 注释） | `lexer.rs` 识别 C++ Token；区分 C-auto 与 C++-auto |
| W3 | Parser 扩展（class / template / auto / 范围 for / new / delete） | 解析通过，生成 C++ AST |
| W4 | 内置容器类型布局表 | `builtin_layout.rs`；`vector<int>`/`string` 硬编码布局 |

### 8.3 Phase 2：TypeChecker + BytecodeGen 扩展（5 周）

| 周 | 任务 | 产出 |
|----|------|------|
| W5 | TypeChecker 扩展（auto 推导 + 引用语义） | `cpp_auto.rs` |
| W6 | BytecodeGen 扩展（this / new / delete） | `cpp_this_new_delete.rs` |
| W7 | 模板受限单态化（TypeChecker） | `cpp_monomorph.rs`；递归深度 ≤ 8 |
| W8 | 类布局分析 + BytecodeGen 虚函数调用 | `cpp_class_layout.rs`, `cpp_member_call.rs` |
| W9 | 范围 for + Lambda 轻量降解 | `cpp_container.rs`, `cpp_lambda.rs` |

### 8.4 Phase 3：容器库实现 + 集成（3 周）

| 周 | 任务 | 产出 |
|----|------|------|
| W10 | 参照 klib 手写 vec_int / vec_float / string | `runtime_libc/cide/*.c` |
| W11 | 预编译容器库 + 方法映射替换 | `bytecode_libc_data.json` 含容器函数 |
| W12 | 容器方法调用一致性测试 | `tests/cpp_container/` |

### 8.5 Phase 4：高级特性 + 双模式（3 周）

| 周 | 任务 | 产出 |
|----|------|------|
| W13 | 右值引用 / 移动语义 | `cpp_rvalue_ref.rs` |
| W14 | unique_ptr / shared_ptr（简化版） | `cpp_smart_ptr.rs` |
| W15 | CppImplicit 模式（作用域守卫） | `implicit_dtor.rs` |

### 8.6 Phase 5：测试防线 + 教材回归（3 周）

| 周 | 任务 | 产出 |
|----|------|------|
| W16 | Host Contract 测试（C++ ↔ C 降解一致性） | `tests/cpp_lower/` |
| W17 | Bytecode Consistency 测试 | `tests/bytecode/` |
| W18 | Differential 测试 + 教材回归 | `tests/diff/`, 教材验证 |

**总计**：C++ 拓展 18 周 ≈ **4.5 个月**。

---

## 九、测试防线

延续 Cide 的五层测试防线：

### 9.1 第一层：轻量降解审计测试

```cpp
// tests/cpp_light_lower/test_vector_basic.cpp
#include <cide_vector>

int main() {
    vector<int> v;
    v.push_back(1);
    v.push_back(2);
    assert(v.size() == 2);
    assert(v[0] == 1);
    v.__destroy();
    return 0;
}
```

验证：
1. `--show-lowered` 输出轻量降解产物（范围 for 展开、Lambda 闭包 struct）
2. 轻量降解产物与手写等价 C 代码**逐字节一致**
3. TypeChecker 容器方法映射正确（`v.push_back` → `cide_vec_push_int`）

### 9.2 第二层：Bytecode Consistency 测试

同一 C++ 源码在 Cide 编译两次，生成的字节码逐指令一致。

### 9.3 第三层：Differential 测试

```cpp
vector<int> v;
v.push_back(3);
v.push_back(1);
v.push_back(4);
sort(v.begin(), v.end());
```

对比：Cide 编译执行结果 vs GCC -O0 编译执行结果。数组顺序必须一致。

### 9.4 第四层：错误诊断测试

```cpp
vector<int> v;
auto x = v[100];   // 越界访问 → E3001 TrapBounds
```

验证：C++ 代码产生的错误码与 C 代码完全一致。

### 9.5 第五层：教材回归测试

选取国内主流教材/ OJ 题目（洛谷、PTA、LeetCode 简单题），用 Cide C++ 子集重写，验证通过。

---

## 十、Dogfooding 与 C++ 容器验证

> **当前状态（2026-06-10）**：Stage 0 已完成，`runtime_libc/cide/*.c` 全部预编译通过；`vector<int/float/char>`、`string`、`list<int>`、`sort_int`  layouts.toml / builtin_layout.rs / type_map.rs / cpp_container.rs 已对齐。Stage 2 栈 RAII 已完成：`Class c;` 自动调用默认构造函数，scope exit / return / break / continue 自动按 LIFO 调用析构函数。Stage 3 `new[]/delete[]` 元素构造析构已完成：`new A[n]` 在 `base[-4]` 存元素 count，`delete[]` 逆序调用析构函数；临时变量槽位从 3 个扩展至 4 个。Stage 4 引用声明与基本语义已完成：`int& r = x` 全链路通过；`T&` 函数参数/返回值支持；引用自动解引用；引用参数隐式取地址；返回引用的函数调用识别为左值。Stage 5 Dogfooding 基础设施已完成：`native/tests/test_utils.rs` 提供 `compile_cpp_bytecode` + `assert_bytecode_equivalent`（Jump/Call 归一化 + diff 输出）；`native/tests/cpp_dogfooding_test.rs` 提供 harness 和工具自验证。Stage 6 `vector<int>` Dogfooding 已启动：C++ 模板类 `vector<int>`（使用 `new[]/delete[]` + 循环复制）编译通过并运行正确，stdout 与 C 基线 `cide_vec_int` 一致（`3\n1\n4\n`）。**构造函数成员初始化列表 `Class() : field(val) {}` 已修复**：Parser 在两个构造函数分支增加 `parse_ctor_init_list()`，降解为 `this->field = expr;` 赋值语句插入 `Block` 开头。**Stage 1 Dogfooding 验证推进**：
> - `vector<int/float/char>`、`string`、`list<int>` C++ 纯实现运行时 stdout 与 C 基线完全一致
> - `sort_int` C++ 模板函数实现运行时 stdout 与 C 基线完全一致（排序结果 `1\n1\n3\n4\n5\n`）
> - **字节码等价验证**：`get`/`size` 等简单访问方法在 C++ class 与 C struct 字段布局对齐后，逐指令等价（StepEvent 行号归一化 + Call 目标名归一化 + 启动代码排除）
> - **已知差异（诚实记录）**：`push_back`/`pop_back` 等方法因 C++ 使用 `new[]/delete[]` + 循环复制，C 使用 `realloc`，算法实现不同，字节码不等价；`list<int>::get` 因 C++ 使用 `if/return` 而 C 使用三元运算符 `?:`，控制流结构不同，字节码不等价
> - **P0 编译器 bug 修复（Dogfooding 过程中发现）**：
>   1. 函数模板隐式实例化：`Array` 实际参数无法匹配 `Pointer` 形式参数进行模板参数推断 → 修复 `infer_template_arg` 增加 `Array → Pointer` 退化分支
>   2. 函数模板实例化后 AST 未重写：`CallPtr { callee: "foo" }` 实例化为 `foo__int` 后，BytecodeGen 仍看到原始名称 → 修复 `resolve_expr_type` 将 `CallPtr` 重写为 `Call { name: mangled }`
>   3. 递归模板调用已实例化回退失败：`try_monomorphize_func` 对已实例化函数返回 `None`，caller 无法获取 mangled 名称 → 改为返回 `Some((mangled, None))`
>   4. 模板实例化函数体未检查：`pending_instantiations` 在 `exit_scope` 后才 drain，函数体从未被 TypeChecker 遍历 → 增加 Pass 3.6 循环检查直至收敛
> - Dogfooding 测试总计 25 个（10 个原有 + 15 个新增），全部通过；600+ Rust 单元测试保持全绿
> - **M5 隐式移动构造函数自动生成（Stage 5）**：类含指针/资源字段时自动生成 `__ctor__{Class}__move`；`std::move` 初始化调用移动构造并置空源指针；Dogfooding 测试 +2（总计 28 个，全绿）

### 10.1 Stage 0：验证 BytecodeGen（已完成 ✅）

开发团队用 Cide C++ 子集写一个 class template 封装，调用 Stage 0 的 C 容器函数：

```cpp
// tests/dogfooding/stage0_vec_cpp_wrapper.h
// 验证 Cide C++ BytecodeGen 是否能正确生成对 C 容器的调用

template<class T>
class vector {
public:
    vector() { cide_vec_init(this); }
    void push_back(T x) { cide_vec_push(this, x); }
    T get(size_t i) { return cide_vec_get(this, i); }
    size_t size() { return cide_vec_size(this); }
    ~vector() { cide_vec_destroy(this); }
};
```

**注意**：不使用运算符重载（已排除），使用显式 `get()` 方法。

**验证流程**：
1. 用 Cide C++ 编译器编译 `stage0_vec_cpp_wrapper.h` → 生成字节码 A
2. 手写等价的 C 代码（直接调用 `cide_vec_*`）→ 生成字节码 B
3. 对比 A ≡ B（逐指令一致）→ 证明 BytecodeGen 正确

### 10.2 Stage 1：Dogfooding 验证（进行中 🔄，核心验证通过）

当 Cide C++ 编译器成熟后，用 Cide C++ 子集写**不依赖 C 容器函数**的纯 C++ 容器：

```cpp
// tests/dogfooding/stage1_vec_cpp.h
// 目标：用 Cide C++ 子集实现一个自包含的 vector<int>
// 不调用任何 cide_vec_* C 函数，完全用 C++ 语法实现

template<class T>
class vector {
    int size_;
    int capacity_;
    T* data;
public:
    vector() : size_(0), capacity_(0), data(nullptr) {}
    
    void push_back(T x) {
        if (size_ >= capacity_) {
            int new_cap = capacity_ == 0 ? 4 : capacity_ * 2;
            T* new_data = new T[new_cap];  // Cide new → HostMalloc
            for (int i = 0; i < size_; i++) new_data[i] = data[i];
            delete[] data;                 // Cide delete → HostFree
            data = new_data;
            capacity_ = new_cap;
        }
        data[size_++] = x;
    }
    
    T get(int i) { return data[i]; }
    int size() { return size_; }
    
    ~vector() { delete[] data; }
};
```

**Dogfooding 验证流程**：
1. 用 Cide C++ 编译器编译 `stage1_vec_cpp.h` → 生成字节码 C
2. 对比 C ≡ B（Stage 0 的 C 版本字节码）
3. 如果 C ≡ B → **Dogfooding 通过**，Cide C++ 编译器可以正确编译 C++ 项目（容器库）
4. 进入 Stage 2：用 `stage1_vec_cpp.h` 替换 `cide_vec_int.c`，删除 C 实现

**Dogfooding 失败处理**：
- 若 C ≠ B → 分析差异，修复 BytecodeGen/TypeChecker
- 循环 1-4 直到 C ≡ B
- Dogfooding 是编译器正确性的**终极测试**，不通过不发布 Stage 2

---

## 十一、风险与缓解

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| 模板单态化爆炸 | 中 | 编译时间/内存爆炸 | 仅支持类型参数，限制递归模板深度 ≤ 8 |
| BytecodeGen 复杂度膨胀 | 中 | 新增 C++ 节点分支难以维护 | 模块化拆分（`cpp_member_call.rs` / `cpp_lambda.rs`），单测覆盖 |
| CppImplicit 作用域守卫 bug | 中 | 内存泄漏/双重释放 | 严格 CFG 分析 + 单元测试覆盖所有跳转组合 |
| AST serde 兼容性破坏 | 中 | 快照/Flutter Bridge 序列化失败 | 新增变体向后兼容测试；若失败则追加 serde 版本号 |
| 学生混淆 C/C++ 语法 | 高 | 教学体验下降 | IDE 层面区分模式提示，错误消息标注 "C++ 模式" |
| klib 参照实现偏差 | 低 | 容器行为与预期不一致 | 逐函数对照 klib 源码，Differential 测试验证 |
| `Auto` Token 语义冲突 | 中 | C-auto 与 C++-auto 解析歧义 | Parser 根据上下文区分（C 模式 vs C++ 模式） |
| **Dogfooding 失败** | **中** | **C++ 编译器无法正确编译 C++ 容器，Stage 2 无法实现** | **Stage 0 C 容器永久保留作为 fallback；Dogfooding 不阻塞 M8 发布** |

---

## 十二、里程碑

| 里程碑 | 时间 | 验收标准 |
|--------|------|----------|
| M1：编译器核心就绪 | T+2 周 | class / auto / 范围 for / new / delete 解析通过，AST 结构正确 |
| M2：TypeChecker 扩展完成 | T+5 周 | auto 推导、模板单态化、类布局分析通过全部单元测试 |
| M3：BytecodeGen 扩展完成 | T+7 周 | this / MemberCall / 虚函数调用字节码与手写 C 一致 |
| M3.5：Stage 0.5 容器收口完成 | T+0 周 | `list<int>` / `vector<char>` / `sort_int` 测试绿；`CPP_FAILURES.md` 创建；C++ Parser/TypeChecker/BytecodeGen 三 tier 纳入 CI；55/55 C++ 单元测试通过 |
| M4：容器库集成完成 | T+10 周 | vector<int> / string 预编译通过，`v.push_back` 生成正确字节码 |
| M4.5：Stage 2 栈 RAII 完成 | T+1 周 | 局部类对象自动调用默认构造函数；scope exit / return / break / continue 自动按 LIFO 调用析构函数；嵌套 scope + early return + loop 跳转测试全绿 |
| M4.6：Stage 3 `new[]/delete[]` 元素构造析构完成 | T+0 周 | `new A[n]` 逐元素调用构造函数；`delete[]` 从 `base[-4]` 读取 count 并逆序调用析构函数；临时变量槽位扩展至 4 个；新增 2 个数组构造析构测试全绿 |
| M5：高级特性完成 | T+13 周 | **移动语义 ✅（隐式移动构造函数自动生成）** / unique_ptr / CppImplicit 模式通过测试 |
| M6：测试防线完成 | T+16 周 | 五层测试防线全部通过，50 道教材题目回归通过 |
| M7：Beta 发布 | T+18 周 | 内部试用，收集反馈 |
| M8：正式发布 | T+22 周 | 文档完整，教学场景验证通过 |
| **M9：容器 Dogfooding（Stage 1）** | **T+26 周** | **用 Cide C++ 编译器编译 C++ 容器源码，字节码与 C 版本逐指令一致** |
| **M10：全面迁移（Stage 2）** | **T+30 周** | **运行时库全部替换为 C++ 实现，删除所有手写 C 容器** |

---

## 十三、Cide 平台内用户体验保障

### 13.1 SourceLoc 天然保留

**优势**：BytecodeGen 直接处理 C++ AST 节点，`&loc` 始终是原始 C++ 代码位置。

```rust
Expr::MemberCall { object, method, is_virtual: true, loc, .. } => {
    // ... 生成 vtable 间接调用 ...
    self.emit(OpCode::CallPtr, args.len() as i32 + 1, &loc);
    // loc = 原始 C++ 代码位置，VM trap 直接指向 obj.area() 行号
}
```

### 13.2 编译速度目标

| 指标 | 目标 | 测试方法 |
|------|------|----------|
| C++ 编译时间 / C 编译时间 | `< 1.5x` | 同规模代码对比 |
| TypeChecker 扩展耗时 | `< 20%` 总编译时间 | `--profile` 输出 |
| 单文件编译延迟 | `< 30ms`（教学代码量级） | 100 行 C++ 代码测试 |

### 13.3 CLI 集成

```bash
# C 用户：零变化
cide_cli compile hello.c
cide_cli run hello.c
cide_cli step hello.c

# C++ 用户：扩展名自动检测，命令完全一致
cide_cli compile hello.cpp        # 自动启用 C++ 模式
cide_cli run hello.cpp            # 运行
cide_cli step hello.cpp           # 源码级调试

# 教学专用
cide_cli compile hello.cpp --show-lowered   # 显示降解产物
cide_cli compile hello.cpp --show-ast       # 显示 C++ AST
```

---

## 十四、附录

### A. 错误码预留

| 范围 | 用途 | 当前最大 |
|------|------|----------|
| E1001~E1013 | Lexer 错误 | E1013 |
| E2001~E2008 | Parser 错误 | E2008 |
| E3001~E3071 | TypeChecker / 运行时错误 | E3071 |
| **E4001~E4999** | **C++ 扩展预留** | — |
| W3050~W3057 | Warning / Hint | W3057 |

### B. 类型映射表完整版

详见 `native/src/compiler/cpp_frontend/type_map.rs`（已实现，覆盖 vector<int/float/char>、list<int>、string）。

### C. 相关文档

| 文档 | 路径 | 说明 |
|------|------|------|
| C 子集规范 | `docs/current/C_SUBSET_SPEC.md` | C++ 拓展的前置语法基座 |
| 标准库支持矩阵 | `docs/current/SUPPORTED_LIBC.md` | 容器库集成的测试防线定义 |
| 错误码体系 | `native/src/diagnostics/error_codes.rs` | 新增 C++ 诊断码预留区间 E4001~E4999 |

---

**计划制定**: Kimi Code CLI  
**待审批**: 项目负责人  
**下一步**: 召开技术评审会，确认 Phase 1 启动日期。
