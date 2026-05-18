import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../providers/unified_provider.dart';
import 'array_visualizer.dart';

class ArrayVisTab extends ConsumerWidget {
  final bool isDark;

  const ArrayVisTab({super.key, required this.isDark});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final unifiedState = ref.watch(unifiedProvider);
    final frameCache = unifiedState.frameCache;
    final currentStep = unifiedState.currentStep;

    if (frameCache.isEmpty || currentStep < 0 || currentStep >= frameCache.length) {
      return _buildEmpty('运行程序以查看数组');
    }

    final payload = frameCache[currentStep];
    final arraySnapshots = payload.arraySnapshots;

    if (arraySnapshots.isEmpty) {
      return _buildEmpty('未检测到数组变量');
    }

    // 解析 vis_events 中的比较上下文，高亮对应元素
    final highlighted = _parseHighlightedIndices(payload.visEvents);
    // 解析语义标签中的交换索引
    final swapped = _parseSwappedIndices(payload.semanticLabel);

    return ListView.builder(
      padding: const EdgeInsets.all(12),
      itemCount: arraySnapshots.length,
      itemBuilder: (context, index) {
        final snap = arraySnapshots[index];
        return ArrayVisualizer(
          name: snap.name,
          elementTy: snap.elementTy,
          elements: snap.elements,
          highlightedIndices: highlighted[snap.name] ?? const {},
          swappedIndices: swapped[snap.name] ?? const {},
          isDark: isDark,
        );
      },
    );
  }

  Widget _buildEmpty(String message) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(Icons.bar_chart, size: 40, color: Colors.grey[500]),
          const SizedBox(height: 12),
          Text(message, style: TextStyle(fontSize: 14, color: Colors.grey[500])),
        ],
      ),
    );
  }

  /// 解析 VisEvent.context 如 "arr[i]:arr[i+1]" → { "arr": {i, i+1} }
  Map<String, Set<int>> _parseHighlightedIndices(List<dynamic> visEvents) {
    final result = <String, Set<int>>{};
    for (final ev in visEvents) {
      final ctx = ev.context as String? ?? '';
      if (ctx.isEmpty) continue;
      // 格式: "arr[i]:arr[j]" 或 "arr[i]"
      final parts = ctx.split(':');
      for (final part in parts) {
        final match = RegExp(r'(\w+)\[(\d+)\]').firstMatch(part.trim());
        if (match != null) {
          final arrName = match.group(1)!;
          final idx = int.tryParse(match.group(2)!) ?? -1;
          if (idx >= 0) {
            result.putIfAbsent(arrName, () => <int>{}).add(idx);
          }
        }
      }
    }
    return result;
  }

  /// 解析语义标签中的交换信息，如 "交换 arr[2]↔arr[3]" → { "arr": {2, 3} }
  Map<String, Set<int>> _parseSwappedIndices(String semanticLabel) {
    final result = <String, Set<int>>{};
    if (semanticLabel.isEmpty) return result;

    // 匹配 "交换 arr[i]↔arr[j]"
    final swapMatch = RegExp(r'交换\s+(\w+)\[(\d+)\]↔\w+\[(\d+)\]').firstMatch(semanticLabel);
    if (swapMatch != null) {
      final arrName = swapMatch.group(1)!;
      final i = int.tryParse(swapMatch.group(2)!) ?? -1;
      final j = int.tryParse(swapMatch.group(3)!) ?? -1;
      if (i >= 0 && j >= 0) {
        result[arrName] = {i, j};
      }
    }
    return result;
  }
}
