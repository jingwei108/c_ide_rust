# S6 Dogfooding  readiness Assessment（全面评估报告）

> **评估日期**: 2026-06-09  
> **评估人**: Kimi Code CLI  
> **结论**: **不满足进入 S6 条件。建议团队暂停工作，全面加固 2 个月以上。**  
> **核心依据**: 当前存在 12 项 P0 编译器缺陷、5 项 P0 容器代码缺陷、以及大量测试覆盖盲区。用标准 C++ 语法编写容器代码时，必须频繁使用 workaround 迎合编译器缺陷，直接违背"狗吃自己狗粮"的诚实原则。

---

## 一、执行摘要

### 1.1 表面数据 vs 实质质量

| 指标 | 表面数据 | 实质评估 |
|------|---------|---------|
| C++ 单元测试通过率 | 74/74 (100%) | ⚠️ 测试用例过于简单，未覆盖标准 C++ 容器编写所需的关键边界 |
| `vector<int>` Dogfooding | ✅ 运行一致性通过 | ✅ 确实通过，但 vector 是**特例**（无内部节点类型） |
| `string` Dogfooding | ✅ 运行一致性通过 | ⚠️ 只能用非模板 `class string`，无法写 `template<class T> class basic_string` |
| `list<int>` Dogfooding | ❌ 模板版本完全失败 | 🔴 类模板内嵌 struct / 模板 struct 均不支持，**标准 list<T> 根本无法编写** |
| CI 三 tier | ✅ 全绿 | ⚠️ 不覆盖 C++ Dogfooding 场景 |

### 1.2 关键发现（一句话总结）

- **Parser**: 类模板内不能定义 struct，模板 struct 不支持，struct tag 不自动作为类型别名，Lambda 不支持多捕获，构造函数初始化列表不支持 cast，成员函数类外定义不支持。
- **TypeChecker**: `auto&` 推导为 `auto`（引用丢失），`const int&` 不能绑定右值，`auto` 推导 `new struct Node` 丢失指针层级。
- **BytecodeGen**: 基本功能工作，但部分场景（如复杂 RAII + 异常路径）缺乏充分验证。
- **C 容器代码**: string 不是 C 字符串（无 `\0` 结尾），空容器 pop 越界，sort 为无保护 quicksort， layouts.toml 有类型错误。

---

## 二、编译器缺陷清单（P0 / P1 / P2）

### 🔴 P0 — 阻塞标准 C++ 容器编写的缺陷（必须修复）

#### P0-001: C++ 模式下 struct tag 名未自动作为类型别名

**标准 C++ 代码**:
```cpp
struct Node { int x; };
Node* p;           // 标准 C++: OK
```

**当前行为**: ❌ 编译失败，报错 `预期 ';'`（第 4 列）。

**影响**: 所有使用自定义 struct 的 C++ 代码必须写 `struct Node* p;`，代码风格严重不标准。`list` 容器需要节点结构，此缺陷直接阻碍。

**根因**: Parser/TypeChecker 未将 struct tag 注册为 C++ 模式下的类型别名。

---

#### P0-002: 类模板内部不支持定义 struct

**标准 C++ 代码**:
```cpp
template<class T>
class list {
    struct Node {      // ❌ Parser: 预期标识符名称
        T data;
        Node* next;
    };
    Node* head;
    ...
};
```

**当前行为**: ❌ 解析失败，级联错误。

**影响**: `std::list<T>` 的标准实现范式（内部节点类型）在 Cide 中**完全不可用**。这是 list Dogfooding 的根本阻塞项。

**根因**: `parse_class_decl` 在解析类成员时，未处理 `struct` 作为嵌套类型定义的情况。

---

#### P0-003: 不支持模板 struct

**标准 C++ 代码**:
```cpp
template<class T>
struct Pair {       // ❌ Parser: 预期函数名称
    T first, second;
};
```

**当前行为**: ❌ `template<class T> struct` 被 Parser 当成函数模板解析，期望函数名但遇到 `struct`。

**影响**: 无法在外部定义模板节点类型作为类模板内嵌 struct 的替代方案。

**根因**: `parse_template_decl` 只处理 `template<class T> class` 和 `template<class T> 返回类型 函数名(...)`，未处理 `template<class T> struct`。

---

#### P0-004: 构造函数初始化列表不支持显式 cast 表达式

**标准 C++ 代码**:
```cpp
struct Foo {
    int* p;
    Foo() : p((int*)0) {}    // ❌ 在非模板类中解析失败
};
```

**当前行为**: ❌ 报错 `预期标识符名称`（在 `(int*)0` 的 `(` 处）。有趣的是，在**类模板**中 `(T*)0` 可以工作（vector<int> Dogfooding 已验证），但在**非模板类**中失败。

**影响**: 标准 C++ 中显式 cast 初始化是常见写法。当前必须用 `0` 妥协，产生 E3054 警告。

**根因**: 非模板类的 `parse_ctor_init_list` 与模板类的初始化列表解析路径不一致。

---

#### P0-005: 不支持成员函数类外定义

**标准 C++ 代码**:
```cpp
class Bar {
public:
    void set(int v);
};
void Bar::set(int v) { ... }   // ❌ Parser: 全局变量声明后预期 ';'
```

**当前行为**: ❌ `Bar::set` 语法完全不被识别。

**影响**: 标准 C++ 项目（包括 STL 风格容器）中，类外定义成员函数是基本组织方式。此缺陷迫使所有代码必须写成 header-only 风格（类内定义全部方法），严重限制代码组织能力。

**根因**: Parser 未实现 `ClassName::methodName` 的 qualified name 解析。

---

#### P0-006: Lambda 不支持多捕获（逗号分隔）

**标准 C++ 代码**:
```cpp
auto f = [a, &b](int x) { ... };   // ❌ Parser: Lambda 预期 ']'
```

**当前行为**: ❌ 解析在逗号处失败。

**影响**: 教学场景和实际代码中多变量捕获非常常见。

**根因**: `parse_lambda_expr` 只解析单个捕获项，未处理逗号分隔列表。

**已验证支持**: `[]`、`[x]`、`[&x]`、`[=]`、`[&]`。
**不支持**: `[a, &b]`、`[this]`、`[=, &a]`、`[&, a]`。

---

#### P0-007: 范围 for 不支持引用形式

**标准 C++ 代码**:
```cpp
for (auto& x : arr) { ... }        // ❌ TypeChecker: 预期 ';'
for (const auto& x : arr) { ... }  // ❌ 同上
```

**当前行为**: ❌ `auto&` 和 `const auto&` 均编译失败。仅 `for (auto x : arr)` 支持。

**影响**: 范围 for 遍历容器时无法修改元素（缺少 `auto&`），也无法避免拷贝（缺少 `const auto&`）。这导致 `vector<int>` 的 Dogfooding 代码中无法使用标准的 `for (auto& x : v)` 写法。

**根因**: Parser 在解析范围 for 的变量声明时，未处理 `auto&` / `const auto&` 的引用声明符。

---

#### P0-008: `auto&` 类型推导丢失引用修饰符

**标准 C++ 代码**:
```cpp
int x = 42;
auto& r = x;            // ❌ TypeChecker: 类型不匹配，无法将 'int' 赋值给 'auto&'
const auto& cr = x;     // ❌ 同上
```

**当前行为**: ❌ `auto&` 被当成普通 `auto` 处理，引用修饰符丢失。

**影响**: 引用类型推导完全不可用，必须显式写 `int& r = x;`。

**根因**: `deduce_auto_type` 的 match 未处理 `VarDecl` 的声明符中 `auto` 后带 `&` / `const&` 的情况。TypeChecker 在 `decl.rs` 中可能将 `auto&` 整体作为类型传递，但 `deduce_auto_type` 只接收 init 表达式，不知道声明符中的引用要求。

---

#### P0-009: `const int&` 不能绑定到右值

**标准 C++ 代码**:
```cpp
const int& r = 5;       // ❌ TypeChecker: 非 const 引用必须绑定到左值 (E4029)
```

**当前行为**: ❌ 报错 E4029，但错误消息说"非 const 引用"，实际代码是 `const int&`。

**影响**: 标准 C++ 中 `const T&` 绑定到右值是核心语义（延长临时对象生命周期）。此缺陷意味着标准惯用法 `const string& s = getString();` 不可用。

**根因**: 引用绑定检查时未区分 `const int&` 和 `int&`，将所有引用绑定到右值的行为统一禁止。

---

#### P0-010: `auto` 推导 `new struct Node` 丢失指针层级

**标准 C++ 代码**:
```cpp
auto p = new struct Node;   // TypeChecker 推导 p 为 Node 而非 Node*
p->x = 42;                  // ❌ 报错: 类 'Node' 没有成员 'x'
```

**当前行为**: ❌ `auto` 被推导为 `Node`（类类型）而非 `Node*`（指针类型）。

**影响**: `auto p = new T;` 这一最常见的 C++ 惯用法在 T 为 struct 时完全错误。显式写 `Node* p = new Node;` 才可工作。

**根因**: `deduce_auto_type` 中 `Expr::New { elem_type, .. }` 返回 `Type::pointer_to(elem_type.clone())`。但 `elem_type` 在 `new struct Node` 中可能是 `Type::Struct { name: "Node" }`，而后续 TypeChecker 在解析 `p->x` 时，将 `p` 的类型当作 `Node`（因为 `auto` 推导结果未正确保留指针）。需要进一步确认是推导问题还是后续解引用处理的问题。

**注**: `auto p = new int;` 可以正确工作。问题仅出现在 `new struct Node` 的场景。

---

#### P0-011: `std::move` / `std::forward` 未被识别

**标准 C++ 代码**:
```cpp
int&& r = std::move(x);   // ❌ Parser: 变量声明后预期 ';'
```

**当前行为**: ❌ `std::move` 被 Parser 在变量声明上下文中无法识别。

**影响**: 移动语义的核心入口 `std::move` 不可用。

**根因**: `std::` prefix 的命名空间支持完全缺失（`using namespace` 也不支持）。虽然文档说 `std::` 前缀会被自动擦除，但实际 Parser 遇到 `std::move` 时无法处理。

---

#### P0-012: 构造函数重载不支持

**标准 C++ 代码**:
```cpp
class Box {
public:
    Box() { x = 0; }
    Box(int v) { x = v; }   // ❌ Parser: 预期标识符名称
};
Box b(42);                  // 同上
```

**当前行为**: ❌ 第二个构造函数解析失败。

**影响**: 标准容器的构造函数重载是基本特性（如 `vector()` 默认构造、`vector(int n)` 填充构造）。当前只能有单个构造函数。

**根因**: Parser 在类内遇到多个同名函数（构造函数都名为类名）时，可能将第二个当成字段重声明或其他错误。

---

### 🟡 P1 — 严重降低代码规范性的缺陷（强烈建议修复）

#### P1-001: 不支持 `static` 类成员

**标准 C++ 代码**:
```cpp
class A {
    static int count;       // ❌ Parser: 预期标识符名称
};
int A::count = 0;           // ❌ Parser: 全局变量声明后预期 ';'
```

**影响**: 无法编写带引用计数的 `shared_ptr` 简化版，也无法编写需要静态成员的容器辅助类。

---

#### P1-002: 不支持 `const` 成员函数

**标准 C++ 代码**:
```cpp
int get() const { return x; }   // ❌ Parser: 方法声明后预期 ';'
```

**影响**: `const` 正确性是 C++ 教学的核心内容。当前完全缺失。

---

#### P1-003: 不支持 `explicit` 构造函数

**标准 C++ 代码**:
```cpp
explicit Box(int v) { ... }    // ❌ Parser: 预期标识符名称
```

**影响**: 隐式转换控制是 C++ 重要教学点。

---

#### P1-004: `nullptr` 不是关键字，被当成普通标识符处理

**标准 C++ 代码**:
```cpp
int* p = nullptr;       // ✅ 编译通过，但报 E3054（整数隐式转指针）
```

**当前行为**: ⚠️ 编译通过，但 `nullptr` 未被识别为关键字，被当成值为 0 的整数常量，产生 E3054 警告。

**影响**: `sizeof(nullptr)` 返回 `sizeof(int)` 而非 `sizeof(void*)`。`nullptr` 与 `0` 在类型系统上无区分。

---

#### P1-005: `using namespace` 不支持但报错信息极差

**标准 C++ 代码**:
```cpp
using namespace std;    // ❌ 报错位置: 2:1014（完全错误的行列号）
```

**影响**: 报错位置 2:1014 意味着字符串解析混乱，学生看到此诊断会完全困惑。

---

### 🟢 P2 — 教学体验降级（可延后）

| # | 缺陷 | 说明 |
|---|------|------|
| P2-001 | 多重继承 | 文档已明确排除单继承以上，但报错应为明确错误码而非解析失败 |
| P2-002 | `operator` 重载 | 文档已排除，但遇到 `operator+` 时 Parser 崩溃级联错误，应给出 E4002 |
| P2-003 | `friend` 关键字 | 不支持，但应给出明确错误而非级联错误 |
| P2-004 | 嵌套 class | `class Outer { class Inner {}; };` 未测试，预计与嵌套 struct 同样失败 |
| P2-005 | `volatile` 成员函数 | 文档已标注不支持，但需确认报错是否明确 |

---

## 三、C 容器代码缺陷清单

### 🔴 P0 — 安全/语义阻塞

| # | 文件 | 问题 | 后果 |
|---|------|------|------|
| C-P0-001 | `vec_*.c` | `pop_back` 在 `n==0` 时执行 `--v->n` → `-1`，访问 `a[-1]` | **数组越界，VM 崩溃或内存损坏** |
| C-P0-002 | `string.c` | `pop_back` 同上越界 | 同上 |
| C-P0-003 | `string.c` | `push_back` 不追加 `\0`，string 不是 C 字符串 | 无法安全传递给 `printf("%s")`、`strlen` 等，与 kstring 核心语义背离 |
| C-P0-004 | `layouts.toml` | `list_int.head`/`tail` 类型声明为 `int*` | 语义错误，应为节点指针类型（虽然当前指针宽度一致，但类型系统层面错误） |

### 🟡 P1 — 规范/一致性

| # | 文件 | 问题 | 后果 |
|---|------|------|------|
| C-P1-001 | `string.c` | `realloc(str->s, str->m)` 未显式乘 `sizeof(char)` | 与 `vec_*.c` 的 `sizeof(T) * m` 风格不一致，教学代码规范性差 |
| C-P1-002 | `string.c` | 扩容策略为 2 倍，与 kstring 的 `kroundup32` 不一致 | 行为偏离参照实现 |
| C-P1-003 | `list_int.c` | 无 `clear` 方法 | layouts.toml 未声明，C 实现也未提供 |
| C-P1-004 | `list_int.c` | `pop_back` 为 O(n) 遍历 | 单向链表尾部删除本就不高效，但频繁 pop_back 性能极差 |
| C-P1-005 | `sort_int.c` | 简单 quicksort，无随机 pivot、无 introsort 退化保护 | 最坏 O(n²)，与 ksort 工业级实现差距显著 |
| C-P1-006 | 全部 `.c` | 空指针写法不一致：`0` vs `(T*)0` | 应统一风格 |

### 🟢 P2 — 功能缺失

| # | 缺失 | 说明 |
|---|------|------|
| C-P2-001 | `vector::capacity()` | kvec 有 `kv_max`，Cide 未暴露 |
| C-P2-002 | `vector::front()`/`back()` | 常用方法，一行代码实现 |
| C-P2-003 | `list::front()`/`back()`/`pop_front()` | `pop_front` 是 O(1)，比 `pop_back` 更实用 |
| C-P2-004 | `string::c_str()` | 依赖 `\0` 结尾修复 |

---

## 四、测试覆盖盲区

### 4.1 Parser 测试盲区

| 特性 | 已有测试 | 缺失测试 | 风险 |
|------|---------|---------|------|
| 类模板内嵌 struct | ❌ | ❌ | P0-002 完全未覆盖 |
| 模板 struct | ❌ | ❌ | P0-003 完全未覆盖 |
| struct tag 别名 | ❌ | ❌ | P0-001 完全未覆盖 |
| 初始化列表 cast | ✅ 模板内 | ❌ 非模板类 | P0-004 仅在模板场景有测试 |
| 成员函数类外定义 | ❌ | ❌ | P0-005 完全未覆盖 |
| Lambda 多捕获 | ❌ | ❌ | P0-006 完全未覆盖 |
| 范围 for 引用 | ❌ | ❌ | P0-007 完全未覆盖 |
| 构造函数重载 | ❌ | ❌ | P0-012 完全未覆盖 |
| `static` 成员 | ❌ | ❌ | P1-001 完全未覆盖 |
| `const` 成员函数 | ❌ | ❌ | P1-002 完全未覆盖 |
| `explicit` | ❌ | ❌ | P1-003 完全未覆盖 |
| `std::move` | ❌ | ❌ | P0-011 完全未覆盖 |

### 4.2 TypeChecker 测试盲区

| 特性 | 已有测试 | 缺失测试 |
|------|---------|---------|
| `auto&` 推导 | ❌ | ❌ |
| `const int&` 绑定右值 | ❌ | ❌ |
| `auto` 推导 `new struct Node` | ❌ | ❌ |
| 引用返回的左值识别（复杂场景） | ✅ 基础 | ❌ 嵌套调用 |
| 类继承后的成员访问控制 | ✅ 基础 | ❌ 多级继承 |
| 虚函数表布局 | ✅ 基础 | ❌ 多级虚继承（已排除） |

### 4.3 BytecodeGen 测试盲区

| 特性 | 已有测试 | 缺失测试 |
|------|---------|---------|
| new struct 的 size 计算 | ⚠️ 未充分 | `new Node` 曾出现 malloc(0) |
| 复杂嵌套 scope RAII | ✅ 基础 | ❌ 3+ 层嵌套 + 混合跳转 |
| goto 跳过 dtor scope | ✅ 报错 | ❌ 边界 case |
| 虚函数调用 + 多重继承 | N/A | N/A（已排除） |

---

## 五、建议的 2 个月打牢计划

### Phase A: Parser 加固（3 周）

**目标**: 修复所有 P0 Parser 缺陷，使标准 C++ 容器代码可被正确解析。

| 周 | 任务 | 产出 |
|----|------|------|
| W1 | 修复 P0-001（struct tag 别名）+ P0-003（模板 struct）+ P0-004（初始化列表 cast） | 新增 parser 测试 6 个 |
| W2 | 修复 P0-002（类模板内嵌 struct）+ P0-005（成员函数类外定义） | 新增 parser 测试 4 个 |
| W3 | 修复 P0-006（Lambda 多捕获）+ P0-007（范围 for 引用）+ P0-012（构造函数重载） | 新增 parser 测试 6 个 |

### Phase B: TypeChecker 加固（3 周）

**目标**: 修复引用语义和 auto 推导缺陷，使类型系统能正确处理标准容器代码。

| 周 | 任务 | 产出 |
|----|------|------|
| W4 | 修复 P0-008（`auto&` 推导）+ P0-009（`const int&` 绑定右值） | 新增 typeck 测试 4 个 |
| W5 | 修复 P0-010（`auto` 推导 `new struct Node`）+ P0-011（`std::move` 识别） | 新增 typeck 测试 4 个 |
| W6 | 修复 P1-001（`static` 成员）+ P1-002（`const` 成员函数）+ P1-003（`explicit`） | 新增 typeck 测试 6 个 |

### Phase C: C 容器代码加固（1 周）

**目标**: 修复安全缺陷，补齐方法，统一规范。

| 任务 | 产出 |
|------|------|
| 修复 C-P0-001/002（pop 空保护） | 3 处修改 |
| 修复 C-P0-003（string `\0` 结尾）+ 添加 `c_str()` | string.c 重构 |
| 修复 C-P0-004（layouts.toml list_int 字段类型） | 1 处修改 |
| 补齐 C-P1-006（capacity/front/back/clear/pop_front） | 新增方法 + toml 同步 |
| 统一 realloc 风格 | 全部 .c 文件 |

### Phase D: BytecodeGen 加固与压力测试（2 周）

**目标**: 验证复杂场景下的 RAII、虚函数、new/delete 正确性。

| 周 | 任务 | 产出 |
|----|------|------|
| W7 | 复杂嵌套 RAII 测试（3+ 层 scope + return/break/continue/goto） | 新增黑盒测试 8 个 |
| W8 | 虚函数 + 继承 + 析构链测试；new/delete 类类型数组压力测试 | 新增黑盒测试 6 个 |

### Phase E: 测试防线建设（1 周）

**目标**: 补充全部缺失的白盒/黑盒测试，建立 C++ Shadow Verification。

| 任务 | 产出 |
|------|------|
| 补充 Parser 缺失测试（14 个） | parser_cpp_unit_test.rs |
| 补充 TypeChecker 缺失测试（14 个） | typeck_cpp_unit_test.rs |
| 补充 BytecodeGen 缺失测试（14 个） | bytecode_gen_cpp_unit_test.rs |
| 建立 C++ Shadow Verification 框架 | `scripts/shadow_verify_cpp.py` |
| 跑完全量回归 | `cargo test` 全绿、`ci_three_tier_check.py` 全绿 |

---

## 六、修订后的 S6 Go/No-Go 检查点

在进入 S6 之前，**必须**全部满足以下检查点：

### Parser 层

- [ ] `struct Node {}; Node* p;` 编译通过（P0-001）
- [ ] `template<class T> class list { struct Node { T val; Node* next; }; ... };` 编译通过（P0-002）
- [ ] `template<class T> struct Pair { T a, b; };` 编译通过（P0-003）
- [ ] `Foo() : p((int*)0) {}` 在非模板类中编译通过（P0-004）
- [ ] `void Class::method() {}` 编译通过（P0-005）
- [ ] `[a, &b]`、`[this]` 多捕获 Lambda 编译通过（P0-006）
- [ ] `for (auto& x : v)` 和 `for (const auto& x : v)` 编译通过（P0-007）
- [ ] 构造函数重载编译通过（P0-012）

### TypeChecker 层

- [ ] `auto& r = x;` 和 `const auto& cr = x;` 类型正确（P0-008）
- [ ] `const int& r = 5;` 编译通过（P0-009）
- [ ] `auto p = new struct Node;` 推导为 `Node*`（P0-010）
- [ ] `std::move(x)` 识别并生成 `RValueRef`（P0-011）

### C 容器层

- [ ] 空容器 pop 不越界（C-P0-001/002）
- [ ] string 以 `\0` 结尾，提供 `c_str()`（C-P0-003）
- [ ] layouts.toml 类型声明正确（C-P0-004）
- [ ] 补齐 `capacity`/`front`/`back`/`clear`/`pop_front`（C-P2-001~004）

### Dogfooding 验证层

- [ ] `template<class T> class vector<T>` C++ 版本编译通过，stdout 与 C 基线一致
- [ ] `template<class T> class list<T>` C++ 版本编译通过，stdout 与 C 基线一致
- [ ] `class string`（或 `template<class T> class basic_string<char>`）C++ 版本编译通过，stdout 与 C 基线一致
- [ ] 上述 C++ 版本代码**不依赖任何 workaround**（无 `struct` 前缀、无 `0` 代替 cast、无非模板妥协）

### 测试防线层

- [ ] C++ 白盒测试 ≥ 100 个（当前 74 个，需新增 26+）
- [ ] C++ 黑盒测试 ≥ 50 个
- [ ] C++ Shadow Verification ≥ 20 个用例
- [ ] `cargo test` 全量无回归
- [ ] `ci_three_tier_check.py` 全绿

---

## 七、最终结论

> **当前状态: NO-GO for S6。**

`vector<int>` 的 Dogfooding 通过是一个**好消息**，但它掩盖了编译器在更复杂场景下的系统性缺陷。`list<T>` 的标准实现需要类模板内嵌 struct，这是当前编译器的**绝对盲区**。如果带着这些缺陷进入 S6，团队将被迫：

1. 写非模板的 `class list_int` 来"假装" Dogfooding 通过；
2. 在所有 struct 指针前加 `struct` 前缀；
3. 避免使用 `auto&`、`const int&`、`std::move` 等标准惯用法；
4. 接受 string 不是 C 字符串、pop 空容器会崩溃的 C 基线。

这些妥协**直接违背**"cpp代码的编写必须极其标准规范且完善，不能扭曲迎合本项目前面留下的问题"的要求。

**建议**: 采纳 2 个月打牢计划，按 Phase A→E 顺序执行。每个 Phase 完成时进行内部评审，全部 Go/No-Go 检查点通过后再正式启动 S6。
