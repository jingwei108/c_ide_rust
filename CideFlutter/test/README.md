# CideFlutter 测试说明

本目录包含 CideFlutter 的单元测试与 Widget 测试，集成测试位于 `../integration_test/`。

详细测试框架文档请参见：

- [docs/current/FLUTTER_TESTING.md](../../docs/current/FLUTTER_TESTING.md)

## 快速开始

```bash
# 运行全部单元/widget 测试
flutter test

# 运行单个目录
flutter test test/editor/
flutter test test/widgets/
flutter test test/providers/

# 运行单个文件
flutter test test/editor/cide_document_test.dart
```

## 目录结构

- `editor/`：编辑器内核测试
- `models/`：数据模型测试
- `providers/`：Riverpod Provider/Notifier 测试
- `services/`：服务层测试
- `widgets/`：UI 组件测试
- `helpers/`：测试辅助函数与工厂
- `mocks/`：Mock 类
