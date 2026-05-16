import 'package:flutter/material.dart';
import 'package:cide/src/rust/api/cide.dart' as rust;
import 'memory_map_visualizer.dart';

class MemoryTab extends StatelessWidget {
  final bool isDark;

  const MemoryTab({super.key, required this.isDark});

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<List<rust.MemoryRegion>>(
      future: rust.getMemoryRegions(),
      builder: (context, snapshot) {
        final regions = snapshot.data ?? [];
        if (regions.isEmpty) {
          return Center(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(Icons.memory, size: 40, color: Colors.grey[500]),
                const SizedBox(height: 12),
                Text('无内存信息', style: TextStyle(fontSize: 14, color: Colors.grey[500])),
              ],
            ),
          );
        }
        return MemoryMapVisualizer(regions: regions, isDark: isDark);
      },
    );
  }
}
