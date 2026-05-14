# 事故报告：Parser 误判 `struct*` 返回类型导致无限循环内存泄漏

**日期**：2026-04-27  
**影响**：`test_new_features.exe` / `LinkedListReverseTest.exe` 运行时占满 ~18GB 内存，进程被超时杀死  
**状态**：已修复，已加固  

---

## 1. 事故摘要

在添加**链表反转算法检测**（`DetectLinkedListReverse`）测试用例时，发现编译以下代码会导致测试进程 hang 死并持续泄漏内存：

```c
struct Node { int val; struct Node* next; };

struct Node* reverse(struct Node* head) {
    struct Node* prev = 0;
    struct Node* curr = head;
    while (curr) {
        struct Node* next = curr->next;
        curr->next = prev;
        prev = curr;
        curr = next;
    }
    return prev;
}

int main() { return 0; }
```

| 进程 | 占用内存 | 状态 |
|------|---------|------|
| `test_new_features.exe` | ~18 GB | 被 timeout kill |
| `LinkedListReverseTest.exe` | ~18 GB | 被 timeout kill |

最小复现代码（仅需一行）：
```c
struct Node { int val; struct Node* next; };
struct Node* foo(struct Node* x) { return x; }
int main() { return 0; }
```

---

## 2. 根因分析

### 2.1 直接原因：Parser 死循环

`ParseProgram()` 中处理顶层声明的分支逻辑：

```cpp
while (!IsAtEnd()) {
    if (Check(TokenType::Struct)) {
        program->structs.push_back(ParseStructDecl());  // ❌
    } else if (IsTypeToken()) {
        // 区分函数 vs 全局变量...
    }
}
```

当 Parser 遇到 `struct Node* reverse(...)` 时：
1. `Check(TokenType::Struct)` → **true**（当前 token 确实是 `struct`）
2. 错误地调用 `ParseStructDecl()`，将其当作**结构体声明**处理

### 2.2 ParseStructDecl 的无限循环

```cpp
StructDecl Parser::ParseStructDecl() {
    Consume(TokenType::Struct, "...");
    auto nameTok = Consume(TokenType::Identifier, "...");
    Consume(TokenType::LBrace, "预期 '{'");   // ❌ 失败，当前 token 是 *

    while (!Check(TokenType::RBrace) && !IsAtEnd()) {
        auto [ftype, fname] = ParseTypeAndName();     // 解析 * reverse 等无效内容
        Consume(TokenType::Semicolon, "预期 ';'");    // 持续失败但不消耗 token
        decl.fields.push_back({ftype, fname});        // 每次循环都分配内存
    }
    // ...
}
```

关键问题：**`Consume()` 失败时不消耗任何 token**，当前位置永远卡在 `(`：

| 循环轮次 | 当前 token | 操作 | token 是否前进 |
|---------|-----------|------|--------------|
| 1 | `*` | `ParseTypeAndName()` 解析出 `{Pointer, "foo"}` | ✅ 消耗 `* foo` |
| 2 | `(` | `ParseTypeAndName()` 无法解析，错误返回 | ❌ 不前进 |
| 3 | `(` | 同上 | ❌ 不前进 |
| ... | `(` | 同上 | ❌ 不前进 |

`IsAtEnd()` 永远为 false，`RBrace` 永远不会出现 → **无限循环 + 持续 `push_back`** → 内存泄漏至 18GB。

### 2.3 深层原因

- **顶层分支逻辑过于简单**：仅通过 `Check(TokenType::Struct)` 判断结构体声明，未考虑 `struct Type* func(...)` 这种函数返回类型场景
- **缺少死循环保护**：`ParseStructDecl` 的字段解析循环没有检测"零进度"情况
- **Consume 的容错设计**：失败时不抛异常也不前进，完全依赖上层逻辑保证进展

---

## 3. 修复措施

### 3.1 ParseProgram：正确区分结构体声明 vs 函数/变量声明

通过 **peek ahead** 检查 `struct Name` 后的 token 是否为 `{`：

```cpp
} else if (Check(TokenType::Struct)) {
    auto checkpoint = pos_;
    Advance(); // consume 'struct'
    Consume(TokenType::Identifier, "预期结构体名称");
    bool isStructDecl = Check(TokenType::LBrace);
    pos_ = checkpoint;  // 回退

    if (isStructDecl) {
        program->structs.push_back(ParseStructDecl());
    } else {
        // struct 返回类型的函数或全局变量
        auto [type, name] = ParseTypeAndName();
        auto nameTok = Previous();
        if (Check(TokenType::LParen)) {
            pos_ = checkpoint;
            program->funcs.push_back(ParseFuncDecl());
        } else {
            // 全局变量声明...
        }
    }
}
```

### 3.2 ParseStructDecl：添加死循环保护（防御性编程）

```cpp
while (!Check(TokenType::RBrace) && !IsAtEnd()) {
    auto fieldCheckpoint = pos_;
    auto [ftype, fname] = ParseTypeAndName();
    if (pos_ == fieldCheckpoint) {
        // 字段解析零进度，跳过 token 避免死循环
        Advance();
        break;
    }
    Consume(TokenType::Semicolon, "预期 ';'");
    decl.fields.push_back({ftype, fname});
}
```

---

## 4. 验证结果

### 4.1 独立测试

```
LinkedListReverseTest.exe
TEST1 compiling...
TEST1 result=0
TEST1 PASS          <-- struct Node* 返回类型编译通过
TEST2 compiling...
TEST2 result=0
TEST2 PASS          <-- 完整链表反转代码编译通过
```

### 4.2 完整回归测试

```
test_new_features.exe
...
linked_list_traversal PASS (confidence=85, line=6)
linked_list_reverse PASS (confidence=80, line=7)  <-- 新增检测正常工作
```

### 4.3 CMake 全量回归

| 测试套件 | 结果 |
|---------|------|
| Phase2Regression | ✅ Passed |
| Phase3Batch1 | ✅ Passed |
| Phase3Batch2 | ✅ Passed |
| Phase3Batch3 | ✅ Passed |
| Phase3Batch4 | ✅ Passed |
| Phase3Step | ✅ Passed |
| Stage2Diagnostic | ✅ Passed |

**7/7 全部通过，零回归。**

---

## 5. 后续建议

1. **Parser 分支逻辑统一审查**：检查其他类似 `Check(TokenType::Xxx)` 的分支是否也存在 peek 不足的问题（如 `enum`、`typedef` 等）
2. **所有循环解析添加零进度保护**：`while (!IsAtEnd())` 类循环应统一检测 pos 是否前进
3. **测试覆盖**：在 Parser 单元测试中增加 "struct 指针返回类型" 的用例，防止回归
