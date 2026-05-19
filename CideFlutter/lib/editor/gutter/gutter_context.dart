import 'package:cide/src/rust/unified/types.dart' as rust_unified;

/// ---------------------------------------------------------------------------
/// GutterContext — Gutter 渲染所需的共享上下文
/// ---------------------------------------------------------------------------
/// 由 GutterView 构造后传递给各 GutterColumn，避免每列重复计算。
/// ---------------------------------------------------------------------------

class GutterContext {
  final int currentLine; // 1-based，光标所在行
  final int currentDebugLine; // 1-based，调试当前行
  final bool isStepMode;
  final Set<int> breakpoints;
  final Map<int, int> diagMap; // line -> severity
  final List<rust_unified.AccessedVar> accessedVars;
  final rust_unified.HeatmapData? heatmap;
  final bool isDark;

  GutterContext({
    required this.currentLine,
    required this.currentDebugLine,
    required this.isStepMode,
    required this.breakpoints,
    required this.diagMap,
    required this.accessedVars,
    this.heatmap,
    required this.isDark,
  });

  /// 获取指定行的诊断严重级别（取最严重）
  int? severityForLine(int line) => diagMap[line];

  /// 获取指定行的热力图计数
  int heatmapCountForLine(int line) {
    if (heatmap == null) return 0;
    for (final entry in heatmap!.lineCounts) {
      if (entry.$1 == line) return entry.$2.toInt();
    }
    return 0;
  }

  int get heatmapMaxCount {
    final max = heatmap?.maxCount.toInt() ?? 0;
    return max > 0 ? max : 1;
  }
}
