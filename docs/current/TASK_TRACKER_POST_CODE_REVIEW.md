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

### 3. double 类型阶段 1（最小实现）
- [x] Lexer：`"double"` 关键字映射为 `TokenType::Float`（教学宽松模式）
- [x] 8 个端到端测试覆盖：基本运算、数组、printf 精度、函数参数/返回值、隐式转换、比较、强制转换、复合赋值
- [x] 影子验证：3 个 double 用例全部编译通过（2 个输出匹配，1 个有 float/double 精度差异为预期行为）
- [x] 匹配率从 **57.8% → 68%**

### 4. 文档整理
- [x] 归档过时文档（CODE_REVIEW_REPORT_20260514、FLUTTER_MIGRATION_PLAN 重复件、Tauri 方案等）
- [x] 更新 `docs/README.md` 索引
- [x] 更新 `CODE_REVIEW_5_16.md`：标记 double / printf 精度为已修复
- [x] 更新 `AGENTS.md`：double 支持状态更新
- [x] 更新 `DOUBLE_TYPE_SUPPORT_PLAN.md`：标记阶段 1 已完成

---

## ⏳ 进行中 / 待启动

### 高优先级（工程债偿还）

#### 🔴 double 类型全管线支持（阶段 2A → 2B → 3）
> **当前状态**：阶段 1 已完成（映射为 float），但 `sizeof(double)=4`、精度 7 位，与标准 C 语义不符，欠下工程债。
>
> **方案**：分阶段还款（见 `DOUBLE_TYPE_SUPPORT_PLAN.md`）

| 阶段 | 目标 | 预估工作量 | 风险 | 前置条件 |
|------|------|------------|------|----------|
| **2A** | Lexer/Parser/AST/TypeChecker 真正区分 `double`（`TypeKind::Double`） | ~4h | 低 | 无 |
| **2B** | BytecodeGen 保留 double 类型信息，但继续生成 float 指令（兼容现有 VM） | ~2h | 低 | 2A 完成 |
| **3** | VM 栈 `Vec<i32>` → `Vec<u64>` + 新增 `PushConstD`/`AddD`/... 指令 | ~2d | **高** | 2B 完成 |

**阶段 2A 具体任务**：
- [ ] Lexer：新增 `TokenType::Double`，`"double"` 不再映射为 `Float`
- [ ] AST：`TypeKind` 新增 `Double`，`Type::double()` 构造函数
- [ ] Parser：`parse_base_type()` 新增 `Double` 分支；`is_type_token()` 包含 `Double`
- [ ] TypeChecker：
  - [ ] `is_scalar()` 加入 `Double`
  - [ ] 隐式转换链扩展：**char → int → float → double**
  - [ ] `double` → `float`/`int`/`char`：允许，warning 提示精度丢失
  - [ ] 二元运算类型提升：有 `double` → 结果为 `double`
  - [ ] `printf`/`scanf` 支持 `%lf` 格式符

**阶段 3 核心风险**：
- VM 栈改为 `u64` 后，**所有现有指令**（`Add`/`LoadMem`/`Call`/`Return` 等）的 push/pop 需适配
- `Call`/`Return` 栈帧偏移计算从 4 字节 → 8 字节
- `frb_generated.rs` 可能需重新生成（若 `Instruction` 或 `CideVM` 结构变化）
- **缓解**：每改一组指令后立即 `cargo test`，确保无回归

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

## 📊 影子验证基线数据

```
总用例数: 45
完全匹配: 31 (68%) ← 目标：> 90%
编译缺口: 13 (28%)
运行时缺口: 0
输出差异: 1 (double_basic 的 float/double 精度差异)
```

**baseline 类别**：27/27 全部匹配 ✅  
**已知缺失特性**：13 个编译缺口均为 Cide 明确不支持的语法

---

## 🗓️ 建议的后续 Sprint 规划

### Sprint 1（当前）：Post-Code-Review 收尾
- [ ] double 阶段 2A/2B（类型系统真区分）
- [ ] 3.8 / 3.9 优化项（可选，视时间余量）

### Sprint 2：VM 栈扩展 + double 阶段 3
- [ ] VM 栈 `Vec<i32>` → `Vec<u64>`
- [ ] 现有指令适配 + 新增 double 指令
- [ ] 端到端测试全覆盖

### Sprint 3：C 子集扩展（数据驱动）
- [ ] function_pointer 完整支持（2 个用例，ROI 最高）
- [ ] file_io 宿主函数（2 个用例）
- [ ] union 支持（1 个用例，C 核心特性）

---

## 📝 决策记录

| 日期 | 决策 | 理由 |
|------|------|------|
| 2026-05-17 | double 阶段 1 采用映射为 float 的妥协方案 | 快速消除编译失败，但承认是工程债 |
| 2026-05-17 | 影子验证框架优先使用 `expected_category` 分类 | 消除 `[unknown]`，使数据可直接驱动优先级决策 |
| 2026-05-17 | 文档整理：归档 5_14 及更早的过时报告 | 避免信息冗余，current/ 只保留最新有效文档 |
