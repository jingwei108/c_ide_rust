# 数据结构教材语法支持路线图

> 针对严蔚敏《数据结构》（C 语言版）及其配套习题集、扩展实现所需的 C 语法子集进行全面审计，明确当前编译器与 C 标准的偏差，制定下一阶段的语法拓展计划。

---

## 一、编译器现状深度审计

### 1.1 🔴 与 C 标准存在偏差的语法（影响教材代码正确性）

| # | 语法点 | 当前行为 | 标准 C 行为 | 教材影响 | 优先级 |
|---|--------|---------|------------|---------|--------|
| 1 | **函数参数数组退化** | `void f(int a[5])` 中 `a` 符号表类型仍为 `Array`，`sizeof(a)=20` | 数组参数一律退化为指针，`sizeof(a)=sizeof(void*)` | `sizeof(arr)/sizeof(arr[0])` 在函数内部失效；误导学生 | **P0** |
| 2 | **`unsigned` 全链路语义** | 仅 Parser 保留 `is_unsigned` 标志；TypeChecker 仅发 `W3056` 警告；Codegen/VM 完全按有符号处理 | 无符号运算、无符号比较、逻辑右移 | 哈希表大小计算、位掩码 `&` 操作在边界值时行为错误 | **P0** |
| 3 | **结构体数组嵌套初始化** | `struct S arr[] = {{1,2},{3,4}};` 不支持，Codegen 报错"初始化列表只能在变量声明中使用" | 允许嵌套 `InitList` 作为数组元素 | 图的边数组、顶点信息数组初始化被阻断 | **P0** |
| 4 | **`const` 语义检查** | AST 的 `Type` 带 `is_const` 标志，但 TypeChecker/Codegen 完全不利用 | 禁止对 `const` 变量赋值；禁止通过非 `const` 指针修改 `const` 数据 | 现代 C 代码风格（`const char*` 等）无保护 | **P1** |
| 5 | **`extern` 声明** | Lexer 无 `extern` 关键字，遇到视为普通标识符 | 声明外部符号，不分配存储空间 | 多文件代码组织、头文件模式无法使用 | **P2** |
| 6 | **`goto` / 标号语句** | Lexer 无 `goto` 关键字，AST 无节点，完全未实现 | 无条件跳转 + 标号语句 | 严蔚敏教材采用结构化编程风格，**几乎不使用** | **P3** |

### 1.2 🟡 已实现但文档/边缘 case 有遗漏的语法

| 语法点 | 实现状态 | 问题说明 |
|--------|---------|---------|
| `switch / case / default / break` | ✅ 全链路支持 | AGENTS.md 未提及；fallthrough 行为与 C 一致 |
| `continue` | ✅ 全链路支持 | AGENTS.md 未提及 |
| `do ... while` | ✅ 全链路支持 | AGENTS.md 未提及 |
| `sizeof expr`（无括号） | ✅ 支持 | AGENTS.md 未提及 |
| `enum` 显式赋值 | ✅ 支持 | AGENTS.md 未提及 |
| 数组大小推断 `int arr[] = {...}` | ✅ 支持 | AGENTS.md 未提及 |
| 数组参数退化警告 | ⚠️ 有 `W3052` 警告 | 但退化未真正替换符号表类型，导致 `sizeof` 行为错误 |
| `long` / `short` / `signed` 独立修饰符 | ⚠️ Lexer 识别，Parser 消费后忽略 | 退化为 `int`，无独立语义 |

### 1.3 🟢 完全支持且正确的语法（无需改动）

`if/else`，`while`，`for`（含 C99 变量声明），`return`，`break`，`typedef`，`struct`/`union`（含匿名、嵌套、成员访问 `.`/`->`），`enum`，`sizeof`（类型/表达式），指针算术（含步长缩放），函数指针（含间接调用、结构体成员、typedef、多级），`static` 局部变量，多维数组（声明、初始化、索引、函数参数），复合赋值，三目运算符 `?:`，位运算符 `& | ^ ~ << >>`，短路求值 `&&`/`||`，`#define` 宏（对象宏/参数化宏/嵌套调用），`malloc`/`free`/`realloc`，`qsort`，`printf`/`scanf`/`fprintf`/`fgets`/`fputs` 等。

---

## 二、数据结构教材语法需求分析

### 2.1 严蔚敏教材高频代码模式与当前支持对照

| # | 教材典型代码 | 当前支持 | 问题 |
|---|-------------|---------|------|
| 1 | `typedef struct LNode { ElemType data; struct LNode *next; } LNode, *LinkList;` | ✅ 完全支持 | 无 |
| 2 | `p = (LNode*)malloc(sizeof(LNode));` | ✅ 完全支持 | 无 |
| 3 | `L->next = NULL;` | ✅ 完全支持 | 无 |
| 4 | `for (i = 0; i < sizeof(arr)/sizeof(arr[0]); i++)` | ⚠️ 部分支持 | 若 `arr` 是**函数参数**则失效（返回指针大小 vs 数组大小混淆） |
| 5 | `int *arr = malloc(n * sizeof(int));` | ✅ 完全支持 | VLA `int arr[n];` 也已支持（栈动态分配），教材两种写法均可编译 |
| 6 | `unsigned int size; unsigned mask = size - 1; hash & mask;` | ❌ 不支持 | `unsigned` 按有符号处理，`size=0` 时 `0-1=-1`，`hash & -1` 行为与预期不同 |
| 7 | `const SqList* L;` / `const char* filename;` | ⚠️ 语法支持 | 类型标志保留，但**不阻止修改**，`const` 形同虚设 |
| 8 | `struct Edge edges[] = {{u,v,w}, {u2,v2,w2}};` | ❌ 不支持 | 结构体数组嵌套初始化被阻断 |
| 9 | `extern int global_var;` | ❌ 不支持 | 多文件头文件模式无法编译 |
| 10 | `int arr[][COLS]` 二维数组传参 | ✅ 支持 | 已支持退化 + 第二维指定 |

### 2.2 关键结论

1. **`sizeof` 数组退化方向错误是教学痛点**：
   - 标准 C 中，数组传参后**退化为指针**，`sizeof(arr)` 返回指针大小（4）。
   - 当前编译器**反向偏差**：函数参数中 `sizeof(arr)` 仍返回数组总大小（如 `int arr[5]` 返回 20）。
   - 教材中反复强调"数组传参后 sizeof 失效"，当前行为会**误导学生**形成错误认知。

2. **`unsigned` 看似低频实则关键**：
   - 严蔚敏原版教材中 `unsigned` 出现频率较低（教材使用"类 C 语言"，倾向 `int`）。
   - 但在**配套习题集、扩展实现**（哈希表、位掩码、循环队列标志位）中，`unsigned` 用于容量计算和位运算。
   - 当前 `unsigned` 按有符号处理，导致 `hash & (size-1)` 等操作在边界值时产生与标准 C 明显不符的结果。

3. **结构体数组嵌套初始化是最大功能缺口**：
   - 图的邻接表/邻接矩阵定义大量使用结构体数组初始化。
   - 当前完全不支持，导致此类教材代码无法直接编译运行。

---

## 三、下一阶段完整任务表

### Phase A：P0 语义修复（必须完成）✅ 已完成

#### A1：函数参数数组退化语义修复

| 属性 | 说明 |
|------|------|
| **任务描述** | 函数参数中的 `T arr[N]` / `T arr[]` 在符号表中显式替换为 `T*`；`sizeof(arr)` 返回指针大小 4（或 8，视目标平台）。 |
| **根因分析** | Parser 将 `int arr[5]` 解析为 `Type::Array`；Codegen 仅在栈空间分配时硬编码为 4 字节，但**符号表类型未改**；`sizeof` 读取符号表类型得到 `Array`，故返回总大小。 |
| **涉及模块** | Parser（参数类型解析上下文）、TypeChecker（符号表注册）、Codegen（确认无额外硬编码依赖） |
| **实现要点** | 在 Parser `parse_param()` 或 TypeChecker `check_func_decl()` 中，若参数类型为 `Array`，将其替换为 `Pointer { pointee: element_type, is_const }`。 |
| **验证标准** | `void f(int a[5]) { printf("%d", sizeof(a)); }` 输出 `4`（而非 `20`）。 |
| **工作量** | 中 |

#### A2：`unsigned` 全链路语义支持

| 属性 | 说明 |
|------|------|
| **任务描述** | 实现无符号整数的完整语义：无符号比较、无符号除法/取模、逻辑右移、类型推导保留无符号性。 |
| **根因分析** | Parser 和 AST 保留 `is_unsigned` 标志，但 TypeChecker 二元运算结果类型推导完全忽略该标志；Codegen 比较/除法/右移指令选择只看 `TypeKind`；VM 只有一套有符号 32/64 位指令，`Shr` 为算术右移。 |
| **涉及模块** | TypeChecker（二元运算类型推导、比较/除法/移位类型检查）、Codegen（指令选择）、VM（新增指令 + 执行逻辑） |
| **实现要点** | 1. VM 新增指令：`ULt`, `UGt`, `ULe`, `UGe`, `UDiv`, `UMod`, `LShr`（逻辑右移）<br>2. TypeChecker 二元运算结果类型：若两边均为 `unsigned`，结果保留 `unsigned`<br>3. Codegen 根据 `is_unsigned()` 选择有符号/无符号指令<br>4. `printf`/`scanf` 格式检查识别 `%u`/`%lu` 匹配 `unsigned int`/`unsigned long long` |
| **验证标准** | `unsigned a = 0; a -= 1; if (a > 1) printf("yes");` 应输出 `yes`；`unsigned x = 0xFFFFFFFF; x = x >> 1; printf("%u", x);` 应输出 `2147483647`。 |
| **工作量** | 大 |

#### A3：结构体数组嵌套初始化

| 属性 | 说明 |
|------|------|
| **任务描述** | 支持 `struct S arr[] = {{a,b}, {c,d}};` 形式的嵌套初始化列表。 |
| **根因分析** | Parser 已支持 `Expr::InitList`；TypeChecker 的 `check_array_initializer` 支持扁平初始化列表；但**嵌套 InitList（即 InitList 的元素本身也是 InitList）**在 Codegen 中被拒绝，报错"初始化列表只能在变量声明中使用"。 |
| **涉及模块** | Parser（确认嵌套 InitList 被正确解析为数组元素）、TypeChecker（递归检查嵌套 InitList 与 struct 字段类型匹配）、Codegen（`flatten_init_list` 处理嵌套，按 struct 字段偏移量展开） |
| **实现要点** | 1. Parser：允许数组初始化列表的元素为 `InitList`<br>2. TypeChecker：递归校验每个嵌套 `InitList` 的元素类型与 struct 字段类型一一匹配<br>3. Codegen：`flatten_init_list` 递归展开，遇到 struct 类型时按字段偏移写入 |
| **验证标准** | `struct S { int x, y; }; struct S arr[] = {{1,2}, {3,4}}; printf("%d %d", arr[1].x, arr[1].y);` 输出 `3 4`。 |
| **工作量** | 中 |

### Phase B：P1 质量提升（推荐完成）✅ 已完成

#### B1：`const` 语义检查

| 属性 | 说明 |
|------|------|
| **任务描述** | 实现对 `const` 变量和 `const` 指针目标的写保护。 |
| **涉及模块** | TypeChecker（赋值检查、指针解引用检查） |
| **实现要点** | 1. 禁止对 `const` 变量直接赋值（如 `const int x = 1; x = 2;` 报错）<br>2. 禁止通过非 `const` 指针修改 `const` 数据（如 `const int* p = &x; *p = 1;` 报错）<br>3. 允许 `const int*` → `const int*` 赋值；禁止 `const int*` → `int*` 隐式转换（或至少报 Warning） |
| **验证标准** | 上述两个场景编译期报错，错误码建议 `E3057_ConstViolation`。 |
| **工作量** | 小 |

#### B2：`%u` / `%lu` 格式说明符支持

| 属性 | 说明 |
|------|------|
| **任务描述** | `printf`/`scanf` 支持 `%u`（unsigned int）和 `%lu`（unsigned long long）格式说明符。 |
| **涉及模块** | TypeChecker（printf/scanf 格式字符串静态检查）、VM/host（`printf` 实现读取无符号值） |
| **实现要点** | 1. TypeChecker 格式匹配：将 `%u` 视为匹配 `unsigned int` 类型<br>2. Host `printf` 实现：识别 `%u`，将栈上的 `i32` 按 `u32` 解释输出<br>3. 同理 `%lu` 对应 `unsigned long long` |
| **验证标准** | `unsigned int x = 4294967295; printf("%u", x);` 输出 `4294967295`。 |
| **工作量** | 小 |

#### B3：`extern` 声明支持

| 属性 | 说明 |
|------|------|
| **任务描述** | 支持 `extern` 全局变量/函数声明。 |
| **涉及模块** | Lexer（新增 `extern` Token）、Parser（识别 `extern` 声明）、TypeChecker（标记外部符号，不分配存储空间） |
| **实现要点** | 1. Lexer `keyword_type` 添加 `"extern" => Some(TokenType::Extern)`<br>2. Parser `parse_global_decl()` 中识别 `extern`，生成带 `is_extern: true` 的 `GlobalDecl`/`FuncDecl`<br>3. TypeChecker/Codegen：对外部符号不进行存储分配，仅注册类型信息供链接/调用检查 |
| **验证标准** | `extern int global_var; int main() { return global_var; }` 编译通过（运行时若无定义则链接错误）。 |
| **工作量** | 小 |

#### B4：`sizeof` 数组退化警告增强

| 属性 | 说明 |
|------|------|
| **任务描述** | 在函数参数中使用 `sizeof(arr)` 时，发出专门的教学友好型警告。 |
| **涉及模块** | TypeChecker（`sizeof` 表达式检查） |
| **实现要点** | 当 `sizeof` 的 operand 是函数参数且类型为 `Pointer`（退化后的数组参数）时，发出提示："数组参数已退化为指针，sizeof 结果为指针大小，而非数组总大小。" |
| **验证标准** | 编译 `void f(int a[5]) { sizeof(a); }` 时，输出 Warning 并解释原因。 |
| **工作量** | 小 |

### Phase C：P2 预处理器拓展（条件成熟再做）

| # | 任务 | 涉及模块 | 工作量 | 说明 |
|---|------|---------|--------|------|
| C1 | 条件编译 `#ifdef`/`#ifndef`/`#if`/`#else`/`#elif`/`#endif` | Lexer/预处理器 | 大 | 实现预处理器条件栈；支持宏定义存在性判断；C 语言头文件防重复包含的基础 |
| C2 | `#include` 文件包含 | Lexer/预处理器 | 大 | 支持 `#include "file.h"`；处理包含路径、递归包含、循环包含检测 |

### Phase D：P3 低优先级（教材几乎不用）

| # | 任务 | 涉及模块 | 说明 |
|---|------|---------|------|
| D1 | `goto` / 标号语句 | 全链路 | 严蔚敏教材采用结构化编程风格，**几乎不使用** `goto` |
| D2 | VLA 变长数组 `int arr[n];` | ✅ **已完成** | 教材使用 `malloc` 动态分配替代；VLA 在 C11 中已变为可选特性，但已实现以覆盖更多代码模式 |
| D3 | `stdarg.h` 变长参数函数 | Parser, Codegen | 教材仅 `InitArray` 等极少数函数使用，可通过固定参数或数组替代 |

---

## 四、技术方案要点

### 4.1 A1：数组退化修复

```rust
// 关键修改点：Parser 或 TypeChecker 中，函数参数上下文
fn normalize_param_type(ty: &mut Type) {
    if let Type::Array { element, is_const, .. } = ty {
        *ty = Type::Pointer {
            pointee: element.clone(),
            is_const: *is_const,
        };
    }
}
```

- 在 Parser `parse_param()` 返回前调用，或在 TypeChecker `check_func_decl()` 注册符号前调用。
- 确保 Codegen 中**不再**依赖 `p.ty.is_array()` 硬编码分配 4 字节，而是统一使用 `type_size(&p.ty)`（此时 `Pointer` 的 `type_size` 已为 4）。

### 4.2 A2：`unsigned` 全链路支持

```rust
// VM 新增指令枚举
ULt, UGt, ULe, UGe,     // 无符号比较
UDiv, UMod,             // 无符号除法/取模
LShr,                   // 逻辑右移（unsigned >> n）

// VM 执行示例
OpCode::ULt => {
    let b = self.pop() as u32;
    let a = self.pop() as u32;
    self.push(if a < b { 1 } else { 0 });
}
OpCode::LShr => {
    let b = self.pop() as u32;
    let a = self.pop() as u32;
    self.push((a >> b) as i32);
}
```

- TypeChecker 二元运算类型推导：`unsigned OP unsigned -> unsigned`，`unsigned OP signed -> unsigned`（C 标准中的通常算术转换）。
- Codegen 中统一封装 `emit_cmp(op, is_unsigned)` 辅助函数，根据类型自动选择 `Lt`/`ULt` 等。

### 4.3 A3：结构体数组嵌套初始化

```rust
// TypeChecker：递归校验嵌套 InitList
fn check_nested_init_list(
    &mut self,
    init: &Expr,
    target_ty: &Type,  // 期望类型（如 struct S）
    loc: &SourceLoc,
) -> Result<(), Error> {
    match init {
        Expr::InitList { elements, .. } => {
            if target_ty.is_struct() {
                let fields = self.get_struct_fields(target_ty);
                for (i, elem) in elements.iter().enumerate() {
                    if let Some(field_ty) = fields.get(i) {
                        self.check_nested_init_list(elem, field_ty, loc)?;
                    } else {
                        self.report_error("初始化元素过多", loc, E3015);
                    }
                }
            } else if target_ty.is_array() {
                // ... 类似处理
            }
        }
        _ => { /* 单一表达式，检查类型兼容 */ }
    }
    Ok(())
}
```

- Codegen 中 `flatten_init_list` 需要递归展开：遇到 struct 类型时，遍历其字段定义，计算每个字段的偏移量，将嵌套 `InitList` 的元素按偏移写入目标内存区域。

---

## 五、推荐执行顺序

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  第一阶段：P0 核心修复                                                        │
│  ├── A1：函数参数数组退化语义修复                                              │
│  └── A3：结构体数组嵌套初始化                                                  │
│         （两项可并行，彼此独立）                                               │
├─────────────────────────────────────────────────────────────────────────────┤
│  第二阶段：P0 语义补完                                                        │
│  └── A2：unsigned 全链路语义支持                                               │
│         （工作量最大，涉及 VM 指令集扩展，建议单独阶段）                        │
├─────────────────────────────────────────────────────────────────────────────┤
│  第三阶段：P1 质量提升                                                        │
│  ├── B1：const 语义检查                                                       │
│  ├── B2：%u / %lu 格式说明符                                                  │
│  ├── B3：extern 声明支持                                                      │
│  └── B4：sizeof 数组退化警告增强                                               │
│         （四项均为小工作量，可集中完成）                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│  第四阶段：P2 预处理器                                                        │
│  ├── C1：条件编译 #ifdef / #ifndef / #endif                                   │
│  └── C2：#include 文件包含                                                    │
│         （根据教学场景中多文件代码组织的需求迫切程度决定是否启动）               │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 六、与 AGENTS.md 的联动

本路线图中的 **Phase A + Phase B + Phase D(VLA)** 已完成，已在 `AGENTS.md` 中同步更新以下条目：

- **新增支持**：函数参数数组退化语义、`unsigned` 无符号语义、`const` 语义检查、`%u`/`%lu` 格式说明符、`extern` 声明、结构体数组嵌套初始化、**VLA 变长数组**、**函数按值返回结构体**、**多级指针**。
- **文档补录**：`switch/case/default/break`、`continue`、`do...while`、`sizeof expr`（无括号）、`enum` 显式赋值、数组大小推断——这些已支持但 AGENTS.md 未提及的语法应补充到"已支持的关键特性"章节。
- **已知限制更新**：`goto`、`stdarg.h` 明确不实现；VLA 已实现但边界检查、`sizeof(VLA类型)` 暂不支持。
