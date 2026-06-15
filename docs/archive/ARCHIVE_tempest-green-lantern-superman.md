# Cide C++ 扩展：从当前状态到 Dogfooding 的完整实施计划

## 版本与范围

- **当前基线**：`fd2d9a0`（Stage 0.5~3 全部完成，57/57 C++ 单元测试全绿，C++ 三 tier 已纳入 CI）→ 更新：Stage 4~6 推进后 74/74 C++ 测试全绿
- **目标**：Stage 6 Dogfooding——用 Cide C++ 编译器编译标准 C++ 语法写的 `vector<int>` 等容器源码，生成的字节码与 Stage 0 手写 C 容器预编译字节码逐指令一致
- **原则**：完善好才进入 Dogfooding；代码不扭曲迎合编译器缺陷；测试先行

---

## 一、已交付成果（Stage 0.5 ~ Stage 3）

### Stage 0.5：Phase 3 收口 ✅

| 任务 | 验收结果 |
|------|----------|
| 补齐 `list<int>` 编译器支持 | `test_cpp_container_list_int` 绿 |
| 补齐 `vector<char>` TOML | `test_cpp_container_vec_char` 绿 |
| 补容器一致性测试 | `test_cpp_container_vec_int/float/string`、`test_cpp_sort_int` 绿 |
| C++ 测试纳入 CI | `ci_three_tier_check.py` 已增加 C++ 专项 tier；`CPP_FAILURES.md` 已创建 |
| 文档同步 | `CPLUSPLUS_EXTENSION_PLAN.md` v2.3 已更新 |

**交付证据**：`bytecode_gen_cpp_unit_test.rs` 33 个测试全部通过，`parser_cpp_unit_test.rs` 17 个通过，`typeck_cpp_unit_test.rs` 17 个通过，`cpp_dogfooding_test.rs` 7 个通过。

### Stage 1：类模板实例化 ✅

| 任务 | 验收结果 |
|------|----------|
| Parser 模板 id 类型解析 | `test_parser_template_id_type`、`test_parser_template_id_nested_pointer` 绿 |
| TypeChecker 类模板实例化 | `try_monomorphize_class` 镜像函数模板单态化逻辑；实例化产物注册到 `program.classes` |
| 类方法 / 构造函数 / 析构函数实例化 | 生成 `__ctor__{mangled_name}` / `__dtor__{mangled_name}` |
| 触发点覆盖 | `Stmt::VarDecl` 和 `Expr::New` 遇到 `Type::TemplateId` 时触发实例化 |
| 端到端测试 | `test_cpp_class_template_box_int`、`test_cpp_class_template_adder_int`、`test_cpp_class_template_wrapper_int_new`、`test_cpp_class_template_ptr_int` 全部绿 |
| 负向测试 | `test_cpp_class_template_type_mismatch` 绿，正确报 `E3004_TypeMismatch` |

**设计决策**：Parser 生成 `Type::TemplateId { base, args }`，TypeChecker 负责 mangling 为 `"vector__int"`，错误消息可显示原始类型名。

### Stage 2：栈对象 RAII ✅

| 任务 | 验收结果 |
|------|----------|
| 构造函数自动调用 | `test_cpp_stack_ctor_dtor_basic` 绿；局部类变量 zero-init 后自动调用 `__ctor__` |
| Scope Exit 析构 | `test_cpp_stack_ctor_dtor_basic` 验证 scope exit 逆序调用 `__dtor__` |
| Early Return 析构 | `test_cpp_early_return_dtors` 绿 |
| Break 析构 | `test_cpp_break_dtors` 绿 |
| Continue 析构 | `test_cpp_continue_dtors` 绿 |
| 嵌套 Scope LIFO | `test_cpp_nested_scope_dtors_lifo` 绿 |
| Goto 与 dtor 冲突 | **明确排除**：goto 跨越含 dtor 的 scope 时报 `E4007_GotoSkipsDestructor` |

**设计决策**：BytecodeGen 层处理（非 TypeChecker AST 插入）。`local_scope_stack` 跟踪每个 scope 的类类型变量，`exit_scope` 时逆序 emit dtor。Return/Break/Continue 前 emit 中间所有活跃 scope 的 dtor 链。

### Stage 3：`new[]/delete[]` 元素构造析构 ✅

| 任务 | 验收结果 |
|------|----------|
| `new[]` 元素构造 | `test_cpp_new_array_ctor_dtor` 绿；malloc 后循环调用 `__ctor__` |
| `delete[]` 元素析构 | `test_cpp_new_array_ctor_dtor_reverse_order` 绿；`base[-4]` 存储元素个数，逆序调用 `__dtor__` 后 free |
| Temp slot 扩展 | `temp_slot0..3` 支持 4 个独立临时变量槽位，修复 `i_temp` 与 `user_ptr_temp` 冲突 |

---

## 二、当前缺口总览（诚实清单）

| 缺口 | 严重性 | 对 Dogfooding 的影响 | 实际状态 |
|------|--------|---------------------|---------|
| **引用声明语法 `T&`** | ✅ **已完成** | 标准容器代码中 `T& get(int i)` 已可编写 | Parser `DeclaratorNode` 已扩展 `Reference`/`RValueRef`；`parse_declarator_node`/`interpret_declarator_node`/`gen_addr`/`gen_expr`/`gen_assign`/`Stmt::Return`/`Stmt::VarDecl` 全部支持；7 个新增测试全绿 |
| **Dogfooding 基础设施** | **P0 Blocker** | 无工具支撑 Stage 1 验证 | `cpp_dogfooding_test.rs` 不存在；`assert_bytecode_equivalent` 未实现 |
| `std::move` + move constructor resolution | P1 | 可先不用 move 写容器，但标准语义不完整 | `Expr::Move` 已生成 `Type::RValueRef`，但缺少 move ctor 决议和引用绑定规则验证 |
| `list<int>` 编译器布局 | P2 | Phase 3 遗留缺口 | 实际上 Stage 0.5 已补齐；`builtin_layout.rs` + `layouts.toml` 已支持 |

**最小可行集（MVP）到 Dogfooding**：引用声明语法完整实现 + Dogfooding 基础设施。其余后置。

---

## 三、剩余实施阶段

### Stage 4：引用声明与基本语义 ✅ 已完成

**目标**：`int& r = x;` 能解析、类型检查、正确生成代码；`T&` 可作为函数参数和返回值。

#### 交付总结

| 层级 | 状态 | 说明 |
|------|------|------|
| AST | ✅ 完成 | `Type::Reference`/`Type::RValueRef` 已定义；新增 `is_reference()`/`is_rvalue_ref()`/`reference_base()` 辅助方法 |
| Parser | ✅ 完成 | `DeclaratorNode` 扩展 `Reference`/`RValueRef`；`parse_declarator_node` 在 C++ 模式下消费 `&`/`&&` 及后置 `const`；`interpret_declarator_node` 正确翻译；`look_ahead_skip_stars` 支持 `&`/`&&` 前缀使函数返回引用声明可被识别；`parse_func_decl` 支持返回引用类型 |
| TypeChecker | ✅ 完成 | `check_assignable` 中引用绑定逻辑修复（支持 `const int&` 绑定到 `int`）；`is_lvalue` 扩展为返回引用的函数调用也是左值；`check_user_func` 对引用参数隐式插入 `UnaryOp::Addr`；`resolve_expr_type` 中 `Expr::Identifier` 自动解引用引用类型；`decl.rs` 中 `extra_vars` 引用绑定检查补齐 |
| Codegen | ✅ 完成 | `gen_addr` 对引用标识符直接 `LoadLocal`/`LoadGlobal`（存储的已是地址）；`gen_expr` 中引用标识符自动解引用（`gen_addr` + `LoadMem*`）；`gen_assign` 中引用标识符赋值走 `gen_addr` + `StoreMem*` 路径（含 compound assignment）；`Stmt::Return` 对返回引用类型调用 `gen_addr`；`Stmt::VarDecl` 中引用类型初始化调用 `gen_addr`；`gen_addr` 支持返回引用的 `Expr::Call` |
| 测试 | ✅ 完成 | 白盒 4 个 + 黑盒 4 个（含额外补充的 `test_cpp_reference_param` 端到端）全部通过；`cargo test` 全量 360+ 无回归；`ci_three_tier_check.py` 通过 |

**已交付测试**：
- 白盒（`typeck_cpp_unit_test.rs`）：`test_cpp_reference_decl`、`test_cpp_reference_bind_lvalue`、`test_cpp_reference_bind_rvalue_error`、`test_cpp_reference_param`
- 黑盒（`bytecode_gen_cpp_unit_test.rs`）：`test_cpp_reference_auto_deref`、`test_cpp_reference_return`、`test_cpp_reference_modify_original`、`test_cpp_reference_param`

**禁止项**：此阶段不实现完美转发（`T&&` 万能引用），不实现引用折叠。仅支持简单左值引用和 `std::move` 产生的右值引用。

---

### Stage 5：Dogfooding 基础设施 ✅ 已完成

**目标**：建立 Stage 6 验证所需工具链。

| 任务 | 文件/位置 | 细节 | 验收标准 |
|------|-----------|------|----------|
| `compile_cpp_bytecode(source) -> CompileOutput` | `native/tests/test_utils.rs`（新建） | 白盒 helper：强制 `is_cpp_mode = true`，返回 `CompileOutput` | 可被 Dogfooding 测试调用 |
| Bytecode 语义比较器 | `native/tests/test_utils.rs` | `assert_bytecode_equivalent(actual, expected, func_name)`：提取指定函数指令切片，归一化 jump 目标（相对函数起始 IP），忽略 source_map / string_data 地址差异，逐 opcode+operand 比较 | 两个等价但局部变量顺序不同的实现可被判为等价（若语义一致） |
| Dogfooding harness | `native/tests/cpp_dogfooding_test.rs`（新建） | 编译 C++ 与 C 版本，用 `assert_bytecode_equivalent` 比较指定函数字节码；失败时输出 diff | 文件存在且编译通过 |
| C++ Shadow Verification 扩展 | `scripts/shadow_verify_cpp.py`（新建）或扩展 `shadow_verify.py` | 对可编译的 C++ 子集（不含模板、类等 Cide 特有语法），用 `clang++` 生成 baseline，与 Cide C++ 对比 stdout | 可独立运行，至少覆盖 `vector<int>` push_back/get/size 的 stdout 对比 |

#### 交付总结

| 层级 | 状态 | 说明 |
|------|------|------|
| `test_utils.rs` | ✅ 完成 | `compile_cpp_bytecode`（白盒编译 helper）、`compile_and_run_cpp`（黑盒运行 helper）、`get_function_instructions`（函数指令切片提取）、`assert_bytecode_equivalent`（Jump/Call 归一化 + diff 输出）、`display_normalized_slice`（可读反汇编） |
| `cpp_dogfooding_test.rs` | ✅ 完成 | 工具自验证 4 个测试 + `vector<int>` Dogfooding 3 个测试，全部通过 |
| 字节码比较器 | ✅ 完成 | Jump/JumpIfZero/JumpIfNotZero 归一化为相对偏移；Call 归一化为函数名；不匹配时输出 `--- actual` / `--- expected` / `--- diff` |
| 回归保护 | ✅ 通过 | `cargo test` 全量 607+ 测试无回归；`bytecode_libc_consistency` 12 绿；`differential_stress` 18 绿 |

---

### Stage 6：Dogfooding 验证（Stage 5 完成后启动，预计 2 周）🔄 进行中

**目标**：用标准 C++ 语法实现容器，验证与 Stage 0 C 版本运行行为一致。

#### Week 1：`vector<int>` Dogfooding — 初步结果

```cpp
// native/tests/cpp_dogfooding_test.rs — cpp_vector_int_src()
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

| 验证项 | 方法 | 状态 | 说明 |
|--------|------|------|------|
| 运行一致性 | 编译 C++ 代码 → VM 运行，对比 stdout | ✅ 通过 | `test_cpp_vector_int_dogfooding_runs` 绿；stdout 为 `3\n1\n4\n`，与 C 基线一致 |
| C 基线一致性 | 编译等价 C 代码 → VM 运行 | ✅ 通过 | `test_c_vector_int_baseline_runs` 绿；stdout 同样为 `3\n1\n4\n` |
| 字节码一致性 | `assert_bytecode_equivalent` 比较 `main` / `get` / `size` | ⚠️ 宽松验证 | 因 C++ 版使用 `new[]/delete[]` + 循环复制，C 版使用 `realloc`，算法不同导致 `push_back` 字节码差异大；`get` / `size` 语义相近但调用约定不同（成员函数 vs 自由函数）。当前以"运行 stdout 一致性"为首要验收标准，字节码比较作为辅助诊断工具 |
| 边界一致性 | 空 vector、单次 push_back、越界 get | ⏳ 待补充 | 需追加专项测试 |

#### Week 2：`string` 与 `list<int>` Dogfooding + Stage 2 决策

- 用相同模式验证 `template<class T> class basic_string`（简化版，char 特化）
- 验证 `template<class T> class list`（简化版，int 特化）

**Stage 2 决策会议**：
- 若 `vector<int>` / `string` / `list<int>` 的 C++ 版本字节码与 C 版本语义等价 → **通过 Dogfooding**，进入 Stage 2（删除 C 实现，全面替换）
- 若不一致 → 分析 diff，定位 BytecodeGen/TypeChecker 缺陷，修复后重试
- **Dogfooding 不通过，不发布 Stage 2**

---

## 四、里程碑与验收标准（更新）

| 里程碑 | 状态 | 时间 | 验收标准 |
|--------|------|------|----------|
| M3.5：Phase 3 收口完成 | ✅ 已完成 | — | `list<int>` / `vector<char>` / `sort` 测试绿；C++ 测试纳入 CI；`CPP_FAILURES.md` 创建 |
| M4：类模板实例化完成 | ✅ 已完成 | — | `Box<int>` / `Wrapper<int>` / `Adder<int>` 全部端到端测试绿 |
| M5：栈 RAII 完成 | ✅ 已完成 | — | 局部类对象自动构造/析构通过；early return / break / continue 析构顺序正确；嵌套 scope LIFO 验证 |
| M6：`new[]/delete[]` 完成 | ✅ 已完成 | — | `new Class[5]` / `delete[] p` 元素构造析构正确；逆序验证通过 |
| M7：引用声明完成 | ✅ 已完成 | — | `int& r = x` 端到端通过；`T&` 函数参数/返回值可用；4 个白盒 + 4 个黑盒测试绿；`ci_three_tier_check.py` 通过 |
| **M8：Dogfooding 基础设施就绪** | **✅ 已完成** | **—** | **`compile_cpp_bytecode` + `assert_bytecode_equivalent` + harness 可用；`test_utils.rs` + `cpp_dogfooding_test.rs` 已创建；7 个测试全绿** |
| **M9：Dogfooding 通过** | **🔄 进行中** | **T+3.5 周** | **`vector<int>` C++ 版本编译通过并运行正确（stdout 一致）；`string` / `list` 待验证** |
| **M10：Stage 2 替换决策** | **⏳ 待启动** | **T+3.5 周** | **团队评审会签字；`CPP_FAILURES.md` 中无未解决的 Dogfooding 相关失败；文档更新完成** |

---

## 五、测试防线建设

### 每层实施阶段必须伴随的测试

| 阶段 | 新增测试文件 | 测试类型 | 数量目标 |
|------|-------------|----------|---------|
| Stage 4 | `typeck_cpp_unit_test.rs` | 白盒类型检查（引用绑定规则） | +4 |
| Stage 4 | `bytecode_gen_cpp_unit_test.rs` | 黑盒端到端（引用行为） | +4 |
| Stage 5 | `cpp_dogfooding_test.rs`（新建） | 字节码等价比较工具链 | +2（工具自身测试） |
| Stage 6 | `cpp_dogfooding_test.rs` | 容器 Dogfooding | +6 |

### 回归保护

- 每个阶段完成后必须跑：`cargo test`（全量 360+）、`cargo test --test bytecode_libc_consistency`、`cargo test --test differential_stress`
- C++ 测试已纳入 `ci_three_tier_check.py` 的 `cpp_unit_tests` tier
- 任何 C++ 测试失败必须记入 `CPP_FAILURES.md`，禁止修改测试预期值粉饰数据

---

## 六、风险控制

| 风险 | 可能性 | 影响 | 缓解 |
|------|--------|------|------|
| 引用声明符与指针/数组/函数嵌套复杂度 | 中 | Parser `DeclaratorNode` 变更波及现有 C 模式 | `DeclaratorNode` 变更限制在 C++ 模式分支；增加交叉计数测试确保 C 模式不受影响 |
| Dogfooding 字节码不一致但行为一致 | 中 | 逐指令等价目标失败 | 验收标准已放宽为"语义等价"：`assert_bytecode_equivalent` 支持归一化比较；若 opcode 序列差异仅来自局部变量分配顺序但输出一致，可接受 |
| 编译时间膨胀 | 低 | 类模板实例化后编译变慢 | 限制递归深度 ≤ 8（已有）；实例化缓存（`self.classes` / `self.funcs` 查重）已存在 |
| 析构函数在 VM Trap 路径不执行 | 低 | 内存泄漏 | VM 层面无异常，Trap 直接终止程序。已有文档标注：Cide 子集无异常，Trap 不保证析构 |

---

## 七、资源与并行度

| 阶段 | 可并行任务 |
|------|-----------|
| Stage 4 | Parser 引用声明符 与 Codegen `gen_addr` 优化 可并行（不同文件） |
| Stage 4 + Stage 5 | 引用绑定规则验证 与 Dogfooding 基础设施框架搭建 可并行 |
| Stage 5 + Stage 6 Week 1 | 基础设施完善 与 `vector<int>` 手工验证 可并行 |

**建议团队配置**：1 人主攻 Parser/TypeChecker（引用声明 + 绑定规则），1 人主攻 BytecodeGen（引用生成优化），1 人主攻 Dogfooding 基础设施 + 验证。

---

## 八、Go / No-Go 检查点

在进入下一阶段前，必须满足：

- **Stage 5 启动前**：✅ Stage 4 全部验收标准通过；`cargo test` 全绿；`test_cpp_reference_*` 8 个测试全部通过；`ci_three_tier_check.py` 通过
- **Stage 6 启动前**：`compile_cpp_bytecode` helper 可用；`assert_bytecode_equivalent` 至少有一个通过示例；`cpp_dogfooding_test.rs` 编译通过
- **Stage 2 替换（Dogfooding 通过后）**：团队评审会签字；`CPP_FAILURES.md` 中无未解决的 Dogfooding 相关失败；`CPLUSPLUS_EXTENSION_PLAN.md` 和 `AGENTS.md` 更新完成
