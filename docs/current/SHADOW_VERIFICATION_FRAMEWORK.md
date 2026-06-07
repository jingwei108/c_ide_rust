# Cide 影子验证框架（Shadow Verification）

> 目的：用 Clang 作为"影子"编译器，数据驱动自研编译器扩展决策。

---

## 一、核心思想

**不是让差距永远靠降级填补，而是缩小子集差距。**

传统 CI 双轨验证回答的是："已实现的特性对不对？"
影子验证回答的是："下一步该实现什么特性？"

两者分层协作：
```
CI 流水线
    │
    ├─ 影子验证（收集缺失特性数据）
    │      └─ 输出：《缺失特性频率报告》→ 驱动 ROADMAP 优先级
    │
    └─ 双轨对比（验证已支持特性正确性）
           └─ 输出：《语义偏差报告》→ 驱动 Bug 修复
```

---

## 二、使用方法

### 2.1 运行验证

```bash
cd native/tests/shadow_verification
python shadow_verify.py
```

**前置条件**：
- `clang` 已安装并在 PATH 中
- `native/target/release/cide_native.dll` 已编译：`cd native && cargo build --release`

**输出**：
- `reports/shadow_report_YYYYMMDD_HHMMSS.md`：人类可读报告
- `reports/shadow_data_YYYYMMDD_HHMMSS.json`：机器可读数据

### 2.2 添加新用例

用例优先以独立 `.c` 文件形式存放，支持热加载：

```bash
# baseline 用例（Cide 应支持）
native/tests/cases/baseline/<name>.c

# gap 用例（Cide 暂不支持）
native/tests/cases/gap/<name>.c

# 模板生成的用例（从 templates/ 自动同步）
native/tests/cases_template_generated/<name>.c
```

`.c` 文件头部可选添加分类注释：
```c
// @category: baseline
int main() { ... }
```

`shadow_verify.py` 启动时会自动扫描上述目录加载用例；加载失败时 fallback 到硬编码 `SHADOW_CASES` 列表。

**分类命名规范**：
- `baseline`：Cide 已支持的特性（用于验证基准）
- `double` / `function_pointer` / `file_io` 等：预期缺失的特性

### 2.3 解读报告

报告输出三个关键指标：

| 指标 | 含义 |
|------|------|
| **完全匹配** | Clang ≡ Cide，该特性已完整支持 |
| **编译缺口** | Clang 编译通过，Cide 编译失败 → **缺失特性或 Bug** |
| **输出差异** | 两者都编译运行，但 stdout 不同 → **语义偏差** |

**缺失特性频率排序**按影响用例数从高到低排列，直接指导扩展优先级。

---

## 三、数据驱动扩展流程

```
        ┌─────────────────┐
        │   收集标准 C    │
        │   测试用例      │
        └────────┬────────┘
                 │
                 ▼
        ┌─────────────────┐
        │  跑影子验证     │
        │  (Clang vs Cide)│
        └────────┬────────┘
                 │
       ┌─────────┴─────────┐
       ▼                   ▼
┌─────────────┐    ┌─────────────┐
│ 编译缺口    │    │ 完全匹配    │
│ (40%+)      │    │ (50%+)      │
└──────┬──────┘    └──────┬──────┘
       │                  │
       ▼                  ▼
┌─────────────┐    ┌─────────────┐
│ 分类统计    │    │ 双轨对比    │
│ 按频率排序  │    │ 确保无回归  │
└──────┬──────┘    └─────────────┘
       │
       ▼
┌─────────────┐
│ 确定 Top 3  │
│ 缺失特性    │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ 进入开发    │
│ 实现特性    │
└─────────────┘
```

---

## 四、与 CI 双轨验证的关系

| 维度 | 影子验证 | CI 双轨验证 |
|------|---------|------------|
| **目标** | 发现尚未支持的特性 | 发现已支持特性的语义偏差 |
| **对比对象** | Clang 编译成功但 Cide 失败 | VM 输出 vs Clang 输出 |
| **数据用途** | 指导编译器扩展优先级 | 修复 VM Bug |
| **频率** | 每次用例库迭代（每周） | 每次 commit/PR |
| **输出** | 缺失特性频率排序 | pass/fail + 差异报告 |

**比喻**：
- 影子验证 = 分析考卷发现"哪些知识点还没学过，哪些最常考"
- 双轨验证 = 检查"已学过的知识点有没有答错"

---

## 五、Shadow 用例 C 源码字符串转义规范

Shadow 用例的 C 源码以 Python 单引号字符串形式存储。由于 Python 字符串本身会解析转义序列，若不注意会导致传给 Cide 的源码被截断或变形，产生**假性编译/运行时缺口**。

### 5.1 常见陷阱

| Python 源码写法 | Python 解析后 | Cide 收到 | 后果 |
|----------------|--------------|----------|------|
| `'...\n...'` | 实际换行符 `0x0A` | 字符串跨行 | `E1003` 字符串跨行 |
| `'...\0...'` | NUL 字符 `0x00` | 字符串截断 | `E1002` 字符字面量未闭合 |
| `'...\t...'` | Tab 字符 `0x09` | 制表符 | 可能不影响，但列号偏移 |

**根因**：`python -c "..."` 中的反斜杠还会被 Shell 二次处理，导致本地调试与文件直接执行的观察结果不一致。

### 5.2 正确写法

在 `shadow_verify.py` 的单引号 Python 字符串中，C 转义序列必须写成**双反斜杠**：

```python
# ✅ 正确：Cide 收到的是 C 源码 \n（换行转义序列）
ShadowCase("hanoi_tower",
    '...printf("Move 1 from %c to %c\\n", from, to);...',
    "baseline")

# ✅ 正确：Cide 收到的是 C 源码 \0（NUL 字符字面量）
ShadowCase("string_reverse",
    '...while (str[len] != \'\\0\') len++;...',
    "baseline")

# ❌ 错误：Python 会把 \n 解析为实际换行符
'...printf("...\n", ...);...'

# ❌ 错误：Python 会把 \0 解析为 NUL，截断整个字符串
'...while (str[len] != \'\0\')...'
```

**验证方法**：用 `python check_escapes.py`（已内置在仓库）扫描全部用例，确保没有包含 NUL / 换行 / Tab 的异常解析。

### 5.3 已修复的历史事故

- **2026-06-06**：修复 14 个用例的转义问题，包括 `string_reverse`（`\0`）、`hanoi_tower`（`\n`）、`circular_queue`/`hash_table_linear`/`josephus_ring` 等（`\n`）。修复前产生 1 个假性编译缺口、1 个假性 match(both fail)。
- **2026-06-06（本次发现）**：为"数据结构教材算法模板体系扩展"新增的 8 个用例（`parametric_macro_max`/`swap`/`square`/`nested`、`static_local_counter`/`init_once`/`array`、`fgets_fputs_basic`）同样存在 `\n` 转义陷阱——Python 字符串中的 `\n` 被解析为字面量 `\` + `n`，导致 `#define` 与后续代码、函数定义之间缺少实际换行。结果 Clang 与 Cide 均编译失败，产生**假性 match（both fail）**。此前 AGENTS.md 记录"全部与 Clang 输出匹配"系基于错误观察。真实状态待修复 `\n` → `\n` 后重新验证。

---

## 六、已知限制

1. **测试用例维度**：当前 274 个用例覆盖有限，需扩展至 300+ 才能更准确反映学生代码模式
2. **分类精度**：`classify_compile_error` 基于错误消息关键词匹配，可能存在误分类
3. **平台依赖**：框架依赖本地 Clang，Windows 上需 MSVC 运行时；Android/iOS 无法直接运行
4. **输出对比**：仅对比 stdout，不对比 stderr、返回值、内存状态

---

## 七、历史报告

| 日期 | 用例数 | 完全匹配 | 编译缺口 | 关键发现 |
|------|--------|---------|---------|---------|
| 2026-05-17 | 45 | 26 (58%) | 18 (40%) | 发现 2 个 baseline Bug + 1 个 printf 精度差异 |
| 2026-05-17 | 61 | 48 (78%) | 13 (21%) | union / long_long 已移回 baseline；新增 16 个用例覆盖 switch/do-while/递归/字符串库等 |
| 2026-05-17 | 79 | 66 (83%) | 13 (16%) | 新增 18 个 baseline 用例覆盖 cast/复合赋值/全局结构体/宿主函数/十六进制/字符字面量/块注释 |
| 2026-05-17 | 103 | 90 (87%) | 13 (12%) | 新增 24 个 baseline 用例覆盖 short/signed/unsigned/前置自增/逻辑运算/取模/负数/void函数/指针比较/结构体赋值/数组参数退化/枚举算术/嵌套调用/指针差值等 |
| 2026-05-17 | 120 | 107 (89%) | 13 (10%) | 新增 17 个 baseline 用例；**发现第 4 个真实 bug：八进制字面量 `077` 被误解析为十进制 77** → Lexer 修复 |
| 2026-05-17 | 234 | 219 (93%) | 13 (5%) | 扩展至 234 用例；新增 114 个 baseline 覆盖算法/控制流/指针/结构体/数组等；**发现第 5 个 bug：`&&`/`||` 无短路求值** → BytecodeGen 修复；2 个已知问题待修复（循环变量作用域、字符串存储） |
| 2026-06-06 | 274 | 266 (97%) | 6 (2%) | 扩展至 274 用例（新增 43 个算法/数据结构模板用例）；**修复 Shadow 框架 Python 字符串转义陷阱**（`\n`/`\0` 被 Python 解析为实际换行/NUL，导致 14 个假性失败）；**发现第 6 个真实 bug：BytecodeGen 对复杂左值赋值的副作用重复执行**（`queue[rear++] = v` 中 `rear++` 执行两次）→ `gen_assign` 中 Index/Deref/Member 分支改为保存地址临时变量，避免重新求值左值表达式 |
| 2026-06-06 | 295 | 282 (95%) | 8 (2%) | 为 **函数按值返回结构体 / 多级指针 cast / Phase 27 语法拓展 / VLA** 新增 21 个影子验证用例；将 `variable_length_array` 从缺失特性移回 baseline（VLA 已支持）；`sizeof_array_param` 标记为 `arch_diff_bug`（32位 VM 指针大小为 4，非 bug）；**发现 2 个真实问题**：(1) `0xFFFFFFFFU` 不被支持 — Lexer 将 `0xFFFFFFFF` 按有符号 int 解析导致溢出，且不支持 `U` 后缀；(2) `extern int foo(int);` 函数原型声明不被 Parser 接受 — 报"预期 struct、函数或全局变量声明" |
| 2026-06-06 | 295 | 275 (93%) | 8 (2%) | **算法模板与验证解耦**：82 个 Dart 硬编码模板全部文件化为 `templates/<key>/source.c` + `meta.yaml`；295 个影子用例全部文件化到 `cases/baseline/` 和 `cases/gap/`；`shadow_verify.py` 改造为从文件热加载；新增 `scripts/sync_templates.py` 一键同步模板 → 测试用例 → Golden → Flutter assets；新增 `native/tests/cide_e2e.rs` Rust e2e 测试（baseline 275 + template 74 全绿，8 已知失败监控）；**修复全局二维数组嵌套初始化子元素大小计算错误**（`flatten_global_init` 中 `elem_type_size` 递归取最内层类型导致 `int[2][3]` 子数组大小算成 4 而非 12）；Flutter 端完成运行时加载骨架（`TemplateLoader` + `FutureBuilder`），`explanations` 与 `focusLines` 通过从 Dart 硬编码提取实现数据完整性 |
| 2026-06-06 | 295 | 285 (97%) | 5 (1%) | **修复影子验证发现的 2 个真实问题**：(1) Lexer 按 C 标准实现整数常量类型推导 — 支持 `U`/`u` 后缀、八进制/十六进制超出有符号 int 范围自动提升为 `unsigned int`（`0xFFFFFFFF` → `UnsignedLiteral`）；(2) Parser 修复 `extern int foo(int);` 纯原型声明不消费分号导致后续解析失败的 bug；`unsigned_cmp_wrap`/`unsigned_lshr`/`extern_func` 3 个用例从 gap 移回 baseline；剩余 5 个编译缺口均为明确不支持的特性（`goto`/`static_assert`/`typeof`/`designated_initializer`/`inline_asm`） |
| 2026-06-07 | 295 | 286 (97%) | 4 (1%) | **P0 语法拓展**：通用逗号运算符 `a, b`、Designated Initializer `.field = val` / `[i] = val`、`offsetof(struct S, field)` 全管线实现；`designated_init.c` 从 gap 移回 baseline；修复 Parser `extra_vars`/`for-init` 错误调用 `parse_expression()`（会解析逗号运算符）导致多变量声明失败的问题；剩余 4 个编译缺口（`goto`/`static_assert`/`typeof`/`inline_asm`） |
| 2026-06-07 | 449 | 396 (88%) | 14 (3%) | **运行完整影子验证（baseline + gap + template + K&R 全量）**：新增 `comma_operator.c`/`offsetof_struct.c` baseline 用例，`designated_init.c` 从 gap 移回 baseline；编译缺口降至 14（`unknown`×10 / `goto`×1 / `inline_asm`×1 / `static_assert`×1 / `typeof`×1），`designated_initializer` 已彻底从缺口列表中移除；运行时缺口 27 个（主要为 K&R `getchar` 交互式输入差异 + 模板已知失败） |

---

*框架位置：`native/tests/shadow_verification/shadow_verify.py`*
