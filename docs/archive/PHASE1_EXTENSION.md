# Phase 1 核心语法扩展报告

## 概述

本次工作扩展了 C 子集编译器以支持 educoder C 语言程序设计课程（路径 1940）所需的核心语法，包括 `do...while`、`break`、`continue`、`char` 类型、`sizeof`、`switch/case`、`typedef`，并添加了用户隐式类型转换警告机制。

---

## 已完成特性

### 1. `do...while` 循环
- **Parser**: `ParseDoWhileStmt()` 解析 `do stmt while (expr);`
- **AST**: `DoWhileStmt` 节点（`StmtKind::DoWhile`）
- **TypeChecker**: `VisitDoWhile` 检查 body 和 condition
- **CodeGen**: `VisitDoWhile` 生成 `block` + `loop` + body + condition + `br_if 0`

### 2. `break` / `continue`
- **Parser**: `ParseBreakStmt()` / `ParseContinueStmt()`
- **AST**: `BreakStmt` / `ContinueStmt` 节点
- **TypeChecker**: 检查 `loopDepth_ > 0`（continue）或 `loopDepth_ > 0 || switchDepth_ > 0`（break）
- **CodeGen**: 引入 `breakStack_` 和 `loopStack_` 分别追踪 break/continue 目标
  - `break`: `br` 到最近的 break-target block end
  - `continue`: `br` 到最近的 loop start

### 3. `char` 类型
- **Lexer**: `TokenType::Char` 关键字
- **Parser**: `IsTypeToken()` 支持 `char`
- **TypeChecker**: `IsInt()` 包含 `Int` 和 `Char`
- **CodeGen**: 所有标量按 4 字节处理（`i32` 模拟）

### 4. `sizeof`
- **Lexer**: `TokenType::Sizeof` 关键字
- **Parser**: `ParseSizeof()` 支持 `sizeof(type)` 和 `sizeof expr`
- **AST**: `SizeofExpr` 节点（`isTypeQuery` + `targetType` / `operand`）
- **TypeChecker**: `VisitSizeof` 设置 `type = Int`
- **CodeGen**: `VisitSizeof` 编译期计算大小，生成 `i32.const size`

### 5. `switch` / `case` / `default`
- **Lexer**: `Switch`, `Case`, `Default` 关键字，`:` 对应 `Colon`
- **Parser**: `ParseSwitchStmt()` / `ParseCaseStmt()`，`case` body 支持多语句
- **AST**: `SwitchStmt` + `CaseStmt` 节点
- **TypeChecker**: `VisitSwitch` 检查 cond 为整数，`VisitCase` 检查 label 为整数常量
- **CodeGen**: `VisitSwitch` 生成嵌套 `block` 结构，支持 fall-through 语义
  - 源代码顺序：case1 → case2 → default
  - block 嵌套（从内到外）：case1 → case2 → default → break
  - `br_if i` 跳到对应 case block end，执行 case body

### 6. `typedef`
- **Lexer**: `TokenType::Typedef` 关键字
- **Parser**: `ParseTypedef()` 解析 `typedef type name;`，维护 `typedefNames_` 映射
- `ParseBaseType()` 和 `IsTypeToken()` 支持识别 typedef 名称
- **TypeChecker/CodeGen**: 无额外修改（Parser 已将别名解析为实际类型）

### 7. `enum`
- **Lexer**: `Enum` 关键字
- **Parser**: `ParseEnumDecl()` 解析 `enum Name { A, B=5, C };`，将枚举常量作为全局 `int` 变量加入 `ProgramNode`
- enum 名称自动注册为 `typedef` 别名（映射到 `int`）
- **TypeChecker/CodeGen**: 无额外修改（enum 常量即 `int`）

### 8. `unsigned`
- **Lexer**: `Unsigned` 关键字
- **Parser**: `ParseBaseType()` 支持 `unsigned` 和 `unsigned int`，映射为 `TypeKind::Int`
- **TypeChecker/CodeGen**: 无额外修改（与 `int` 同样处理）

### 9. 隐式类型转换警告
- **TypeChecker**: `warnings_` 列表 + `ReportWarning()`
- `IsAssignable()` 在允许隐式转换时（如 `int`→`pointer`、`int`→`char`）生成提示
- **C API**: `cide_diagnostic_get` 支持读取 warning 的 `fixSuggestion`

---

## 关键 Bug 修复

### Bug 1: TypeChecker::VisitBlock 缺少新语句分支
**根因**: `VisitBlock` 使用手写 `switch`，未包含 `DoWhile`、`Break`、`Continue` case，导致 do-while 体内的语句完全未经类型检查。
**影响**: 赋值表达式 `type` 保持默认 `Void`，`VisitExprStmt` 未 emit `drop`，WASM stack 不平衡。
**修复**: 将 `VisitBlock` 改为调用 `DispatchStmt(*stmt)`。

### Bug 2: VisitIf 未跟踪 labelDepth
**根因**: `OP_IF` 是 WASM block 结构，但 `VisitIf` 未增加/减少 `ctx_->labelDepth`。
**影响**: 嵌套在 `if` 内的 `break` 计算错误（`br 1` 跳到 loop start 而非 block end），导致无限循环。
**修复**: `VisitIf` 中 `OP_IF` 后 `labelDepth++`，`OP_END` 前 `labelDepth--`。

### Bug 3: continue 的 label 公式错误
**根因**: `label = ctx_->labelDepth - loopStack_.back()` 计算出的 relative depth 过大。
**修复**: 改为 `label = ctx_->labelDepth - (loopStack_.back() + 2)`。

### Bug 4: 函数体双 END
**根因**: `Generate()` 和 `Build()` 各自在函数体末尾添加 `OP_END`。
**影响**: `return` 后有两个 `end`，wasm3 在 block/loop 嵌套下验证失败。
**修复**: 移除 `Generate()` 中的多余 `EmitOp(OP_END)`。

### Bug 5: Switch case body 与 block 不匹配
**根因**: `VisitSwitch` 中 `OP_END` 关闭的是最内层 block，但生成的 body 是按 cases 逆序，导致 case1 block end 之后执行了 case2 body。
**修复**: 关闭 block 时按源代码正序生成 body（`for (size_t i = 0; i < cases.size(); i++)`）。

---

## 回归测试结果

| 测试套件 | 结果 |
|---------|------|
| Phase 2 Regression | 5/5 passed |
| Batch 1 (内存操作) | 9/9 passed |
| Batch 2 (运行时错误) | 2/3 passed（除零乱码为已知终端编码问题）|
| Batch 3 (Memory View) | 1/1 passed |
| Batch 4 (printf/scanf) | 6/6 passed |
| New Features | 17/17 passed |

---

## 新特性测试详情

| 测试用例 | 说明 | 状态 |
|---------|------|------|
| char_basic | `char c = 65; return c;` | ✅ |
| char_int_conv | `char` ↔ `int` 隐式转换 | ✅ |
| dowhile_basic | 简单 `do...while` 累加 | ✅ |
| break_basic | `while(1)` + `break` | ✅ |
| continue_basic | `while` + `continue` | ✅ |
| break_for | `for` + `break` | ✅ |
| dowhile_break | `do...while` + `break` | ✅ |
| sizeof_int | `sizeof(int)` | ✅ |
| sizeof_char | `sizeof(char)` | ✅ |
| sizeof_var | `sizeof(a)` | ✅ |
| sizeof_ptr | `sizeof(p)` | ✅ |
| switch_basic | 多 case + default | ✅ |
| switch_default | default 匹配 | ✅ |
| switch_fallthrough | case1 → case2 fall-through | ✅ |
| switch_no_default | 无匹配无 default | ✅ |
| typedef_basic | `typedef int MyInt;` | ✅ |
| typedef_ptr | `typedef int* IntPtr;` | ✅ |
| enum_basic | `enum Color { Red, Green, Blue }; return Green;` | ✅ |
| enum_with_value | `enum { OK=0, Error=1, Warning=2 };` | ✅ |
| enum_var | `Color c = Blue; return c;` | ✅ |
| unsigned_basic | `unsigned a = 5;` | ✅ |
| unsigned_int | `unsigned int a = 10;` | ✅ |
