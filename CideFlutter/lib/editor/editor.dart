// Cide Editor Kernel — 自研编辑器内核
//
// 基于 Gesture Proxy 架构：
// - EditableText 作为手势/IME 代理（完全透明）
// - CustomPaint 负责所有可见渲染
// - CideDocument 提供文档模型、选区、Undo/Redo
//
// 使用方式：
// ```dart
// import 'package:cide/editor/editor.dart';
//
// final doc = CideDocument();
// doc.setText('int main() {\n  return 0;\n}');
//
// CideEditor(
//   document: doc,
//   style: TextStyle(fontSize: 14, fontFamily: 'Consolas'),
//   layers: [
//     TextLayer(baseStyle: textStyle),
//     SelectionLayer(),
//     ComposingLayer(),
//   ],
// )
// ```

export 'autocomplete_controller.dart';
export 'autocomplete_overlay.dart';
export 'cide_document.dart';
export 'cide_editor.dart';
export 'diagnostic_layer.dart';
export 'editor_layers.dart';
export 'editor_painter.dart';
export 'syntax_highlight_layer.dart';
