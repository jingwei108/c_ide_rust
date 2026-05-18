import 'package:flutter/material.dart';
import 'package:cide/src/rust/api/types.dart' as rust;
import 'memory_map_visualizer.dart';

class MemoryTab extends StatefulWidget {
  final bool isDark;

  const MemoryTab({super.key, required this.isDark});

  @override
  State<MemoryTab> createState() => _MemoryTabState();
}

class _MemoryTabState extends State<MemoryTab> {
  late final Future<Map<String, dynamic>> _memoryFuture;

  @override
  void initState() {
    super.initState();
    _memoryFuture = Future.wait([
      rust.getMemoryRegions(),
      rust.getMemorySize(),
    ]).then((results) => {
      'regions': results[0] as List<rust.MemoryRegion>,
      'memorySize': results[1] as int,
    });
  }

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<Map<String, dynamic>>(
      future: _memoryFuture,
      builder: (context, snapshot) {
        final regions = snapshot.data?['regions'] as List<rust.MemoryRegion>? ?? [];
        final memorySize = snapshot.data?['memorySize'] as int? ?? 1024 * 1024;
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
        return MemoryMapVisualizer(
          regions: regions,
          isDark: widget.isDark,
          memorySize: memorySize,
        );
      },
    );
  }
}
