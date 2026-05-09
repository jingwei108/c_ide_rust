# [已归档] OCR 照片导入与智能纠错设计

> **状态**: 已归档。OCR 相关代码（Cide.Client/Core/Ocr/ 等）已于 2026-05-04 清理移除。
> 本文档保留作为历史设计参考。

> 核心问题：拍摄识别的代码错误能否修正？如何与现有修复系统结合？

---

## 1. 场景分析

### 1.1 用户使用流程

```
打开 IDE → 点击 📷 导入 → 拍摄/选择照片
                                    ↓
                            OCR 引擎识别
                                    ↓
                            原始文本（含错误）
                                    ↓
                            ┌───────────────┐
                            │ 智能纠错引擎   │
                            │ • OCR 后处理   │
                            │ • 编译器反馈   │
                            │ • 用户确认     │
                            └───────┬───────┘
                                    ↓
                            修正后的代码
                                    ↓
                            导入编辑器
```

### 1.2 OCR 典型错误清单

| 错误类别 | OCR 输出 | 实际代码 | 原因 |
|:---|:---|:---|:---|
| **数字/字母混淆** | `int a = 1O;` | `int a = 10;` | 0 ↔ O/o |
| | `int a = l;` | `int a = 1;` | 1 ↔ l/I/| |
| | `int a = S;` | `int a = 5;` | 5 ↔ S/s |
| | `int a = Z;` | `int a = 2;` | 2 ↔ Z/z |
| **符号混淆** | `if (a = 5)` | `if (a == 5)` | = ↔ == |
| | `a & b` | `a && b` | & ↔ && |
| | `{` → `(` | `{` | 手写体相似 |
| | `:` → `;` | `;` | 手写体 : 和 ; |
| **空格丢失** | `inta = 5;` | `int a = 5;` | 连续字符间空格被忽略 |
| | `inta=5;` | `int a = 5;` | 所有空格丢失 |
| **换行错误** | `inta\n= 5;` | `int a = 5;` | 换行位置识别错误 |
| **字符遗漏** | `int a = 5` | `int a = 5;` | 行末模糊字符丢失 |
| | `for(int i=0;i<n;i+)` | `for(int i=0;i<n;i++)` | 重复字符漏识别 |
| **注释混淆** | `printf("hello");` | `// printf("hello");` | `//` 被忽略 |
| | `x = 10;` | `/* x = 10; */` | 注释符号丢失 |
| **字符串断裂** | `printf("hel` | `printf("hello");` | 字符串被截断 |

---

## 2. 三级纠错策略

```
OCR 原始文本
    ↓
┌─────────────────────────────────────────────┐
│ Level 1: OCR 后处理（基于置信度和字符混淆表）│
│ • 空格恢复                                  │
│ • 常见字符替换（O→0, l→1）                  │
│ • 符号补全（= → == 在条件中）               │
└─────────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────────┐
│ Level 2: 编译器驱动纠错（核心创新）          │
│ • 将文本送入编译器                          │
│ • 分析错误是否由 OCR 导致                    │
│ • 生成修正假设 → 尝试 → 验证                 │
│ • 循环直到编译成功或无法修正                  │
└─────────────────────────────────────────────┘
    ↓
┌─────────────────────────────────────────────┐
│ Level 3: 用户确认交互                        │
│ • 高置信度修正：自动应用                     │
│ • 中置信度修正：用户确认                     │
│ • 低置信度区域：人工高亮标记                 │
└─────────────────────────────────────────────┘
    ↓
导入编辑器
```

---

## 3. Level 1: OCR 后处理

### 3.1 字符混淆映射表

```csharp
public static class OcrConfusionMap {
    // 正向映射：OCR 可能识别成的字符 → 实际可能的字符
    public static readonly Dictionary<char, List<(char candidate, float probability)>> Map = new() {
        ['O'] = new() { ('0', 0.9f), ('O', 0.1f) },
        ['o'] = new() { ('0', 0.85f), ('o', 0.15f) },
        ['0'] = new() { ('O', 0.7f), ('0', 0.3f) },
        ['l'] = new() { ('1', 0.9f), ('l', 0.05f), ('I', 0.05f) },
        ['I'] = new() { ('1', 0.85f), ('I', 0.1f), ('l', 0.05f) },
        ['1'] = new() { ('l', 0.6f), ('I', 0.3f), ('1', 0.1f) },
        ['5'] = new() { ('S', 0.7f), ('s', 0.2f), ('5', 0.1f) },
        ['S'] = new() { ('5', 0.8f), ('s', 0.15f), ('S', 0.05f) },
        ['s'] = new() { ('5', 0.6f), ('S', 0.3f), ('s', 0.1f) },
        ['2'] = new() { ('Z', 0.7f), ('z', 0.2f), ('2', 0.1f) },
        ['8'] = new() { ('B', 0.6f), ('8', 0.4f) },
        [';'] = new() { (':', 0.5f), (';', 0.5f) },
        [':'] = new() { (';', 0.6f), (':', 0.4f) },
        ['{'] = new() { ('(', 0.4f), ('{', 0.6f) },
        ['}'] = new() { (')', 0.4f), ('}', 0.6f) },
        ['('] = new() { ('{', 0.3f), ('(', 0.7f) },
        [')'] = new() { ('}', 0.3f), (')', 0.7f) },
    };
    
    // 上下文相关的符号映射
    public static readonly Dictionary<string, List<(string candidate, string condition, float probability)>> ContextualMap = new() {
        // 在条件表达式中，= 很可能是 ==
        ["="] = new() {
            ("==", "inside_condition", 0.8f),
            ("=", "assignment", 0.95f),
        },
        // 在逻辑表达式中，& 很可能是 &&
        ["&"] = new() {
            ("&&", "inside_condition", 0.85f),
            ("&", "bitwise", 0.7f),
        },
        // | 很可能是 ||
        ["|"] = new() {
            ("||", "inside_condition", 0.85f),
            ("|", "bitwise", 0.7f),
        },
    };
}
```

### 3.2 空格恢复算法

```csharp
public class SpaceRecoveryEngine {
    // C 语言关键字列表
    private static readonly HashSet<string> Keywords = new() {
        "int", "void", "if", "else", "for", "while", "return",
        "struct", "sizeof", "break", "continue", "malloc", "free"
    };
    
    public string Recover(string text) {
        // 使用最大匹配法，在关键字和标识符之间插入空格
        var result = new StringBuilder();
        int i = 0;
        
        while (i < text.Length) {
            // 尝试匹配最长关键字
            var match = TryMatchKeyword(text, i);
            
            if (match != null) {
                // 关键字前面需要空格（如果前面是标识符或数字）
                if (result.Length > 0 && IsIdentifierChar(result[^1])) {
                    result.Append(' ');
                }
                result.Append(match);
                i += match.Length;
                
                // 关键字后面需要空格（如果后面是标识符）
                if (i < text.Length && IsIdentifierChar(text[i])) {
                    result.Append(' ');
                }
            } else {
                result.Append(text[i]);
                i++;
            }
        }
        
        return result.ToString();
    }
    
    private string? TryMatchKeyword(string text, int start) {
        foreach (var kw in Keywords.OrderByDescending(k => k.Length)) {
            if (start + kw.Length <= text.Length && 
                text.Substring(start, kw.Length) == kw) {
                return kw;
            }
        }
        return null;
    }
}

// 示例
// 输入:  "inta=1O;if(a=5){print("hello");}"
// 输出:  "int a = 1O; if (a = 5) { print ("hello"); }"
//            ↑     ↑    ↑ ↑ ↑ ↑  ↑
//            空格恢复
```

---

## 4. Level 2: 编译器驱动纠错（核心创新）

### 4.1 核心思想

**C 语言语法严格，编译器是最好的 OCR 纠错器。**

传统 OCR 纠错：基于语言模型（NLP）猜测正确的词。
编译器驱动纠错：基于**形式语法**验证猜测，确保修正后的代码在语法和语义上都是合法的。

```
OCR 文本
  ↓
编译器尝试编译 → 发现错误 E1
  ↓
分析 E1：
  • 如果是语法错误 → 可能是 OCR 字符错误
  • 提取错误位置附近的 token
  • 查询字符混淆映射表，生成候选修正
  ↓
应用最可能的修正 → 重新编译
  ↓
成功？→ 输出修正结果
失败？→ 尝试下一个候选，或报告不确定
```

### 4.2 纠错引擎实现

```csharp
public class CompilerDrivenOcrCorrector {
    private readonly CideCompiler _compiler;
    private readonly int _maxIterations = 10;
    
    public OcrCorrectionResult Correct(string ocrText) {
        var corrections = new List<Correction>();
        var currentText = ocrText;
        var appliedPositions = new HashSet<int>();  // 避免重复修正同一位置
        
        for (int iteration = 0; iteration < _maxIterations; iteration++) {
            // 尝试编译当前文本
            var result = _compiler.TryCompile(currentText);
            
            if (result.Success) {
                return new OcrCorrectionResult {
                    Success = true,
                    CorrectedText = currentText,
                    Corrections = corrections,
                    Confidence = CalculateConfidence(corrections)
                };
            }
            
            // 获取第一个可修正的错误
            var fixableError = FindFixableError(result.Errors, currentText, appliedPositions);
            
            if (fixableError == null) {
                // 还有错误但无法自动修正
                return new OcrCorrectionResult {
                    Success = false,
                    CorrectedText = currentText,
                    Corrections = corrections,
                    RemainingErrors = result.Errors,
                    UncertainRegions = MarkUncertainRegions(currentText, result.Errors)
                };
            }
            
            // 应用修正
            corrections.Add(fixableError.Correction);
            currentText = ApplyCorrection(currentText, fixableError.Correction);
            appliedPositions.Add(fixableError.Position);
        }
        
        return new OcrCorrectionResult {
            Success = false,
            CorrectedText = currentText,
            Corrections = corrections,
            Message = "达到最大修正轮数，仍有错误无法自动修复"
        };
    }
    
    private FixableError? FindFixableError(
        List<Diagnostic> errors, string text, HashSet<int> appliedPositions) {
        
        foreach (var error in errors) {
            if (appliedPositions.Contains(error.Position)) continue;
            
            var correction = TryGenerateCorrection(error, text);
            if (correction != null) {
                return new FixableError { Error = error, Correction = correction, Position = error.Position };
            }
        }
        
        return null;
    }
    
    private Correction? TryGenerateCorrection(Diagnostic error, string text) {
        // === 模式 1: 未声明标识符，可能是关键字拼接 ===
        if (error.Code == ErrorCode.E_UNDECLARED_VAR) {
            var token = error.Token;  // e.g., "inta"
            
            // 尝试拆分：inta → int + a
            foreach (var kw in Keywords.Where(k => token.StartsWith(k))) {
                var remainder = token.Substring(kw.Length);
                if (IsValidIdentifier(remainder)) {
                    return new Correction {
                        Type = CorrectionType.InsertSpace,
                        Description = $"OCR 可能将 'int a' 识别为 '{token}'（丢失空格）",
                        Original = token,
                        Replacement = $"{kw} {remainder}",
                        Position = error.Position,
                        Confidence = 0.95f
                    };
                }
            }
        }
        
        // === 模式 2: 未声明标识符，可能是数字字母混淆 ===
        if (error.Code == ErrorCode.E_UNDECLARED_VAR && error.Token.Length == 1) {
            var ch = error.Token[0];
            if (OcrConfusionMap.Map.TryGetValue(ch, out var candidates)) {
                var best = candidates.OrderByDescending(c => c.probability).First();
                return new Correction {
                    Type = CorrectionType.CharSubstitution,
                    Description = $"OCR 可能将 '{best.candidate}' 识别为 '{ch}'",
                    Original = ch.ToString(),
                    Replacement = best.candidate.ToString(),
                    Position = error.Position,
                    Confidence = best.probability
                };
            }
        }
        
        // === 模式 3: 期望数字但遇到字母（如 1O → 10）===
        if (error.Code == ErrorCode.E_UNEXPECTED_TOKEN && 
            IsNumericContext(error, text)) {
            
            var token = error.Token;
            var corrected = new StringBuilder();
            bool changed = false;
            
            foreach (var ch in token) {
                if (OcrConfusionMap.Map.TryGetValue(ch, out var candidates)) {
                    var digitCandidate = candidates.FirstOrDefault(c => char.IsDigit(c.candidate));
                    if (digitCandidate != default) {
                        corrected.Append(digitCandidate.candidate);
                        changed = true;
                        continue;
                    }
                }
                corrected.Append(ch);
            }
            
            if (changed) {
                return new Correction {
                    Type = CorrectionType.CharSubstitution,
                    Description = $"OCR 可能将数字识别为字母：'{token}' → '{corrected}'",
                    Original = token,
                    Replacement = corrected.ToString(),
                    Position = error.Position,
                    Confidence = 0.85f
                };
            }
        }
        
        // === 模式 4: 缺少分号 ===
        if (error.Code == ErrorCode.E_SEMI_MISSING) {
            return new Correction {
                Type = CorrectionType.InsertChar,
                Description = "OCR 可能遗漏了行末的分号",
                Original = "",
                Replacement = ";",
                Position = error.EndPosition,
                Confidence = 0.9f
            };
        }
        
        // === 模式 5: 条件中的赋值（= → ==）===
        if (error.Code == ErrorCode.E_ASSIGN_IN_CONDITION) {
            return new Correction {
                Type = CorrectionType.SymbolExpansion,
                Description = "条件判断建议用 ==，OCR 可能遗漏了一个 =",
                Original = "=",
                Replacement = "==",
                Position = error.Position,
                Confidence = 0.75f
            };
        }
        
        // === 模式 6: 括号不匹配 ===
        if (error.Code == ErrorCode.E_BRACE_MISMATCH) {
            // 检查是否是 { ↔ ( 混淆
            var nearby = text.Substring(error.Position - 5, 10);
            if (nearby.Contains('(') && !nearby.Contains('{')) {
                return new Correction {
                    Type = CorrectionType.CharSubstitution,
                    Description = "OCR 可能将 { 识别为 (",
                    Original = "(",
                    Replacement = "{",
                    Position = error.Position,
                    Confidence = 0.7f
                };
            }
        }
        
        // === 模式 7: 未闭合字符串 ===
        if (error.Code == ErrorCode.E_UNCLOSED_STRING) {
            return new Correction {
                Type = CorrectionType.InsertChar,
                Description = "OCR 可能遗漏了字符串末尾的引号",
                Original = "",
                Replacement = "\"",
                Position = error.EndPosition,
                Confidence = 0.8f
            };
        }
        
        return null;
    }
}
```

### 4.3 纠错示例演示

**输入（OCR 原始输出）**：
```c
inta = 1O;
if (a = 5) {
    printf("hell0 wor1d");
}
```

**第 1 轮编译**：
```
错误 1: 第 1 行，第 1 列：未声明标识符 "inta"
  → 匹配模式 1：inta → int + a（空格丢失）
  → 修正：inta → int a
```

**第 2 轮编译**：
```
错误 1: 第 1 行，第 9 列：未声明标识符 "O"
  → 匹配模式 2：O 是单字符标识符，混淆映射 O→0
  → 修正：1O → 10
```

**第 3 轮编译**：
```
错误 1: 第 2 行，第 7 列：条件中使用了赋值
  → 匹配模式 5：= → ==
  → 修正：a = 5 → a == 5
```

**第 4 轮编译**：
```
成功！
```

**最终输出**：
```c
int a = 10;
if (a == 5) {
    printf("hell0 wor1d");
}
```

**剩余不确定区域**：`"hell0 wor1d"` 中的 `0` 和 `1` 在字符串内，编译器无法判断是否正确，标记为人工确认。

---

## 5. Level 3: 用户确认交互设计

### 5.1 置信度分级

```csharp
public enum CorrectionConfidence {
    AutoApply,      // 置信度 > 0.9，直接应用
    Suggest,        // 置信度 0.7~0.9，默认勾选，用户可取消
    Uncertain,      // 置信度 < 0.7，默认不勾选，需要用户确认
    Unfixable       // 无法自动修正，人工高亮
}
```

### 5.2 照片导入确认界面

```
┌──────────────────────────────────────────────┐
│ 📷 照片导入                              [×]  │
├──────────────────────────────────────────────┤
│                                              │
│ ┌──────────────────────────────────────────┐ │
│ │         [照片预览]                        │ │
│ │                                          │ │
│ │    inta = 1O;                            │ │
│ │    if (a = 5) {                          │ │
│ │        printf("hello");                   │ │
│ │    }                                     │ │
│ └──────────────────────────────────────────┘ │
│                                              │
│ OCR 识别结果：                                │
│ ┌──────────────────────────────────────────┐ │
│ │ 1  inta = 1O;                            │ │
│ │ 2  if (a = 5) {                          │ │
│ │ 3      printf("hello");                   │ │
│ │ 4  }                                     │ │
│ └──────────────────────────────────────────┘ │
│                                              │
│ 📝 自动修正（5 处）：                         │
│                                              │
│ ┌──────────────────────────────────────────┐ │
│ │ ✅ 第 1 行：inta → int a                  │ │
│ │    置信度 95% · OCR 丢失了空格             │ │
│ │    [取消]                                 │ │
│ ├──────────────────────────────────────────┤ │
│ │ ✅ 第 1 行：1O → 10                       │ │
│ │    置信度 90% · 将字母 O 识别为数字 0      │ │
│ │    [取消]                                 │ │
│ ├──────────────────────────────────────────┤ │
│ │ ☑️ 第 2 行：= → ==                        │ │
│ │    置信度 75% · 条件判断建议用 ==          │ │
│ │    [取消]                                 │ │
│ ├──────────────────────────────────────────┤ │
│ │ ☐ 第 3 行：printf → print                 │ │
│ │    置信度 45% · 不确定是否为 printf        │ │
│ │    [接受]                                 │ │
│ └──────────────────────────────────────────┘ │
│                                              │
│ 🔴 不确定区域（需要人工确认）：                │
│ ┌──────────────────────────────────────────┐ │
│ │ 第 1 行第 5 列："a" 前的字符模糊          │ │
│ │    [🔍 放大查看] [✏️ 手动编辑]            │ │
│ └──────────────────────────────────────────┘ │
│                                              │
│ [✅ 应用选中的修正并导入]                    │
│ [✏️ 进入编辑器手动修改]                      │
│                                              │
│ 💡 提示：你可以先点击 [应用] 导入，然后在    │
│    编辑器中使用 💡 快速修复功能继续修改。     │
└──────────────────────────────────────────────┘
```

### 5.3 与编辑器修复的衔接

照片导入的修正和编辑器的 QuickFix 共享同一套基础设施：

```
┌─────────────────────────────────────────────┐
│              统一修正引擎                      │
│                                             │
│  输入：文本 + 上下文（OCR / 编辑器）          │
│                    ↓                        │
│         ┌─────────────────┐                 │
│         │ 错误检测器       │                 │
│         │ • 编译器诊断     │                 │
│         │ • OCR 特征分析   │                 │
│         └────────┬────────┘                 │
│                  ↓                          │
│         ┌─────────────────┐                 │
│         │ 修正生成器       │                 │
│         │ • OCR 专用规则   │                 │
│         │ • 语法修复规则   │                 │
│         │ • 语义修复规则   │                 │
│         └────────┬────────┘                 │
│                  ↓                          │
│         ┌─────────────────┐                 │
│         │ 修正应用器       │                 │
│         │ • Diff 预览      │                 │
│         │ • 撤销/重做      │                 │
│         │ • 置信度评分     │                 │
│         └─────────────────┘                 │
└─────────────────────────────────────────────┘
```

---

## 6. 技术架构

### 6.1 新增模块

```
Cide.Client/
├── Services/
│   ├── OcrService.cs                  # OCR 引擎封装（Azure/Google/Tesseract）
│   └── OcrCorrectionService.cs        # 纠错协调器
│
├── Core/
│   └── OcrCorrection/
│       ├── OcrConfusionMap.cs         # 字符混淆映射表
│       ├── SpaceRecoveryEngine.cs     # 空格恢复
│       ├── CompilerDrivenCorrector.cs # 编译器驱动纠错核心
│       ├── Correction.cs              # 修正数据模型
│       └── OcrCorrectionResult.cs     # 纠错结果
│
├── Views/
│   └── PhotoImportPage.axaml          # 照片导入确认界面
│
└── ViewModels/
    └── PhotoImportViewModel.cs        # 导入页面 VM
```

### 6.2 OCR 引擎选择

| 方案 | 准确率 | 离线 | 成本 | 推荐场景 |
|:---|:---|:---|:---|:---|
| **Azure Computer Vision** | ⭐⭐⭐⭐⭐ | ❌ | 按调用计费 | 在线教学平台 |
| **Google Vision API** | ⭐⭐⭐⭐⭐ | ❌ | 按调用计费 | 在线教学平台 |
| **Tesseract.js** | ⭐⭐⭐ | ✅ | 免费 | 离线使用、原型验证 |
| **本地 ML 模型** | ⭐⭐⭐⭐ | ✅ | 一次性 | 生产环境最佳 |
| **云端代码专用 OCR** | ⭐⭐⭐⭐⭐ | ❌ | 较高 | 高精度需求 |

**推荐 Phase 1**：使用 Azure/Google Vision API（准确率高，快速验证）
**推荐 Phase 2**：训练专用代码 OCR 模型或集成 Tesseract + 自定义训练

### 6.3 与现有系统的集成点

| 现有系统 | 集成方式 | 复用内容 |
|:---|:---|:---|
| C 子集编译器 | 纠错引擎调用编译器进行验证 | 编译错误码、错误位置 |
| 诊断系统 | 共享错误码和中文消息模板 | ErrorCodes.hpp、ErrorCatalog.cs |
| QuickFix 引擎 | OCR 修正使用同样的 TextEdit/Diff 基础设施 | Correction、TextEdit 模型 |
| 知识卡片 | OCR 导入错误也可关联知识卡片 | KnowledgeCard 组件 |

---

## 7. 边界与限制

### 7.1 能修正的

| OCR 错误 | 修正方式 | 成功率 |
|:---|:---|:---|
| 空格丢失 | 关键字匹配 + 编译器验证 | ~95% |
| 0/O 混淆 | 数字上下文分析 | ~90% |
| 1/l/I 混淆 | 数字/标识符上下文分析 | ~85% |
| 缺少分号 | 语法错误修复 | ~95% |
| = → ==（条件中） | 语义分析 | ~80% |
| 引号不匹配 | 语法错误修复 | ~90% |
| 行末字符丢失 | 语法错误修复 | ~85% |

### 7.2 难以修正的

| OCR 错误 | 原因 | 处理方式 |
|:---|:---|:---|
| **手写体过于潦草** | 完全无法识别字符 | 标记为不确定，用户手动输入 |
| **复杂表达式结构错误** | 缺少多个括号，结构混乱 | 标记为不确定，建议用户重写 |
| **注释内容丢失** | OCR 忽略注释 | 不影响编译，低优先级 |
| **字符串内容错误** | 字符串内字符无法语义验证 | 标记为不确定 |
| **算法逻辑错误** | 代码语法正确但逻辑错误（如排序写错） | 这不是 OCR 错误，属于正常代码错误，由编辑器 QuickFix 处理 |

### 7.3 关键原则

1. **宁可漏修，不可错修**：低置信度修正默认不应用，避免引入更难发现的错误
2. **Diff 预览必须**：所有修正都必须以 Diff 形式展示给用户
3. **一键撤销**：导入后保留原始 OCR 文本，用户可随时对比和撤销
4. **逐步学习**：系统记录用户常接受的修正类型，逐步提高自动应用阈值

---

## 8. 实施计划

### Phase 1: 基础 OCR 导入（快速验证）
- [ ] 集成 Azure/Google Vision API
- [ ] 照片选择和预览界面
- [ ] 原始 OCR 文本直接导入编辑器（无纠错）
- [ ] 用户使用数据收集（常见 OCR 错误类型）

### Phase 2: 智能纠错 MVP
- [ ] 字符混淆映射表
- [ ] 空格恢复引擎
- [ ] 编译器驱动纠错（前 5 个最常见错误模式）
- [ ] 修正确认界面（勾选列表）

### Phase 3: 完善纠错能力
- [ ] 扩展错误模式覆盖（20+ 种 OCR 错误）
- [ ] 上下文感知修正（条件中的 =/==、循环边界等）
- [ ] 置信度模型优化
- [ ] 批量导入支持（多页 PDF）

### Phase 4: 离线 OCR
- [ ] 集成 Tesseract.js 或本地模型
- [ ] 针对代码优化的 OCR 训练
- [ ] 完全离线可用

---

## 9. 总结

### 照片识别的错误能否修正？

**能修正大部分常见 OCR 错误**，成功率约 80-95%。

**核心策略**：
1. **Level 1 预处理**：字符混淆映射 + 空格恢复（解决 60% 问题）
2. **Level 2 编译器驱动**：将 OCR 文本送入编译器，利用语法和语义错误反馈指导修正（解决 30% 问题）
3. **Level 3 用户确认**：低置信度修正由用户确认（兜底 10% 不确定区域）

### 与现有修复系统的关系

| 场景 | 系统 | 修复能力 |
|:---|:---|:---|
| **照片导入时** | OCR 纠错引擎 | 修正 OCR 识别错误（字符混淆、空格丢失、符号遗漏） |
| **编辑器中编写时** | QuickFix 引擎 | 修正语法/逻辑错误（补分号、改边界、加初始化） |
| **运行时执行时** | 诊断引擎 | 诊断运行时错误（越界、空指针），提供修复建议 |

**三者共享**：
- 统一错误码体系
- 统一中文消息模板
- 统一 Diff 预览组件
- 统一 TextEdit 应用机制

**一句话**：OCR 纠错是现有修复系统向"导入场景"的自然延伸，不是独立的系统，而是统一修复引擎的一个输入来源。
