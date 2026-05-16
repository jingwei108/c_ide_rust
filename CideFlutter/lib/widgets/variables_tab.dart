import 'package:flutter/material.dart';
import 'package:cide/src/rust/api/cide.dart' as rust;

class VariablesTab extends StatelessWidget {
  final bool isDark;

  const VariablesTab({super.key, required this.isDark});

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<List<rust.VariableSnapshot>>(
      future: rust.getVariables(),
      builder: (context, snapshot) {
        final vars = snapshot.data ?? [];
        if (vars.isEmpty) {
          return Center(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(Icons.data_object, size: 40, color: Colors.grey[500]),
                const SizedBox(height: 12),
                Text('无变量信息', style: TextStyle(fontSize: 14, color: Colors.grey[500])),
              ],
            ),
          );
        }
        return ListView.builder(
          itemCount: vars.length,
          itemBuilder: (context, index) {
            final v = vars[index];
            return ListTile(
              dense: true,
              title: Row(
                children: [
                  Text(v.name, style: const TextStyle(fontFamily: 'monospace', fontSize: 13)),
                  const SizedBox(width: 8),
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                    decoration: BoxDecoration(
                      color: isDark ? const Color(0xff2a2a2a) : const Color(0xffe5e5e5),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(v.tyName, style: TextStyle(fontSize: 10, color: isDark ? Colors.grey : Colors.black54, fontFamily: 'monospace')),
                  ),
                ],
              ),
              subtitle: Text('值: ${v.value}  地址: 0x${v.addr.toRadixString(16).toUpperCase().padLeft(4, '0')}',
                  style: const TextStyle(fontSize: 11, color: Colors.grey, fontFamily: 'monospace')),
            );
          },
        );
      },
    );
  }
}
