import 'package:flutter/material.dart';
import 'package:cide/src/rust/api/cide.dart' as rust;

/// 内存区域类型颜色
class _MemoryColors {
  static const stack = Color(0xFF0A84FF);
  static const heap = Color(0xFFFF9F0A);
  static const global = Color(0xFF32D74B);
  static const code = Color(0xFFFF453A);
  static const free = Color(0xFF3A3A3C);
  static const trap = Color(0xFFBF5AF2);
}

/// 内存映射可视化组件
///
/// 将 256KB 内存划分为 64 个 4KB 块，以网格形式展示各区域的占用情况。
/// 点击某块可查看该范围内所有内存区域的详细信息。
class MemoryMapVisualizer extends StatelessWidget {
  final List<rust.MemoryRegion> regions;
  final bool isDark;

  const MemoryMapVisualizer({
    super.key,
    required this.regions,
    this.isDark = false,
  });

  Color _getRegionColor(rust.MemoryRegion region) {
    if (region.isFreed) return _MemoryColors.free.withValues(alpha: 0.3);
    if (region.addr < 64) return _MemoryColors.trap;
    if (region.isHeap) return _MemoryColors.heap;
    if (region.addr < 0x1000) return _MemoryColors.stack;
    if (region.name.contains('global') || region.name.contains('static')) {
      return _MemoryColors.global;
    }
    return _MemoryColors.code;
  }

  String _getRegionLabel(rust.MemoryRegion region) {
    if (region.name.isNotEmpty) return region.name;
    if (region.isHeap) return '堆';
    return '区域';
  }

  void _showBlockDetails(BuildContext context, int blockIndex, List<rust.MemoryRegion> blockRegions) {
    const blockSize = 4096;
    final addr = blockIndex * blockSize;
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(12)),
      ),
      builder: (ctx) {
        return DraggableScrollableSheet(
          initialChildSize: 0.45,
          minChildSize: 0.3,
          maxChildSize: 0.8,
          expand: false,
          builder: (_, scrollController) {
            return Column(
              children: [
                Container(
                  padding: const EdgeInsets.symmetric(vertical: 12),
                  child: Container(
                    width: 40,
                    height: 4,
                    decoration: BoxDecoration(
                      color: Colors.grey,
                      borderRadius: BorderRadius.circular(2),
                    ),
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: Row(
                    children: [
                      Expanded(
                        child: Text(
                          '块 #${blockIndex + 1} (0x${addr.toRadixString(16).toUpperCase().padLeft(5, '0')})',
                          style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
                        ),
                      ),
                      Text(
                        '${blockRegions.length} 个区域',
                        style: const TextStyle(fontSize: 12, color: Colors.grey),
                      ),
                    ],
                  ),
                ),
                const SizedBox(height: 8),
                const Divider(height: 1),
                Expanded(
                  child: blockRegions.isEmpty
                      ? const Center(child: Text('该块暂无占用区域', style: TextStyle(color: Colors.grey)))
                      : ListView.builder(
                          controller: scrollController,
                          padding: const EdgeInsets.symmetric(vertical: 8),
                          itemCount: blockRegions.length,
                          itemBuilder: (context, i) {
                            final r = blockRegions[i];
                            final color = _getRegionColor(r);
                            return ListTile(
                              dense: true,
                              leading: Container(
                                width: 12,
                                height: 12,
                                decoration: BoxDecoration(
                                  color: color,
                                  borderRadius: BorderRadius.circular(3),
                                ),
                              ),
                              title: Text(
                                r.name.isEmpty ? '(未命名)' : r.name,
                                style: const TextStyle(fontSize: 14),
                              ),
                              subtitle: Text(
                                '地址: 0x${r.addr.toRadixString(16).toUpperCase().padLeft(5, '0')} · 大小: ${r.size}B · 类型: ${r.ty.isEmpty ? '未知' : r.ty}',
                                style: const TextStyle(fontSize: 11, color: Colors.grey),
                              ),
                              trailing: Row(
                                mainAxisSize: MainAxisSize.min,
                                children: [
                                  if (r.isHeap)
                                    const Padding(
                                      padding: EdgeInsets.only(right: 4),
                                      child: Text('堆', style: TextStyle(fontSize: 10, color: Colors.orangeAccent)),
                                    ),
                                  if (r.isFreed)
                                    const Text('已释放', style: TextStyle(fontSize: 10, color: Colors.grey)),
                                ],
                              ),
                            );
                          },
                        ),
                ),
              ],
            );
          },
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    const memorySize = 256 * 1024; // 256KB
    const blockSize = 4096; // 每块 4KB
    const blockCount = memorySize ~/ blockSize; // 64 块

    // 构建块颜色和标签映射
    final blockColors = List<Color>.filled(blockCount, _MemoryColors.free);
    final blockLabels = List<String?>.filled(blockCount, null);
    final blockRegions = List<List<rust.MemoryRegion>>.generate(blockCount, (_) => []);

    for (final region in regions) {
      final startBlock = region.addr ~/ blockSize;
      final endBlock = (region.addr + region.size) ~/ blockSize;
      final color = _getRegionColor(region);
      final label = _getRegionLabel(region);
      for (var i = startBlock; i <= endBlock && i < blockCount; i++) {
        blockColors[i] = color;
        if (blockLabels[i] == null) {
          blockLabels[i] = label;
        }
        blockRegions[i].add(region);
      }
    }

    return LayoutBuilder(
      builder: (context, constraints) {
        const cols = 8;

        return Column(
          children: [
            // 图例
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              child: Wrap(
                spacing: 12,
                runSpacing: 4,
                children: [
                  _LegendItem(color: _MemoryColors.stack, label: '栈'),
                  _LegendItem(color: _MemoryColors.heap, label: '堆'),
                  _LegendItem(color: _MemoryColors.global, label: '全局'),
                  _LegendItem(color: _MemoryColors.code, label: '代码/数据'),
                  _LegendItem(color: _MemoryColors.trap, label: 'NULL陷阱'),
                  _LegendItem(color: _MemoryColors.free.withValues(alpha: 0.3), label: '空闲/已释放'),
                ],
              ),
            ),
            const SizedBox(height: 8),
            // 内存网格
            Expanded(
              child: GridView.builder(
                padding: const EdgeInsets.all(8),
                gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
                  crossAxisCount: cols,
                  crossAxisSpacing: 2,
                  mainAxisSpacing: 2,
                ),
                itemCount: blockCount,
                itemBuilder: (context, index) {
                  final addr = index * blockSize;
                  final color = blockColors[index];
                  final label = blockLabels[index];
                  return GestureDetector(
                    onTap: () => _showBlockDetails(context, index, blockRegions[index]),
                    child: Tooltip(
                      message: label != null
                          ? '0x${addr.toRadixString(16).toUpperCase().padLeft(5, '0')} - $label'
                          : '0x${addr.toRadixString(16).toUpperCase().padLeft(5, '0')} (空闲)',
                      child: Container(
                        decoration: BoxDecoration(
                          color: color,
                          borderRadius: BorderRadius.circular(2),
                          border: Border.all(
                            color: isDark ? Colors.white12 : Colors.black12,
                            width: 0.5,
                          ),
                        ),
                        child: Center(
                          child: Text(
                            '${index + 1}',
                            style: TextStyle(
                              fontSize: 9,
                              color: color.computeLuminance() > 0.5 ? Colors.black : Colors.white,
                            ),
                          ),
                        ),
                      ),
                    ),
                  );
                },
              ),
            ),
          ],
        );
      },
    );
  }
}

class _LegendItem extends StatelessWidget {
  final Color color;
  final String label;

  const _LegendItem({required this.color, required this.label});

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: 10,
          height: 10,
          decoration: BoxDecoration(
            color: color,
            borderRadius: BorderRadius.circular(2),
          ),
        ),
        const SizedBox(width: 4),
        Text(label, style: const TextStyle(fontSize: 10, color: Colors.grey)),
      ],
    );
  }
}
