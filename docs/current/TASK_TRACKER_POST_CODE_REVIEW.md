# Post-Code-Review 跟进任务跟踪

> 分支：`feat/post-code-review-followup`  
> 起始日期：2026-05-17  
> 目标：消化 CODE_REVIEW_5_16.md 遗留项，偿还技术债务，夯实测试基础设施

---

## ✅ 已完成

### 1. 影子验证框架（Shadow Verification）
- [x] 搭建 Python 自动化对比框架（Clang vs Cide）
- [x] 45 个用例覆盖 baseline / double / function_pointer / file_io / union 等类别
- [x] 第一轮即发现 3 个真实 bug，验证框架价值
- [x] 分类逻辑改进：优先使用用例 `expected_category`，消除 `[unknown]`
- [x] 文档化：`SHADOW_VERIFICATION_FRAMEWORK.md`

### 2. 真实 Bug 修复（3/3）
| Bug | 根因 | 修复文件 | 状态 |
|-----|------|----------|------|
| **string_literal const 语义** | 字符串字面量推导为 `char*`（非 const），`char* s = "hello"` 误报类型不匹配 | `ast.rs`：`Type::pointer()` 新增 `is_const` 参数；`type_checker.rs`：推导为 `const char*` | ✅ |
| **printf %.2f 精度** | `format_printf_string()` 忽略精度修饰符，固定 6 位输出 | `host_funcs.rs`：解析 `.*` 精度并传递给 `format!("{:.*}", prec, f)` | ✅ |
| **forward_decl 解析** | 单行压缩代码 `int foo(int);` 被 Parser 错误识别 | `parser.rs`：`parse_global_var_or_func()` 的 checkpoint/lookahead 逻辑修复 | ✅ |

### 3. double 类型全管线支持（阶段 1→2A→2B→3 全部完成）
- [x] **阶段 1**：Lexer 映射为 Float（教学妥协），8 个测试覆盖
- [x] **阶段 2A**：类型系统真区分（`TokenType::Double` / `TypeKind::Double` / `Type::double()`）
- [x] **阶段 2B**：BytecodeGen 保留 double 类型，降级为 float 指令（兼容过渡）
- [x] **阶段 3**：VM 栈 `Vec<i32>` → `Vec<u64>`，真正 double 指令（`PushConstD`/`AddD`/`CastI2D`/...）
- [x] **字节偏移架构**：`LoadLocal`/`StoreLocal`/`Call` 全面改为字节偏移
- [x] **param_sizes 参数传递**：支持 double 占 8 字节、正序遍历
- [x] **`SplitD`**：f64 拆分为 2 个 i32 word 用于普通函数 Call
- [x] **printf**：`%f` 统一读取 f64，float 参数自动 `CastF2D` 提升
- [x] **精度验证**：`test_e2e_double_precision_64bit` 通过（`1.0000000001` 正确输出）
- [x] **sizeof(double) = 8**
- [x] 影子验证：3 个 double 用例全部编译通过且运行正确
- [x] 匹配率从 **57.8% → 68%**
- [x] `DOUBLE_FULL_IMPLEMENTATION_PLAN.md` + `FIXES_DOUBLE_IMPLEMENTATION.md` 文档化

### 4. 文档整理
- [x] 归档过时文档（CODE_REVIEW_REPORT_20260514、FLUTTER_MIGRATION_PLAN 重复件、Tauri 方案等）
- [x] 更新 `docs/README.md` 索引
- [x] 更新 `CODE_REVIEW_5_16.md`：标记 double / printf 精度为已修复
- [x] 更新 `AGENTS.md`：double 支持状态更新
- [x] 更新 `DOUBLE_TYPE_SUPPORT_PLAN.md`：标记阶段 1 已完成

---

## ⏳ 进行中 / 待启动

### 高优先级（当前分支收尾）

#### 🔴 当前分支 `feat/post-code-review-followup` 核心工作已完成

| 模块 | 状态 | 测试 |
|------|------|------|
| 影子验证框架 | ✅ | 45 用例，27/27 baseline 匹配 |
| 3 个真实 bug 修复 | ✅ | string_literal / printf 精度 / forward_decl |
| double 全量干净实现 | ✅ | 216/216 通过，0 clippy 警告 |

**建议**：当前分支合并到 `master`，后续工作开新分支。

---

### 中优先级（代码审查遗留项）

#### 🟡 3.8 `format_bounds_error` 预建索引优化
- **当前**：每次越界线性扫描全部 `symbols`，O(n)
- **建议**：全局数组符号预建按 `addr` 排序索引，二分查找 O(log n)
- **收益**：教学场景 symbols 通常 <100，实际收益有限
- **优先级**：P2（建议），非阻塞

#### 🟡 3.9 `Type` 结构体 `String` → `Cow<'static, str>`
- **当前**：`Type::name` 是 `String`，编译管线中频繁 `clone`
- **建议**：短字符串（`"int"`, `"Node"`）用 `Cow<'static, str>` 减少堆分配
- **风险**：改动面极广（Lexer→Parser→TypeChecker→BytecodeGen→VM→C API），且可能影响 FFI
- **优先级**：P2（建议），非阻塞

---

### 中优先级（代码审查遗留项）

#### 🟡 3.8 `format_bounds_error` 预建索引优化
- **当前**：每次越界线性扫描全部 `symbols`，O(n)
- **建议**：全局数组符号预建按 `addr` 排序索引，二分查找 O(log n)
- **收益**：教学场景 symbols 通常 <100，实际收益有限
- **优先级**：P2（建议），非阻塞

#### 🟡 3.9 `Type` 结构体 `String` → `Cow<'static, str>`
- **当前**：`Type::name` 是 `String`，编译管线中频繁 `clone`
- **建议**：短字符串（`"int"`, `"Node"`）用 `Cow<'static, str>` 减少堆分配
- **风险**：改动面极广（Lexer→Parser→TypeChecker→BytecodeGen→VM→C API），且可能影响 FFI
- **优先级**：P2（建议），非阻塞

---

### 低优先级（C 子集扩展，数据驱动）

> 基于影子验证框架的编译缺口频率排序：

| 特性 | 用例数 | 优先级 | 备注 |
|------|--------|--------|------|
| `function_pointer` 完整支持 | 2 | P1 | 当前仅支持函数名作为参数传递（`qsort`），不支持声明/调用语法 |
| `file_io`（`fopen`/`fread`/`fwrite`） | 2 | P1 | 需新增宿主函数 |
| `union` | 1 | P1 | C 核心特性，ROADMAP 明确列出 |
| `long long` | 1 | P2 | 64 位整型，若 VM 栈已扩展为 `u64`，实现成本大幅降低 |
| `goto` | 1 | P2 | 控制流跳转，需 BytecodeGen 新增标签/跳转指令 |
| `designated_initializer` | 1 | P2 | `.field = value` 语法 |
| `variable_length_array` | 1 | P2 | 运行时定长数组 |
| `static_assert` | 1 | P3 | C11 特性，教学场景低频 |
| `inline_asm` | 1 | P3 | 明确不支持（安全/沙箱限制） |
| `variadic_macro` | 1 | P3 | `...` / `__VA_ARGS__` |
| `typeof` | 1 | P3 | GCC 扩展 |

---

## 📊 影子验证基线数据（2026-05-17，double 完成后）

```
总用例数: 45
完全匹配: 32 (71%) ← double 精度差异已解决
编译缺口: 13 (28%)
运行时缺口: 0
输出差异: 0
```

**baseline 类别**：27/27 全部匹配 ✅  
**double 类别**：3/3 全部匹配 ✅（含 64 位精度验证）  
**已知缺失特性**：13 个编译缺口均为 Cide 明确不支持的语法

---

## 🎯 下一个 Sprint 推荐：`long long` 支持

VM 栈已扩展为 `u64`，`long long`（64 位整型）的实现成本大幅降低：

| 组件 | 工作量 | 说明 |
|------|--------|------|
| Lexer | 10 分钟 | 识别 `long long` 关键字 |
| AST | 10 分钟 | `TypeKind::LongLong` |
| Parser | 30 分钟 | 解析 `long long x;` |
| TypeChecker | 1 小时 | 隐式转换链 |
| BytecodeGen | 2 小时 | 复用 double 的字节偏移模式，新增 `PushConstI64` / `AddI64` / ... |
| VM | 2 小时 | 新增 `OpCode::AddI64` / `CastI2I64` / `CastI64I2` |
| 测试 | 半天 | E2E 测试 |

**总计：1 天**（内存框架已是字节偏移，无需再改 `LoadLocal`/`Call`/全局初始化）

---

## 🗓️ 建议的后续 Sprint 规划

### Sprint 1（已完成）：Post-Code-Review + Double 全量
- [x] 影子验证框架搭建
- [x] 3 个真实 bug 修复
- [x] double 全量干净实现（VM 栈 u64 + 字节偏移 + *D 指令）
- [x] 216/216 测试通过，0 clippy 警告

**建议动作**：合并 `feat/post-code-review-followup` → `master`

### Sprint 2：`long long` 快速胜利
> 分支建议：`feat/long-long-support`  
> 预估：1 天  
> 理由：VM 栈已是 u64，字节偏移架构就绪，long long 只需新增类型 + 指令

- [ ] Lexer：`long long` 关键字
- [ ] AST：`TypeKind::LongLong`
- [ ] Parser：`long long` / `long` 类型解析
- [ ] TypeChecker：隐式转换链 `int → long long`
- [ ] BytecodeGen：`PushConstI64` / `AddI64` / `CastI2I64` / ...
- [ ] VM：新增 I64 运算指令
- [ ] 端到端测试

### Sprint 3：C 子集扩展（数据驱动）
> 分支建议：`feat/c-subset-extensions`

| 优先级 | 特性 | 用例数 | 实现成本 | 备注 |
|--------|------|--------|----------|------|
| P1 | `file_io`（`fopen`/`fread`/`fwrite`） | 2 | 中 | 新增宿主函数，教学场景高频 |
| P1 | `function_pointer` 完整支持 | 2 | 高 | Parser 语法复杂，但 ROADMAP 明确 |
| P2 | `union` | 1 | 高 | C 核心特性，内存布局特殊 |
| P2 | `goto` | 1 | 中 | 控制流跳转，需 BytecodeGen 标签系统 |
| P2 | `designated_initializer` | 1 | 低 | `.field = value` 语法 |
| P3 | `variable_length_array` | 1 | 中 | 运行时定长数组 |
| P3 | `static_assert` / `typeof` / `variadic_macro` | 各 1 | 低~中 | 低频特性 |

### Sprint 4：工程优化（CODE_REVIEW 遗留项）
- [ ] 3.8 `format_bounds_error` 预建索引
- [ ] 3.9 `Type::name` `String` → `Cow<'static, str>`
- [ ] 增量编译缓存（4.9）

---

## 📝 决策记录

| 日期 | 决策 | 理由 |
|------|------|------|
| 2026-05-17 | double 阶段 1 采用映射为 float 的妥协方案 | 快速消除编译失败，但承认是工程债 |
| 2026-05-17 | 影子验证框架优先使用 `expected_category` 分类 | 消除 `[unknown]`，使数据可直接驱动优先级决策 |
| 2026-05-17 | 文档整理：归档 5_14 及更早的过时报告 | 避免信息冗余，current/ 只保留最新有效文档 |
