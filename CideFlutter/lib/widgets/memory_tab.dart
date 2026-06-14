import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../providers/unified_provider.dart';
import 'memory_map_visualizer.dart';

/// 内存映射 Tab。
///
/// 统一从 [unifiedProvider] 读取内存状态，不再独立调用 Rust FFI，
/// 保证 MemoryTab 与当前执行步骤的内存视图一致。
class MemoryTab extends ConsumerWidget {
  final bool isDark;

  const MemoryTab({super.key, required this.isDark});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final unifiedState = ref.watch(unifiedProvider);
    final regions = unifiedState.memoryRegions;
    final memorySize = unifiedState.memorySize;
    final fragments = unifiedState.memoryFragments;
    final heapStats = unifiedState.heapStats;

    if (regions.isEmpty) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.memory, size: 40, color: Colors.grey[500]),
            const SizedBox(height: 12),
            Text(
              '无内存信息',
              style: TextStyle(fontSize: 14, color: Colors.grey[500]),
            ),
          ],
        ),
      );
    }

    return MemoryMapVisualizer(
      regions: regions,
      fragments: fragments,
      heapStats: heapStats,
      isDark: isDark,
      memorySize: memorySize,
    );
  }
}
