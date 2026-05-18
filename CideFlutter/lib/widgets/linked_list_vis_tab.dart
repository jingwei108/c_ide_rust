import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../providers/unified_provider.dart';
import 'linked_list_visualizer.dart';

/// 链表可视化 Tab：集成到统一模式，从 StepPayload 中读取链表头指针并渲染。
class LinkedListVisTab extends ConsumerWidget {
  final bool isDark;

  const LinkedListVisTab({super.key, required this.isDark});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final unifiedState = ref.watch(unifiedProvider);
    final frameCache = unifiedState.frameCache;
    final currentStep = unifiedState.currentStep;

    if (frameCache.isEmpty || currentStep < 0 || currentStep >= frameCache.length) {
      return _buildEmpty('运行程序以查看链表');
    }

    final payload = frameCache[currentStep];
    final localVars = payload.localVars;

    // 查找 struct Node* 类型的指针变量作为链表头
    final headVars = localVars.where((v) {
      final ty = v.tyName.toLowerCase();
      return ty.contains('struct') && ty.contains('node') && ty.contains('*');
    }).toList();

    if (headVars.isEmpty) {
      return _buildEmpty('未检测到链表头指针');
    }

    // 收集链表相关的 visEvents
    final listVisEvents = payload.visEvents.where((ev) {
      return ev.ty == 4 || ev.ty == 6 || ev.ty == 7; // NodeCreate / NodeAccess / NodeDelete
    }).toList();

    return ListView.builder(
      padding: const EdgeInsets.all(12),
      itemCount: headVars.length,
      itemBuilder: (context, index) {
        final hv = headVars[index];
        final headAddr = int.tryParse(hv.value) ?? 0;
        final structName = _extractStructName(hv.tyName);

        return Card(
          margin: const EdgeInsets.only(bottom: 12),
          color: isDark ? const Color(0xff2a2a2a) : const Color(0xfff8f8f8),
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Text(
                      hv.name,
                      style: TextStyle(
                        fontSize: 14,
                        fontWeight: FontWeight.bold,
                        color: isDark ? Colors.white : Colors.black87,
                      ),
                    ),
                    const SizedBox(width: 8),
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                      decoration: BoxDecoration(
                        color: isDark ? const Color(0xff3a3a3a) : const Color(0xffe0e0e0),
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Text(
                        hv.tyName,
                        style: TextStyle(fontSize: 11, color: Colors.grey[600], fontFamily: 'monospace'),
                      ),
                    ),
                    const Spacer(),
                    Text(
                      '0x${headAddr.toRadixString(16).toUpperCase().padLeft(4, '0')}',
                      style: TextStyle(fontSize: 11, color: Colors.grey[500], fontFamily: 'monospace'),
                    ),
                  ],
                ),
                const SizedBox(height: 8),
                SizedBox(
                  height: 120,
                  child: LinkedListVisualizer(
                    headAddr: headAddr,
                    structName: structName,
                    visEvents: listVisEvents,
                    isDark: isDark,
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildEmpty(String message) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(Icons.linear_scale, size: 40, color: Colors.grey[500]),
          const SizedBox(height: 12),
          Text(message, style: TextStyle(fontSize: 14, color: Colors.grey[500])),
        ],
      ),
    );
  }

  /// 从类型字符串提取结构体名称，如 "struct Node *" → "Node"
  String _extractStructName(String tyName) {
    final lowered = tyName.toLowerCase();
    final match = RegExp(r'struct\s+(\w+)').firstMatch(lowered);
    if (match != null) {
      return match.group(1)!;
    }
    return 'Node';
  }
}
