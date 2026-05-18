# 多文件/项目模式实现计划

> 状态：**✅ 已全部实现（2026-05-18）**

## 目标

将 Cide 从单文件编译扩展为多文件合并编译模式，支持 `// @include "other.c"` 语法和前端文件列表管理，实现跨文件的统一符号表与 `static` 作用域隔离。

## 教学价值

- 学生第一次理解"为什么要把代码拆到多个文件"
- 理解"头文件和实现文件的区别"
- 理解 `static` 函数的作用域隔离

## 技术路径

```
当前:  main.c → compile → run
扩展:  main.c + utils.c + sort.c → merge → compile → 统一符号表
```

## 方案：真正的多文件 AST 合并编译

- 每个文件独立 Lexer → Parser → AST
- AST 级别合并（structs/unions/globals/funcs 聚合到单个 ProgramNode）
- TypeChecker 阶段实现 `static` 跨文件隔离
- 诊断信息携带文件名，前端正确渲染多文件错误

---

## 实现阶段

### Phase 1: 后端 AST & Parser 扩展 ✅ 已完成

**文件修改：**
- `native/src/compiler/ast.rs`
  - `FuncDecl` 新增 `is_static: bool`
  - `GlobalDecl` 新增 `is_static: bool`
- `native/src/compiler/parser.rs`
  - `parse_func_decl()`：保留 `static` 标记到 `FuncDecl.is_static`
  - `parse_global_var_or_func()`：保留 `static` 标记到 `GlobalDecl.is_static`

### Phase 2: 多文件编译管线 ✅ 已完成

**新增文件：** `native/src/engine/multi_file.rs`

```rust
pub struct FileRange {
    pub filename: String,
    pub start_line: i32,
    pub end_line: i32,
}

fn merge_compile_units(units: &[CompileUnit]) -> (String, Vec<FileRange>);
fn run_multi_file_pipeline(session: &mut Session, units: Vec<CompileUnit>) -> Result<(), String>;
```

**实现逻辑：**
1. 遍历所有 `CompileUnit`，按顺序拼接为单个 `full_source`
2. 记录每个文件的 `line_offset`，构建 `file_ranges`
3. 调用现有 `run_compile_pipeline` 编译合并后的源码
4. TypeChecker/BytecodeGen 阶段利用 `file_ranges` 实现 static 隔离

**折中方案（推荐）：**
- 为 `FuncDecl` 和 `GlobalDecl` 添加 `source_file: String` 字段
- 每个文件独立 Lexer → Parser → 设置 `source_file` → 合并 AST
- 统一 TypeChecker → BytecodeGen

### Phase 3: TypeChecker static 隔离 ✅ 已完成

**修改文件：** `native/src/compiler/type_checker.rs`

**新增字段：**
```rust
static_funcs: HashMap<String, String>,      // func_name → source_file
static_globals: HashMap<String, String>,    // var_name → source_file
current_file: String,
```

**修改逻辑：**
- Pass 2（注册函数签名）：
  - `f.is_static` → 插入 `static_funcs`
  - 非 static → 插入 `funcs`（保持现有行为）
- Pass 3（检查函数体）：
  - 函数调用时：
    - 先在 `funcs` 查找
    - 如果找不到，检查 `static_funcs`
      - 如果存在且 `current_file != static_funcs[name]`，报 `E3005`
      - 如果存在且相同文件，允许调用
  - 全局变量访问类似处理

**新增错误码：** `native/src/diagnostics/error_codes.rs`
- `E3005_StaticFuncAccess = 3005`："static 函数 '{}' 在其他文件中不可见"
- `E3006_StaticGlobalAccess = 3006`："static 全局变量 '{}' 在其他文件中不可见"

### Phase 4: 诊断增强 ✅ 已完成

**修改文件：** `native/src/session.rs`
- `Diagnostic` 新增 `filename: String`（默认 `"main.c"`）

**修改文件：** `native/src/engine/compile_pipeline.rs`
- `push_one` / `push_diagnostics` / `push_warnings` / `push_hints`：
  - 接收 `file_ranges: &[FileRange]`
  - 根据错误行号推断所属文件名，填充到 `Diagnostic.filename`

### Phase 5: FRB API 扩展 ✅ 已完成

**修改文件：** `native/src/api/cide.rs`
- 新增 `#[frb]` 类型：`CodeFile { filename: String, source: String }`
- 新增 `#[frb] pub fn compile_multi(files: Vec<CodeFile>) -> CompileResult`
- 新增 `#[frb] pub fn compile_and_run_multi(files: Vec<CodeFile>) -> UnifiedRunResult`

**修改文件：** `native/src/flutter_bridge.rs`
- 新增 `compile_multi(files: Vec<CodeFile>) -> CompileResult`
- 新增 `compile_and_run_multi(files: Vec<CodeFile>) -> UnifiedRunResult`
- 内部将 `CodeFile` 转换为 `CompileUnit`，调用 `run_multi_file_pipeline`

**注意：** FRB 代码生成需要运行 `flutter_rust_bridge_codegen generate`。

### Phase 6: 前端状态管理 ✅ 已完成

**修改文件：** `CideFlutter/lib/models/ide_state.dart`
- 新增 `CodeFile` 类：`{ String filename, String source }`
- `IdeState` 修改：
  - 保留 `source`（当前文件内容，向后兼容）
  - 新增 `files: List<CodeFile>`
  - 新增 `currentFile: String`（当前活跃文件名）
- 默认状态：包含一个 `main.c` 文件

**修改文件：** `CideFlutter/lib/providers/ide_notifier.dart`
- 新增方法：
  - `addFile(String filename)`：添加新文件
  - `removeFile(String filename)`：删除文件（至少保留一个）
  - `switchFile(String filename)`：切换当前文件
  - `updateFileSource(String filename, String source)`：更新指定文件内容
- 修改 `compile()`：调用 `rust.compileMulti(files: state.files)`
- 修改 `applyFix()`：根据 `diag.filename` 定位到正确文件

**修改文件：** `CideFlutter/lib/providers/unified_provider.dart`
- 修改 `compileAndRun()`：调用 `rust.compileAndRunMulti(files: state.files)`

### Phase 7: 前端 UI ✅ 已完成

**修改文件：** `CideFlutter/lib/screens/ide_screen.dart`
- 在 `EditorPanel` 上方添加文件标签栏
- 文件标签：水平滚动列表，显示文件名，当前文件高亮
- 操作按钮：
  - "+" 新建文件（弹出对话框输入文件名）
  - 标签上的 "×" 关闭文件
- 文件切换时：保存当前编辑器内容，加载新文件内容

**新增文件：** `CideFlutter/lib/widgets/file_tab_bar.dart`
- 文件标签栏组件
- 支持水平滚动、点击切换、关闭按钮

**修改文件：** `CideFlutter/lib/widgets/editor_panel.dart`
- 支持根据 `currentFile` 重建/切换内容
- `_controller.text` 绑定当前文件内容

### Phase 8: 测试 ✅ 已完成

**新增 E2E 测试：** `native/tests/e2e_multi_file.rs`
- 测试 1：两个文件，main.c 调用 utils.c 的函数，编译运行成功
- 测试 2：utils.c 中定义 static 函数，main.c 调用时报错 `E3005`
- 测试 3：utils.c 中定义 static 全局变量，main.c 访问时报错 `E3006`
- 测试 4：同一文件内的 static 函数调用成功
- 测试 5：跨文件 struct 定义共享

---

## 风险与缓解

1. **FRB 代码生成失败**：`flutter_rust_bridge_codegen` 对新类型（`Vec<CodeFile>`）的支持
   - 缓解：先用简单类型测试，如 `Vec<String>` 文件名 + `Vec<String>` 内容分开传递

2. **诊断行号映射**：多文件合并后行号连续，前端需要正确映射
   - 缓解：`Diagnostic.filename` 字段直接指明文件，前端不需要自己计算

3. **算法验证兼容性**：`validateAlgorithm` 目前使用单文件编译
   - 缓解：算法验证时只使用 `main.c` 的内容（因为测试框架注入自己的 main）

4. **统一模式兼容性**：`UnifiedEngine` 的快照/恢复依赖 VM 状态
   - 缓解：多文件只影响编译阶段，不影响运行时 VM

---

## 工作量估算

| 阶段 | 预计文件数 | 复杂度 |
|------|----------|--------|
| Phase 1: AST/Parser | 2 | 低 |
| Phase 2: 多文件编译管线 | 1 新 + 2 改 | 中 |
| Phase 3: TypeChecker static | 1 | 中 |
| Phase 4: 诊断增强 | 2 | 低 |
| Phase 5: FRB API | 3 | 中 |
| Phase 6: 前端状态 | 3 | 中 |
| Phase 7: 前端 UI | 3 | 中 |
| Phase 8: 测试 | 1 新 | 低 |
| **总计** | **~15 文件** | **高** |

## 实施顺序

1. Phase 1-4（后端核心）
2. Phase 5（FRB 桥接）
3. Phase 6-7（前端 UI）
4. Phase 8（测试）
