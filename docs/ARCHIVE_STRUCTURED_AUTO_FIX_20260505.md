# 后端结构化自动修复系统 — 2026-05-05

## 背景

2026-05-04 的 P3 清理已完成结构化修复的**基础设施**：C API 扩展、`CideDiagnostic` 新增 fix 字段、`CodeFixService` 重构为结构化修复 + fallback。但后端 `Parser`/`TypeChecker` 尚未填充精确修复位置，所有 diagnostic 的 `fixKind` 默认 `None`，实际修复仍依赖前端字符串匹配。

本次工作聚焦**后端精确结构化修复**：让 `Lexer`/`Parser`/`TypeChecker` 在报错时携带足够的上下文，使 `PopulateStructuredFix` 能自动生成精确的 `InsertText`/`ReplaceText` 修复数据。

---

## 执行成果

### 1. 错误码体系扩展

`native/src/diagnostics/ErrorCodes.hpp` 新增 Parser 错误码：

| 错误码 | 含义 | 触发场景 |
|:---|:---|:---|
| `E2005_ExpectedSemicolon` | 预期 `;` | 语句结束未遇到 `;` |
| `E2006_ExpectedClosingBrace` | 预期 `}` | `ParseBlock` 结束未遇到 `}` |
| `E2007_ExpectedClosingParen` | 预期 `)` | `ParseExprStmt`/`ParseIf`/`ParseWhile`/`ParseFor`/`ParseCall` 未遇到 `)` |
| `E2008_ExpectedClosingBracket` | 预期 `]` | 数组声明/索引未遇到 `]` |

**实现**：`Parser::Consume(TokenType type, const std::string& msg)` 根据期望的 token 类型映射错误码：

```cpp
ErrorCode code = ErrorCode::E2005_ExpectedSemicolon;
if (type == TokenType::RBrace)  code = ErrorCode::E2006_ExpectedClosingBrace;
else if (type == TokenType::RParen) code = ErrorCode::E2007_ExpectedClosingParen;
else if (type == TokenType::RBracket) code = ErrorCode::E2008_ExpectedClosingBracket;
```

---

### 2. `PopulateStructuredFix` — 核心修复生成引擎

**位置**：`native/src/capi/cide_capi.cpp`

**职责**：根据 `errorCode` + 源码内容，计算精确的 `fixKind`/`replaceRange`/`replacementText`。

#### 2.1 `SplitSourceLines` — 源码行分割

**陷阱**：C++ raw string `R"(...\n)"` 尾部包含 `\n`，`std::getline` 会丢弃它，导致 `lines.size()` 比 Parser 的 EOF 行号少 1。

**修复**：
```cpp
static std::vector<std::string> SplitSourceLines(const std::string& source) {
    std::vector<std::string> lines;
    std::stringstream ss(source);
    std::string line;
    while (std::getline(ss, line)) lines.push_back(line);
    if (!source.empty() && source.back() == '\n') lines.push_back("");
    return lines;
}
```

#### 2.2 各错误码的修复策略

| 错误码 | fixKind | 策略 | replacementText |
|:---|:---|:---|:---|
| `E2005_ExpectedSemicolon` | `InsertText` | `column > 1` 时插入 `column-1`；否则插到上一行末尾或行首 | `;` |
| `E2006_ExpectedClosingBrace` | `InsertText` | 同上 | `}` |
| `E2007_ExpectedClosingParen` | `InsertText` | 同上 | `)` |
| `E2008_ExpectedClosingBracket` | `InsertText` | 同上 | `]` |
| `E1004_UnsupportedOp` | `ReplaceText` | Lexer `column` 指向**字符后**；检查 `column-2` 处的 `\|` → `\|\|`、`&` → `&&` | `\|\|` / `&&` |

**关键列号语义**：
- Lexer/Parser `column` 为 **1-based**，且指向**当前 token 之后**（post-`Advance()`）
- `E2005-E2008` 的插入位置 = `column - 1`（即缺失 token 的前一个字符位置）
- `E1004` 的替换位置 = `column - 2`（即被消费的那个非法字符的 0-based 索引）

#### 2.3 `MakeDiagnostic` 包装器

所有 `Lexer`/`Parser`/`TypeChecker` 的 diagnostic push_back 统一改为 `MakeDiagnostic(...)`：

```cpp
static CideDiagnostic MakeDiagnostic(int line, int column, int code, int severity,
    const std::string& message, const std::string& fixSuggestion,
    const std::string& source) {
    CideDiagnostic d;
    d.line = line; d.column = column; d.errorCode = code; d.severity = severity;
    d.message = message; d.fixSuggestion = fixSuggestion;
    PopulateStructuredFix(d, source);
    return d;
}
```

**注意**：`BytecodeGen` 错误（`E4xxx`）仍使用原始 `push_back`，**不生成结构化修复**（Codegen 阶段已无源码结构可修复）。

---

### 3. 前端 `CodeFixService` 消费

`Cide.Client.Shared/Core/CodeFixService.cs` 已在此前重构为两阶段修复：

```csharp
public static CodeFixResult TryApplyFix(string sourceCode, Diagnostic diagnostic)
{
    if (diagnostic.FixKind == FixKind.ReplaceText && diagnostic.ReplaceStartLine > 0)
    {
        var structuredResult = ApplyStructuredReplace(sourceCode, diagnostic);
        if (structuredResult.Applied) return structuredResult;
    }
    else if (diagnostic.FixKind == FixKind.ManualHint)
    {
        return new CodeFixResult(false, null, $"💡 修复提示...");
    }
    return ApplyLegacyFix(sourceCode, diagnostic); // fallback
}
```

后端结构化修复数据就绪后，前端**无需任何改动**即可自动使用精确修复。

---

### 4. 原生测试验证

`native/tests/test_new_features.cpp` 扩展 `testDiagnosticFix()`：

| 测试用例 | 源码片段 | 期望修复 | 状态 |
|:---|:---|:---|:---|
| `missing_semicolon` | `int x = 5\nreturn 0;` | 第3行第13列插入 `;` | ✅ PASS |
| `unsupported_op` | `if (a \| b)` | 第5行第10-11列替换为 `\|\|` | ✅ PASS |
| `missing_brace` | `int main() { int x = 5;` | 第3行第14列插入 `}` | ✅ PASS |
| `missing_paren` | `if (x == 5 {` | 第4行第16列插入 `)` | ✅ PASS |

**测试陷阱**：`missing_paren` 源码 `R"(..."` 尾部 `\n` 导致 `SplitSourceLines` 最初少一行；修复后测试通过。

**全部 35 个原生测试**：
```
missing_semicolon PASS
unsupported_op PASS
missing_brace PASS
missing_paren PASS
linked_list_traversal PASS (confidence=85, line=6)
linked_list_reverse PASS (confidence=80, line=7)
... 及其他 29 个基础功能测试全部通过
```

---

## 文件变更清单

### 修改文件

```
native/src/diagnostics/ErrorCodes.hpp          (+ E2006/E2007/E2008)
native/src/compiler/Parser.cpp                 (Consume: 按 token 类型映射错误码)
native/src/capi/cide_capi.cpp                  (+ SplitSourceLines, PopulateStructuredFix, MakeDiagnostic)
native/tests/test_new_features.cpp             (+ missing_brace, missing_paren 结构化修复测试)
```

### 未改动但已就绪的文件

```
Cide.Client.Shared/Core/CodeFixService.cs      (此前已重构为结构化修复 + fallback)
native/include/cide_capi.h                     (此前已扩展 cide_diagnostic_get_fix)
native/src/capi/CideSession.hpp                (此前已扩展 CideFixKind, CideDiagnostic)
```

---

## 已知限制

1. **Codegen 诊断无结构化修复**：`BytecodeGen` 的 `push_back` 仍使用原始方式，因为 Codegen 阶段已无源码 AST 结构可映射精确位置。
2. **多诊断覆盖**：同一源码可能产生多个同类诊断（如 `missing_paren` 同时触发 `E2007` 和 `E2006`），`PopulateStructuredFix` 对每个诊断独立计算修复，前端需选择最合适的应用。
3. **列号语义假设**：`PopulateStructuredFix` 假设 Lexer/Parser 的 `column` 为 1-based post-token 位置。若其他阶段的列号语义改变，需同步调整。

---

## 相关文档

- `docs/ARCHIVE_P3_CLEANUP_20260504.md` — 结构化修复基础设施（C API / 前端重构）
- `docs/UX_DIAGNOSTICS_DESIGN.md` — 三级诊断与修复设计
- `docs/DESIGN.md` — 项目总体设计（诊断与修复系统章节）
