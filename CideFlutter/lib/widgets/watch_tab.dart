import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:cide/src/rust/api/cide.dart' as rust;
import '../providers/ide_provider.dart';

class WatchTab extends ConsumerWidget {
  final List<String> watchExpressions;
  final bool isDark;

  const WatchTab({super.key, required this.watchExpressions, required this.isDark});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final controller = TextEditingController();
    return Column(
      children: [
        // 输入栏
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          decoration: BoxDecoration(
            border: Border(bottom: BorderSide(color: Theme.of(context).dividerColor.withValues(alpha: 0.2))),
          ),
          child: Row(
            children: [
              Expanded(
                child: TextField(
                  controller: controller,
                  style: TextStyle(fontSize: 13, color: isDark ? Colors.white : Colors.black),
                  decoration: const InputDecoration(
                    isDense: true,
                    border: InputBorder.none,
                    hintText: '输入变量名（如 a、arr[0]）',
                    hintStyle: TextStyle(fontSize: 13),
                  ),
                  onSubmitted: (value) {
                    if (value.trim().isNotEmpty) {
                      ref.read(ideProvider.notifier).addWatchExpression(value.trim());
                      controller.clear();
                    }
                  },
                ),
              ),
              TextButton(
                onPressed: () {
                  final value = controller.text.trim();
                  if (value.isNotEmpty) {
                    ref.read(ideProvider.notifier).addWatchExpression(value);
                    controller.clear();
                  }
                },
                child: const Text('添加'),
              ),
            ],
          ),
        ),
        // 表达式列表
        Expanded(
          child: watchExpressions.isEmpty
              ? const Center(child: Text('暂无监视表达式', style: TextStyle(color: Colors.grey)))
              : FutureBuilder<List<rust.VariableSnapshot>>(
                  future: rust.getVariables(),
                  builder: (context, snapshot) {
                    final vars = snapshot.data ?? [];
                    return ListView.builder(
                      itemCount: watchExpressions.length,
                      itemBuilder: (context, index) {
                        final expr = watchExpressions[index];
                        final result = _evalWatchExpression(expr, vars);
                        return Container(
                          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                          decoration: BoxDecoration(
                            border: Border(bottom: BorderSide(color: Theme.of(context).dividerColor.withValues(alpha: 0.1))),
                          ),
                          child: Row(
                            children: [
                              Expanded(
                                child: Column(
                                  crossAxisAlignment: CrossAxisAlignment.start,
                                  children: [
                                    Text(
                                      expr,
                                      style: TextStyle(
                                        fontSize: 13,
                                        fontFamily: 'monospace',
                                        color: isDark ? const Color(0xffd4d4d4) : const Color(0xff333333),
                                      ),
                                    ),
                                    const SizedBox(height: 2),
                                    Text(
                                      result,
                                      style: TextStyle(
                                        fontSize: 12,
                                        color: result.startsWith('值:') ? Colors.green : Colors.orange,
                                        fontFamily: 'monospace',
                                      ),
                                    ),
                                  ],
                                ),
                              ),
                              IconButton(
                                icon: const Icon(Icons.close, size: 16, color: Colors.grey),
                                onPressed: () => ref.read(ideProvider.notifier).removeWatchExpression(expr),
                              ),
                            ],
                          ),
                        );
                      },
                    );
                  },
                ),
        ),
      ],
    );
  }

  String _evalWatchExpression(String expr, List<rust.VariableSnapshot> vars) {
    // 简单数组索引：arr[0]
    final arrMatch = RegExp(r'^(\w+)\[(\d+)\]$').firstMatch(expr);
    if (arrMatch != null) {
      final name = arrMatch.group(1)!;
      final idx = int.tryParse(arrMatch.group(2)!) ?? 0;
      final v = vars.where((x) => x.name == name).firstOrNull;
      if (v != null) {
        // 异步读取内存，这里返回提示
        return '数组 $name，地址 0x${v.addr.toRadixString(16)}，索引 $idx';
      }
      return '未找到变量: $name';
    }
    // 简单变量名匹配
    final v = vars.where((x) => x.name == expr).firstOrNull;
    if (v != null) {
      return '值: ${v.value}  (0x${v.addr.toRadixString(16).toUpperCase().padLeft(4, '0')})';
    }
    return '未找到变量: $expr';
  }
}
