import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:cide/src/rust/api/types.dart' as rust;
import '../models/algorithm_validation.dart';
import '../providers/ide_provider.dart';

class AlgorithmTab extends ConsumerWidget {
  final List<rust.AlgorithmMatch> matches;
  final bool isDark;

  const AlgorithmTab({super.key, required this.matches, required this.isDark});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
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
}
