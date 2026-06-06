import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:cide/src/rust/api/types.dart' as rust;
import 'package:cide/src/rust/unified/types.dart' show AlgorithmStepSnapshot;
import '../models/algorithm_validation.dart';
import '../providers/ide_provider.dart';
import '../providers/unified_provider.dart';

class AlgorithmTab extends ConsumerWidget {
  final List<rust.AlgorithmMatch> matches;
  final bool isDark;

  const AlgorithmTab({super.key, required this.matches, required this.isDark});

  static const _algorithmPhaseFlows = {
    'bubble_sort': ['outer_loop', 'inner_loop', 'compare', 'swap', 'finish'],
    'selection_sort': ['outer_loop', 'inner_loop', 'compare', 'swap', 'finish'],
    'insertion_sort': ['outer_loop', 'compare', 'inner_loop', 'insert', 'finish'],
    'quick_sort': ['recursive', 'partition_init', 'partition_scan', 'partition_swap', 'finish'],
    'merge_sort': ['recursive_split', 'merge', 'finish'],
    'binary_search': ['loop', 'mid_calc', 'compare', 'narrow_left', 'narrow_right', 'found', 'not_found'],
    'heap_sort': ['build_heap', 'extract', 'swap', 'heapify', 'finish'],
    'bfs': ['loop', 'dequeue', 'visit', 'enqueue', 'finish'],
    'dfs': ['recursive', 'visit', 'scan', 'finish'],
    'dp': ['outer_loop', 'inner_loop', 'transition', 'finish'],
    'shell_sort': ['outer_loop', 'insert', 'inner_loop', 'finish'],
    'counting_sort': ['count', 'collect', 'place', 'finish'],
    'linked_list_delete': ['delete_head', 'search', 'unlink', 'free', 'finish'],
    'bst_insert': ['recursive', 'compare', 'create', 'finish'],
    'string_reverse': ['measure', 'swap', 'finish'],
    'gcd': ['loop', 'mod', 'finish'],
    'is_prime': ['check_small', 'test_divisor', 'found_factor', 'finish'],
    'hanoi': ['base', 'recursive', 'move', 'finish'],
    'seq_list': ['check', 'shift', 'place', 'update_len', 'finish'],
    'linked_list_append': ['find_tail', 'link', 'finish'],
    'circular_queue': ['check', 'enqueue', 'dequeue', 'wrap', 'finish'],
    'linked_stack': ['push', 'pop', 'finish'],
    'linked_queue': ['enqueue', 'dequeue', 'free', 'finish'],
    'level_order': ['enqueue', 'dequeue', 'enqueue_left', 'enqueue_right', 'finish'],
    'bst_search': ['recursive', 'compare', 'hit', 'miss', 'finish'],
    'hash_table': ['hash', 'probe', 'hit', 'finish'],
    'josephus': ['init', 'eliminate', 'rotate', 'finish'],
  };

  static const _phaseDisplayNames = {
    'outer_loop': '外层循环',
    'inner_loop': '内层循环',
    'compare': '比较',
    'swap': '交换',
    'insert': '插入',
    'partition_init': '选取枢轴',
    'partition_scan': '分区扫描',
    'partition_swap': '分区交换',
    'recursive': '递归分割',
    'recursive_split': '递归分割',
    'merge': '合并',
    'loop': '搜索范围',
    'mid_calc': '计算中点',
    'narrow_left': '缩左边界',
    'narrow_right': '缩右边界',
    'found': '找到结果',
    'not_found': '未找到',
    'finish': '完成',
    'build_heap': '建堆',
    'extract': '取堆顶',
    'heapify': '堆化',
    'dequeue': '出队',
    'enqueue': '入队',
    'visit': '访问标记',
    'scan': '扫描邻居',
    'transition': '状态转移',
    'count': '统计',
    'collect': '收集',
    'place': '放置元素',
    'delete_head': '删头节点',
    'unlink': '断链',
    'free': '释放内存',
    'create': '创建节点',
    'measure': '求长度',
    'mod': '取模',
    'check_small': '排除小数',
    'test_divisor': '试除',
    'found_factor': '发现因子',
    'move': '移动盘子',
    'check': '检查条件',
    'shift': '移动元素',
    'update_len': '更新长度',
    'find_tail': '找尾节点',
    'link': '链接节点',
    'wrap': '循环绕回',
    'push': '入栈',
    'pop': '出栈',
    'enqueue_left': '左子入队',
    'enqueue_right': '右子入队',
    'hit': '命中',
    'miss': '未命中',
    'probe': '线性探测',
    'init': '初始化',
    'eliminate': '淘汰',
    'rotate': '循环移动',
  };

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final unifiedState = ref.watch(unifiedProvider);
    final currentStep = unifiedState.currentStep >= 0 && unifiedState.currentStep < unifiedState.frameCache.length
        ? unifiedState.frameCache[unifiedState.currentStep].algorithmStep
        : null;

    if (matches.isEmpty) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.auto_graph_outlined, size: 40, color: Colors.grey[500]),
            const SizedBox(height: 12),
            Text(
              '未检测到算法模式',
              style: TextStyle(fontSize: 14, color: Colors.grey[500]),
            ),
          ],
        ),
      );
    }
    return StatefulBuilder(
      builder: (context, setState) {
        final validationResults = <int, AlgorithmValidationResult>{};
        final validating = <int, bool>{};
        final expandedVis = <int, bool>{};
        final hasAutoValidated = <int, bool>{};

        // 统一模式执行完成后自动验证算法
        if (unifiedState.phase == ExecutionPhase.playback) {
          for (var i = 0; i < matches.length; i++) {
            if (hasAutoValidated[i] != true && validationResults[i] == null) {
              hasAutoValidated[i] = true;
              final match = matches[i];
              final notifier = ref.read(ideProvider.notifier);
              Future.microtask(() async {
                setState(() => validating[i] = true);
                final res = await notifier.validateAlgorithm(match);
                setState(() {
                  validating[i] = false;
                  validationResults[i] = res;
                });
              });
            }
          }
        }

        return ListView.builder(
          itemCount: matches.length,
          itemBuilder: (context, index) {
            final match = matches[index];
            final result = validationResults[index];
            final isValidating = validating[index] ?? false;
            final isVisExpanded = expandedVis[index] ?? false;
            return Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              decoration: BoxDecoration(
                border: Border(
                  bottom: BorderSide(color: Theme.of(context).dividerColor.withValues(alpha: 0.1)),
                ),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Expanded(
                        child: Text(
                          match.displayName.isEmpty ? match.name : match.displayName,
                          style: const TextStyle(fontSize: 14, fontWeight: FontWeight.bold),
                        ),
                      ),
                      Text('置信度 ${match.confidence}%', style: const TextStyle(fontSize: 12, color: Colors.grey)),
                    ],
                  ),
                  if (match.suggestion.isNotEmpty)
                    Padding(
                      padding: const EdgeInsets.only(top: 4),
                      child: Text(match.suggestion, style: TextStyle(fontSize: 12, color: Colors.grey[400])),
                    ),
                  const SizedBox(height: 8),
                  Row(
                    children: [
                      TextButton.icon(
                        onPressed: isValidating
                            ? null
                            : () async {
                                setState(() => validating[index] = true);
                                final notifier = ref.read(ideProvider.notifier);
                                final res = await notifier.validateAlgorithm(match);
                                setState(() {
                                  validating[index] = false;
                                  validationResults[index] = res;
                                });
                              },
                        icon: isValidating
                            ? const SizedBox(width: 14, height: 14, child: CircularProgressIndicator(strokeWidth: 2))
                            : const Icon(Icons.search, size: 14),
                        label: const Text('验证算法', style: TextStyle(fontSize: 12)),
                        style: TextButton.styleFrom(
                          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                          minimumSize: Size.zero,
                          tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                        ),
                      ),
                      if (match.visEvents.isNotEmpty)
                        TextButton.icon(
                          onPressed: () => setState(() => expandedVis[index] = !isVisExpanded),
                          icon: Icon(
                            isVisExpanded ? Icons.visibility_off : Icons.visibility,
                            size: 14,
                          ),
                          label: Text(
                            isVisExpanded ? '收起事件' : '可视化事件 (${match.visEvents.length})',
                            style: const TextStyle(fontSize: 12),
                          ),
                          style: TextButton.styleFrom(
                            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                            minimumSize: Size.zero,
                            tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                          ),
                        ),
                    ],
                  ),
                  if (result != null)
                    Container(
                      margin: const EdgeInsets.only(top: 4),
                      padding: const EdgeInsets.all(8),
                      decoration: BoxDecoration(
                        color: result.passed ? Colors.green.withValues(alpha: 0.1) : Colors.red.withValues(alpha: 0.1),
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Row(
                        children: [
                          Icon(
                            result.passed ? Icons.check_circle : Icons.error,
                            size: 16,
                            color: result.passed ? Colors.green : Colors.red,
                          ),
                          const SizedBox(width: 6),
                          Expanded(
                            child: Text(
                              result.message,
                              style: TextStyle(
                                fontSize: 12,
                                color: result.passed ? Colors.green[300] : Colors.red[300],
                              ),
                            ),
                          ),
                        ],
                      ),
                    ),
                  // 算法步骤流程预览
                  _buildPhaseFlow(match, currentStep, isDark),
                  if (isVisExpanded && match.visEvents.isNotEmpty)
                    Container(
                      margin: const EdgeInsets.only(top: 6),
                      padding: const EdgeInsets.all(8),
                      decoration: BoxDecoration(
                        color: isDark ? const Color(0xFF2A2A2C) : const Color(0xFFF5F5F7),
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          const Text(
                            '关键比较事件',
                            style: TextStyle(fontSize: 11, fontWeight: FontWeight.bold, color: Colors.grey),
                          ),
                          const SizedBox(height: 4),
                          Wrap(
                            spacing: 6,
                            runSpacing: 4,
                            children: match.visEvents.asMap().entries.map((entry) {
                              final i = entry.key;
                              final ev = entry.value;
                              return Tooltip(
                                message: '第 ${ev.line} 行: ${ev.context}',
                                child: Container(
                                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                                  decoration: BoxDecoration(
                                    color: Colors.blueAccent.withValues(alpha: 0.15),
                                    borderRadius: BorderRadius.circular(4),
                                    border: Border.all(color: Colors.blueAccent.withValues(alpha: 0.3)),
                                  ),
                                  child: Text(
                                    '${i + 1}. ${ev.context}',
                                    style: const TextStyle(fontSize: 11, color: Colors.blueAccent),
                                  ),
                                ),
                              );
                            }).toList(),
                          ),
                        ],
                      ),
                    ),
                ],
              ),
            );
          },
        );
      },
    );
  }

  Widget _buildPhaseFlow(rust.AlgorithmMatch match, AlgorithmStepSnapshot? currentStep, bool isDark) {
    final phases = _algorithmPhaseFlows[match.name];
    if (phases == null || phases.isEmpty) return const SizedBox.shrink();

    final isActive = currentStep != null && currentStep.algorithmName == match.name;

    return Container(
      margin: const EdgeInsets.only(top: 6),
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
      decoration: BoxDecoration(
        color: isDark ? const Color(0xFF252527) : const Color(0xFFF0F0F2),
        borderRadius: BorderRadius.circular(4),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            '步骤流程',
            style: TextStyle(fontSize: 11, fontWeight: FontWeight.bold, color: Colors.grey),
          ),
          const SizedBox(height: 6),
          SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            child: Row(
              children: phases.asMap().entries.map((entry) {
                final idx = entry.key;
                final phase = entry.value;
                final isCurrent = isActive && currentStep.phase == phase;
                final display = _phaseDisplayNames[phase] ?? phase;

                final bgColor = isCurrent
                    ? Colors.blueAccent
                    : (isDark ? const Color(0xFF3A3A3C) : const Color(0xFFE0E0E2));
                final fgColor = isCurrent
                    ? Colors.white
                    : (isDark ? Colors.grey[400] : Colors.grey[700]);

                return Row(
                  children: [
                    if (idx > 0)
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 4),
                        child: Icon(
                          Icons.arrow_forward,
                          size: 12,
                          color: isDark ? Colors.grey[600] : Colors.grey[400],
                        ),
                      ),
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
                      decoration: BoxDecoration(
                        color: bgColor,
                        borderRadius: BorderRadius.circular(12),
                      ),
                      child: Text(
                        display,
                        style: TextStyle(
                          fontSize: 11,
                          color: fgColor,
                          fontWeight: isCurrent ? FontWeight.bold : FontWeight.normal,
                        ),
                      ),
                    ),
                  ],
                );
              }).toList(),
            ),
          ),
        ],
      ),
    );
  }
}
