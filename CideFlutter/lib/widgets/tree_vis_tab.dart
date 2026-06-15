import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../providers/unified_provider.dart';
import 'tree_visualizer.dart';

/// 树可视化 Tab：集成到统一模式，从 StepPayload 中读取树根指针并渲染。
class TreeVisTab extends ConsumerWidget {
  final bool isDark;

  const TreeVisTab({super.key, required this.isDark});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final unifiedState = ref.watch(unifiedProvider);
    final frameCache = unifiedState.frameCache;
    final currentStep = unifiedState.currentStep;
    final cacheIdx = currentStep - unifiedState.frameCacheStartStep;

    if (frameCache.isEmpty ||
        cacheIdx < 0 ||
        cacheIdx >= frameCache.length) {
      return _buildEmpty('运行程序以查看树');
    }

    final payload = frameCache[cacheIdx];
    final localVars = payload.localVars;

    // 查找 struct TreeNode* 类型的指针变量作为树根
    final rootVars = localVars.where((v) {
      final ty = v.tyName.toLowerCase();
      return ty.contains('struct') && ty.contains('tree') && ty.contains('*');
    }).toList();

    if (rootVars.isEmpty) {
      return _buildEmpty('未检测到树根指针');
    }

    // 收集树相关的 visEvents
    final treeVisEvents = payload.visEvents.where((ev) {
      return ev.ty == 4 || ev.ty == 6 || ev.ty == 7; // NodeCreate / NodeAccess / NodeDelete
    }).toList();

    return ListView.builder(
      padding: const EdgeInsets.all(12),
      itemCount: rootVars.length,
      itemBuilder: (context, index) {
        final rv = rootVars[index];
        final rootAddr = int.tryParse(rv.value) ?? 0;
        final structName = _extractStructName(rv.tyName);

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
                      rv.name,
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
                        rv.tyName,
                        style: TextStyle(fontSize: 11, color: Colors.grey[600], fontFamily: 'monospace'),
                      ),
                    ),
                    const Spacer(),
                    Text(
                      '0x${rootAddr.toRadixString(16).toUpperCase().padLeft(4, '0')}',
                      style: TextStyle(fontSize: 11, color: Colors.grey[500], fontFamily: 'monospace'),
                    ),
                  ],
                ),
                const SizedBox(height: 8),
                SizedBox(
                  height: 280,
                  child: TreeVisualizer(
                    rootAddr: rootAddr,
                    structName: structName,
                    visEvents: treeVisEvents,
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
          Icon(Icons.account_tree, size: 40, color: Colors.grey[500]),
          const SizedBox(height: 12),
          Text(message, style: TextStyle(fontSize: 14, color: Colors.grey[500])),
        ],
      ),
    );
  }

  /// 从类型字符串提取结构体名称，如 "struct TreeNode *" → "TreeNode"
  String _extractStructName(String tyName) {
    final lowered = tyName.toLowerCase();
    final match = RegExp(r'struct\s+(\w+)').firstMatch(lowered);
    if (match != null) {
      return match.group(1)!;
    }
    return 'TreeNode';
  }
}
