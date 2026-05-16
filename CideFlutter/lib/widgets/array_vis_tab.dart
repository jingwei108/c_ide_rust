import 'package:flutter/material.dart';
import 'package:cide/src/rust/api/cide.dart' as rust;
import '../widgets/array_visualizer.dart';

class ArrayVisTab extends StatelessWidget {
  final bool isDark;

  const ArrayVisTab({super.key, required this.isDark});

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<List<rust.VariableSnapshot>>(
      future: rust.getVariables(),
      builder: (context, varSnapshot) {
        final vars = varSnapshot.data ?? [];
        // 筛选可能是数组的变量（类型名包含 [ 或名字包含 arr）
        final arrayVars = vars.where((v) {
          return v.tyName.contains('[') || v.name.toLowerCase().contains('arr');
        }).toList();

        if (arrayVars.isEmpty) {
          return Center(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(Icons.bar_chart, size: 40, color: Colors.grey[500]),
                const SizedBox(height: 12),
                Text('未检测到数组变量', style: TextStyle(fontSize: 14, color: Colors.grey[500])),
              ],
            ),
          );
        }

        return ListView.builder(
          padding: const EdgeInsets.all(12),
          itemCount: arrayVars.length,
          itemBuilder: (context, index) {
            final v = arrayVars[index];
            return ArrayVisualizer(
              name: v.name,
              addr: v.addr,
              tyName: v.tyName,
              isDark: isDark,
            );
          },
        );
      },
    );
  }
}
