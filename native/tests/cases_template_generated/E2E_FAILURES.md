# Template Generated E2E 失败记录

> 生成时间：每次 `cargo test --test cide_e2e` 运行后自动对比 Golden 输出。
> 原则：不修改模板源码来迎合测试，只记录根因与分类。

## 当前失败列表（10 / 82）

### bTree_default
- **现象**: Runtime error — 访问了 NULL 指针区域（地址 0x0010）
- **根因**: B 树插入/分裂逻辑访问了未初始化的子节点指针。模板代码未在创建节点时将 `children[]` 数组全部初始化为 NULL。
- **分类**: 模板代码缺陷（未初始化指针）
- **是否 Cide 限制**: 否，标准 C 编译器执行也会因未定义行为崩溃（虽然 Clang 可能因内存布局巧合不崩溃）

### bellmanFord_default
- **现象**: Parser 错误 E2005 — 预期分号（第7行 `int u, v, w;`）
- **根因**: 待确认。可能与 `struct Edge { ... }` 后的解析状态有关，或函数名 `BellmanFord` 大写开头的标识符解析问题。
- **分类**: 待分类
- **是否 Cide 限制**: 待确认

### bfs_default
- **现象**: 输出不匹配 — Cide 输出 `"0 1 2"`，Golden `"0 1 2 3 4"`
- **根因**: ✅ **已修复**（2026-06-06）。Cide `codegen/mod.rs` 的 `flatten_global_init` 在处理多维数组嵌套初始化时，子元素大小计算错误。`elem_type_size(target_ty)` 通过 `base_kind` 递归取最内层类型，导致 `int[2][3]` 的子元素大小被算成 4（`int`）而非 12（`int[3]`）。修复：改用 `type_size(&inner_ty)`。
- **分类**: Cide 编译器缺陷（全局数组初始化）
- **是否 Cide 限制**: 否

### dfs_default
- **现象**: 输出不匹配 — Cide 输出 `"0 1 2"`，Golden `"0 1 3 4 2"`
- **根因**: ✅ **已修复**（2026-06-06）。同 `bfs_default`，全局二维数组嵌套初始化子元素大小计算错误。
- **分类**: Cide 编译器缺陷（全局数组初始化）
- **是否 Cide 限制**: 否

### binarySearchTreeValidation_default
- **现象**: `INT_MIN`/`INT_MAX` 未声明（E3023）；`isValidBST` 第 2/3 参数类型不匹配（E3038）
- **根因**: Cide 不支持 `<limits.h>` 的 `INT_MIN`/`INT_MAX` 宏；`long long` 类型作为函数参数可能未被 TypeChecker 正确处理。
- **分类**: ✅ **已修复**（2026-06-07）。新增 `<limits.h>` 支持，`INT_MIN`/`INT_MAX` 已预定义。
- **是否 Cide 限制**: 否

### dfs_default
- **现象**: 输出不匹配 — Cide 输出 `"0 1 2"`，Golden `"0 1 3 4 2"`
- **根因**: 同 `bfs_default`，待确认。
- **分类**: 待分类

### infixEvaluation_default
- **现象**: `isdigit` 未声明（E3023）；对非函数指针类型进行调用（E3045）
- **根因**: Cide 不支持 `<ctype.h>` 的 `isdigit` 函数。模板代码中 `isdigit(ch)` 被当作未声明标识符，后续解析错误。
- **分类**: Cide 已知限制 + 模板依赖未支持头文件
- **是否 Cide 限制**: ✅ 是。`<ctype.h>` 不在 AGENTS.md 支持列表中。

### polynomialAdd_default
- **现象**: 逻辑运算类型错误 E3019 — 逻辑运算要求两边都是 int 或 float
- **根因**: `while (pa && pb)` 中 `pa`/`pb` 为 `struct Term*` 指针。Cide TypeChecker 不支持指针到 bool 的隐式转换。
- **分类**: Cide 语义差异
- **是否 Cide 限制**: ✅ 是。Cide 暂不支持指针作为逻辑运算操作数（标准 C 允许）。

### redBlackTree_default
- **现象**: 逻辑运算类型错误 E3019（多处）
- **根因**: 同 `polynomialAdd_default`，`while (z->parent && ...)` 中指针参与 `&&` 运算。
- **分类**: Cide 语义差异
- **是否 Cide 限制**: ✅ 是。

### spfa_default
- **现象**: 同 `bellmanFord_default`，Parser 错误 E2005
- **根因**: 待确认（同 bellmanFord）。
- **分类**: 待分类

### threadedBinaryTree_default
- **现象**: Parser 错误 E2005 — `typedef enum { Link, Thread } PointerTag;` 无法解析
- **根因**: Cide Parser 不支持 `typedef enum { ... } Name;` 的合并声明语法。
- **分类**: Cide 语法限制
- **是否 Cide 限制**: ✅ 是。`typedef enum` 合并声明不在 AGENTS.md 支持列表中。

---

## 记录规范

新增失败时按以下格式追加到本文件顶部（保持时间倒序）：

```markdown
### <case_name>
- **现象**: 
- **根因**: 
- **分类**: 模板代码缺陷 / Cide 已知限制 / Cide 语义差异 / Cide 语法限制 / 待分类
- **是否 Cide 限制**: 是 / 否 / 待确认
```
