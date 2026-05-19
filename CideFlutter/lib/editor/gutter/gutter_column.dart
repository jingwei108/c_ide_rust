import 'package:flutter/material.dart';
import 'gutter_context.dart';

/// ---------------------------------------------------------------------------
/// GutterColumn — Gutter 列插件接口
/// ---------------------------------------------------------------------------
/// 每个 GutterColumn 负责一类信息的渲染（如行号、热力图、断点等）。
/// GutterView 负责按需（懒加载）调用 buildCell。
/// ---------------------------------------------------------------------------

abstract class GutterColumn {
  /// 列宽度（固定）
  double get width;

  /// 构建指定行的单元格。
  /// [line] 为 1-based 行号。
  Widget buildCell(BuildContext context, int line, GutterContext ctx);
}
