# Cide C++ 扩展：从当前状态到 Dogfooding 的完整实施计划

## 版本与范围

- **当前基线**：`d1f7eb4`（P0-P2 完成，Stage 0 C 容器预编译通过，14/14 C++ e2e 测试绿）
- **目标**：Stage 1 Dogfooding——用 Cide C++ 编译器编译标准 C++ 语法写的 `vector<int>` 等容器源码，生成的字节码与 Stage 0 手写 C 容器预编译字节码逐指令一致
- **原则**：完善好才进入 Dogfooding；代码不扭曲迎合编译器缺陷；测试先行

---

## 一、当前缺口总览（诚实清单）

| 缺口 | 严重性 | 对 Dogfooding 的影响 | 预估工作量 |
|------|--------|---------------------|-----------|
| **类模板实例化** | **P0 Blocker** | `template<class T> class vector` 无法编译 | 2.5 周 |
| **栈对象 RAII（ctor/dtor 自动调用）** | **P0 Blocker** | `vector<int> v;` 只 zero-init 不构造；scope exit 不析构 | 2.5 周 |
| `new[]` 数组元素构造 + `delete[]` 元素析构 | P1 | `new T[n]` / `delete[] data` 对类类型元素需逐个 ctor/dtor | 4 天 |
| **引用声明语法 `T&`** | P1 | 标准容器代码中 `T& get(int i)` 无法编写 | 1 周 |
| `std::move` + move constructor resolution | P2 | 可先不用 move 写容器，但标准语义不完整 | 1 周 |
| 测试基础设施（bytecode diff + C-vs-C++ harness） | P1 | Dogfooding 验证无工具支撑 | 4 天 |
| `list<int>` 编译器布局缺失 | P2 | Phase 3 遗留缺口，不影响 Dogfooding 核心 | 2 天 |

**最小可行集（MVP）到 Dogfooding**：类模板实例化 + 栈 RAII + `new[]/delete[]` 元素构造析构 + 测试基础设施。其余后置。

---

## 二、实施阶段

### Stage 0.5：Phase 3 收口（第 1 周，3 人日）

**目标**：把 Stage 0 容器集成固化为可交付状态，补全测试与文档。

| 任务 | 细节 | 验收标准 |
|------|------|----------|
| 补齐 `list<int>` 编译器支持 | `builtin_layout.rs` 加 `cide_list_int` ClassLayout；`layouts.toml` 加 `[list_int]`；`cpp_container.rs` 确认方法降级路径 | `list<int> l; l.push_back(1); assert(l.size()==1);` 端到端通过 |
| 补齐 `vector<char>` TOML | `layouts.toml` 加 `[vector_char]` | TOML 与 `builtin_layout.rs`、预编译符号三者一致 |
| 补容器一致性测试 | `bytecode_gen_cpp_unit_test.rs` 新增：`test_cpp_container_list_int`、`test_cpp_container_vec_char`、`test_cpp_sort_int`；覆盖空容器/越界/重复 destroy 边界 | 3 个新测试绿，全量 331+ 测试零回归 |
| C++ 测试纳入 CI | `ci_three_tier_check.py` 增加 C++ 专项 tier；创建 `CPP_FAILURES.md` | CI 能机械检查 C++ 测试与失败文档的一致性 |
| 文档同步 | 更新 `CPLUSPLUS_P0_P2_GAP_REPORT.md`，标记 Phase 3 完成；在 `CPLUSPLUS_EXTENSION_PLAN.md` 中明确 Stage 0/1/2 边界 | 文档与代码事实一致 |

**禁止项**：此阶段绝不触碰类模板、RAII、引用语法。

---

### Stage 1：类模板实例化（第 2-4 周，核心 2.5 周）

**目标**：`template<class T> class vector { ... };` 能被正确解析、实例化为 `vector__int`，并参与 TypeChecker 布局分析与 BytecodeGen 代码生成。

#### Week 2：Parser 模板 id 类型解析

| 任务 | 文件 | 细节 |
|------|------|------|
| Parser 识别模板名表 | `parser/mod.rs` | 在 Parser 中维护 `template_names: HashSet<String>`，当 `parse_template_decl` 解析完模板定义后，将模板名加入该集合 |
| `parse_base_type` 扩展 | `parser/mod.rs` | 当 `is_cpp_mode` 为真且 identifier 在 `template_names` 中时，向后查看 `<`；若存在则递归解析 `<type, type, ...>`，生成 `Type::Class { name: "vector__int" }`（预 mangling）或新增 `Type::TemplateId { base, args }` |
| `DeclaratorNode` 无需改动 | — | 模板 id 作为整体类型名出现在 `Base` 位置即可，不触及 pointer/array/function 解析 |
| 测试 | `parser_cpp_unit_test.rs` | `test_parser_template_id_type`：`vector<int> v;` 解析后 `var_type` 为 `Type::Class { name: "vector__int" }`（或 TemplateId）；`test_parser_template_id_nested_pointer`：`vector<int>* p;` |

**决策点**：是否在 Parser 阶段做 mangling？
- **推荐**：Parser 生成 `Type::TemplateId { base: "vector", args: [Type::Int] }`，TypeChecker 负责 mangling 为 `"vector__int"`。这样 TypeChecker 错误消息可显示原始类型名。

#### Week 3：TypeChecker 类模板实例化

| 任务 | 文件 | 细节 |
|------|------|------|
| `try_monomorphize_class` | `typeck/cpp_monomorph.rs` | 镜像 `try_monomorphize_func`：从 `self.templates` 取 `Templateable::Class`，构建 `type_map`，deep-clone `ClassDecl`，替换字段类型 / 方法参数 / 返回值 / 构造函数 / 析构函数中的模板参数；mangling 为 `"vector__int"` |
| 类方法 / 构造函数 / 析构函数实例化 | `typeck/cpp_monomorph.rs` | 对 `ClassDecl.members` 中每个 `Method` / `Constructor` / `Destructor`，生成对应的 `FuncDecl`（与现有 `check_class_methods` 生成的 `__ctor__{name}` 格式一致，但使用 mangled 类名） |
| 注册实例化产物 | `typeck/mod.rs` | 新增 `pending_class_instantiations: Vec<(String, ClassDecl)>`；在 Pass 3 之后 drain 到 `program.classes`；同步注册 `ClassSymbol` 到 `self.classes` |
| 触发点 | `typeck/decl.rs` + `typeck/expr.rs` | `Stmt::VarDecl` 遇到 `Type::TemplateId` 时触发实例化；`Expr::New` 遇到 `Type::TemplateId` 时触发；实例化后替换为 `Type::Class { name: mangled }` |
| 布局复用 | `typeck/cpp_class_layout.rs` | 确保实例化后的 `ClassDecl` 走 `analyze_class` 路径，计算 size、vtable、继承偏移 |

#### Week 4：集成测试与边界

| 任务 | 验收标准 |
|------|----------|
| 端到端测试：`template<class T> class Box { public: T value; }; Box<int> b; b.value = 42;` | stdout 输出 42 |
| 端到端测试：类模板构造函数 `template<class T> class Wrapper { public: T v; Wrapper(T x) { v = x; } }; Wrapper<int> w(10);` | w.v == 10 |
| 端到端测试：类模板方法 `template<class T> class Adder { public: T add(T a, T b) { return a+b; } };` | `adder.add(3,4) == 7` |
| 端到端测试：类模板嵌套指针 `template<class T> class Ptr { public: T* p; }; Ptr<int> ptr; ptr.p = new int(5);` | `*ptr.p == 5` |
| 负向测试：`Box<int> b; b.value = "hello";` | 报 `E3004_TypeMismatch` |
| 零回归 | 全量 `cargo test` 绿 |

---

### Stage 2：栈对象 RAII（第 4-7 周，核心 2.5 周）

**目标**：局部 `vector<int> v;` 自动调用默认构造函数；scope exit / return / break / continue 自动按 LIFO 顺序调用析构函数。

**设计决策**：BytecodeGen 层处理（非 TypeChecker AST 插入）。理由：控制流（return/break/continue/goto）的 scope 退出信息在 BytecodeGen 的 `local_scope_stack` 中最清晰，AST 层插入会严重污染结构。

#### Week 4-5：构造函数自动调用 + Scope Exit 析构

| 任务 | 文件 | 细节 |
|------|------|------|
| 跟踪 scope 内类类型变量 | `codegen/mod.rs` | 扩展 `local_scope_stack` 记录：每个 scope 维护 `Vec<(String, local_offset, class_name)>`（类类型变量列表） |
| VarDecl 零初始化路径 emit ctor | `codegen/stmt.rs` | 在 zero-init 后（或替代 zero-init），若 `vty.is_class()` 且存在 `__ctor__{name}`，emit `GetFrameBase + PushConst offset + Add + Call __ctor__` |
| VarDecl init-expr 路径 emit ctor | `codegen/stmt.rs` | 若 init 是 `Expr::Call`（构造函数语法）或拷贝初始化，调用 `resolve_constructor_overload` 选择 ctor，emit 对应 `Call` |
| `exit_scope` emit dtor | `codegen/mod.rs` | 在恢复 shadow 变量前，逆序遍历当前 scope 的类变量，emit `GetFrameBase + PushConst offset + Add + Call __dtor__{name}` |
| 验证 | `bytecode_gen_cpp_unit_test.rs` | `test_cpp_stack_ctor_dtor`：类含 static flag，局部变量构造/析构时修改 flag，验证执行顺序 |

#### Week 5-6：Early Return / Break / Continue 析构

| 任务 | 文件 | 细节 |
|------|------|------|
| `Return` 前 emit dtors | `codegen/stmt.rs` | 在 `Ret` / `RetVoid` 前，从当前 scope 到函数最外层，按逆序遍历所有活跃 scope 的类变量，emit dtor 调用链 |
| `Break` / `Continue` 前 emit dtors | `codegen/stmt.rs` | 计算 break/continue 目标 scope 深度，emit 中间所有 scope 的 dtor 调用；利用现有 `break_patches` / `continue_patches` 机制，在 patch 前插入 dtor 代码 |
| 测试 | `bytecode_gen_cpp_unit_test.rs` | `test_cpp_early_return_dtors`、`test_cpp_break_dtors`、`test_cpp_nested_scope_dtors` |

#### Week 6-7：`goto` 与复杂控制流（可选降级）

| 方案 | 说明 |
|------|------|
| **方案 A（推荐）**：不支持 goto 跨越含 dtor 的 scope | 若 `goto` 的源 scope 深度 > 目标 scope 深度，且中间有类类型变量，TypeChecker 报 `E4007_GotoSkipsDestructor` 错误。实现简单，教学场景极少需要此模式。 |
| 方案 B：完整支持 | 需在 label 解析后二次 pass 插入 dtor，或维护运行时 cleanup 表。工作量翻倍，收益低。 |

---

### Stage 3：`new[]/delete[]` 元素构造析构（第 7 周前半，4 天）

**目标**：`new T[n]` 对类类型元素逐个调用默认构造函数；`delete[] p` 对类类型元素逆序调用析构函数。

| 任务 | 文件 | 细节 |
|------|------|------|
| `new[]` 元素构造 | `codegen/cpp_this_new_delete.rs` | `gen_new` 中，若 `size_expr` 存在且 `elem_type.is_class()`，malloc 后循环 `0..n`，计算 `base + i * elem_sz`，对每个元素 emit `Call __ctor__` |
| `delete[]` 元素析构 | `codegen/cpp_this_new_delete.rs` | `gen_delete` 中，若 `is_array` 为真且元素为 class，先逆序循环 emit `Call __dtor__`，再 `free` |
| 测试 | `bytecode_gen_cpp_unit_test.rs` | `test_cpp_new_array_ctor`、`test_cpp_delete_array_dtor`：类含 static counter，new[]/delete[] 后 counter 正确 |

---

### Stage 4：引用声明与基本语义（第 7-8 周，1 周）

**目标**：`int& r = x;` 能解析、类型检查、正确生成代码；`T&` 可作为函数参数和返回值。

| 任务 | 文件 | 细节 |
|------|------|------|
| Parser 支持 `&`/`&&` declarator | `parser/mod.rs` | `DeclaratorNode` 新增 `Reference { base, is_const }` / `RValueRef { base }`；`parse_declarator_node` 消费 `Ampersand` / `AndAnd`；`interpret_declarator_node` 生成 `Type::Reference` / `Type::RValueRef` |
| Codegen auto-deref | `codegen/expr.rs` | `Expr::Identifier` 若类型为 `Reference` / `RValueRef`，`gen_expr` 先 `LoadLocal` 取地址，再 `LoadMem` 解引用；`gen_addr` 直接返回存储的地址（不再加 frame base） |
| 绑定规则验证 | `typeck/mod.rs` | 复用现有 `check_assignable` 中的引用绑定逻辑；确保非 const 引用只能绑定左值 |
| 测试 | `typeck_cpp_unit_test.rs` + `bytecode_gen_cpp_unit_test.rs` | `test_cpp_reference_decl`、`test_cpp_reference_bind_lvalue`、`test_cpp_reference_auto_deref`、`test_cpp_reference_param` |

---

### Stage 5：Dogfooding 基础设施（第 8 周，4 天）

**目标**：建立 Stage 1 验证所需工具链。

| 任务 | 文件/位置 | 细节 |
|------|-----------|------|
| `compile_cpp_bytecode(source) -> CompileOutput` | `native/tests/test_utils.rs`（新建或复用） | 白盒 helper：强制 `is_cpp_mode = true`，返回 `CompileOutput` |
| Bytecode 语义比较器 | `native/tests/test_utils.rs` | `assert_bytecode_equivalent(actual, expected, func_name)`：提取指定函数指令切片，归一化 jump 目标（相对函数起始 IP），忽略 source_map / string_data 地址差异，逐 opcode+operand 比较 |
| Dogfooding harness | `native/tests/cpp_dogfooding_test.rs`（新建） | `dogfood_cpp_vs_c(cpp_src, c_src, func_name)`：编译两者，用 `assert_bytecode_equivalent` 比较指定函数字节码；失败时输出 diff |
| C++ Shadow Verification 扩展 | `scripts/shadow_verify_cpp.py`（新建）或扩展 `shadow_verify.py` | 对可编译的 C++ 子集（不含模板、类等 Cide 特有语法），用 `clang++` 生成 baseline，与 Cide C++ 对比 stdout |

---

### Stage 6：Dogfooding 验证（第 9-10 周，2 周）

**目标**：用标准 C++ 语法实现容器，验证与 Stage 0 C 版本字节码一致。

#### Week 9：`vector<int>` Dogfooding

```cpp
// tests/dogfooding/stage1_vec_int.cpp
template<class T>
class vector {
    T* data;
    int size_;
    int capacity_;
public:
    vector() : data((T*)0), size_(0), capacity_(0) {}
    void push_back(T x) {
        if (size_ >= capacity_) {
            int new_cap = capacity_ == 0 ? 4 : capacity_ * 2;
            T* new_data = new T[new_cap];
            for (int i = 0; i < size_; i++) new_data[i] = data[i];
            delete[] data;
            data = new_data;
            capacity_ = new_cap;
        }
        data[size_++] = x;
    }
    T get(int i) { return data[i]; }
    int size() { return size_; }
    ~vector() { delete[] data; }
};

int main() {
    vector<int> v;
    v.push_back(3);
    v.push_back(1);
    v.push_back(4);
    for (int i = 0; i < v.size(); i++) {
        printf("%d\n", v.get(i));
    }
    return 0;
}
```

| 验证项 | 方法 |
|--------|------|
| 运行一致性 | 编译上述 C++ 代码 → VM 运行 → stdout 为 `3\n1\n4\n` |
| 字节码一致性 | `dogfood_cpp_vs_c` 比较 `main` 函数与等价的 C 版本 `cide_vec_int` 调用代码 |
| 边界一致性 | 空 vector、单次 push_back、越界 get（应产生 TrapBounds 或 segfault，与 C 版本行为一致） |

#### Week 10：`string` 与 `list<int>` Dogfooding + Stage 2 决策

- 用相同模式验证 `template<class T> class basic_string`（简化版，char 特化）
- 验证 `template<class T> class list`（简化版，int 特化）

**Stage 2 决策会议**：
- 若 `vector<int>` / `string` / `list<int>` 的 C++ 版本字节码与 C 版本逐指令一致 → **通过 Dogfooding**，进入 Stage 2（删除 C 实现，全面替换）
- 若不一致 → 分析 diff，定位 BytecodeGen/TypeChecker 缺陷，修复后重试
- **Dogfooding 不通过，不发布 Stage 2**（文档风险缓解策略）

---

## 三、里程碑与验收标准

| 里程碑 | 时间 | 验收标准 |
|--------|------|----------|
| M3.5：Phase 3 收口完成 | T+1 周 | `list<int>` / `vector<char>` / `sort` 测试绿；C++ 测试纳入 CI；`CPP_FAILURES.md` 创建 |
| M4：类模板实例化完成 | T+4 周 | `template<class T> class Box/Wrapper/Adder` 全部端到端测试绿；`Box<int>` 与手写 `Box_int` 字节码等价 |
| M5：栈 RAII 完成 | T+7 周 | 局部类对象自动构造/析构通过；early return / break / continue 析构顺序正确；嵌套 scope LIFO 验证 |
| M6：引用 + `new[]/delete[]` 完成 | T+8 周 | `int& r = x` 端到端通过；`new Class[5]` / `delete[] p` 元素构造析构正确 |
| M7：Dogfooding 基础设施就绪 | T+8.5 周 | `compile_cpp_bytecode` + `assert_bytecode_equivalent` + harness 可用；有示例 diff 输出 |
| **M8：Dogfooding 通过** | **T+10 周** | **`vector<int>` C++ 版本与 C 版本字节码逐指令一致；`string` / `list` 运行行为一致** |
| M9：Stage 2 替换决策 | T+10 周 | 团队评审会：确认 C++ 容器源码可完全替代 C 实现；制定删除 `runtime_libc/cide/*.c` 的迁移计划 |

---

## 四、测试防线建设

### 每层实施阶段必须伴随的测试

| 阶段 | 新增测试文件 | 测试类型 | 数量目标 |
|------|-------------|----------|---------|
| Stage 0.5 | `bytecode_gen_cpp_unit_test.rs` | 黑盒端到端（容器操作） | +3 |
| Stage 1 | `parser_cpp_unit_test.rs` | 白盒 AST（模板 id 解析） | +4 |
| Stage 1 | `typeck_cpp_unit_test.rs` | 白盒类型检查（类模板实例化） | +6 |
| Stage 1 | `bytecode_gen_cpp_unit_test.rs` | 黑盒端到端（类模板使用） | +8 |
| Stage 2 | `bytecode_gen_cpp_unit_test.rs` | 黑盒端到端（RAII 行为） | +6 |
| Stage 2 | 新建 `codegen_cpp_whitebox_test.rs` | 白盒 bytecode 检查（ctor/dtor emit） | +4 |
| Stage 4 | `typeck_cpp_unit_test.rs` + `bytecode_gen_cpp_unit_test.rs` | 引用声明与解引用 | +4 |
| Stage 5 | `cpp_dogfooding_test.rs` | 字节码等价比较（C++ vs C） | +3 |
| Stage 6 | `cpp_dogfooding_test.rs` | 容器 Dogfooding | +6 |

### 回归保护

- 每个阶段完成后必须跑：`cargo test`（全量 331+）、`cargo test --test bytecode_libc_consistency`、`cargo test --test differential_stress`
- C++ 测试纳入 `ci_three_tier_check.py` 的新 tier：`cpp_unit_tests`
- 任何 C++ 测试失败必须记入 `CPP_FAILURES.md`，禁止修改测试预期值粉饰数据

---

## 五、风险控制

| 风险 | 可能性 | 影响 | 缓解 |
|------|--------|------|------|
| 类模板实例化与函数模板单态化代码重复 | 中 | 维护负担 | 提取公共 trait / 宏：`TemplateReplacer` 统一处理 `replace_template_type` 对 FuncDecl 和 ClassDecl 的递归替换 |
| 栈 RAII 与 `goto` 冲突 | 中 | 实现复杂度爆炸 | **明确排除**：goto 跨越含 dtor 的 scope 时 TypeChecker 报错 `E4007`，文档标注限制 |
| 析构函数在异常路径（VM Trap）不执行 | 中 | 内存泄漏 | VM 层面目前无异常； Trap 直接终止程序。文档标注：Cide 子集无异常，Trap 不保证析构 |
| Dogfooding 字节码不一致但行为一致 | 中 | 逐指令等价目标失败 | 放宽验收标准：若 opcode 序列差异仅来自局部变量分配顺序或临时变量命名，但语义等价（相同 VM 输出），可接受为"等价"；需在 harness 中实现语义级比较模式 |
| 编译时间膨胀 | 低 | 类模板实例化后编译变慢 | 限制递归深度 ≤ 8（已有）；实例化缓存（`self.classes` / `self.funcs` 查重）已存在 |

---

## 六、资源与并行度

| 阶段 | 可并行任务 |
|------|-----------|
| Stage 0.5 + Stage 1 Week 2 | Phase 3 收口可与 Parser 模板 id 解析并行（不同文件） |
| Stage 1 Week 3 + Stage 4 | 类模板实例化（TypeChecker）可与引用 Parser 支持并行 |
| Stage 2 + Stage 3 | 栈 RAII 与 `new[]/delete[]` 元素构造可并行（不同文件） |
| Stage 5 + Stage 6 Week 9 | Dogfooding 基础设施建设可与 `vector<int>` 手工验证并行 |

**建议团队配置**：1 人主攻 TypeChecker/Parser（类模板 + 引用），1 人主攻 BytecodeGen（RAII + new/delete），1 人主攻测试基础设施 + Dogfooding 验证。

---

## 七、文档同步清单

每完成一个 Stage，必须更新以下文档：

1. `docs/current/CPLUSPLUS_EXTENSION_PLAN.md` — 修正里程碑日期、标记已完成阶段
2. `native/tests/CPP_FAILURES.md` — 记录已知 C++ 失败项（即使为零也要显式声明）
3. `CHANGELOG.md` — 按 Phase 记录新增特性
4. `AGENTS.md` — 若新增构建命令或测试命令则更新

---

## 八、Go / No-Go 检查点

在进入下一阶段前，必须满足：

- **Stage 1 启动前**：Stage 0.5 全部验收标准通过；`cargo test` 全绿；`ci_three_tier_check.py` 通过
- **Stage 2 启动前**：`Box<int>` / `Wrapper<int>` / `Adder<int>` 类模板端到端测试绿；`template<class T> class vector { T* data; ... }` 能解析并通过 TypeChecker（无需运行，只需编译通过）
- **Stage 6 启动前**：栈 RAII 测试绿；`vector<int> v;` 在 main 中自动构造/析构可观测；`new[]/delete[]` 元素构造析构测试绿
- **Stage 2 替换（Dogfooding 通过后）**：团队评审会签字；`CPP_FAILURES.md` 中无未解决的 Dogfooding 相关失败；文档更新完成
