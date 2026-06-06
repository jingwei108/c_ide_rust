# 算法模板与验证解耦方案

> 状态：Phase 1~4 已实施完成，旧 Dart 硬编码模板框架已移除  
> 核心原则：**算法模板即合法 C 代码** + **语法验证双重验证参照** + **自验证降低维护成本**

---

## 一、问题背景

### 1.1 算法模板与前端耦合

~~当前 82 个算法模板硬编码在 `CideFlutter/lib/models/templates/*.dart` 中~~（**已移除**）。

新框架下：

- 所有模板源码为合法 C 代码，存放于 `templates/<key>/source.c`
- 运行时通过 `TemplateLoader` 从 `assets/templates/index.json` + `.c` 加载
- 影子验证直接复用 `native/tests/cases_template_generated/*.c`
- 前端替换时仅需复制 `assets/templates/` 目录

### 1.2 语法验证重复劳动

同一个 `for` 循环用例当前维护多份：

| 位置 | 形式 | 问题 |
|------|------|------|
| Rust 单元测试 | C 字符串 | Parser/TypeChecker/VM 各测一遍 |
| Python `SHADOW_CASES` | Python 字符串 | 同一份代码再写一遍，且有 `\n` 转义陷阱 |
| 算法模板 Dart 文件 | Dart 字符串 | 算法代码第三遍 |

### 1.3 隐形成本（本次设计重点解决）

| 隐形成本 | 根因 | 后果 |
|---------|------|------|
| 模板无法直接编译检查 | `{{n}}` 不是合法 C | 作者无法立即用 Clang 验证语法 |
| expected_stdout 静默过期 | 与源码示例数据分离 | 改数据后 expected 不匹配 |
| Tutorial 锚点漂移 | phase 与源码行号弱关联 | 重构后高亮错位 |

---

## 二、核心设计

### 2.1 算法模板：合法 C + 注释参数

**占位符改为合法 C 注释标记**，模板本身就是可编译的 C 代码。

```c
// templates/bubble_sort/source.c
#include <stdio.h>

void bubbleSort(int arr[], int n) {
    // @tutorial-anchor: outer_loop
    for (int i = 0; i < n - 1; i++) {
        // @tutorial-anchor: inner_loop
        for (int j = 0; j < n - i - 1; j++) {
            if (arr[j] > arr[j + 1]) {
                int temp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
        }
    }
}

int main() {
    int arr[/*__PARAM_n__*/ 5] = {5, 3, 8, 1, 2};
    int expected[/*__PARAM_n__*/ 5] = {1, 2, 3, 5, 8};
    int n = /*__PARAM_n__*/ 5;

    bubbleSort(arr, n);

    int ok = 1;
    for (int i = 0; i < n; i++) if (arr[i] != expected[i]) ok = 0;
    printf(ok ? "OK\n" : "FAIL\n");
    return 0;
}
```

**关键规则**：

| 元素 | 语法 | 说明 |
|------|------|------|
| 参数占位 | `/*__PARAM_n__*/ 5` | 合法 C 注释，后接默认值，可被替换 |
| 教程锚点 | `// @tutorial-anchor: outer_loop` | 源码中的显式标记，重构时跟着代码走 |
| 自验证 | `expected[]` + 循环比较 | 模板自身断言正确性，输出稳定 `"OK\n"` |

**替换逻辑**：`sync_templates.py` 扫描 `/*__PARAM_{key}__*/\s*(\S+)`，将匹配到的整段替换为目标值。

**验证方式**：模板作者随时可 `clang templates/bubble_sort/source.c` 编译运行，确认语法和逻辑正确。

### 2.2 元数据：`templates/<key>/meta.yaml`

纯声明式，不重复源码中已存在的信息：

```yaml
key: bubble
name: 冒泡排序
category: 排序

params:
  n:
    label: 数组长度
    type: int
    default: 5
    mutations:
      - name: small
        value: 2
      - name: large
        value: 10

tutorial:
  steps:
    - title: 外层循环
      description: 控制趟数，每趟将最大元素沉底
      anchor: outer_loop       # 对应 source.c 中的 @tutorial-anchor 标记
    - title: 内层循环
      description: 相邻元素比较
      anchor: inner_loop

knowledge_nodes:
  - Array
  - BubbleSort
  - BoundaryCondition
```

### 2.3 双重验证参照

| 维度 | Rust e2e 集成测试 | Python 影子验证 |
|------|------------------|----------------|
| **参照系** | Golden `.out` 文件（Clang 生成并锁定） | Clang 实时输出 |
| **验证目标** | Cide 行为是否符合锁定预期 | Cide 行为是否与标准 C 一致 |
| **发现问题** | Cide 自身 bug / Golden 过期 | 缺失特性 / 语义偏差 / 标准不符 |
| **运行方式** | `cargo test --test cide_e2e` | `python shadow_verify.py` |
| **依赖** | 仅需 Cide 自身 | 需 Clang |
| **反馈速度** | 秒级 | 分钟级 |

**交叉验证场景**：

| Rust e2e | Python 影子 | 结论 |
|---------|------------|------|
| ✅ | ✅ | 健康 |
| ❌ | ❌ | 预期值/Golden 写错，或两者都对但输出不同 |
| ✅ | ❌ | **Cide 行为自洽但与标准 C 偏差** |
| ❌ | ✅ | **Golden 文件过期**，需重新生成 |
| - | compile_gap | **缺失特性或编译器 bug** |

---

## 三、目录结构

```
project_root/
│
├── templates/                          # 算法模板（人类维护）
│   ├── bubble_sort/
│   │   ├── source.c                    # 合法 C，含 /*__PARAM_n__*/ 和 @tutorial-anchor
│   │   └── meta.yaml                   # 元数据 + 教程声明
│   ├── quick_sort/
│   │   └── ...
│   └── ...
│
├── native/
│   ├── tests/
│   │   ├── cases/                      # 语法验证用例（单一事实来源）
│   │   │   ├── baseline/               # 已支持特性
│   │   │   │   ├── hello_world.c
│   │   │   │   ├── for_loop.c
│   │   │   │   └── ...
│   │   │   └── gap/                    # 已知缺失特性（可选）
│   │   │       └── goto_basic.c
│   │   ├── cases_golden/               # 模板 Golden 输出（CI 生成+锁定）
│   │   │   ├── bubble_sort_default.out
│   │   │   └── ...
│   │   ├── cases_template_generated/   # CI 从 templates/ 自动生成
│   │   │   ├── bubble_sort_default.c
│   │   │   └── ...
│   │   └── shadow_verification/
│   │       ├── shadow_verify.py        # 影子验证框架
│   │       └── reports/                # 输出报告
│   └── src/...
│
├── CideFlutter/
│   └── assets/templates/               # Flutter 打包
│       ├── index.json                  # sync_templates.py 生成
│       └── bubble_sort.c               # 按需加载的模板源码
│
└── scripts/
    └── sync_templates.py               # 核心同步脚本
```

---

## 四、文件格式规范

### 4.1 算法模板：`templates/<key>/source.c`

```c
#include <stdio.h>

void bubbleSort(int arr[], int n) {
    // @tutorial-anchor: outer_loop
    for (int i = 0; i < n - 1; i++) {
        for (int j = 0; j < n - i - 1; j++) {
            if (arr[j] > arr[j + 1]) {
                int temp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
        }
    }
}

int main() {
    int arr[/*__PARAM_n__*/ 5] = {5, 3, 8, 1, 2};
    int expected[/*__PARAM_n__*/ 5] = {1, 2, 3, 5, 8};
    int n = /*__PARAM_n__*/ 5;

    bubbleSort(arr, n);

    int ok = 1;
    for (int i = 0; i < n; i++) if (arr[i] != expected[i]) ok = 0;
    printf(ok ? "OK\n" : "FAIL\n");
    return 0;
}
```

### 4.2 语法验证用例：`native/tests/cases/baseline/*.c`

```c
// @category: baseline
// @features: for_loop, array_index, printf
// @expected_stdout: 0 1 2
int main() {
    for (int i = 0; i < 3; i++) printf("%d ", i);
    return 0;
}
```

| 注释标记 | 用途 | 消费者 |
|---------|------|--------|
| `// @category:` | baseline / gap / arch_diff_bug | Python shadow, Rust e2e |
| `// @features:` | 特性标签（逗号分隔） | 特性矩阵生成 |
| `// @expected_stdout:` | Rust e2e 断言预期 | Rust e2e |

### 4.3 Golden 输出文件：`cases_golden/<name>.out`

纯文本，无注释，就是 stdout 的精确字节：

```
ative/tests/cases_golden/bubble_sort_default.out
---
OK
```

生成方式：`sync_templates.py` 先用 Clang 渲染并运行，stdout 写入 `.out`，首次需人工 Review 后提交锁定。

---

## 五、消费端实现

### 5.1 `sync_templates.py`（核心同步脚本）

```python
#!/usr/bin/env python3
"""
职责：
1. 渲染模板 source.c → cases_template_generated/*.c（替换 /*__PARAM_*/）
2. 用 Clang 运行生成 Golden .out → cases_golden/
3. 生成 Flutter JSON Index → CideFlutter/assets/templates/index.json
"""

import re
import yaml
import subprocess
from pathlib import Path

PARAM_RE = re.compile(r'/\*__PARAM_(\w+)__\*/\s*(\S+)')

def render_template(source: str, args: dict) -> str:
    def repl(m):
        key = m.group(1)
        return args.get(key, m.group(2))
    return PARAM_RE.sub(repl, source)

def scan_tutorial_anchors(source: str) -> dict:
    """扫描 // @tutorial-anchor: name，返回 {name: line_number}"""
    anchors = {}
    for i, line in enumerate(source.splitlines(), 1):
        m = re.search(r'@tutorial-anchor:\s*(\w+)', line)
        if m:
            anchors[m.group(1)] = i
    return anchors

def sync():
    tpl_dir = Path("templates")
    gen_dir = Path("native/tests/cases_template_generated")
    golden_dir = Path("native/tests/cases_golden")
    flutter_dir = Path("CideFlutter/assets/templates")
    
    gen_dir.mkdir(parents=True, exist_ok=True)
    golden_dir.mkdir(parents=True, exist_ok=True)
    flutter_dir.mkdir(parents=True, exist_ok=True)
    
    index = {"templates": []}
    
    for d in tpl_dir.iterdir():
        if not d.is_dir():
            continue
        
        source_c = (d / "source.c").read_text()
        meta = yaml.safe_load((d / "meta.yaml").read_text())
        anchors = scan_tutorial_anchors(source_c)
        
        # 生成 Flutter JSON Index
        index["templates"].append({
            "key": meta["key"],
            "name": meta["name"],
            "category": meta["category"],
            "params": meta.get("params", {}),
            "tutorialAnchors": anchors,
            "knowledgeNodes": meta.get("knowledge_nodes", []),
        })
        
        # 复制源码到 Flutter assets（按需加载）
        (flutter_dir / f"{meta['key']}.c").write_text(source_c)
        
        # 生成 shadow 用例 + Golden
        for sc in meta.get("shadow_cases", []):
            rendered = render_template(source_c, sc["args"])
            case_name = f"{meta['key']}_{sc['name']}"
            
            # 写入生成的 .c
            c_path = gen_dir / f"{case_name}.c"
            lines = [f"// @category: {sc.get('category', 'baseline')}"]
            lines.append(rendered)
            c_path.write_text("\n".join(lines))
            
            # 生成/更新 Golden .out（仅当 .out 不存在时）
            out_path = golden_dir / f"{case_name}.out"
            if not out_path.exists():
                out = run_with_clang(rendered)  # 调用 Clang 编译运行
                out_path.write_text(out.stdout)
                print(f"[NEW GOLDEN] {out_path}")
    
    # 写入 Flutter Index
    (flutter_dir / "index.json").write_text(
        json.dumps(index, ensure_ascii=False, indent=2)
    )
    
    print(f"Synced {len(index['templates'])} templates")

def run_with_clang(source: str) -> subprocess.CompletedProcess:
    ...  # 复用 shadow_verify.py 中的 clang 运行逻辑

if __name__ == "__main__":
    sync()
```

### 5.2 Rust e2e 集成测试

```rust
// native/tests/cide_e2e.rs

use std::fs;
use std::path::Path;

struct TestCase {
    name: String,
    source: String,
    expected_stdout: String,
    category: String,
}

fn load_cases(dir: &str) -> Vec<TestCase> {
    let mut cases = vec![];
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().unwrap_or_default() != "c" {
            continue;
        }
        let source = fs::read_to_string(&path).unwrap();
        let expected = extract_comment_tag(&source, "@expected_stdout");
        let category = extract_comment_tag(&source, "@category")
            .unwrap_or_else(|| "baseline".into());
        
        cases.push(TestCase {
            name: path.file_stem().unwrap().to_string_lossy().into(),
            source: source.clone(),
            expected_stdout: expected.unwrap_or_default(),
            category,
        });
    }
    cases
}

fn load_golden_cases(cases_dir: &str, golden_dir: &str) -> Vec<TestCase> {
    let mut cases = vec![];
    for entry in fs::read_dir(cases_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().unwrap_or_default() != "c" {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let golden_path = Path::new(golden_dir).join(format!("{}.out", name));
        if !golden_path.exists() {
            continue;  // 无 Golden 则跳过（或报错）
        }
        let source = fs::read_to_string(&path).unwrap();
        let expected = fs::read_to_string(&golden_path).unwrap();
        let category = extract_comment_tag(&source, "@category")
            .unwrap_or_else(|| "baseline".into());
        
        cases.push(TestCase { name, source, expected_stdout: expected, category });
    }
    cases
}

#[test]
fn e2e_baseline_cases() {
    for case in load_cases("tests/cases/baseline") {
        if case.category == "gap" { continue; }
        let output = run_cide(&case.source);
        assert_eq!(output.stdout.trim(), case.expected_stdout.trim(),
            "[{}] stdout mismatch", case.name);
    }
}

#[test]
fn e2e_template_cases() {
    for case in load_golden_cases(
        "tests/cases_template_generated",
        "tests/cases_golden"
    ) {
        let output = run_cide(&case.source);
        assert_eq!(output.stdout.trim(), case.expected_stdout.trim(),
            "[{}] golden mismatch", case.name);
    }
}
```

### 5.3 Python 影子验证

`shadow_verify.py` **完整保留**现有 `analyze_diff` 四分类逻辑，仅简化用例加载：

```python
def load_case_files() -> List[ShadowCase]:
    """从 .c 文件加载用例，替代硬编码 SHADOW_CASES 列表"""
    cases = []
    for root in [
        Path("tests/cases/baseline"),
        Path("tests/cases_template_generated")
    ]:
        if not root.exists():
            continue
        for path in root.glob("*.c"):
            source = path.read_text(encoding="utf-8")
            meta = parse_comment_tags(source)
            cases.append(ShadowCase(
                name=path.stem,
                source=source,
                category=meta.get("category", "baseline")
            ))
    return cases

SHADOW_CASES = load_case_files()

# analyze_diff、classify_compile_error、generate_report 完全保留原有逻辑
```

### 5.4 Flutter 前端

运行时加载 `assets/templates/index.json` + 按需加载 `assets/templates/<key>.c`：

```dart
class TemplateRepository {
  static Future<List<CodeTemplate>> loadAll() async {
    final idx = await rootBundle.loadString('assets/templates/index.json');
    final data = jsonDecode(idx);
    // 解析 params, tutorialAnchors, knowledgeNodes
  }
  
  static Future<String> loadSource(String key) async {
    return rootBundle.loadString('assets/templates/$key.c');
  }
}
```

**高亮逻辑**：`source.c` 中的 `// @tutorial-anchor: outer_loop` 注释位置即高亮行号，无需额外维护。

---

## 六、特性矩阵生成（可选增值）

从 `// @features:` 注释自动生成特性支持矩阵：

```python
# scripts/feature_matrix.py
from collections import defaultdict

features = defaultdict(lambda: {"cases": 0, "match": 0})
for case in load_case_files():
    for feat in case.features:
        features[feat]["cases"] += 1
        if case.diff_type == "match":
            features[feat]["match"] += 1

# 输出：for_loop: 15/15 (100%), array_index: 12/12 (100%), ...
```

教学价值：直接回答"Cide 支持哪些 C 语法特性"。

---

## 七、迁移路径

### Phase 1：文件化影子用例 ✅

1. ✅ 创建 `native/tests/cases/baseline/`
2. ✅ 将 `shadow_verify.py` 中 baseline 用例逐个提取为 `.c` 文件
3. ✅ 每个 `.c` 文件添加 `// @category` 和 `// @expected_stdout` 注释
4. ✅ `shadow_verify.py` 改为 `load_case_files()` 扫描目录
5. ✅ 验证：跑一次 `python shadow_verify.py`，match 率不下降

### Phase 2：模板文件化 + sync 脚本 ✅

1. ✅ 创建 `templates/<key>/source.c`（合法 C + `/*__PARAM__*/`）+ `meta.yaml`
2. ✅ 实现 `scripts/sync_templates.py`：
   - 渲染 `/*__PARAM__*/` → `cases_template_generated/*.c`
   - Clang 运行生成 `cases_golden/*.out`
   - 生成 `CideFlutter/assets/templates/index.json`
3. ✅ 验证 Rust e2e 和 Python 影子都能加载新生成的模板用例

### Phase 3：Rust e2e 集成测试 ✅

1. ✅ 新建 `native/tests/cide_e2e.rs`
2. ✅ 实现 `load_cases()` 扫描 `cases/baseline/`
3. ✅ 实现 `load_golden_cases()` 扫描 `cases_template_generated/` + `cases_golden/`
4. ✅ `cargo test --test cide_e2e` 跑通全部 baseline + 模板用例

### Phase 4：Flutter 运行时加载 + 删除旧框架 ✅

1. ✅ `pubspec.yaml` 注册 `assets/templates/`
2. ✅ `TemplateBar` / `TemplateParamDialog` / `TemplateTutorialPanel` 改为消费 JSON Index + `.c` 文件
3. ✅ 删除 `CideFlutter/lib/models/templates/*.dart` 硬编码模板文件（11 个 Dart 文件已移除）
4. ✅ `template_registry.dart` 清理：移除 `allTemplates` fallback 及全部旧 import
5. ✅ `code_template.dart` 移除旧语法 `{{key:defaultValue}}` 支持，仅保留 `/*__PARAM__*/`
6. ✅ `@tutorial-anchor` 扫描驱动高亮

---

## 八、与现有系统兼容性

| 现有系统 | 影响 | 措施 |
|---------|------|------|
| `shadow_verify.py` 报告格式 | 无 | `analyze_diff` / `generate_report` 逻辑不变 |
| `SHADOW_CASES` 硬编码列表 | 删除 | 改为 `load_case_files()` 扫描 `.c` |
| Rust 单元测试（分层） | 无 | `lexer/tests.rs` / `parser/tests.rs` 等保持原样 |
| `CodeTemplate` Dart API | 微调 | 数据字段不变，加载来源从代码变为文件 |
| `template_registry.dart` | 删除 | 改为运行时加载 JSON Index |
| 教程高亮 | 增强 | 从行号绑定升级为 `@tutorial-anchor` 源码标记 |

---

## 九、风险与回退

| 风险 | 缓解措施 |
|------|---------|
| `/*__PARAM_n__*/` 默认值与替换值类型不匹配 | `sync_templates.py` 渲染后先调 Clang 编译，失败则阻断 |
| Golden `.out` 首次生成错误 | 必须人工 Review 后提交锁定，CI 检查 `.out` 变更需审批 |
| `@tutorial-anchor` 注释遗漏 | `sync_templates.py` 校验 meta.yaml 中的 anchor 必须在 source.c 中存在 |
| Flutter 运行时加载性能 | Index JSON 仅几 KB，`.c` 文件按需加载，无性能问题 |
| Windows 无 Clang | Python 影子标记为可选 CI 步骤；Rust e2e 为必过项 |

---

## 十、总结

> **算法模板是合法 C 代码（`/*__PARAM__*/` 注释占位），作者可直接用 Clang 验证；模板自身断言正确性（`expected[]` 自验证），输出稳定 `"OK\n"`，Golden 文件由 Clang 生成并锁定；同一份 `.c` 文件，Rust e2e 以 Golden 为参照验证 Cide 自洽性，Python 影子以 Clang 实时输出为参照验证标准一致性——双重独立，交叉验证，Python 不降级。**
