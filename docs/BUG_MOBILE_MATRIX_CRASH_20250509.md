# 移动端矩阵代码运行崩溃问题记录

## 基本信息
- **日期**: 2026-05-09
- **测试设备**: Android, 16GB+12GB 内存 / Windows Desktop
- **应用**: Cide.Client.Maui (MAUI Blazor Hybrid) / Cide.Client.Desktop (Avalonia)
- **触发代码**: 包含二维数组的 C 代码

## 症状对比

### 模板代码（正常运行）
```c
int main() {
    int sum = 0;
    for (int i = 1; i <= 5; i = i + 1) {
        sum = sum + i;
    }
    printf("%d", sum);
    return sum;
}
```
- 点击运行：红色停止按钮**一闪而过**
- 点击下一步：正常执行
- 输出面板：正常显示结果

### 用户矩阵代码（崩溃）
```c
#include<stdio.h>
void transMatrix(int matrix[][3])
{
    for (int i = 0; i < 3; i++)
    {
        for (int j = i+1; j < 3; j++)
        {
            int temp = matrix[i][j];
            matrix[i][j] = matrix[j][i];
            matrix[j][i] = temp;
        }
    }
}
void printMatrix(int matrix[][3])
{
    for (int i = 0; i < 3; i++)
    {
        for (int j = 0; j < 3; j++)
        {
            printf("%d ", matrix[i][j]);
        }
        printf("\n");
    }
}
int main()
{
    int date[3][3] = { {1,2,3},{4,5,6},{7,8,9} };
    transMatrix(date);
    printMatrix(date);
}
```
- 点击运行：红色停止按钮**永久固化**（IsRunning = true 后未恢复）
- 输出面板：**无任何输出**
- 随后：应用**自动闪退**
- 系统弹窗：**.NET 运行内存异常**，应用被强制关闭

## 已尝试的修复（未解决问题）

### 1. JS 层资源清理
- `codemirror-interop.js`: 添加 `destroy(id)` 清理 MutationObserver / scroll / resize 监听器
- `CodeMirrorEditor.razor`: `DisposeAsync()` 中调用 JS destroy
- **结果**: 问题依旧

### 2. CTS 释放修复
- `MainViewModel.cs`: 修复 `_runCts` 和 `_flashCts` 的 Cancel/Dispose 逻辑
- `FinishExecution()` / `StopExecution()` 中增加 Dispose
- **结果**: 问题依旧

### 3. ConsoleOutput 大小限制
- 添加 `TruncateOutput()`，限制最大 50KB
- **结果**: 问题依旧

### 4. 后台线程执行
- 将 `EnsureCompiled()` 和 `RunFullSpeed()` 放到 `Task.Run` 中执行
- UI 更新通过 `MainThread.InvokeOnMainThreadAsync` 调度
- **结果**: UI 不再卡死（主线程未阻塞），但后台线程崩溃导致应用闪退

## 关键发现：编译器不支持多维数组

### Parser 层面
`Parser::ParseTypeAndName()` 只处理**一层** `[]`：
```cpp
if (Match(TokenType::LBracket)) {
    if (Check(TokenType::Number)) {
        auto sizeTok = Advance();
        int size = std::atoi(sizeTok.text.c_str());
        Consume(TokenType::RBracket, "预期 ']'");
        return {Type{TypeKind::Array, baseType.name, size, baseType.kind}, nameTok.text};
    }
    // ...
}
```
- `int date[3][3]` → 被解析为 `int date[3]`，后面的 `[3]` 未被消费
- `int matrix[][3]` → 同样只解析 `int matrix[]`

### TypeChecker 层面
即使声明被部分解析，`matrix[i][j]` 也会失败：
- `matrix[i]` → TypeChecker 返回 `int`
- `int[j]` → TypeChecker 报错："只有数组和指针才能用 [] 索引"

### 表达式层面
`ParsePostfix()` 支持连续索引（`date[i][j]` 解析为 `IndexExpr(IndexExpr(date, i), j)`），但类型检查不通过。

## 异常分析

### 核心矛盾
按正常逻辑，用户代码应该**编译失败**，然后 `PresentCompileError()` 应该在输出面板显示错误信息。但实际结果是：
- **输出面板为空**
- **应用直接闪退**

这说明崩溃发生在 **native 层**（C++ 编译器/VM），导致整个进程被操作系统终止，C# 的 `try/catch` 无法捕获。

### 可能崩溃点（待桌面端验证）

#### 假设 A: 编译阶段崩溃
`cide_compile_all()` 在处理 Parser 错误时，可能访问了无效内存：
- `MakeDiagnostic()` → `PopulateStructuredFix()` → `SplitSourceLines()` 处理大字符串
- `CheckArrayInitializer()` 在处理不合法的 InitListExpr 时越界
- `std::make_move_iterator(ast->structs.begin())` 如果 ast 异常但 HasErrors() 未正确检测

#### 假设 B: 运行阶段崩溃
如果编译器在某些错误情况下仍返回了 `compiled = true`：
- VM 执行 `CallHost` 时 stack 参数不匹配（`printf` 参数错误）
- `Call` 指令的 `frameSize` 计算错误导致栈溢出
- `RetVoid` 从空 callStack 返回（未定义函数调用）

#### 假设 C: P/Invoke 缓冲区溢出
`CompilerService.GetCompileErrors()` 分配固定 4096 字节 buffer，如果错误信息超过此长度，`cide_get_compile_errors_buf` 会截断，不会溢出。但需验证。

## 桌面端测试计划

### 测试环境
- Windows 桌面端（Cide.Client.Desktop）
- Visual Studio 2022，启用 **Native Code Debugging**
- 或者使用 WinDbg 捕获崩溃 dump

### 测试步骤

#### 测试 1：确认编译是否成功
1. 在桌面端运行应用
2. 输入用户矩阵代码
3. 点击运行
4. 观察：
   - 是否显示编译错误？（预期：应显示多维数组不支持错误）
   - 还是直接崩溃？

#### 测试 2：Native 调试定位崩溃点
1. VS → 项目属性 → 调试 → 启用本机代码调试
2. 在以下位置设置断点：
   - `native/src/capi/cide_capi.cpp: cide_compile_all()`
   - `native/src/capi/cide_capi.cpp: cide_run()`
   - `native/src/compiler/TypeChecker.cpp: CheckArrayInitializer()`
   - `native/src/vm/CideVM.cpp: Step() / CallHost`
3. 逐步执行，观察在哪个调用点触发 Access Violation

#### 测试 3：验证 InitList 解析
1. 单步跟踪 `Parser::ParseInitList()` 处理 `{ {1,2,3}, {4,5,6}, {7,8,9} }`
2. 观察内层 `{1,2,3}` 是否被正确消费还是导致解析混乱
3. 观察 `CheckArrayInitializer` 接收到怎样的 InitListExpr

#### 测试 4：简化代码测试
逐步简化用户代码，找到最小崩溃复现：
```c
// 测试 4a: 最简二维数组声明
int main() {
    int date[3][3];
    return 0;
}

// 测试 4b: 二维数组参数
void f(int m[][3]) {}
int main() {
    f(0);
    return 0;
}

// 测试 4c: 一维数组（确认基础功能正常）
int main() {
    int arr[3] = {1,2,3};
    printf("%d", arr[0]);
    return 0;
}
```

### 预期结果记录表

| 测试项 | 预期结果 | 实际结果 | 崩溃点 |
|--------|---------|---------|--------|
| 模板代码 | 正常运行 | 待记录 | - |
| 最简 2D 数组声明 | 编译错误 | 待记录 | 待记录 |
| 2D 数组参数 | 编译错误 | 待记录 | 待记录 |
| 用户完整代码 | 编译错误/崩溃 | 待记录 | 待记录 |
| 一维数组初始化 | 正常运行 | 待记录 | - |

---

## 根因确认（2026-05-09 补充）

### 桌面端 Native 调试结果
- 在 `cide_compile_all()` 入口命中断点
- 按 F10 单步跟踪几次后**卡住**，说明问题在**编译阶段**
- 任务管理器显示进程内存约 **21GB**，且是**缓慢增长**（约两分钟以上达到 20GB）

### 内存快照分析
- **托管内存（.NET）增长量很小**：仅几百 KB（Avalonia UI 对象正常增长）
- **泄漏绝大部分发生在非托管/本机内存（Native Heap）**
- 确认泄漏源在 C++ native 后端（Parser）

### 死循环根因：`ParseBlock()` 缺少零进度保护

当函数体内出现 `int date[3][3];` 时：

1. `ParseVarDeclStmt()` 调用 `ParseTypeAndName()` 消费了 `int date[3]`
2. 当前 token 停在第二个 `[`
3. `Consume(Semicolon)` 在 `[` 处失败，**不消费 token**
4. 回到 `ParseBlock()` 的 `while` 循环，当前 token 仍是 `[`
5. `ParseStatement()` → `IsTypeToken()` 为 false → `ParseExprStmt()`
6. `ParseExpression()` → ... → `ParsePrimary()` 在 `[` 上：
   - 不是 Number、String、Identifier、LParen
   - 报错"预期表达式"，返回 `LiteralExpr(0)`，**不消费 token**
7. `ParseExprStmt()` 的 `Consume(Semicolon)` 再次失败，**不消费 token**
8. **死循环**：`ParseBlock()` 的 `while` 循环永远在同一个 `[` token 上打转

每次循环都分配新的 AST 节点（`LiteralExpr`、`ExprStmt`）和错误消息，`block->stmts` 不断 `push_back`，native 内存持续增长至 21GB。

> `ParseStructDecl()` 在 2026-04-27 事故后已添加零进度保护（`pos_ == fieldCheckpoint` 时 `Advance()`），但 `ParseBlock()` 遗漏了同样的保护。

### 修复措施

在 `native/src/compiler/Parser.cpp` 的 `ParseBlock()` 中添加零进度保护：

```cpp
StmtPtr Parser::ParseBlock() {
    Consume(TokenType::LBrace, "预期 '{'");
    auto block = std::make_unique<BlockStmt>();
    while (!Check(TokenType::RBrace) && !IsAtEnd()) {
        auto stmtCheckpoint = pos_;
        block->stmts.push_back(ParseStatement());
        if (pos_ == stmtCheckpoint) {
            // Failed to parse a statement, skip one token to avoid infinite loop
            Advance();
        }
    }
    Consume(TokenType::RBrace, "预期 '}'");
    return block;
}
```

### 修复后预期行为
- 矩阵代码不再触发死循环和内存泄漏
- 编译器会正常返回编译错误（如"预期 ';'"）
- 应用正常显示错误信息，不再闪退

---

## 修复优先级

### P0：Parser 死循环修复 ✅（已完成）
- `ParseBlock()` 添加零进度保护
- 所有 `while (!IsAtEnd())` 类循环应统一检测 pos 是否前进

### P1：支持多维数组（进行中）
- 修改 `Type` 结构支持多维尺寸（`std::vector<int> dims`）
- 修改 `ParseTypeAndName` 循环解析多个 `[]`
- 修改 TypeChecker 处理多维索引（`date[i]` 返回子数组类型）
- 修改 BytecodeGen 生成线性偏移计算

### P2：支持 #define 宏
- 添加 Preprocessor 阶段
- 简单常量替换

### P3：printf 可变参数
- 扩展 HostFunctions 支持更多参数
