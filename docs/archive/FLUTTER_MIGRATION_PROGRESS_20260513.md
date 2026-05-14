# Flutter 前端迁移推进记录

> 日期：2026-05-13
> 基于：`CODE_REVIEW_COMPREHENSIVE_20260513.md`

---

## 一、本次完成内容（P0 + 部分 P1）

### Rust 后端

| 模块 | 修改内容 | 状态 |
|------|----------|------|
| `compiler/algorithm_detector.rs` | 新建 AST 启发式算法检测器，支持冒泡/选择/插入/快速/归并/二分查找 6 种算法识别 | ✅ |
| `flutter_bridge.rs` | `collect_algorithm_matches` 接入检测器，不再清空数据 | ✅ |
| `capi/mod.rs` | 编译成功后填充 `algorithm_matches` | ✅ |
| `api/cide.rs` | `VisEvent` 新增 `extra0/extra1/extra2` 字段 | ✅ |
| `frb_generated.rs` / Dart 绑定 | 重新生成，Dart 端 `VisEvent` 同步包含 extra 字段 | ✅ |
| `rust_builder/windows/CMakeLists.txt` | 优先使用预构建 DLL，修复 cargokit 依赖缺失导致的 Windows 构建失败 | ✅ |

### Flutter 前端

| 功能 | 文件 | 说明 | 状态 |
|------|------|------|------|
| 指针视图面板 | `ide_screen.dart` | 筛选指针变量，显示 `变量名 → 目标地址（目标名称）` | ✅ |
| 诊断"应用修复"按钮 | `ide_screen.dart` + `ide_provider.dart` | 支持结构化替换 + 启发式修复（补分号/括号/`=`→`==`/`<=`→`<` 等），修复后自动重新编译 | ✅ |
| 算法验证 UI | `ide_screen.dart` + `ide_provider.dart` | 每个算法匹配显示"验证算法"按钮，生成测试用例编译运行，显示通过/失败详情 | ✅ |
| 执行速度滑块 | `ide_screen.dart` + `ide_provider.dart` | 调试模式下工具栏显示 `0~500ms` 滑块，控制单步执行延迟 | ✅ |
| 编辑器 `setText` | `editor_panel.dart` | 添加 `setText` 方法供修复后同步编辑器文本 | ✅ |

### 构建验证

- `cargo check` / `cargo test`：✅ 0 错误，全部测试通过
- `flutter_rust_bridge_codegen generate`：✅ 成功
- `flutter build windows --debug`：✅ 构建成功
- `flutter analyze`（业务代码）：✅ 无严重错误

---

## 二、剩余任务（按优先级）

### P1 — 重要缺失

| 条目 | 模块 | 预估工作量 |
|------|------|----------|
| 链表图可视化（CustomPainter） | Flutter 新增 `widgets/linked_list_viz.dart` | 3–5 天 |
| FRB 暴露 `cide_algorithm_match_vis_event_count/get` | Rust `api/cide.rs` + 重新生成 FRB | 0.5–1 天 |
| VS 风格 Enter 自动格式化 | Flutter `editor_panel.dart` | 1–2 天 |
| 介绍 / 教程覆盖层 | Flutter 新增 `widgets/intro_overlay.dart` | 1–2 天 |

### P2 — 次要缺失

| 条目 | 模块 | 预估工作量 |
|------|------|----------|
| DRY 重构：抽取共享诊断/VM 初始化代码 | Rust 新增 `engine` 模块 | 1–2 天 |
| `once_cell` → `std::sync::LazyLock` | Rust `flutter_bridge.rs` | 0.5 天 |
| 补全 unsafe 安全注释 | Rust 全局 | 0.5 天 |

---

## 三、下一步计划

推进 **P1 链表图可视化**，参考 MAUI 端 `CanvasVisualizer.razor` + `canvas-interop.js` 的实现逻辑，使用 Flutter `CustomPainter` 绘制：
- 矩形节点 + 带箭头的边
- 通过 `VisEvent` 的 `extra` 字段获取节点地址
- 支持节点颜色闪烁（绿/蓝/红）对应 Create/Access/Delete 事件
