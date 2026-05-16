import 'package:flutter/material.dart';
import 'package:cide/src/rust/api/types.dart' as rust;

class CallstackTab extends StatelessWidget {
  const CallstackTab({super.key});

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<List<rust.TraceEntry>>(
      future: rust.getCallstack(),
      builder: (context, snapshot) {
        final entries = snapshot.data ?? [];
        if (entries.isEmpty) {
          return Center(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(Icons.account_tree, size: 40, color: Colors.grey[500]),
                const SizedBox(height: 12),
                Text('调用栈为空', style: TextStyle(fontSize: 14, color: Colors.grey[500])),
              ],
            ),
          );
        }
        return ListView.builder(
          itemCount: entries.length,
          itemBuilder: (context, index) {
            final e = entries[index];
            final isCurrent = index == 0;
            return ListTile(
              dense: true,
              title: Text(e.operation, style: const TextStyle(fontFamily: 'monospace', fontSize: 13)),
              subtitle: Text('返回行 ${e.line}', style: const TextStyle(fontSize: 11, color: Colors.grey)),
              trailing: isCurrent
                  ? Container(
                      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                      decoration: BoxDecoration(
                        color: Colors.blueAccent.withValues(alpha: 0.2),
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: const Text('当前', style: TextStyle(fontSize: 10, color: Colors.blueAccent)),
                    )
                  : null,
            );
          },
        );
      },
    );
  }
}
