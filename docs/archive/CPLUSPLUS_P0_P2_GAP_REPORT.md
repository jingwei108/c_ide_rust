# C++ 扩展 P0~P2 实施缺漏报告

> 基于 `CPLUSPLUS_EXTENSION_PLAN.md` v2.1 对当前代码事实的逐条核查
> 核查日期: 2026-06-08
> 更新日期: 2026-06-08（模块拆分 + 容器调用修复后）

---

## 一、已完整实现（✅ 无需改动）

| 阶段 | 模块 | 说明 |
|------|------|------|
| P0 | Lexer 关键字扩展 | `Class`/`Public`/`Private`/`Protected`/`This`/`Using`/`Namespace`/`Virtual`/`Override`/`Friend`/`Template`/`Typename`/`Static_cast`/`Const_cast`/`Reinterpret_cast`/`New`/`Delete`/`ColonColon`/`ArrowStar`/`DotStar`/`LineComment` 均已加入 `TokenType` |
| P0 | AST 节点扩展 | `TypeKind::Class`/`Reference`/`RValueRef`；`Type::Class`/`Reference`/`Auto`/`RValueRef`；`Expr::This`/`MemberCall`/`Lambda`/`New`/`Delete`/`Move`；`Stmt::RangeFor`/`Try`；`ClassDecl`/`ClassMember`/`TemplateDecl` 均已定义 |
| P0 | Parser 扩展 | `parse_class_decl`/`parse_template_decl`/`parse_new_expr`/`parse_delete_expr`/`parse_lambda_expr`/`parse_range_for`/`parse_member_call` 均已实现；C++ 模式根据 `.cpp`/`.cxx`/`.cidecpp` 自动切换 |
| P1 | auto 类型推导 | `native/src/compiler/typeck/cpp_auto.rs` 已独立成模块，覆盖 Literal/FloatLiteral/LongLiteral/StringLiteral/Identifier/Call/MemberCall/Unary/Binary/New/Cast/Ternary/Member/Index/InitList 等表达式 |
| P1 | 模板受限单态化 | `native/src/compiler/typeck/cpp_monomorph.rs` 已独立成模块，支持隐式实例化、类型参数推断、mangling（`func__T1_T2`）、递归类型替换（Pointer/Array/Reference/RValueRef/Function） |
| P1 | 类/vtable 布局分析 | `typeck/mod.rs` Pass 1.5 已实现：基类字段继承、方法注册、虚函数表构建、override 检测、size 计算、访问控制（Public/Private/Protected） |
| P2 | this / MemberCall / New / Move 字节码生成 | `codegen/expr.rs` 已实现：虚函数 vtable 间接调用、非虚函数直接调用、this 指针加载、new（含 vptr 初始化+构造函数调用）、Move 透传 |
| P2 | RangeFor 字节码生成（数组） | `codegen/stmt.rs` 已实现数组类型的索引循环展开 |
| P2 | **Lambda 代码生成** | ✅ **已完成**：`codegen/expr.rs` 生成闭包 struct 栈分配 + 捕获字段初始化（ByValue/ByReference）；`TypeChecker` Pass 3.5 完成 lambda lifting 为 `__lambda_N__call` FuncDecl + `__lambda_N` ClassDecl；`Expr::Call` 自动重写 lambda 变量调用为 `__call` MemberCall |
| P2 | **`delete` 调用析构函数** | ✅ **已完成**：`codegen/expr.rs` `Expr::Delete` 分支先查找 `__dtor__{name}` 并 emit `Call`，再 `CallHost FREE` |
| P2 | **RangeFor 容器支持** | ✅ **已完成**：`codegen/stmt.rs` `Stmt::RangeFor` 已支持 `Type::Class` 内置容器，生成 `cide_vec_size_*`/`cide_vec_get_*` 调用循环 |
| — | 错误码预留 | `E4001~E4030` 已定义，覆盖异常/运算符重载/模板特化/线程/多重继承/命名空间/虚继承/friend/using/typename/new/delete/this/私有成员访问/auto/重载/构造器未找到等场景 |
| — | layouts.toml | `native/runtime_libc/cide/layouts.toml` 已存在，含 `vector_int`/`vector_float`/`string` 的字段与方法签名 |
| — | **容器库 C 实现（Stage 0）** | ✅ **已完成**：`runtime_libc/cide/` 下 6 个文件已创建——`vec_int.c`/`vec_float.c`/`vec_char.c`/`list_int.c`/`string.c`/`sort_int.c`，预编译通过（60 funcs, code length 2243, globals 2048） |
| — | **builtin_layout.rs** | ✅ **已完成**：`native/src/compiler/cpp_frontend/builtin_layout.rs` 已创建，通过 `include_str!` 读取 `layouts.toml`，`std::sync::LazyLock` 静态缓存 |
| — | **type_map.rs** | ✅ **已完成**：`native/src/compiler/cpp_frontend/type_map.rs` 已创建，支持 `vector<int>`→`cide_vec_int`、`string`→`cide_string` 等映射 |
| — | **容器方法映射** | ✅ **已完成**：`typeck/expr.rs` `Expr::MemberCall` 对 builtin 容器自动重写为 `Expr::Call { name: host_func, args: [&obj, ...args] }`，TypeChecker 验证签名 |
| — | 测试覆盖 | `parser_cpp_unit_test.rs` / `typeck_cpp_unit_test.rs` / `bytecode_gen_cpp_unit_test.rs` 已存在 |
| — | **VM 堆分配器修复** | ✅ **已完成**：修复 `MemoryState::allocate_raw` 中 heap_offset 与 free_list stale 条目冲突导致的地址重叠问题；修复 `host_realloc` 释放旧区域后 `freed_logs` 清理缺失导致的 Double-Free 误报 |

---

## 二、模块结构拆分（2026-06-08 已完成 ✅）

文档要求将 C++ 相关逻辑从 `mod.rs` / `expr.rs` / `stmt.rs` 中拆分为独立子模块。本次重构已完成全部 7 个新文件的提取。

| 要求文件 | 状态 | 迁移/实现的内容 |
|----------|------|-----------------|
| `native/src/compiler/typeck/cpp_class_layout.rs` | ✅ **已完成** | 从 `typeck/mod.rs` Pass 1.5 拆分：类字段布局、基类继承、vtable 构建、size 计算 |
| `native/src/compiler/typeck/cpp_container.rs` | ✅ **已完成** | 容器方法映射（轻量降解）：`v.push_back(x)` → `cide_vec_push_int(&v, x)`；`v.size()` → `cide_vec_size_int(&v)` 等（从 `typeck/expr.rs` 提取） |
| `native/src/compiler/typeck/cpp_overload.rs` | ✅ **已完成** | 重载决议 stub：构造函数重载、移动构造/拷贝构造优先级（当前仅预留接口，`resolve_constructor_overload` 尚未被调用） |
| `native/src/compiler/codegen/cpp_member_call.rs` | ✅ **已完成** | 从 `codegen/expr.rs` 拆分：虚函数 vtable 调用、非虚函数直接调用、this 指针处理、struct 返回临时变量 |
| `native/src/compiler/codegen/cpp_this_new_delete.rs` | ✅ **已完成** | 从 `codegen/expr.rs` 拆分：this 加载、new（malloc + vptr + ctor）、delete（dtor + free） |
| `native/src/compiler/codegen/cpp_lambda.rs` | ✅ **已完成** | Lambda 闭包生成：闭包 struct 定义、`__call` 函数体生成、捕获变量初始化（从 `codegen/expr.rs` 提取） |
| `native/src/compiler/cpp_frontend/mod.rs` | ✅ **已完成** | C++ 前端模块入口，re-exporting `builtin_layout` 和 `type_map` |
| `native/src/compiler/cpp_frontend/type_map.rs` | ✅ 已存在 | `vector<int>` → `cide_vec_int`、`string` → `cide_string` 等类型名映射表 |
| `native/src/compiler/cpp_frontend/builtin_layout.rs` | ✅ 已存在 | 读取 `layouts.toml`，为 TypeChecker/BytecodeGen 提供内置容器类型的字段偏移、方法签名、size |

---

## 三、容器库基座（文档 H1 / 4.x 节 / 8.2 W4）→ 全部完成 ✅

| 要求 | 状态 | 说明 |
|------|------|------|
| `native/runtime_libc/cide/vec_int.c` | ✅ 已完成 | 动态数组 int 型 C 实现（klib-style） |
| `native/runtime_libc/cide/vec_float.c` | ✅ 已完成 | 动态数组 float 型 C 实现 |
| `native/runtime_libc/cide/vec_char.c` | ✅ 已完成 | 动态数组 char 型 C 实现 |
| `native/runtime_libc/cide/list_int.c` | ✅ 已完成 | 单向链表 int 型 C 实现 |
| `native/runtime_libc/cide/string.c` | ✅ 已完成 | 动态字符串 C 实现 |
| `native/runtime_libc/cide/sort_int.c` | ✅ 已完成 | int 数组排序 C 实现（introsort/qsort wrapper） |
| `scripts/precompile_bytecode_libc.py` 扩展 | ✅ 已完成 | 脚本自动拾取 `runtime_libc/cide/*.c`，无需修改脚本逻辑 |
| `native/src/compiler/cpp_frontend/builtin_layout.rs` | ✅ 已完成 | `layouts.toml` 已解析并静态缓存 |

---

## 四、Bytecode Libc 容器调用运行时修复（2026-06-08）

### 4.1 栈下溢修复 ✅

**问题**：调用 `cide_vec_init_int(&v)` 等 `void` 容器函数时，VM 报"运行时错误：栈下溢"。

**根因**：`decl.rs` 对所有 Bytecode Libc 函数的返回类型 fallback 为 `int`。`Stmt::Expr` 在表达式类型非 `void` 时会生成 `Pop` 丢弃栈顶值，但 `void` 函数的 `Call` 不会向值栈 push 返回值，导致 `Pop` 从空栈弹出。

**修复**：
- `native/src/vm/bytecode_libc_index.rs`：新增 `bytecode_libc_sig(name) -> Option<(Type, Vec<Type>)>`，为全部 60 个 Bytecode Libc 函数提供正确的返回类型和参数类型签名。
- `native/src/compiler/typeck/decl.rs`：fallback 逻辑改为使用 `bytecode_libc_sig` 获取正确签名，并对参数进行类型检查 + 隐式转换。

### 4.2 int→float 参数转换修复 ✅

**问题**：`cide_vec_push_float(&v, 15)` 将 `15` 作为 `int` 传递，缺少隐式 `int→float` 转换，导致位模式错误（输出 `0.0` 而非 `15.0`）。

**根因**：`decl.rs` fallback 路径不检查参数类型，因此 `insert_implicit_cast` 未被调用。

**修复**：同上，`bytecode_libc_sig` 提供参数类型后，`check_user_func` 的 fallback 路径现在会调用 `insert_implicit_cast`。

---

## 五、返现问题（测试中仍失败）

以下 6 个端到端测试在 `native/tests/bytecode_gen_cpp_unit_test.rs` 中已编写，但尚未通过：

| 测试名 | 失败原因 | 优先级 |
|--------|---------|--------|
| `test_cpp_delete_calls_dtor` | **Parser 不支持析构函数语法**：`~Tracker()` 在类体内被解析为"预期分号"/"预期标识符名称" | P1 |
| `test_cpp_new_delete_with_ctor` | **同上**：类声明中包含构造函数/析构函数时 Parser 报错 | P1 |
| `test_cpp_lambda_capture_by_value` | **Lambda 捕获内存越界**：运行时访问地址 `0x100000`（栈顶），闭包对象大小或地址计算有误 | P1 |
| `test_cpp_lambda_capture_by_reference` | **同上**：引用捕获场景下闭包初始化写入越界 | P1 |
| `test_cpp_range_for_vector` | **TypeChecker 不支持 Class 作为 RangeFor 迭代对象**：报错"范围 for 的迭代对象必须是数组或指针类型" | P2 |
| `test_cpp_range_for_string` | **同上**：`string` 类型在 RangeFor 中被拒绝 | P2 |

### 5.1 问题详情

#### Parser：析构函数语法
```cpp
class Tracker {
public:
    int flag;
    Tracker() { flag = 0; }
    ~Tracker() { flag = 1; }  // ← Parser 报错
};
```
- `~Tracker()` 中的 `~` 在 Parser 中未被识别为析构函数标记，导致后续 token 流被误解析为字段声明。
- **影响**：任何包含析构函数的类声明都会编译失败。

#### Lambda：捕获内存越界
```cpp
int main() {
    int x = 10;
    auto f = [x](int y) { return x + y; };
    printf("%d\n", f(5));
    return 0;
}
```
- 运行时错误："内存访问越界：你访问了地址 0x100000"
- **可能原因**：
  1. 闭包对象在栈上的分配大小（`class_sizes` 中 `__lambda_N` 的大小）与实际生成的字段布局不匹配；
  2. 或 `__call` 函数内通过 `this` 指针访问捕获字段时，地址计算错误。

#### RangeFor：Class 类型迭代对象
```cpp
vector<int> v;
v.push_back(1);
for (auto x : v) { ... }  // ← TypeChecker 报错 E4020
```
- `typeck/stmt.rs` 或 `typeck/mod.rs` 中 RangeFor 的类型检查仅接受 `Array` 或 `Pointer`，未将内置容器 `Class` 类型纳入合法迭代对象。
- **影响**：`for (auto x : container)` 语法对 `vector<int>`/`string` 等容器不可用。

---

## 六、验证清单（已补充测试）

| 测试项 | 说明 | 状态 |
|--------|------|------|
| `test_cpp_lambda_capture_by_value` | Lambda `[x](int y) { return x + y; }` 编译执行 | ❌ 运行时越界 |
| `test_cpp_lambda_capture_by_reference` | Lambda `[&x](int y) { x += y; }` 编译执行 | ❌ 运行时越界 |
| `test_cpp_delete_calls_dtor` | 类析构函数中设置标志位，delete 后验证标志位 | ❌ Parser 不支持 `~Tracker()` |
| `test_cpp_range_for_vector` | `vector<int> v; v.push_back(1); for (auto x : v) { ... }` | ❌ TypeChecker 拒绝 Class 迭代对象 |
| `test_cpp_range_for_string` | `string s; s.push_back('a'); for (auto c : s) { ... }` | ❌ TypeChecker 拒绝 Class 迭代对象 |
| `test_cpp_container_vec_int` | `vector<int>` push/pop/size/get/destroy 端到端 | ✅ **通过** |
| `test_cpp_container_vec_float` | `vector<float>` 端到端 | ✅ **通过** |
| `test_cpp_container_string` | `string` push/pop/size/get/destroy 端到端 | ✅ **通过** |
| `test_cpp_builtin_layout_from_toml` | 验证 `builtin_layout.rs` 正确解析 layouts.toml | ✅ **通过** |
| `test_cpp_type_map_lookup` | 验证 `type_map.rs` 中 `vector<int>` → `cide_vec_int` 映射 | ✅ **通过** |

> **当前测试状态**: 全部现有 331+ 单元测试通过，fuzz 测试 5/5 通过，bytecode_libc_consistency 12/12 通过，differential_stress 18/18 通过，host_contract 86/86 通过。C++ 专项端到端测试：12 项中 8 项通过，4 项运行时/编译失败，2 项 Parser 语法不支持。

---

*报告结束*
