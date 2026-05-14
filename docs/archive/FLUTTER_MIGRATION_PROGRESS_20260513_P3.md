# Flutter 前端迁移推进记录 — P2 DRY 重构 + 依赖升级

> 日期：2026-05-13（第四轮）

---

## 一、本次完成内容

### Rust 后端 — 架构优化

| 修改项 | 说明 | 状态 |
|--------|------|------|
| `engine/compile_pipeline.rs` | 新建共享模块，抽取 `CompileError` trait、`push_diagnostics`/`push_warnings`/`push_hints`、`setup_vm`，消除 `flutter_bridge.rs` 与 `capi/mod.rs` 约 200 行重复代码 | ✅ |
| `flutter_bridge.rs` | 接入 `engine::compile_pipeline`，删除本地重复实现 | ✅ |
| `capi/mod.rs` | 接入 `engine::compile_pipeline`，删除本地重复实现 | ✅ |
| `once_cell` → `LazyLock` | `flutter_bridge.rs` 全局 Session 改用 `std::sync::LazyLock`，移除 `once_cell` 依赖 | ✅ |
| `Cargo.toml` | 移除 `once_cell = "1.19"` | ✅ |
| clippy 警告清理 | 修复未使用导入（`FuncMeta`/`VMSymbol`/`CideVM`）、未使用变量（`init`/`step`）、snake_case 命名（`readMemory`→`read_memory`、`getStructFields`→`get_struct_fields`） | ✅ |
| unsafe 注释 | `engine/compile_pipeline.rs` 的 `setup_vm` 中 `slice::from_raw_parts_mut` 已补充 `// Safety:` 注释 | ✅ |

### 构建验证

- `cargo check`：✅ 0 错误
- `cargo clippy`：✅ 从 ~50 警告降至 ~35 警告（剩余主要为 FRB 生成的 `frb_expand` cfg 警告）
- `cargo test`：✅ 全部通过
- `flutter_rust_bridge_codegen generate`：✅ 成功
- `flutter build windows --debug`：✅ 构建成功

---

## 二、已完成的全部迁移功能汇总

| 优先级 | 功能 | 状态 |
|--------|------|------|
| **P0** | 算法检测器（Rust） | ✅ |
| **P0** | VisEvent extra 字段（FRB） | ✅ |
| **P0** | 指针视图面板 | ✅ |
| **P1** | 诊断"应用修复"按钮 | ✅ |
| **P1** | 算法验证 UI | ✅ |
| **P1** | 链表图可视化（CustomPainter） | ✅ |
| **P1/P2** | 执行速度滑块 | ✅ |
| **P2** | VS 风格 Enter 格式化（缩进+分号补齐） | ✅ |
| **P2** | 介绍/教程覆盖层 | ✅ |
| **P2** | DRY 重构（engine 共享模块） | ✅ |
| **P2** | `once_cell` → `LazyLock` | ✅ |
| **P2** | unsafe 安全注释 + clippy 清理 | ✅ |

### 剩余任务

| 优先级 | 条目 | 模块 | 预估工作量 |
|--------|------|------|----------|
| P1 | FRB 暴露 `cide_algorithm_match_vis_event_count/get` | Rust `api/cide.rs` | 0.5 天 |
| P2 | 触摸滑动切换底部标签 | Flutter `ide_screen.dart` | 0.5 天 |
| P2 | Canvas 通用可视化组件 | Flutter 新增组件 | 1–2 天 |
