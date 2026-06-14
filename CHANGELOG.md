# Changelog

All notable changes to the Cide project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed (架构重构)
- **内置容器布局解耦（CPP_BUILTIN_LAYOUT_DECOUPLING_PLAN）**：将 `vector<int>`/`vector<float>`/`vector<char>`/`string`/`list<int>` 的布局与方法签名从 Rust 硬编码迁移到 `.cpp` 接口声明文件
  - 新建 `native/runtime_libc/cide/{vector_int,vector_float,vector_char,string,list_int}.cpp` 作为唯一真相来源，通过 `clang++ -fsyntax-only` 语法验证
  - 新增 `scripts/extract_cpp_builtin_layout.py` 轻量解析脚本，从 `.cpp` 提取字段、方法签名并生成 `native/src/compiler/cpp_frontend/builtin_layout_data.json`
  - 重写 `builtin_layout.rs`：改为 `include_str!("builtin_layout_data.json")` + `LazyLock` 加载，零硬编码容器信息
  - 重写 `type_map.rs`：改为 JSON 加载 `cpp_type_to_cide` / `map_container_method`，零硬编码方法映射
  - `codegen/mod.rs` 与 `typeck/cpp_class_layout.rs` 中的硬编码 `container_mappings` 列表改为动态遍历 `builtin_class_mappings()`
  - `scripts/precompile_bytecode_libc.py` 扩展 glob 支持 `.cpp`（当前仅识别，不预编译为字节码）
  - 删除已废弃的 `native/runtime_libc/cide/layouts.toml`
  - 全部 600+ 测试通过，零回归

### Fixed (代码审查报告推进)
- **移除生产代码中的调试输出**：删除 `capi/mod.rs` 中 `CAPI: calling run_multi_file_pipeline` 与 `DUMP: VarDecl` 的 `println!`，以及 `engine/compile_pipeline.rs` 中对 `dump_var_decls` 的调用，避免污染程序 stdout 导致 Shadow Verification 误判
- **修复 `printf`/`putchar` 输出缓冲行为**：`RuntimeState::output()` 从 `output_lines.join("\n")` 改为 `join("")`，与 C 标准一致：只有格式字符串显式包含 `\n` 或调用 `puts` 时才换行，不再为每次 `printf` 自动换行
- **struct 体支持多字段声明**：`parser/mod.rs` `parse_struct_body` 改为先 `parse_base_type` 再 `parse_declarator`，并支持逗号分隔的多个声明符（如 `int u, v, w;`）
- **支持 `typedef enum { ... } Alias;`**：新增 `parse_typedef_enum_decl`，解析匿名枚举 typedef；顶层 Enum 分支改为可选消费 `Identifier`
- **支持匿名 `enum { ... };` 声明**：移除顶层 Enum 分支对枚举名的强制要求
- **三目运算符支持数组到指针的通常转换**：`typeck/expr.rs` 对三目分支中的 `Array` 类型统一 decay 为指向元素的指针，使 `" "`（char[2]）与 `""`（char[1]）可统一为 `char*`
- **回归测试扩展**：`parser_unit_test` +3、`type_checker_unit_test` +1、`end_to_end_extra_test` +1
- **Flutter 测试金字塔**：新增 10 个测试文件、90 个测试 + 4 个集成测试
  - `test/models/ide_state_test.dart`：默认值、copyWith、hasErrors/hasWarnings
  - `test/models/unified_state_test.dart`：默认值、copyWith、ExecutionPhase getter 矩阵
  - `test/models/code_template_test.dart`：占位符替换、默认参数、模型字段
  - `test/providers/theme_notifier_test.dart`：主题切换
  - `test/providers/ide_notifier_test.dart`：build、文件管理、面板管理、watch 表达式、学习进度、教程
  - `test/providers/unified_notifier_test.dart`：build、播放控制、onCodeChanged
  - `test/services/learning_progress_service_test.dart`：SharedPreferences load/save/clear、非法 JSON 回退
  - `test/widgets/custom_keyboard_test.dart`：字母/数字/符号模式、配对键、Shift、Space、滚动
  - `test/widgets/file_tab_bar_test.dart`：渲染、切换、关闭按钮、关闭回调、新建文件
  - `test/widgets/tool_button_test.dart`：图标渲染、点击、禁用、自定义颜色
  - `integration_test/app_test.dart`：端到端 smoke 测试，覆盖应用启动、核心 UI 渲染、主题切换、底部 Tab 切换、新建文件
  - 添加 `mocktail: ^1.0.4` 到 `pubspec.yaml` 作为未来 mock Rust API 抽象层的基础
  - 测试揭露并修复：`closeFloatingPanel` 因 `IdeState.copyWith` 的 null 语义无法清除 `activeFloatingPanel`
  - 测试揭露并修复：`PanelItem` 缺失 `intent` 定义导致默认底部 Tab "意图" 无法渲染
  - 测试揭露并修复：`EditorPanelV2` 初始 build 时访问尚未 attach 到 ScrollView 的 `ScrollController.offset`
  - 集成测试适配：`lib/main.dart` 中 `RustLib.init()` 增加幂等保护，避免多个 `app.main()` 调用触发 "Should not initialize flutter_rust_bridge twice"
- **CI Windows 构建修复**：`.github/workflows/ci.yml` 在 `flutter build windows` 前清理 `build/windows/x64` 缓存，避免 CMake 使用缓存中的 `Visual Studio 16 2019` generator 导致在仅有 VS2022 的 runner 上失败
- **Rust 测试 warnings 清理**：`native/tests/b10_new_array_rollback.rs` 移除未使用导入；`native/tests/test_utils.rs` 添加 `#![allow(dead_code)]`
- **失败记录同步**：`bellmanFord_default` 从 `KNOWN_TEMPLATE_FAILURES` 移除；`kr_5_16` 从 `KNOWN_KR_FAILURES` 移除；更新 `E2E_FAILURES.md` / `KR_FAILURES.md`
- **Parser 支持函数指针类型转换（cast）的抽象声明符**：`parser/expr.rs` 的 `parse_type_only` 改为调用 `parse_abstract_declarator`，使 `(int (*)(void *, void *))func` 这类类型转换可被正确解析
  - 解锁 K&R `kr_5_8`（函数指针 qsort 比较器）与 `kr_5_14`（排序字段选项）
  - `kr_5_8` 的 `cases_golden/knr/kr_5_8.out` 已按 Clang + `<stdlib.h>` 重新生成
  - 新增 `parser_unit_test.rs` 回归测试 `test_parser_function_pointer_cast_type`
- **VM 支持 `main(int argc, char *argv[])`**：
  - 新增 `OpCode::PushArgc` / `PushArgv`，VM 在全局数据区后为 argv 分配内存并记录地址
  - `compiler/codegen/mod.rs` 的入口包装代码根据 `main` 参数个数自动推送 `argc`/`argv`
  - `engine/compile_pipeline.rs` 的 `setup_vm` 调用 `vm.setup_argv`
  - `flutter_bridge.rs` 新增 `set_argv`，`capi/mod.rs` 新增 `cide_set_argv`
  - `cide_cli run` 支持 `-- arg1 arg2 ...` 传递命令行参数
  - 解锁 K&R `kr_5_10`（echo 命令行参数）
  - 新增 `end_to_end_extra_test.rs` 回归测试 `test_e2e_main_args` / `test_e2e_main_no_args`
- **Shadow Verification 状态更新**：匹配数从 412 提升至 415；编译缺口从 5 降至 3（仅剩 `inline_asm`/`static_assert`/`typeof_operator` 三个已知不支持特性）；运行时缺口从 31 降至 30
- **K&R 失败记录更新**：`KR_FAILURES.md` 中 `kr_5_8`/`kr_5_10`/`kr_5_14` 标记为已修复；剩余已知失败仅 `kr_6_1`
- **清除 MAUI 前端遗留的死 C API 代码**：`native/src/capi/mod.rs` 从 1384 行精简至约 290 行
  - 删除未使用的会话快照/恢复：`SessionSnapshot`、`cide_session_save`、`cide_session_load`
  - 删除未使用的 buf 版本错误获取：`cide_get_compile_errors_buf`、`cide_get_runtime_error_buf`
  - 删除未使用的单步/状态查询 API：`cide_step_next`、`cide_get_current_line`、`cide_callstack_count`、`cide_callstack_get`、`cide_breakpoint_add`/`remove`/`clear`、`cide_input_count`
  - 删除未使用的内存/变量/可视化/算法诊断 API：`cide_memory_region_count`/`get`、`cide_memory_get_value`/`pointer_target`、`cide_diagnostic_count`/`get`/`get_fix`、`cide_sourcemap_lookup`、`cide_trace_count`/`get`、`cide_variable_count`/`get`/`get_type`/`find_by_addr`/`get_field`、`cide_vis_event_count`/`get`/`get_ex`/`clear`、`cide_algorithm_match_count`/`get`/`vis_event_count`/`vis_event_get`
  - 保留的 API（Shadow Verification + 测试实际使用）：`cide_session_create`/`destroy`、`cide_compile`/`compile_unit`/`compile_all`、`cide_get_compile_errors`、`cide_set_argv`、`cide_run`、`cide_get_runtime_error`、`cide_set_input`、`cide_is_waiting_input`、`cide_provide_input_line`、`cide_get_output_length`/`get_output`
  - 移除因此变为死代码的辅助函数 `write_str` 和未使用的 `CideVM`/`setup_vm`/`reset_runtime_for_step`/`inject_preset_files` 导入

### Added (P0 语法拓展)
- **通用逗号运算符 `a, b`**：Parser 在 `parse_assign` 前新增 `parse_comma` 层，AST 新增 `BinaryOp::Comma`
  - TypeChecker 取右操作数类型，CodeGen 生成左值计算 + `Pop` 后保留右值
  - 支持 `while (a--, a > 0)`、`for (; ; a++, b++)`、表达式语句多操作等场景
- **Designated Initializer `.field = val` / `[i] = val`**：AST `InitList` 重构为 `Vec<InitElement>`
  - Parser `parse_init_list` 支持 `.field = expr` 和 `[index] = expr` 两种 designator 语法
  - TypeChecker/CodeGen：struct 按字段名写入、数组先 `Memset` 零填充再按索引写入，未指定元素自动为 0
  - 覆盖局部变量上下文（全局/静态 designated init 暂不支持）
- **`offsetof(struct S, field)`**：新增 `Expr::Offsetof`，Lexer 添加 `offsetof` 关键字，Parser 按 `offsetof(type, identifier)` 语法解析
  - TypeChecker 编译期计算字段偏移（struct 累加、union 为 0），CodeGen 直接 `PushConst(offset)`
  - 支持 struct / union 字段偏移查询
- **新增 10 个 E2E 测试**：`test_e2e_comma_operator`、`test_e2e_designated_struct_init`、`test_e2e_designated_array_init`、`test_e2e_offsetof_struct`、`test_e2e_offsetof_union` 等

### Added (标准库拓展 P0)
- **math.h 全管线支持**：引入 `libm` crate，注册 `sin`/`cos`/`sqrt`/`pow`/`atan`/`log`/`exp` 为 Layer B Rust Host Func
  - TypeChecker 支持 `double` 参数/返回类型，Host Contract 测试覆盖精度、NaN、-inf 边界行为
  - K&R 4.5（栈计算器数学函数）从已知失败中移除
- **头文件存根系统（Stub Headers）**：建立 `native/runtime_libc/include/{stdio.h,stdlib.h,ctype.h,math.h,string.h}`
  - 改造 Lexer：`#include <name.h>` 不再跳过，而是加载对应存根内容到当前翻译单元
  - 存根中声明标准库函数符号，Parser/TypeChecker 自动识别，逐步替代硬编码函数名匹配
  - 预定义宏 `NULL`/`EOF`/`stdin`/`stdout`/`stderr` 在 Lexer 中内置，兼容 K&R 早期示例

### Added (C++ 扩展 M6 — 测试防线收尾)
- **60 个 C++ E2E 回归用例**：新增 `native/tests/cases/cpp/` 目录，覆盖三大类
  - 核心语言（16）：class / ctor / dtor / 引用 / auto / 范围 for / 模板 / 虚函数 / this / 方法重载
  - 容器与算法（15）：自实现 vector<int/float/char> / list<int> / string / 排序 / 栈 / 队列 / 链表 / 二叉树
  - 教学/OJ 题目（29）：Two Sum / 去重 / 移除元素 / 二分 / 最大子数组 / 股票 / 单数 / 多数 / 旋转 / 移动零 / 回文 / 括号 / 反转链表 / 合并链表 / 树深度 / 相同树 / 翻转树 / 爬楼梯 / 帕斯卡 / 平方根 / 罗马数字 / 缺失数字 / 公共前缀 / 首个唯一字符等
- **C++ E2E 测试框架**：扩展 `native/tests/cide_e2e.rs`
  - `compile_and_run_cpp` 通过 `cide_compile_unit(..., "main.cpp", ...)` 自动启用 C++ 模式
  - `load_cpp_cases` / `run_cpp_case` 支持 `.cpp` 用例与 `.in` 输入文件
  - `test_cide_e2e_cpp` / `test_cide_e2e_cpp_known_failures` 及 `KNOWN_CPP_FAILURES` 监控
  - `TEST_REPORT.md` 生成已汇总 C++ 统计
- **Golden 来源**：所有 60 个 `.out` 文件由 Clang++ (`-std=c++14 -O0`) 生成，Cide 输出与之逐行对比，目前 60/60 全绿
- **单元测试扩展**：parser_cpp_unit_test（33）、typeck_cpp_unit_test（28）、bytecode_gen_cpp_unit_test（38）全部通过
- **诚实记录子集边界**：`native/tests/CPP_FAILURES.md` 新增 M6 过程中识别的 Cide C++ 子集边界（如类字段逗号多声明、指针逻辑运算、模板类方法引用参数等），用例已规避，无已知失败

### Fixed (C++ 子集边界消除)
- **指针逻辑运算 `&&` / `||` 支持指针/数组**：`typeck/expr.rs` 放宽 `And`/`Or` 操作数类型检查；`UnaryOp::Not` 同时支持数组
  - `cpp_merge_two_lists.cpp` 恢复标准 `while (l1 && l2)` / `while (h)` 写法
- **类内方法重载**：`ClassSymbol::methods` 从 `HashMap<String, MethodSig>` 改为 `HashMap<String, Vec<MethodSig>>`
  - 新增 `resolve_method_overload` / `overload_match_score`，按参数数量与类型相似度选择最佳签名
  - 方法 mangling 在存在多个重载时使用 `Class__method__N`（N 为用户参数个数），单签名保持向后兼容的 `Class__method`
  - 移除 Pass 2.3 重复注册逻辑；`register_single_class_layout` 统一注册方法/构造/析构函数符号
  - 支持类成员函数体内无显式 `this->` 的方法调用（C++ name hiding），`Call`/`CallPtr` 均会尝试解析为 `MemberCall`
  - 新增 E2E 用例 `cpp_method_overload.cpp`（BST 公有 `insert(int)` + 私有递归 `insert(Node*, int)` + `print` 重载）
- **M6 10 项 C++ 子集边界全部消除**：`CPP_FAILURES.md` 中记录的边界全部修复，`native/tests/cases/cpp/` 60 个用例全部使用标准 C++14 语法，`KNOWN_CPP_FAILURES` 为空

### Fixed (C++ Shadow Verification 3 gap 清零)
- **右值引用绑定函数返回值 `int&& r = foo();`**：`VarDecl` 引用初始化分支区分左值 / 引用表达式 / 纯右值；纯右值创建临时局部变量延长生命周期，再绑定引用地址
- **`const int& r = 5` 绑定字面量右值**：同上临时变量方案，常量左值引用可接受字面量右值
- **`for (auto& x : arr)` 修改数组元素**：
  - `typeck/decl.rs` 修正 `RangeFor` 变量类型推导：`auto&` 推导为 `Reference { base: elem_type }`，`auto&&` 推导为 `RValueRef { base: elem_type }`
  - `codegen/stmt.rs` 数组形式的 `RangeFor` 在循环变量为引用时存储元素地址而非元素值
- **测试扩展**：`bytecode_gen_cpp_unit_test` +3（`test_cpp_rvalue_ref`、`test_cpp_const_ref_rvalue`、`test_cpp_range_for_ref_modify`），`typeck_cpp_unit_test` +1（`test_cpp_auto_ref_range_for`）
- **Shadow Verify 状态**：`scripts/shadow_verify_cpp.py` 中 3 个用例从 `gap` 改为 `baseline`，C++ Shadow Verification 82/82 全绿，0 gap

### Added (C++ 扩展 Stage 1 — 类模板实例化)
- **Parser 模板 id 类型解析**：新增 `Type::TemplateId { base, args }`，`Parser` 维护 `template_names` 集合，`parse_base_type` 识别 `vector<int>` 语法
- **TypeChecker 类模板实例化**：`try_monomorphize_class` 镜像函数模板单态化逻辑，支持字段/方法/构造函数/析构函数中的模板参数替换
  - `resolve_template_id` 递归处理指针/数组/引用等包装器内部的 `TemplateId`
  - 实例化产物立即注册 `ClassSymbol` 并参与 Pass 3.5 `check_class_methods`
- **BytecodeGen 非类 new-init 修复**：`gen_new` 补充非 `Class` 类型（如 `new int(5)`）的 init 直接存储路径
- **MemberCall 参数检查修复**：`user_param_count` 从 `param_types.len() - 1` 修正为 `param_types.len()`（方法签名不含 `this`）
- **zero-size 类 zero-init 跳过**：`sz == 0` 时不 emit `StoreLocal`，避免 `STACK_START` 边界越界
- **集成测试 +5**：`Box<int>` 字段访问、`Adder<int>` 方法调用、`Wrapper<int>` 构造函数 + `new`、`Ptr<int>` 指针字段、类型不匹配负向测试

### Added (C++ 扩展 Stage 6 — `unique_ptr<T>` dogfooding 与构造函数初始化语法)
- **`unique_ptr<T>` 简化版全管线跑通**：模板类 `unique_ptr<T>`（单 `T*` 字段）支持构造、`get()`、`release()`、`reset()`、析构，以及 `std::move` 触发的隐式移动构造转移所有权并置空源对象
  - 新增 `native/tests/cpp_dogfooding_test.rs::test_cpp_unique_ptr_int_dogfooding_runs` 作为 M5 dogfooding 用例
  - 同步更新 `native/runtime_libc/cide/unique_ptr_int.{c,cpp}` 运行时布局与 `bytecode_libc_sig.rs` 签名（内置 `unique_ptr<int>` 容器走 Bytecode Libc 路径）
- **构造函数初始化语法 `Type name(args);`**：Parser `parse_var_decl_stmt` 识别类/模板类变量后的 `(...)` 为构造参数列表，生成占位 `__ctor__{Class}__{N}`；TypeChecker 解析为实际 mangled 构造函数并在参数列表前插入 `&name` 作为 `this`
- **构造函数重载与隐式默认构造**：
  - 显式构造函数按参数数量编码为 `__ctor__{Class}__{N}`，零参数保持 `__ctor__{Class}`
  - 无显式默认构造的类自动注册隐式默认构造函数，支持 `Class c;` 和 `new Class()`
  - `resolve_constructor_overload` 按参数数量匹配，带 fallback 扫描
- **`new` 表达式类型检查修复**：类类型 `new` 的 init 中尚未包含 `this`，改为根据类方法签名直接检查用户参数，避免参数数量不匹配报错
- **函数指针声明解析修复**：`parse_var_decl_stmt` 通过预读 `Identifier (` 精确区分构造初始化与函数指针后缀，恢复 `int (*fp)(int, int);` 等复杂声明符解析
- **Range-for 数组大小推断修复**：`VarDecl` 仅在构造初始化时提前 `declare_var`，避免数组初始化后推断出的大小与符号表类型不一致
- **Dogfooding 测试 +1**：`cpp_dogfooding_test` 总数达 29 个，全绿

### Added (C++ 扩展 Stage 5 — 隐式移动构造函数自动生成)
- **资源检测**：`ClassSymbol` 新增 `has_resource` 字段；`typeck/cpp_class_layout.rs` 在类布局注册后第二遍计算，递归检测指针、`Reference`/`RValueRef`、含资源 class/struct、数组元素等资源字段
- **隐式移动构造函数生成**：`typeck/cpp_overload.rs` 新增 `generate_implicit_move_ctors`，为含资源且无显式移动构造的类自动生成 `__ctor__{Class}__move`；函数体逐字段拷贝，并将源对象指针字段置 `nullptr`，防止双重释放
- **类型系统适配**：
  - `check_assignable` 允许 `RValueRef<Class>` 赋值给 `Class`（触发移动构造）
  - `Expr::Member` 类型检查支持 `Reference`/`RValueRef` 对象访问
- **BytecodeGen 调用路径**：
  - `VarDecl` 初始化时检测到 `RValueRef`/`Expr::Move` 调用 `__ctor__{Class}__move`，按 VM 参数弹出顺序右-to-left 压入 `this`/`other`
  - `gen_member_addr` 与 `get_member_offset` 支持 `Reference`/`RValueRef` 对象地址计算
  - `gen_addr` 对 `std::move(x)` 返回 `x` 的地址而非值；`CallPtr(std__move)` 在 `gen_expr` 中透传参数
  - 调用移动构造函数后记录 `class_vars`，确保作用域退出时析构被调用
- **测试 +2**：`test_implicit_move_ctor_pointer_nulls_source`、`test_implicit_move_ctor_builtin_vector`，Dogfooding 测试总数达 28 个，全绿

### Added (C++ 扩展 Stage 0.5 — Phase 3 收口)
- **容器库编译器支持补全**：
  - `builtin_layout.rs` 新增 `cide_list_int` 布局；`layouts.toml` 新增 `[vector_char]`、`[list_int]`
  - `type_map.rs` 新增 `cide_list_int` 方法映射（push_back/push_front/pop_back/size/get/destroy）
  - `list_int.c` / `vec_char.c` / `sort_int.c` 已预编译为 Bytecode Libc（索引 1000~1059）
- **C++ 容器端到端测试 +3**：`test_cpp_container_vec_char`、`test_cpp_container_list_int`、`test_cpp_sort_int`
  - 覆盖空容器/越界/重复 destroy 边界；22/22 C++ BytecodeGen 端到端测试全绿
- **C++ 测试防线建设**：
  - 创建 `native/tests/CPP_FAILURES.md`（当前零已知失败）
  - `ci_three_tier_check.py` 新增 C++ 三 tier（`parser_cpp_unit_test` 15 例、`typeck_cpp_unit_test` 13 例、`bytecode_gen_cpp_unit_test` 22 例），纳入 CI 一致性监控；C++ 扩展合计 50/50 通过

### Added (C++ 扩展 Stage 2 — 栈对象 RAII)
- **ScopeFrame 重构**：`local_scope_stack` 从 `(String, Option<...>)` 元组向量升级为 `ScopeFrame { shadows, class_vars }`，支持按作用域追踪类类型局部变量
- **构造函数自动调用**：`codegen/stmt.rs` VarDecl zero-init 路径对 `Type::Class` 自动 emit `__ctor__{Class}` 调用，实现 `Class c;` 即构造
- **析构函数自动调用**：
  - `exit_scope` 逆序遍历 `class_vars`，emit `__dtor__{Class}`
  - `Return` / `RetVoid` 前调用 `emit_dtors_for_scope_exit(0)`，覆盖函数最外层 block
  - `Break` / `Continue` 前按 `loop_scope_depths` 计算需退出的嵌套 scope， emit 对应 dtor
- **Loop scope 深度追踪**：新增 `loop_scope_depths` 栈，与 `loop_start_ips` 同步 push/pop，支持 break/continue 的精确析构范围
- **集成测试 +5**：`test_cpp_stack_ctor_dtor_basic`、`test_cpp_nested_scope_dtors_lifo`、`test_cpp_early_return_dtors`、`test_cpp_break_dtors`、`test_cpp_continue_dtors`

### Added (C++ 扩展 Stage 3 — `new[]/delete[]` 元素构造析构)
- **`new A[n]` 元素逐个构造**：`gen_new` 对类类型数组在 `base[-4]` 预存元素 count，`for i = 0..n-1` 调用 `__ctor__{Class}(user_ptr + i * elem_sz)`
- **`delete[] arr` 逆序析构**：`gen_delete` 从 `base[-4]` 读取 count，`for i = n-1..0` 调用 `__dtor__{Class}(user_ptr + i * elem_sz)`，最后 `free(base)`
- **临时变量槽位扩展**：`BytecodeGen` 的 `get_temp_slot` 从 3 个独立 slot 扩展为 4 个（`temp_slot0..3`），避免 `new[]/delete[]` 循环中 `i_temp` 与 `user_ptr_temp` 冲突
- **集成测试 +2**：`test_cpp_new_array_ctor_dtor`（验证构造次数）、`test_cpp_new_array_ctor_dtor_reverse_order`（验证析构逆序）

### Added (标准库全面拓展 P1 — 2026-06-07)
- **新增 19 个 Host Func + 7 个存骨头文件**，覆盖 C89/C99 教学高频函数：
  - `ctype.h`：`isgraph`/`ispunct`/`isblank`
  - `math.h`：`asin`/`acos`/`atan2`/`sinh`/`cosh`/`tanh`
  - `stdlib.h`：`abort`/`strtol`/`strtod`/`llabs`
  - `stdio.h`：`fflush`/`perror`/`clearerr`/`remove`/`rename`
  - `string.h`：`strerror`/`strpbrk`/`strspn`/`strcspn`
  - `time.h`：`time`/`clock` + `time_t`/`clock_t` typedef + `CLOCKS_PER_SEC` 宏
  - `assert.h`：`assert` 宏展开为 `if (!(expr)) __cide_assert_fail()`
  - `errno.h`：`extern int errno` + `EINVAL`/`ERANGE`/`EDOM`/`ENOENT`/`EACCES` 宏，Host Func 支持通过符号表写入
  - `float.h`：`FLT_MAX`/`DBL_MAX`/`FLT_EPSILON`/`DBL_EPSILON` 等宏
  - `stdint.h`/`stddef.h`：`int8_t`~`uint64_t`、`size_t`/`ptrdiff_t` typedef
- **新增 23 个 Host Contract 测试**：覆盖全部新增函数边界条件
- **VFS 扩展**：`VfsDesc` 新增 `error` 字段，支持 `fflush`/`clearerr`/`remove`/`rename`
- **CideVM 公开 API**：新增 `is_finished()`/`exit_code()` getter，供测试框架查询 VM 终止状态

### Fixed (标准库拓展中发现并修复的 Bug — 2026-06-07)
- **严重：7 个新增 Host Func 参数 pop 顺序错误**
  - 根因：新增 Host Func 实现时 `vm.pop()` 顺序错误，与 Cide 编译器「从右到左压栈」约定不匹配
  - 影响函数：`strtol`/`strtod`/`strpbrk`/`strspn`/`strcspn`/`rename`/`atan2`
  - 后果：这些函数在实际 C 代码中被调用时，所有参数全部错位；由于此前无端到端测试覆盖，bug 一直隐藏
  - 修复：调整 `vm.pop()` 顺序，使第一个 pop 得到第一个参数（栈顶）
  - 验证：Host Contract Tests 新增 23 个用例后触发失败，修复后 86 个 Host Contract 测试全部通过

### Fixed (2026-06-04 审阅报告修复)
- **Soundness / 正确性**：
  - `cstr_to_str` 返回 `&'static str` → `Option<String>`，消除 C 端释放后的悬垂引用风险
  - `VM::reset()` 遗漏 `qsort_depth` 重置，两次运行间残留值导致 VFS 行为异常
  - `algorithm_detector::is_adjacent_compare`：字符串匹配 → AST 结构比较（`idx_b` 是否为 `idx_a + 1`）
  - `algorithm_detector` mid 计算检测：字符串匹配 "mid"/"left"/"right" → AST 结构匹配 `(a+b)/2` / `a+(b-a)/2`
  - `algorithm_detector` shift 模式：宽松 `contains('[')` → 精确 `arr[x+1]=arr[x]` 结构匹配
- **性能**：
  - VM 热点路径 `LoadLocal`/`StoreLocal`/`LoadGlobal`/`StoreGlobal` O(n) 符号查找 → O(1) `HashMap`
    - `VMSymbol` 新增 `func_name` 字段，`CideVM` 新增 `local_sym_map`/`global_sym_map`
    - 函数调用/返回时自动重建局部变量映射
  - `Call`/`CallPtr` 帧设置逻辑提取 `do_call` 辅助方法，消除 ~100 行重复
- **代码质量 / DRY**：
  - `VM::check_mem_access` 统一 `load_i32`/`store_i32`/`load_i64`/`store_i64`/`load_i8`/`store_i8` 的 NULL/边界检查
  - `host_funcs` 提取 `parse_format_spec` 共享函数，消除 `parse_format_specs` 与 `format_printf_string` ~80 行重复
  - `ast.rs` 提取 `compute_type_size`/`base_element_type`，消除 `compile_pipeline` 与 `bytecode_gen` 重复
  - `type_checker::insert_implicit_cast`：6 个重复 if-else → `implicit_cast_target` 映射表
  - `type_checker::check_assignable` 拆分为 4 个独立辅助方法（数组指针/函数指针/标量/通用指针）
  - `parser` 提取 `look_ahead_skip_stars` 辅助函数
- **边界检查**：
  - `do_call` 中 `frame_size > MEM_SIZE` 改为 `> STACK_START - NULL_TRAP_SIZE`，更精确反映可用栈空间
- **工程化**：
  - `SessionSnapshot` 增加 `#[serde(deny_unknown_fields)]`，防止加载不兼容数据
  - `cide_session_load` 硬编码 `test.txt`/`numbers.txt` → 从 snapshot 序列化/恢复 VFS 预设文件
  - `UnifiedEngine::seek_to` 正向重放时检查 `is_cancelled`，支持长时间 seek 中断
  - `UnifiedEngine::max_steps` 默认 10,000 → 100,000，减少长程序过早终止
  - `lexer` 十六进制解析：`u64::from_str_radix` + 手动溢出检查 → `u32::from_str_radix`，利用类型系统防溢出
  - `parser` `parse_base_type`：`unsigned` 修饰非法组合时 early return 哨兵类型，避免继续构造无效类型
  - `opcode.rs` 添加扩展空间注释（当前最大值 111，上限 255）
  - `bytecode_gen` `push_f64_constant`/`push_i64_constant` 添加去重，相同常量复用索引
  - `bytecode_gen` `ptr_step_size` 支持指向数组的指针步长（如 `int (*p)[3]` 步长为数组总大小）
- **未使用 import 清理**：`e2e_multi_file.rs` 移除 `Type` import

### Fixed (2026-06-08 全面审阅报告修复)
- **`E3057_ConstViolation` 重命名为 `E3065_ConstViolation`**，消除标签与值不匹配
- **`opcode.rs` 更新最大 opcode 注释**：`CallPtr = 111` → `Strlen = 126`
- **`compute_stride` 增加零/负步长 guard**，防止 VLA size 未解析时的静默数据损坏
- **`codegen/mod.rs` 拆分为 `expr.rs` + `stmt.rs`**，解耦表达式/语句生成逻辑（trait 模块化）
- **`Stmt`/`FuncDecl`/`ProgramNode` 添加 `serde::Serialize/Deserialize`**，解除 C++ AST 序列化阻塞
- **C++ 扩展错误码骨架 E4001-E4020 预声明**，防止多人并行开发时编号冲突
- **Flutter `UnifiedNotifier` 覆盖 `dispose()`**，取消 StreamSubscription 防止内存泄漏
- **Flutter `main.dart` 添加应用生命周期监听**，桌面端窗口关闭时释放 VM Session
- **Flutter CI `flutter-action` 启用 `cache: true`**，减少 CI 构建时间
- **预编译脚本 `precompile_bytecode_libc.py` 适配 `cide/` 目录扫描**

### Fixed (2026-05-18 审查报告修复)
- **Rust 后端 P0 Bug（5 个严重问题）**：
  - `call_user_function` 循环次数错误：拆分 `arg_count` 为 `param_count`（参数个数）和 `param_words`（总 word 数）
  - `restore()` 快照恢复：`.copy_from_slice()` → 安全边界拷贝，防止不同内存配置下 panic
  - 复编译时 `f64_constants` 残留：添加 `clear()` 防止旧常量污染
  - 常量索引越界：`.unwrap_or(0)` → `trap` 报告越界错误
  - `PushConstF` 符号扩展：`operand as u64` → `operand as u32 as u64`，修复负 float 值损坏
- **VM 安全加固**：
  - `TrapBounds`：栈为空时 `trap` 而非静默返回 0
  - C API `cide_get_call_frame`：`vm.as_ref().unwrap()` → 安全匹配
  - `write_cstring`：移除 `#[allow(clippy::int_plus_one)]`，改写边界条件
- **代码质量与重构**：
  - 统一宿主函数名→ID 映射：`host_func_id::by_user_name()` / `is_builtin()` 消除 3 处重复
  - 合并 `gen_struct_copy` / `gen_struct_copy_to_local` → `gen_struct_copy_common`
  - 合并 `parse_abstract_declarator` / `parse_declarator_node`（新增 `is_abstract` 标志）
  - 删除 `Session.errors_buffer` 冗余字段
  - `insert_implicit_cast`：`std::mem::replace` + dummy Literal → `std::mem::take`
  - 删除未使用的 `parse_call_expr`
  - `cargo clippy -- -D warnings` 完全通过（无手动抑制）
- **工程化**：
  - 检查点内存上限：默认最大 50 个快照，防止长程序内存无限增长
  - 字符串字面量上限：`0x8000` (32KB) → `MEM_SIZE / 16` (64KB)
  - CI 新增 Release 构建验证 + Flutter 测试
  - Android `applicationId`：`com.example.cide` → `com.cide.app`
  - `re_editor` 锁定确切版本 `0.8.0`，添加私有 API 依赖注释
  - NDK 配置添加环境变量说明
- **文档同步**：
  - `DESIGN.md`：指令集 `~30 条` → `106 条`，C++ 伪代码 → Rust
  - `AGENTS.md` / `CHANGELOG.md`：测试数量 `44` → `238`
  - `ROADMAP.md`：知识图谱标记为未启动，函数指针标记为已完成
  - `CideFlutter/README.md`：重写为项目说明
- **Flutter 前端加固**：
  - `LinkedListVisualizer` / `TreeVisualizer`：异步 `setState()` 前加 `mounted` 检查
  - `LinkedListVisualizer`：内存上限改为 `rust.getMemorySize()` 动态获取
  - `MemoryTab`：`StatelessWidget` → `StatefulWidget` 缓存 Future
  - `IdeScreen`：键盘状态同步从 `build()` 移至 `didChangeDependencies`

### Added
- **键盘弹出时沉浸编辑模式**（Flutter）：
  - 自定义键盘或系统键盘弹出时，顶部工具栏、模板栏、底部面板通过 `SizeTransition` 平滑收起，编辑器自动拉伸占满剩余空间。
  - 键盘收起后上下栏自动弹出恢复。
  - 系统键盘真实可见性通过 `MediaQuery.viewInsets.bottom` 检测，收起后自动同步状态。
- **编辑器手势优化**（Flutter）：
  - 点击代码字符处：打开键盘。
  - 点击空白处（空行、行尾之后、尾部空白区域）：关闭键盘。
  - 上下滑动（位移 >100px 且垂直方向为主）：关闭键盘。
  - 长按（>600ms）仍弹出上下文菜单，不受单击/滑动逻辑影响。
  - 空白检测通过 `addPostFrameCallback` 延迟到 `re_editor` 内部更新光标位置后执行，避免依赖内部私有 API。
- **Panel drag-and-drop swap logic** (Flutter):
  - All drag interactions now perform **swap** instead of add/remove/move. Both regions (bottom tabs + floating orb) maintain fixed element counts.
  - Cross-region swap: `swapBottomWithFloatingItem(bottomPanelId, floatingIndex)` and `swapFloatingWithBottomItem(floatingPanelId, bottomIndex)` in `ide_notifier.dart`.
  - Item-level `DragTarget` for each floating menu item (`floating_orb_widget.dart`), enabling precise swap with the hovered item.
  - Hover feedback: blue border + shadow on both bottom tabs and floating menu items when a draggable hovers over them.
  - Edge detection: dropping on empty padding/orb area shows a SnackBar "未识别到可交换的目标位置".
  - Same-region filtering: floating menu item `DragTarget` only accepts drags from `PanelLocation.bottom`, preventing accidental same-region swaps.
- **Floating orb menu direction**: menu now prefers expanding **upward** whenever space allows (`_pos.dy >= menuHeight + 28`), making it easier to drag bottom tabs upward into the menu for swapping.

### Changed
- **Flutter bottom panel UI polish**:
  - Output tab empty state now shows `terminal_outlined` icon + "等待执行" text instead of plain text.
  - Diagnostics tab empty state now shows `check_circle_outline` icon + "无诊断信息" text.
  - Algorithm tab empty state now shows `auto_graph_outlined` icon + "未检测到算法模式" text.
  - Copy button in output tab now has a background container (adapts to dark/light theme) and no longer overlaps text (right padding added to scroll view).
  - Removed unused "+" button from bottom tab bar.

### Added
- Host function ID unified constant module (`vm/host_func_id.rs`) to prevent ID mismatch between compile-time and runtime.
- Unified compilation pipeline `run_compile_pipeline()` in `engine/compile_pipeline.rs` to eliminate ~100 lines of DRY violation between `flutter_bridge.rs` and `capi/mod.rs`.
- `rustfmt.toml` for consistent Rust code formatting across the project.
- `CHANGELOG.md` for tracking project evolution.
- **240 unit tests** across all compiler phases (`lexer_unit_test.rs`, `parser_unit_test.rs`, `type_checker_unit_test.rs`, `bytecode_gen_unit_test.rs`, `vm_memory_safety_test.rs`, `compile_pipeline_test.rs`, `end_to_end_test.rs`, `end_to_end_extra_test.rs`, `test_snapshot.rs`).
- **Flutter frontend modularization**: extracted all tab widgets (`AlgorithmTab`, `WatchTab`, `PointerVisTab`, `ArrayVisTab`, `MemoryTab`, `VariablesTab`, `CallstackTab`, `KnowledgeCardTab`), visualizers (`ArrayVisualizer`, `KnowledgeCardItem`), and layout components (`Toolbar`, `SymbolBar`, `TemplateBar`, `HeightResizablePanel`, `DraggablePanelTab`) from `ide_screen.dart` (2004 → 471 lines).
- **Flutter provider split**: extracted `IdeNotifier` to `providers/ide_notifier.dart` (`ide_provider.dart` 726 → 7 lines).
- **数组排序实时条形图可视化**（Flutter + Rust）：
  - Rust: `CideVM::get_array_snapshots()` 遍历符号表识别 `Type::Array`，从 VM 内存逐元素读取（支持 int/char/float/double/long long）。
  - `StepPayload` 新增 `array_snapshots: Vec<ArraySnapshot>`，`StepCollector` 每步自动收集。
  - Flutter: `ArrayVisTab` 从 `unifiedProvider` 零延迟读取；`ArrayVisualizer` 绘制条形图，高度表示数值，负值红色/正值蓝色。
  - VisEvent 比较事件（如 `arr[i]:arr[j]`）自动高亮对应条形（琥珀色 + 发光阴影）。
- **变量级高亮（读/写标记）**（Flutter + Rust）：
  - Rust: `CideVM::step()` 中 `LoadLocal`/`StoreLocal`/`LoadGlobal`/`StoreGlobal` 自动记录 `VariableAccess`（Read/Write）。
  - `StepPayload` 新增 `accessed_vars`。
  - Flutter: `VariablesTab` 被读取变量显示蓝色边框+「读」徽章，被写入显示橙色边框+「写」徽章。
- **编辑器行号区域变量访问指示**：统一模式下当前执行行的行号旁追加 `a=W b=R` 标记。
- **运行时异常智能诊断匹配**（Flutter）：
  - `KnowledgeCard` 新增 `relatedTrapKeywords` 字段和 `findByTrapMessage()` 方法。
  - 新增 5 张运行时异常知识卡片：数组越界、NULL 指针解引用、除零、栈溢出、访问已释放内存。
  - `ExecutionControlPanel` 异常提示条新增「查看帮助」按钮，点击弹出 BottomSheet 展示匹配的知识卡片。
- **学习进度追踪（统一模式）**（Flutter）：
  - `LearningProgress` 新增 `totalUnifiedRuns`/`totalStepsExecuted`/`totalTraps`/`totalSeeks`/`maxStepsInSingleRun`。
  - `IdeNotifier` 新增 `recordUnifiedRun()` / `recordSeek()`。
  - `ProgressTab` 新增「调试探索」卡片，显示运行次数/总步数/异常/Seek/峰值步数。
- **算法检测信息在前端展示**（Flutter）：`ExecutionControlPanel` 顶部显示检测到的算法名称（如「冒泡排序」）+ 时间复杂度说明。
- **IDE 热键支持（Desktop）**（Flutter）：F5 运行/继续、Shift+F5 停止、F10 单步、F9 切换断点；`EditorPanelState` 新增 `getCurrentLine()`。
- **变量值变化检测**（Flutter）：`VariablesTab` 比较当前步与上一步变量值，数值增加显示绿色 ↑，减少显示红色 ↓，非数值变化显示黄色 •。
- **断点列表管理面板**（Flutter）：新增 `BreakpointsTab`，显示所有断点行号+源码预览，支持点击跳转和删除。
- **代码覆盖率统计**（Flutter）：`ExecutionControlPanel` 显示覆盖率百分比（已执行行数/总行数），颜色分级（≥80%绿/≥50%橙/<50%红）。
- **算法事件指示条**（Flutter）：`ExecutionControlPanel` 顶部紫色渐变条显示当前步 VisEvent 上下文（如 `arr[i]:arr[i+1]`）。
- **函数指针高级语法支持**（Rust Parser + TypeChecker + BytecodeGen）：
  - 多级函数指针：`int (**pp)(int) = &fp;` — `interpret_declarator_node` 的 `Function` 分支递归解释 `ptr_inner` 为"以函数指针为基础类型的声明符"。
  - 返回指针的函数指针：`int *(*fp)(int) = greet;`。
  - `sizeof` 函数指针类型：`sizeof(int (*)(int))` — 新增 `parse_abstract_declarator()` 支持抽象声明符（括号、多级指针、数组后缀、函数参数列表）。
  - `typedef` 函数指针：`typedef int (*Op)(int, int);` — `parse_typedef` 改用完整 `parse_declarator()` 替代简陋的 `parse_type_only()`。
  - `static` 局部变量：`static int arr[3] = {1,2,3};` — `parse_statement` 识别 `static` 存储类说明符并跳过。

### Fixed
- **Flutter Overlay popup Material missing**: `FloatingPanelPopup` now wraps its content with `Material(type: MaterialType.transparency)`, eliminating the yellow underline artifacts on text and the red `No Material widget found` crash when opening `WatchTab` (which contains `TextField`) or `ProgressTab` (which contains `TextButton`) from the floating orb.
- **Flutter run/step auto-compile**: `IdeNotifier.run()` and `IdeNotifier.step()` now automatically call `compile()` before executing if the session is not already running. Previously, clicking the play button without manually compiling first resulted in a silent `"程序尚未编译"` error because `state.error` was never displayed in the UI.
- **Flutter error visibility**: `IdeScreen` now listens to `state.error` via `ref.listen` and shows a floating `SnackBar` when a new error occurs, preventing silent failures.
- `printf`/`fprintf` format specifiers now correctly skip width/precision/length modifiers (e.g. `%6d`, `%.2f`, `%ld`), preventing stack imbalance from mis-counted arguments. Shared logic extracted into `parse_format_specs()` + `format_printf_string()` in `host_funcs.rs`.
- `scanf` format parsing now also skips modifiers via `parse_format_specs()`, fixing the same miscount bug.
- Comma-separated multi-variable array declarations now preserve per-variable dimensions (`int a[10], b[20];`). `parse_declarator()` extracted; `Stmt::VarDecl.extra_vars` changed to `Vec<(Type, String, Option<Expr>)>`.
- `unsigned char` no longer mapped to `unsigned int`; now correctly preserves `TypeKind::Char` with `is_unsigned: true`.
- Flutter `IdeNotifier.reset()` is now `async` and properly `await`s `rust.resetSession()`, eliminating the race condition.
- `cide_get_runtime_error()` now uses `error_buffer` snapshot pattern (same as `cide_get_compile_errors()`), eliminating dangling pointer risk across FFI boundary.
- `cide_session_load` now restores VM state via `setup_vm()` instead of overwriting with a blank VM.
- `call_user_function` no longer incorrectly pops stack value on `Trap`; returns `None` instead.
- Hex literal overflow check relaxed from `i32::MAX` to `u32::MAX` (`0x80000000` now accepted).
- Algorithm detector now collects all matching patterns per function instead of returning only the first match.
- `call_user_function` temporarily disables breakpoints to prevent internal `Paused` from terminating `run()`.
- `Type::is_scalar()` now includes `Float`, consistent with `TypeChecker::is_scalar()`.
- `malloc(0)` emits a pedagogical warning about implementation-defined behavior.
- Lexer `make_token` column calculation now uses `text.chars().count()` instead of `text.len()`, fixing multi-byte UTF-8 character inaccuracy.
- **统一模式下断点暂停支持**（Rust + Flutter）：`AutoStepResult` 新增 `paused` 字段；`UnifiedEngine::run_batch` 正确传递 `self.is_paused`；Flutter 端 `_collectBatch` 检测到 `paused` 后取消 Timer 并切换到 `paused` 状态。
- **算法可视化事件 context 修复**（Rust）：`vm.rs` 中 `StepEvent` 生成 `VisEvent` 时 `context` 为空；`CideVM.vis_event_lines` 扩展为 `Vec<(i32, i32, String)>` 保留 context，`compile_pipeline.rs` 传递 `ev.context` 到 VM。
- `cargo clippy` 8 处警告自动修复（`useless_format!` → `.to_string()`，`manual_range_contains` → `(32..=126).contains(&b)`）。

### Changed
- `TypeChecker` now uses `#[derive(Default)]`; `TypeChecker::new()` removed.
- Temp test files (`temp_nested_struct_test.rs`, `temp_ptr_array_test.rs`, `tmp_struct_copy_test.rs`) merged or removed; tests consolidated into `end_to_end_extra_test.rs`.
- `CODE_REVIEW_REPORT.md` updated to reflect actual fix status.
- Lexer internal representation changed from `source: String` (byte-indexed) to `chars: Vec<char>` (char-indexed), making `peek()` and `advance()` O(1) instead of O(n).
- `merge_free_list()` extracted in `host_funcs.rs` to eliminate ~20 lines of duplication between `host_free` and `host_realloc`.
- `push_one()` extracted in `compile_pipeline.rs` to eliminate ~100 lines of duplication between `push_diagnostics` / `push_warnings` / `push_hints`.
- `parse_declarator()` extracted in `parser.rs` to share declarator parsing between `parse_type_and_name()` and comma-separated extra variables.

## [0.1.0] - 2026-05-14

### Added
- **Full C subset compiler pipeline** (Lexer → Parser → TypeChecker → BytecodeGen → CideVM).
- **Float type support** across the entire pipeline (Lexer/Parser/TypeChecker/BytecodeGen/VM).
- **Host functions**: `printf`, `scanf`, `malloc`, `free`, `realloc`, `strlen`, `strcpy`, `strcmp`, `strcat`, `memset`, `getchar`, `putchar`, `rand`, `srand`, `atoi`, `exit`, `fprintf`, `qsort`.
- **C language features**: `struct`/`typedef struct`, `enum`, arrays (multi-dimensional), pointers (arithmetic, dereference, cast), `#define` macros, function forward declarations, `sizeof`, explicit casts, compound assignments (`+=`, `-=`, etc.), ternary operator, bitwise operators (`& | ^ ~ << >>`).
- ** pedagogical diagnostics**: Chinese error messages with emoji, fix suggestions, error catalog with explanations.
- **Algorithm visualization**: Bubble sort, selection sort, insertion sort, quick sort, merge sort, binary search detection with visual event hooks.
- **Memory map visualization**: 1MB VM memory grid with color-coded regions.
- **Flutter frontend**: IDE screen with `re_editor`, console, variable watch, step debugging, algorithm animation panel.
- **Session save/load**: `serde_json`-based snapshot of compile/runtime/memory state.
- **CI/CD**: GitHub Actions workflow for Rust build/test/clippy + C# build/test.

### Fixed
- Parser zero-progress deadlocks (`struct*`, `ParseBlock`, `parse_case_stmt`).
- VM security hardening: u32 overflow on addr arithmetic, step_count overflow, heap limit closure capture, jump target bounds, value stack limits.
- `char` array initialization using `StoreMemByte` instead of `StoreLocal`.
- Implicit cast hint system with severity levels (warning vs hint).
- UTF-8 safety in Lexer (`chars().nth()` instead of `as_bytes()[i] as char`).
- `printf`/`fprintf` format modifiers (`%6d`, `%.2f`, `%ld`) no longer cause stack unbalance.
- Comma-separated multi-variable array declarations (`int a[10], b[20];`) now preserve per-variable dimensions.
- `unsigned char` no longer incorrectly mapped to `unsigned int`.
- `cide_get_runtime_error` dangling pointer: now uses buffer snapshot pattern.
- `call_user_function` return_ip uses `HOST_CALLBACK_SENTINEL` instead of `code.len()`.
- `session.rs` removed misleading `#![forbid(unsafe_code)]`.
- `host_realloc` in-place shrink when old block is at heap boundary.
- `host_qsort` recursion depth limited to `MAX_QSORT_DEPTH = 8`, preventing stack overflow from indirect recursive qsort calls.
- `host_scanf` `%c` no longer skips whitespace (matches standard C semantics).
- `compute_stride` zero-dimension fallback fixed: `dims[i] == 0` now produces stride 0 instead of 1.
- Algorithm validation regex no longer matches `int main(` inside string literals or comments.
- `flutter_riverpod` upgraded from `^3.3.2-dev.2` to stable `^3.3.1`.
- **多维数组初始化回归**：`bytecode_gen.rs` 中 `InitList` 处理在 `elements` 数量少于 `count` 时（如 `{{1,2,3},{4,5,6}}` 的顶层只有两个内层列表，总元素为6），`else` 分支错误 push `0` 而非 `values[i]`，导致数组元素全零。

### Changed
- `host_memset` now uses slice `.fill()` instead of per-byte `store_i8` for large blocks.
- `host_realloc` supports in-place shrink when the old block is at heap boundary.
- `RuntimeState::output()` replaces 13 repeated `output_lines.join("\n")` calls in `flutter_bridge.rs`.
- `TrapBounds` VM instruction now performs full bounds check in a single instruction (was ~15 instructions via manual `Ge`/`Lt`/`JumpIfZero` chain). `gen_index` bytecode shrunk by ~73%.
- `host_memset` now uses slice `.fill()` instead of per-byte `store_i8` for large blocks.

### Refactored
- `Expr::loc()`/`ty()`/`set_ty()` deduplicated with `macro_rules! expr_field!`.
- `merge_free_list()` extracted to eliminate duplication between `host_free` and `host_realloc`.
- `push_one()` unifies `push_diagnostics`/`push_warnings`/`push_hints`.
- `TypeChecker::visit_call()` split into 19 `check_builtin_xxx()` methods + `check_user_func()`.
- `format_type()` in `capi/mod.rs` removed; uses `Type::to_string()` instead.
- FRB duplicate data structures unified: `VisEvent`/`AlgorithmMatch`/`CompileResult`/`RunResult`/`StepResult`/`StepStatus` now single-source in `session.rs`, re-exported by `api/cide.rs`.
- `OpCode::from_u8` auto-generated via `define_opcode!` macro, eliminating manual repr/match maintenance.
- `Lexer::new` takes `&str` instead of `String`, removing `.to_string()` clones in compile pipeline and all tests.
- `flutter_bridge.rs` breakpoint API batchified: `setBreakpoints(Vec<i32>)` replaces N+1 FFI calls.
- `api/cide.rs` now re-exports FRB types from `session.rs`, eliminating duplicate struct definitions between `flutter_bridge.rs` and `api/cide.rs`.

### Security
- `compile_pipeline.rs` unsafe string write bounds validated.
- C API naked pointers documented with lifetime contracts.

---

## Migration History

- **Phase 0** (2025-10): Rust skeleton + C API stubs.
- **Phase 1** (2025-10): VM migration (CideVM + host functions).
- **Phase 2** (2025-11): Compiler frontend migration (Lexer/Parser/TypeChecker/BytecodeGen).
- **Phase 3–5** (2025-11): C# frontend E2E tests, Android builds, C++/CMake cleanup.
- **Phase 6–8** (2025-12–2026-01): Warning cleanup, float support, diagnostic system expansion.
- **Phase 9–10** (2026-02–2026-05): Flutter frontend from scratch, memory canvas, algorithm visualization FRB integration.
