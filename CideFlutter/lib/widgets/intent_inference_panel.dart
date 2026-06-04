import 'package:flutter/material.dart';
import '../src/rust/compiler/intent.dart';

/// P3: Code intent inference panel.
///
/// Displays the inferred high-level intent of the user's code
/// (Sort, Search, Traverse, Compute, Transform) with confidence scores.
class IntentInferencePanel extends StatelessWidget {
  final List<IntentScore> scores;

  const IntentInferencePanel({super.key, required this.scores});

  @override
  Widget build(BuildContext context) {
    if (scores.isEmpty) {
      return const Center(
        child: Text(
          '暂无代码意图分析',
          style: TextStyle(color: Colors.grey),
        ),
      );
    }

    return ListView.builder(
      padding: const EdgeInsets.all(12),
      itemCount: scores.length,
      itemBuilder: (context, index) {
        final s = scores[index];
        return _IntentCard(score: s, rank: index + 1);
      },
    );
  }
}

class _IntentCard extends StatelessWidget {
  final IntentScore score;
  final int rank;

  const _IntentCard({required this.score, required this.rank});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final color = _intentColor(score.intent);
    final display = _intentDisplayName(score.intent);

    return Card(
      margin: const EdgeInsets.only(bottom: 10),
      elevation: 2,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
      child: Padding(
        padding: const EdgeInsets.all(14),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Container(
                  width: 32,
                  height: 32,
                  decoration: BoxDecoration(
                    color: color.withValues(alpha: 0.15),
                    shape: BoxShape.circle,
                  ),
                  child: Center(
                    child: Text(
                      '$rank',
                      style: TextStyle(
                        color: color,
                        fontWeight: FontWeight.bold,
                        fontSize: 14,
                      ),
                    ),
                  ),
                ),
                const SizedBox(width: 10),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        display,
                        style: theme.textTheme.titleMedium?.copyWith(
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                      const SizedBox(height: 2),
                      Text(
                        '置信度 ${score.score} 分',
                        style: theme.textTheme.bodySmall?.copyWith(
                          color: Colors.grey[600],
                        ),
                      ),
                    ],
                  ),
                ),
                _ConfidenceBadge(score: score.score),
              ],
            ),
            const SizedBox(height: 10),
            const Divider(height: 1),
            const SizedBox(height: 8),
            ...score.reasons.map((r) => Padding(
              padding: const EdgeInsets.only(bottom: 4),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Icon(Icons.lightbulb_outline, size: 14, color: color),
                  const SizedBox(width: 6),
                  Expanded(
                    child: Text(
                      r,
                      style: theme.textTheme.bodySmall?.copyWith(
                        color: Colors.grey[700],
                        height: 1.3,
                      ),
                    ),
                  ),
                ],
              ),
            )),
          ],
        ),
      ),
    );
  }

  static Color _intentColor(CodeIntent intent) {
    switch (intent) {
      case CodeIntent.sort:
        return Colors.orange;
      case CodeIntent.search:
        return Colors.blue;
      case CodeIntent.traverse:
        return Colors.green;
      case CodeIntent.compute:
        return Colors.purple;
      case CodeIntent.transform:
        return Colors.teal;
      case CodeIntent.unknown:
        return Colors.grey;
    }
  }

  static String _intentDisplayName(CodeIntent intent) {
    switch (intent) {
      case CodeIntent.sort:
        return '排序算法';
      case CodeIntent.search:
        return '查找算法';
      case CodeIntent.traverse:
        return '遍历操作';
      case CodeIntent.compute:
        return '数值计算';
      case CodeIntent.transform:
        return '数据转换';
      case CodeIntent.unknown:
        return '未知意图';
    }
  }
}

class _ConfidenceBadge extends StatelessWidget {
  final int score;

  const _ConfidenceBadge({required this.score});

  @override
  Widget build(BuildContext context) {
    Color color;
    String label;
    if (score >= 80) {
      color = Colors.green;
      label = '高';
    } else if (score >= 50) {
      color = Colors.orange;
      label = '中';
    } else {
      color = Colors.red;
      label = '低';
    }
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Text(
        label,
        style: TextStyle(
          color: color,
          fontSize: 12,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
  }
}
