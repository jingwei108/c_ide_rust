# 友好中文提示与智能修复深度设计

> 核心问题：如何让初学者真正看懂错误？修复功能能做到什么程度？

---

## 1. 初学者认知模型分析

### 1.1 初学者的典型困惑

| 错误类型 | 编译器原话 | 初学者真实困惑 |
|:---|:---|:---|
| 数组越界 | `Array index out of bounds` | "我明明写了 `i <= n`，为什么不行？" |
| 空指针 | `Segmentation fault` | "什么是 segmentation？我的代码看起来没问题啊？" |
| 缺少分号 | `Expected ';' before '}'` | "哪里缺了？这一行看起来有分号啊？" |
| 类型不匹配 | `Cannot convert int* to int` | "指针和 int 不都是数字吗？为什么不能赋值？" |
| 未定义变量 | `Undeclared identifier` | "我明明在前面定义了，为什么说没定义？" |
| 栈溢出 | `Stack overflow` | "栈是什么？为什么会溢出来？" |

**关键洞察**：初学者的问题往往不是"错误是什么"，而是"**为什么这是错误**"和"**我的代码到底哪里出了问题**"。

### 1.2 信息过载 vs 信息不足

```
❌ 信息不足（传统编译器）
"Segmentation fault"
→ 初学者：这是什么？怎么办？

❌ 信息过载（专家模式）
"SIGSEGV at 0x7ffd3a2b: attempted write to unmapped memory region 
 in function main at offset 0x42, likely caused by dereferencing 
 uninitialized pointer p (declared at line 5, type int*)"
→ 初学者：完全看不懂

✅ 分级信息（本项目目标）
┌─────────────────────────────────────────┐
│ 😵 程序崩溃了                           │
├─────────────────────────────────────────┤
│ 问题：你使用了指针 p，但它还没有指向   │
│       任何有效的内存地址。               │
│                                        │
│ 你的代码（第 7 行）：                    │
│    int* p;                             │
│    *p = 10;      ← 这里出错了          │
│                                        │
│ 原因：声明指针时，它的值是随机的。       │
│       直接往随机地址写数据会导致崩溃。   │
│                                        │
│ ✅ 修复方案：                            │
│    int a = 0;                          │
│    int* p = &a;   ← 让 p 指向 a        │
│    *p = 10;                            │
│                                        │
│ 📚 [什么是指针？] [内存就像酒店房间]    │
└─────────────────────────────────────────┘
```

---

## 2. 三级诊断信息系统

### 2.1 信息架构

```
错误诊断
├── L1 感知层（一眼看懂）
│   ├── 表情图标 + 一句话总结
│   ├── 出错的代码片段（高亮错误位置）
│   └── 修复按钮（如果可自动修复）
│
├── L2 理解层（点击展开）
│   ├── 通俗解释（为什么出错）
│   ├── 代码对比（❌ 你的写法 vs ✅ 正确写法）
│   ├── 运行时状态（当时的变量值）
│   └── 常见原因 checklist
│
└── L3 原理层（深入学习）
    ├── 内存/指针动画演示
    ├── 概念详解（Markdown）
    ├── 相似错误案例
    └── 练习题（巩固理解）
```

### 2.2 L1 感知层设计

**目标**：让用户在 1 秒内理解"出了什么问题"和"能不能一键修好"。

```csharp
public record L1Diagnostic {
    public string Emoji { get; init; }           // "😵" / "🤔" / "⚠️"
    public string Title { get; init; }           // "程序崩溃了"
    public string OneLiner { get; init; }        // "指针 p 还没有指向有效地址"
    public int Line { get; init; }               // 7
    public int Column { get; init; }             // 5
    public string CodeSnippet { get; init; }     // 出错行前后 2 行代码
    public bool HasAutoFix { get; init; }        // true / false
    public string AutoFixLabel { get; init; }    // "添加初始化"
}
```

**UI 呈现**：
```
┌────────────────────────────────────────┐
│ 😵 程序崩溃了                    [×]   │
├────────────────────────────────────────┤
│ 指针 p 还没有指向有效地址              │
│                                        │
│ 第 7 行：                              │
│    5  │ int* p;                        │
│    6  │                                │
│  ► 7  │ *p = 10;    ← 这里            │
│    8  │                                │
│                                        │
│ [🔧 添加初始化]  [📖 详细了解]         │
└────────────────────────────────────────┘
```

### 2.3 L2 理解层设计

**目标**：让用户在 30 秒内理解"为什么会出错"和"怎么改"。

**动态内容生成**：不是静态模板，而是根据**用户的实际代码**和**运行时状态**生成个性化解释。

**示例：数组越界**
```
┌────────────────────────────────────────┐
│ 为什么出错了？                         │
├────────────────────────────────────────┤
│                                        │
│ 你的代码（第 8 行）：                   │
│   int arr[5] = {1,2,3,4,5};           │
│   for (int i = 0; i <= 5; i++) {      │
│       arr[i] = i * 2;                  │
│   }                                    │
│                                        │
│ 当循环执行到 i = 5 时：                 │
│   • 你想访问 arr[5]                   │
│   • 但数组只有 5 个元素                │
│   • 有效索引是 0, 1, 2, 3, 4          │
│   • arr[5] 超出了数组的边界            │
│                                        │
│ 内存布局：                              │
│   arr[0]  arr[1]  arr[2]  arr[3]  arr[4]  [越界!]│
│     1       2       3       4       5    arr[5]? │
│   └────────────── 数组范围 ──────────────┘       │
│                                        │
│ ✅ 修改方案：                           │
│   for (int i = 0; i < 5; i++) {       │
│                    ↑                   │
│                  <= 改为 <              │
│                                        │
│ 💡 记忆口诀：循环条件用 < 数组大小       │
│                                        │
│ [🔧 应用修复] [📚 为什么索引从0开始？]  │
└────────────────────────────────────────┘
```

**关键技术**：
- **代码片段提取**：AST 层提取错误节点及其上下文（前后各 2 行）
- **运行时值注入**：如果是运行时错误，读取当时的变量值填充到消息中
- **内存可视化生成**：根据数组大小和索引自动生成 ASCII/Canvas 内存图

### 2.4 L3 原理层设计

**目标**：建立系统性理解，防止同类错误再次发生。

**内容组织**：
```
知识卡片：数组与内存
├── 🎬 动画演示（Avalonia Canvas）
│   └── 数组在内存中的连续存放演示
├── 📖 概念讲解
│   ├── 什么是数组？
│   ├── 为什么索引从 0 开始？
│   ├── 数组越界会发生什么？（缓冲区溢出）
│   └── C 语言为什么不检查数组越界？
├── 📝 常见错误模式
│   ├── 循环条件写成 <=（应该是 <）
│   ├── 忘记数组大小是 n，不是 n-1
│   └── 字符串末尾的 '\0' 占用一个字节
├── 🧪 练习题
│   ├── 题目 1：找出循环中的边界错误
│   ├── 题目 2：画出 int arr[3][4] 的内存布局
│   └── 题目 3：为什么 arr[-1] 也是越界？
└── 🔗 相关概念
    ├── 指针运算
    ├── 内存对齐
    └── 缓冲区溢出攻击（进阶）
```

---

## 3. 运行时诊断：比静态分析更强大

### 3.1 为什么运行时诊断更懂初学者？

静态编译器只能看到代码结构，而运行时知道**实际发生了什么**：

| 场景 | 静态分析能说的 | 运行时诊断能说的 |
|:---|:---|:---|
| 数组越界 | "索引可能越界" | "当 i = 10 时，arr[10] 越界了。数组大小是 5。当前 n = 10，是用户输入的值。" |
| 空指针 | "p 可能未初始化" | "p 的值是 0x00000000（NULL）。它声明于第 5 行，之后没有任何赋值。" |
| 除零 | "除数可能为 0" | "除数 x 的值是 0。x 在第 3 行初始化为 0，之后未被修改。" |
| 无限循环 | "循环条件可能恒真" | "循环已执行 100,000 步，i 的值始终是 1。i++ 被注释掉了。" |

### 3.2 运行时诊断信息生成

```cpp
// 后端：当 trap 发生时，收集运行时上下文
struct RuntimeContext {
    int currentLine;
    int currentColumn;
    std::string currentFunction;
    
    // 相关变量当前值
    struct VarSnapshot {
        std::string name;
        std::string type;
        int32_t intValue;
        uint32_t ptrValue;
        bool isPointer;
        bool isNull;
    };
    std::vector<VarSnapshot> relatedVars;
    
    // 如果是数组越界
    struct ArrayContext {
        std::string arrayName;
        int arraySize;
        int accessIndex;
    };
    std::optional<ArrayContext> arrayCtx;
};

// 生成个性化错误消息
std::string generatePersonalizedError(const RuntimeContext& ctx) {
    if (ctx.arrayCtx) {
        return format(
            "第 %d 行：数组索引 %d 超出了范围。\n"
            "数组 %s 只有 %d 个元素，有效索引是 0 ~ %d。\n"
            "当你写 %s[%d] 时，就像去酒店找 10 号房间，但酒店只有 5 间房。",
            ctx.currentLine,
            ctx.arrayCtx->accessIndex,
            ctx.arrayCtx->arrayName,
            ctx.arrayCtx->arraySize,
            ctx.arrayCtx->arraySize - 1,
            ctx.arrayCtx->arrayName,
            ctx.arrayCtx->accessIndex
        );
    }
    // ...
}
```

---

## 4. 智能修复系统：从基础到进阶

### 4.1 修复能力分级

```
修复能力
├── Level 1：语法自动修复（100% 自动，无需确认）
│   ├── 补分号、补括号、补引号
│   ├── 修正拼写错误（intt → int）
│   └── 添加缺失的函数声明
│
├── Level 2：语义辅助修复（90% 自动，简单确认）
│   ├── 数组越界：循环 <= 改为 <
│   ├── 空指针：添加初始化（int* p = &a）
│   ├── 未初始化变量：添加 = 0
│   ├── 类型不匹配：添加强制转换
│   └── 死代码删除
│
├── Level 3：逻辑建议修复（50% 准确，需要理解）
│   ├── 赋值误写为比较（if (a = 5) → if (a == 5)）
│   ├── 逻辑运算符误用（&& 误写为 &）
│   ├── 循环变量未递增（导致无限循环）
│   └── 忘记释放内存（malloc 后无 free）
│
└── Level 4：教学引导修复（不推荐自动应用）
    ├── 算法逻辑错误（排序边界条件）
    ├── 递归终止条件遗漏
    └── 性能优化建议（O(n²) → O(n log n)）
```

### 4.2 修复不仅仅是改代码，更是教思路

**反模式**：直接修改代码，学生不知道改了什么。

**正模式**：
```
┌────────────────────────────────────────┐
│ 🔧 修复建议：修改循环条件               │
├────────────────────────────────────────┤
│                                        │
│ 当前代码（第 4 行）：                   │
│   for (int i = 0; i <= n; i++)        │
│                                        │
│ 问题：当 i == n 时，arr[n] 越界。      │
│       数组 arr 的有效索引是 0 ~ n-1。   │
│                                        │
│ 建议修改：                              │
│   for (int i = 0; i < n; i++)         │
│                     ↑                  │
│                   这里变了              │
│                                        │
│ 为什么？                                │
│   • <= 表示 "小于或等于"               │
│   • i 会取到 0,1,2,...,n（共 n+1 个值）│
│   • < 表示 "小于"                      │
│   • i 会取到 0,1,2,...,n-1（共 n 个值）│
│   • 数组有 n 个元素，正好对应 n 个索引  │
│                                        │
│ [✅ 应用这个修复]                       │
│ [📝 我自己改]（高亮提示错误位置）        │
└────────────────────────────────────────┘
```

### 4.3 AST 级修复 vs WASM 级修复

| 层级 | 适用错误 | 修复方式 | 精度 |
|:---|:---|:---|:---|
| **AST 层** | 语法错误、类型错误、简单逻辑错误 | 修改 AST 节点，重新生成代码 | ⭐⭐⭐ 精确到字符 |
| **WASM 层** | 运行时错误（越界、空指针） | 只能诊断，无法修复（已丢失源码结构） | ⭐⭐ 只能定位到行 |

**决策**：
- **可自动修复的错误**：在 AST 层完成，生成修复后的源码
- **运行时错误**：在 WASM 层诊断，返回修改建议（文本级），不自动修改源码

### 4.4 修复示例库

```csharp
public class FixProvider {
    // 根据错误码和 AST 上下文生成修复
    public IEnumerable<CodeFix> GetFixes(Diagnostic diagnostic, ASTNode ast) {
        switch (diagnostic.ErrorCode) {
            case E_SEMI_MISSING:
                yield return new CodeFix {
                    Title = "添加分号",
                    Description = "在语句末尾添加 ';'",
                    Edit = new TextEdit {
                        Start = diagnostic.EndPosition,
                        NewText = ";"
                    },
                    Level = FixLevel.Auto       // 可直接应用
                };
                break;
                
            case E_ARRAY_OOB:
                // 检查是否是循环条件问题
                if (ast.FindParent<ForStmt>(diagnostic.Node) is var forStmt 
                    && forStmt.Condition is BinaryExpr cond
                    && cond.Op == "<=") {
                    yield return new CodeFix {
                        Title = "修改循环条件为 <",
                        Description = "数组有 n 个元素，索引范围是 0 ~ n-1，所以循环条件应该用 < 而不是 <=",
                        Edit = new TextEdit {
                            Start = cond.OpPosition,
                            OldText = "<=",
                            NewText = "<"
                        },
                        Level = FixLevel.Auto,
                        Explanation = "<= 会让 i 取到 n，但 arr[n] 超出了数组范围"
                    };
                }
                break;
                
            case E_NULL_PTR_DEREF:
                // 建议初始化
                var ptrDecl = ast.FindDeclaration(diagnostic.VariableName);
                yield return new CodeFix {
                    Title = "添加指针初始化",
                    Description = $"让 {diagnostic.VariableName} 指向一个有效的变量",
                    Edit = new TextEdit {
                        Start = ptrDecl.EndPosition,
                        OldText = $"int* {diagnostic.VariableName};",
                        NewText = $"int __{diagnostic.VariableName}_target = 0;\nint* {diagnostic.VariableName} = &__{diagnostic.VariableName}_target;"
                    },
                    Level = FixLevel.Suggest,    // 需要确认，因为变量名需要合理
                    Explanation = "声明指针时没有初始化，它的值是随机的。需要先让它指向一个有效的变量。"
                };
                break;
                
            case E_UNINIT_VAR:
                yield return new CodeFix {
                    Title = "添加初始化",
                    Description = $"给 {diagnostic.VariableName} 一个初始值",
                    Edit = new TextEdit {
                        Start = diagnostic.Node.EndPosition,
                        NewText = " = 0"
                    },
                    Level = FixLevel.Auto
                };
                break;
        }
    }
}
```

---

## 5. 知识卡片：不只是文档，是交互式学习

### 5.1 知识卡片内容结构

```yaml
# knowledge_base.yaml
knowledge_card:
  id: pointer_basics
  title: 什么是指针？
  
  # L1：一句话总结
  summary: "指针是存储内存地址的变量。"
  
  # L2：通俗解释
  explanation: |
    想象内存是一排酒店房间，每个房间有门牌号（地址）。
    
    • 普通变量（int a = 10）：你住在 1001 号房，房里放着数字 10。
    • 指针（int* p = &a）：p 住在 1002 号房，房里放着"1001"（即 a 的地址）。
    
    当你说 *p = 20，意思是："去 1001 号房，把里面的数字改成 20"。
  
  # L3：技术细节
  details: |
    在 C 语言中：
    • `&a` 获取变量 a 的地址
    • `*p` 访问指针 p 指向的地址中的值
    • 指针本身也占用内存（32位系统占 4 字节，64位系统占 8 字节）
  
  # 常见错误关联
  related_errors: [E_NULL_PTR_DEREF, E_DANGLING_PTR, E_OOB_ARRAY]
  
  # 可视化：内存布局图
  memory_diagram:
    type: grid
    address_start: 0x1000
    cells:
      - address: 0x1000
        label: "a"
        type: int
        value: 10
      - address: 0x1004
        label: "p"
        type: int*
        value: 0x1000
        arrow_to: 0x1000
  
  # 互动练习
  exercises:
    - question: "执行 int a = 5; int* p = &a; *p = 10; 后，a 的值是多少？"
      options: [5, 10, 15, 不确定]
      answer: 10
      explanation: "*p = 10 修改了 p 指向的变量 a 的值。"
```

### 5.2 动态知识卡片生成

**不是静态 Markdown，而是根据错误动态生成**：

```csharp
public class KnowledgeCardGenerator {
    public KnowledgeCard Generate(Diagnostic diag, ASTNode ast, RuntimeContext? runtime) {
        var card = new KnowledgeCard {
            Title = ErrorCatalog.GetTitle(diag.Code),
            Emoji = ErrorCatalog.GetEmoji(diag.Code)
        };
        
        // 1. 生成个性化解释（注入用户代码和运行时值）
        card.Summary = TemplateEngine.Render(
            ErrorCatalog.GetTemplate(diag.Code, level: 1),
            new { diag.Line, diag.Column, UserCode = GetSnippet(ast, diag) }
        );
        
        // 2. 如果是运行时错误，添加运行时上下文
        if (runtime != null) {
            card.RuntimeSection = GenerateRuntimeSection(diag, runtime);
        }
        
        // 3. 生成内存图（如果是内存相关错误）
        if (diag.Category == DiagnosticCategory.Memory) {
            card.MemoryDiagram = MemoryDiagramGenerator.Generate(
                runtime?.MemorySnapshot,
                highlightAddress: diag.RelatedAddress
            );
        }
        
        // 4. 关联修复建议
        card.SuggestedFixes = FixProvider.GetFixes(diag, ast);
        
        // 5. 关联相关知识
        card.RelatedConcepts = KnowledgeGraph.GetNeighbors(diag.Code);
        
        return card;
    }
}
```

### 5.3 内存可视化：用 Avalonia Canvas 绘制

```csharp
// 知识卡片中的内存图
public class MemoryDiagramCanvas : Control {
    public MemorySnapshot Snapshot { get; set; }
    public uint32_t? HighlightAddress { get; set; }
    
    public override void Render(DrawingContext ctx) {
        // 绘制内存格子
        foreach (var cell in Snapshot.Cells) {
            var rect = GetCellRect(cell.Address);
            var brush = cell.Type switch {
                CellType.Free => Brushes.Gray,
                CellType.Stack => Brushes.LightGreen,
                CellType.Heap => Brushes.LightBlue,
                CellType.Pointer => Brushes.Yellow,
                _ => Brushes.White
            };
            
            ctx.FillRectangle(brush, rect);
            ctx.DrawText(cell.Label, rect.TopLeft + new Point(2, 2));
            
            // 如果是指针，画箭头
            if (cell is PointerCell pc && pc.TargetAddress.HasValue) {
                DrawArrow(ctx, rect.Center, GetCellRect(pc.TargetAddress.Value).Center);
            }
        }
        
        // 高亮出错位置
        if (HighlightAddress.HasValue) {
            ctx.DrawRectangle(new Pen(Brushes.Red, 3), 
                GetCellRect(HighlightAddress.Value));
        }
    }
}
```

---

## 6. 具体实现：错误码设计

### 6.1 错误码体系

```cpp
// ErrorCodes.hpp — 实际实现
namespace cide {

enum class ErrorCode : int {
    // ===== Lexer 错误 (E1xxx) =====
    E1001_UnknownChar       = 1001,  // 无法识别的字符
    E1002_UnterminatedString= 1002,  // 字符串未闭合
    E1003_StringCrossLine   = 1003,  // 字符串不能跨行
    E1004_UnsupportedOp     = 1004,  // 暂不支持的操作符 (| / & 单目误用)

    // ===== Parser 错误 (E2xxx) =====
    E2001_ExpectedType      = 2001,  // 预期类型名称
    E2002_ExpectedArraySize = 2002,  // 预期数组大小或 ']'
    E2003_ExpectedExpr      = 2003,  // 预期表达式
    E2004_ExpectedCaseOrDefault = 2004, // 预期 'case' 或 'default'
    E2005_ExpectedSemicolon = 2005,  // 预期 ';'  → 结构化修复: InsertText ";"
    E2006_ExpectedClosingBrace = 2006, // 预期 '}' → 结构化修复: InsertText "}"
    E2007_ExpectedClosingParen = 2007, // 预期 ')' → 结构化修复: InsertText ")"
    E2008_ExpectedClosingBracket = 2008, // 预期 ']' → 结构化修复: InsertText "]"

    // ===== TypeChecker 错误 (E3xxx) =====
    E3001_VarRedeclared     = 3001,  // 变量重复声明
    E3002_StructRedeclared  = 3002,  // 结构体重复定义
    E3003_FuncRedeclared    = 3003,  // 函数重复定义
    E3004_TypeMismatch      = 3004,  // 类型不匹配
    // ... (E3005-E3047 详见 ErrorCodes.hpp)

    // ===== TypeChecker 警告 (W3xxx) =====
    W3050_AssignInCondition = 3050,  // 条件中使用了赋值 (=)，可能是想使用 ==
    W3051_ArrayBoundOffByOne = 3051, // 循环条件可能是 <=，数组访问可能越界

    // ===== BytecodeGen 错误 (E4xxx) =====
    E4001_NoMainFunc        = 4001,  // 找不到 main 函数
    // ... (E4002-E4014 详见 ErrorCodes.hpp)

    Unknown = 0,
};
```

**已支持结构化自动修复的错误码**：

| 错误码 | 触发场景 | 修复动作 | 自动？ |
|:---|:---|:---|:---|
| `E2005_ExpectedSemicolon` | 语句结束缺少 `;` | `InsertText` `;` | ✅ |
| `E2006_ExpectedClosingBrace` | 块结束缺少 `}` | `InsertText` `}` | ✅ |
| `E2007_ExpectedClosingParen` | 条件/调用缺少 `)` | `InsertText` `)` | ✅ |
| `E2008_ExpectedClosingBracket` | 数组索引缺少 `]` | `InsertText` `]` | ✅ |
| `E1004_UnsupportedOp` | `\|` → `\|\|`, `&` → `&&` | `ReplaceText` | ✅ |
| `W3050_AssignInCondition` | `=` → `==` | `ReplaceText` | ✅ |
| `W3051_ArrayBoundOffByOne` | `<=` → `<` | `ReplaceText` | ✅ |

struct ErrorInfo {
    ErrorCode code;
    std::string emoji;
    std::string title;           // L1: "数组越界"
    std::string templateL1;      // "第 {line} 行：{message}"
    std::string templateL2;      // 详细解释模板
    std::string templateL3;      // 原理讲解模板
    std::string analogy;         // 生活类比
    std::vector<std::string> commonCauses;
    std::vector<FixTemplate> fixes;
};

// 错误信息注册表
const std::map<ErrorCode, ErrorInfo> ErrorRegistry = {
    {ErrorCode::E_ARRAY_OOB, {
        .code = ErrorCode::E_ARRAY_OOB,
        .emoji = "🚫",
        .title = "数组越界",
        .templateL1 = "第 {line} 行：索引 {index} 超出了数组 {array} 的范围（大小 {size}）",
        .templateL2 = 
            "当你访问 {array}[{index}] 时，数组只有 {size} 个元素。\n"
            "有效索引是 0, 1, 2, ..., {lastIndex}。\n"
            "{index} 超出了这个范围，就像去图书馆找第 10 本书，但图书馆只有 5 本。",
        .analogy = "数组就像一排座位，编号从 0 开始。有 5 个座位的话，编号是 0~4。坐到 5 号座位上会摔下来。",
        .commonCauses = {
            "循环条件写成了 <= 而不是 <",
            "忘记数组大小是 n，不是 n-1",
            "用户输入的数字直接作为索引，没有检查范围"
        },
        .fixes = {
            {FixLevel::Auto, "修改循环条件", "将 <= 改为 <"},
            {FixLevel::Auto, "添加边界检查", "if (index >= size) { /* 错误处理 */ }"}
        }
    }},
    
    {ErrorCode::E_NULL_PTR_DEREF, {
        .code = ErrorCode::E_NULL_PTR_DEREF,
        .emoji = "😵",
        .title = "空指针解引用",
        .templateL1 = "第 {line} 行：指针 {name} 还没有指向有效的地址",
        .templateL2 = 
            "你声明了指针 {name}，但没有让它指向任何变量。\n"
            "指针 {name} 的值是 NULL（0），这表示" nowhere"。\n"
            "试图往" nowhere" 写数据会导致程序崩溃。",
        .analogy = "指针就像快递单上的地址。如果你写了" 空地址"，快递员不知道往哪送，就会报错。",
        .commonCauses = {
            "声明指针后忘记初始化",
            "malloc 失败后没有检查返回值是否为 NULL",
            "指针被 free 后再次使用"
        },
        .fixes = {
            {FixLevel::Auto, "初始化指针", "int a = 0; int* p = &a;"},
            {FixLevel::Suggest, "检查 malloc 返回值", "if (p == NULL) { return; }"}
        }
    }},
    
    // ... 更多错误码
};

} // namespace cide
```

### 6.2 中文消息生成引擎

```cpp
class ChineseDiagnosticEngine {
public:
    std::string GenerateMessage(ErrorCode code, int level, 
                                const DiagnosticContext& ctx) {
        auto it = ErrorRegistry.find(code);
        if (it == ErrorRegistry.end()) {
            return "未知错误";
        }
        
        const auto& info = it->second;
        
        switch (level) {
            case 1:
                return formatTemplate(info.templateL1, ctx);
            case 2:
                return formatTemplate(info.templateL2, ctx) + "\n\n💡 " + info.analogy;
            case 3:
                return formatTemplate(info.templateL3, ctx);
            default:
                return info.title;
        }
    }
    
private:
    std::string formatTemplate(const std::string& tpl, const DiagnosticContext& ctx) {
        std::string result = tpl;
        replaceAll(result, "{line}", std::to_string(ctx.line));
        replaceAll(result, "{column}", std::to_string(ctx.column));
        replaceAll(result, "{name}", ctx.variableName);
        replaceAll(result, "{index}", std::to_string(ctx.arrayIndex));
        replaceAll(result, "{size}", std::to_string(ctx.arraySize));
        replaceAll(result, "{array}", ctx.arrayName);
        replaceAll(result, "{lastIndex}", std::to_string(ctx.arraySize - 1));
        return result;
    }
};
```

---

## 7. 前端交互设计

### 7.1 代码编辑器中的错误呈现

```
┌────────────────────────────────────────┐
│ 1 │ int main() {                       │
│ 2 │     int arr[5];                    │
│ 3 │     for (int i = 0; i <= 5; i++) {│
│   │                      ~~~           │
│   │                      │             │
│   │                      ▼             │
│   │         🚫 数组越界                 │
│   │         索引 5 超出范围（大小 5）   │
│   │         [🔧 改为 <] [📖 详情]       │
│ 4 │         arr[i] = i;                │
│ 5 │     }                              │
│ 6 │     return 0;                      │
│ 7 │ }                                  │
└────────────────────────────────────────┘
```

### 7.2 底部错误面板

```
┌────────────────────────────────────────┐
│ ⚠️ 错误 (2)  │  📊 内存  │  ▶️ 输出    │
├────────────────────────────────────────┤
│ 🚫 第 3 行：数组越界                    │
│    索引 5 超出了 arr 的范围（大小 5）    │
│    [🔧 改为 <] [📖 为什么？]            │
├────────────────────────────────────────┤
│ ⚠️ 第 7 行：未使用的变量                │
│    变量 x 声明后未被使用                 │
│    [🗑️ 删除] [💡 用它做什么？]          │
└────────────────────────────────────────┘
```

### 7.3 知识卡片弹窗

```
┌────────────────────────────────────────┐
│ 😵 空指针解引用                    [×]   │
├────────────────────────────────────────┤
│                                        │
│ 问题                                   │
│ 指针 p 还没有指向有效的内存地址。        │
│                                        │
│ 你的代码（第 7 行）                     │
│ ┌──────────────────────────────────┐   │
│ │ 5 │ int* p;                      │   │
│ │ 6 │                              │   │
│ │ 7 │ *p = 10;   ← 这里崩溃了      │   │
│ └──────────────────────────────────┘   │
│                                        │
│ 为什么会这样？                          │
│ ▼                                      │
│ 声明指针时，它的值是随机的（可能是      │
│ 0，也可能是任何数字）。你直接往这个      │
│ 随机地址写数据，程序就会崩溃。          │
│                                        │
│ 💡 想象指针是快递单上的地址。如果        │
│    你写了"空地址"，快递员不知道往       │
│    哪送，就会报错。                     │
│                                        │
│ ✅ 修复方案                             │
│ ┌──────────────────────────────────┐   │
│ │ int a = 0;                       │   │
│ │ int* p = &a;   ← 让 p 指向 a     │   │
│ │ *p = 10;                         │   │
│ └──────────────────────────────────┘   │
│                                        │
│ [🔧 应用修复]                          │
│                                        │
│ 📚 还想了解：                           │
│ • [什么是指针？]                        │
│ • [内存就像酒店房间]                    │
│ • [malloc 和 free 是什么？]             │
└────────────────────────────────────────┘
```

---

## 8. 实施优先级

### Phase 1：让初学者"看得懂"（MVP）
- [x] L1 错误消息（一句话 + 代码片段 + 表情）
- [x] L2 通俗解释 + 代码对比
- [x] 基础自动修复（补分号 `;`、补括号 `}`/`)`/`]`、改 `\|`→`\|\|`、`&`→`&&`）— 后端结构化修复已就绪
- [x] 10 个最常见错误的中文消息（数组越界、空指针、未声明变量、类型不匹配等）

### Phase 2：让初学者"学得会"（知识卡片）
- [ ] L3 知识卡片（Markdown + 内存图）
- [ ] 运行时诊断（注入实际变量值）
- [ ] 20 个错误码的完整知识库
- [ ] 类比系统（酒店房间、快递地址等生活类比）

### Phase 3：让初学者"修得快"（智能修复）
- [x] Level 1 语法自动修复（补分号、补括号、`\|`→`\|\|`、`&`→`&&`）— 后端精确结构化修复 2026-05-05
- [ ] Level 2 辅助修复（类型转换、边界修正 `<=`→`<`）
- [ ] Level 3 逻辑建议（`=` vs `==`、死代码检测）
- [ ] 修复预览（应用前显示 diff）
- [ ] 渐进式 disclosure（根据用户熟练度调整信息深度）

### Phase 4：个性化学习（进阶）
- [ ] 用户错误历史分析（"你经常犯数组越界错误，这里有一份专项练习"）
- [ ] 难度自适应（减少/增加解释深度）
- [ ] 知识图谱可视化（概念之间的关联图）

---

## 9. 结论

### 初学者能看懂吗？

**能**，但需要做到以下几点：
1. **不用术语堆砌**：不说"segmentation fault"，说"程序崩溃了"
2. **给上下文**：不只说"数组越界"，说"当你访问 arr[10] 时，数组只有 5 个元素"
3. **用类比**："指针就像地址标签"、"数组就像一排座位"
4. **给方案**：不只指出错误，给出可直接应用的修复
5. **分层展示**：先给一句话总结，用户感兴趣再展开详细解释

### 修复能做到什么程度？

| 级别 | 能力 | 示例 | 是否自动 |
|:---|:---|:---|:---|
| 基础 | 语法修复 | 补分号、改括号 | ✅ 全自动 |
| 中级 | 语义修复 | 改循环边界、加初始化 | ✅ 全自动 |
| 进阶 | 逻辑建议 | = vs ==、死代码 | ⚠️ 需确认 |
| 高级 | 算法指导 | 递归边界、排序逻辑 | ❌ 仅建议 |

**核心原则**：
- 语法和常见语义错误 → **自动修复**（减少挫败感）
- 逻辑错误 → **建议修复 + 解释原因**（保护学习过程）
- 算法错误 → **仅提供知识卡片**（不替代思考）
