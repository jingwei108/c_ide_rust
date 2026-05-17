import 'package:flutter/material.dart';

/// 面板位置
enum PanelLocation { bottom, floating }

/// 面板元素定义（共 11 个）
class PanelItem {
  final String id;
  final String label;
  final IconData icon;

  const PanelItem._({required this.id, required this.label, required this.icon});

  // 底部默认 3 个
  static const output = PanelItem._(id: 'output', label: '输出', icon: Icons.terminal);
  static const diagnostics = PanelItem._(id: 'diagnostics', label: '诊断', icon: Icons.error_outline);
  static const algorithm = PanelItem._(id: 'algorithm', label: '算法', icon: Icons.auto_fix_high);

  // 悬浮球 8 个
  static const knowledge = PanelItem._(id: 'knowledge', label: '知识卡片', icon: Icons.menu_book);
  static const pointer = PanelItem._(id: 'pointer', label: '指针视图', icon: Icons.polyline);
  static const arrayVis = PanelItem._(id: 'arrayVis', label: '数组可视化', icon: Icons.bar_chart);
  static const memory = PanelItem._(id: 'memory', label: '内存区域', icon: Icons.memory);
  static const variables = PanelItem._(id: 'variables', label: '局部变量', icon: Icons.data_object);
  static const watch = PanelItem._(id: 'watch', label: '监视变量', icon: Icons.visibility);
  static const callstack = PanelItem._(id: 'callstack', label: '调用栈', icon: Icons.account_tree);
  static const progress = PanelItem._(id: 'progress', label: '学习进度', icon: Icons.trending_up);
  static const varHistory = PanelItem._(id: 'varHistory', label: '变量历史', icon: Icons.show_chart);
  static const breakpoints = PanelItem._(id: 'breakpoints', label: '断点', icon: Icons.stop_circle);

  /// 全部 13 个面板元素
  static const List<PanelItem> all = [
    output, diagnostics, algorithm,
    knowledge, pointer, arrayVis, memory, variables, watch, callstack, progress, varHistory, breakpoints,
  ];

  /// 根据 id 查找
  static PanelItem? fromId(String id) {
    for (final p in all) {
      if (p.id == id) return p;
    }
    return null;
  }

  @override
  bool operator ==(Object other) => other is PanelItem && other.id == id;

  @override
  int get hashCode => id.hashCode;
}

/// 面板槽位（携带位置信息）
class PanelSlot {
  final String panelId;
  final PanelLocation location;

  const PanelSlot({required this.panelId, required this.location});

  PanelItem? get item => PanelItem.fromId(panelId);

  PanelSlot copyWith({String? panelId, PanelLocation? location}) {
    return PanelSlot(
      panelId: panelId ?? this.panelId,
      location: location ?? this.location,
    );
  }
}
