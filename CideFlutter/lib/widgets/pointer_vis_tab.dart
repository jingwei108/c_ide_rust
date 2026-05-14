import 'package:flutter/material.dart';
import 'package:cide/src/rust/api/cide.dart' as rust;
import '../visualizers/linked_list_visualizer.dart';

class PointerVisTab extends StatelessWidget {
  final bool isDark;

  const PointerVisTab({super.key, required this.isDark});

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<List<rust.VariableSnapshot>>(
      future: rust.getVariables(),
      builder: (context, snapshot) {
        final vars = snapshot.data ?? [];
        const nullTrapEnd = 64;
        const linearMemorySize = 256 * 1024;
        final pointers = vars.where((v) {
          final val = v.value;
          return v.tyName.contains('*') &&
              val > nullTrapEnd &&
              val < linearMemorySize;
        }).toList();

        // 查找链表头节点（struct Node* 类型）
        final headVars = pointers.where((v) {
          return v.tyName.toLowerCase().contains('struct') &&
              v.tyName.toLowerCase().contains('node');
        }).toList();

        if (pointers.isEmpty && headVars.isEmpty) {
          return const Center(child: Text('未检测到指针变量', style: TextStyle(color: Colors.grey)));
        }

        return Column(
          children: [
            // 链表可视化区域
            if (headVars.isNotEmpty)
              FutureBuilder<List<rust.VisEvent>>(
                future: rust.getVisEvents(),
                builder: (context, visSnapshot) {
                  final visEvents = visSnapshot.data ?? [];
                  return SizedBox(
                    height: 120,
                    child: ListView.builder(
                      scrollDirection: Axis.horizontal,
                      padding: const EdgeInsets.symmetric(horizontal: 12),
                      itemCount: headVars.length,
                      itemBuilder: (context, idx) {
                        final hv = headVars[idx];
                        return Container(
                          margin: const EdgeInsets.only(right: 16),
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Padding(
                                padding: const EdgeInsets.symmetric(vertical: 4),
                                child: Text(
                                  '${hv.name} (${hv.tyName})',
                                  style: const TextStyle(fontSize: 11, color: Colors.grey, fontFamily: 'monospace'),
                                ),
                              ),
                              Expanded(
                                child: LinkedListVisualizer(
                                  headAddr: hv.value,
                                  structName: 'Node',
                                  visEvents: visEvents,
                                  isDark: isDark,
                                ),
                              ),
                            ],
                          ),
                        );
                      },
                    ),
                  );
                },
              ),
            // 指针列表
            Expanded(
              child: ListView.builder(
                padding: const EdgeInsets.all(12),
                itemCount: pointers.length,
                itemBuilder: (context, index) {
                  final p = pointers[index];
                  String targetName = '';
                  final targetAddr = p.value;
                  for (final v in vars) {
                    if (v.addr == targetAddr) {
                      targetName = v.name;
                      break;
                    }
                  }
                  return Container(
                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                    decoration: BoxDecoration(
                      border: Border(
                        bottom: BorderSide(color: Theme.of(context).dividerColor.withValues(alpha: 0.1)),
                      ),
                    ),
                    child: Row(
                      children: [
                        const Icon(Icons.arrow_forward, size: 16, color: Colors.blueAccent),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(
                                p.name,
                                style: TextStyle(
                                  fontSize: 13,
                                  fontFamily: 'monospace',
                                  color: isDark ? const Color(0xffd4d4d4) : const Color(0xff333333),
                                ),
                              ),
                              const SizedBox(height: 2),
                              Text(
                                '0x${p.addr.toRadixString(16).toUpperCase().padLeft(4, '0')} → 0x${targetAddr.toRadixString(16).toUpperCase().padLeft(4, '0')} ${targetName.isNotEmpty ? '($targetName)' : ''}',
                                style: const TextStyle(fontSize: 11, color: Colors.grey, fontFamily: 'monospace'),
                              ),
                            ],
                          ),
                        ),
                        Container(
                          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                          decoration: BoxDecoration(
                            color: Colors.blueAccent.withValues(alpha: 0.1),
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Text(p.tyName, style: const TextStyle(fontSize: 10, color: Colors.blueAccent, fontFamily: 'monospace')),
                        ),
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
  }
}
