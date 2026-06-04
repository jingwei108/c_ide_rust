import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../models/ide_state.dart';
import '../models/knowledge_card.dart';
import '../providers/ide_provider.dart';
import 'learning_path_panel.dart';

class ProgressTab extends ConsumerWidget {
  final IdeState state;

  const ProgressTab({super.key, required this.state});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final progress = state.learningProgress;
    final totalCards = KnowledgeCard.all.length;
    final viewedCards = progress.viewedKnowledgeCards.length;
    final cardProgress = totalCards == 0 ? 0.0 : viewedCards / totalCards;

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 连续活跃天数
          ProgressCard(
            title: '🔥 连续活跃',
            value: '${progress.streakDays} 天',
            subtitle: progress.lastActiveDate.isEmpty ? '开始你的学习之旅吧' : '最后活跃: ${progress.lastActiveDate}',
            icon: Icons.local_fire_department,
            color: Colors.orangeAccent,
          ),
          const SizedBox(height: 12),
          // 编译统计
          ProgressCard(
            title: '📝 编译统计',
            value: '${progress.totalCompiles} 次',
            subtitle: '成功 ${progress.successfulCompiles} · 失败 ${progress.failedCompiles} · 成功率 ${(progress.successRate * 100).toStringAsFixed(1)}%',
            icon: Icons.code,
            color: Colors.blueAccent,
          ),
          const SizedBox(height: 12),
          // 错误修复
          ProgressCard(
            title: '🛠️ 错误修复',
            value: '${progress.totalErrorsFixed} / ${progress.totalErrorsEncountered}',
            subtitle: '已修复 / 遇到',
            icon: Icons.build,
            color: Colors.green,
          ),
          const SizedBox(height: 12),
          // 知识卡片
          ProgressCard(
            title: '📚 知识卡片',
            value: '$viewedCards / $totalCards',
            subtitle: '已阅读 / 总数',
            icon: Icons.menu_book,
            color: Colors.purpleAccent,
            progress: cardProgress,
          ),
          const SizedBox(height: 12),
          // 算法验证
          ProgressCard(
            title: '🔍 算法验证',
            value: '${(progress.algorithmOverallPassRate * 100).toStringAsFixed(1)}%',
            subtitle: progress.algorithmValidationsTotal.isEmpty
                ? '暂无验证记录'
                : progress.algorithmValidationsTotal.entries.map((e) {
                    final passed = progress.algorithmValidationsPassed[e.key] ?? 0;
                    return '${e.key}: $passed/${e.value}';
                  }).join(' · '),
            icon: Icons.auto_fix_high,
            color: Colors.teal,
          ),
          const SizedBox(height: 12),
          // 统一模式探索
          ProgressCard(
            title: '🚀 调试探索',
            value: '${progress.totalUnifiedRuns} 次',
            subtitle: progress.totalUnifiedRuns == 0
                ? '使用统一模式运行代码以开始追踪'
                : '总步数 ${progress.totalStepsExecuted} · 异常 ${progress.totalTraps} · Seek ${progress.totalSeeks} · 峰值 ${progress.maxStepsInSingleRun} 步',
            icon: Icons.play_circle_outline,
            color: Colors.deepOrange,
          ),
          const SizedBox(height: 12),
          // 认知诊断入口
          InkWell(
            onTap: () {
              showModalBottomSheet(
                context: context,
                isScrollControlled: true,
                builder: (context) => DraggableScrollableSheet(
                  initialChildSize: 0.7,
                  minChildSize: 0.4,
                  maxChildSize: 0.95,
                  expand: false,
                  builder: (context, scrollController) => SingleChildScrollView(
                    controller: scrollController,
                    child: const LearningPathPanel(),
                  ),
                ),
              );
            },
            borderRadius: BorderRadius.circular(10),
            child: Container(
              width: double.infinity,
              padding: const EdgeInsets.all(14),
              decoration: BoxDecoration(
                color: Colors.indigo.withValues(alpha: 0.05),
                borderRadius: BorderRadius.circular(10),
                border: Border.all(color: Colors.indigo.withValues(alpha: 0.2)),
              ),
              child: Row(
                children: [
                  const Icon(Icons.psychology, color: Colors.indigo, size: 20),
                  const SizedBox(width: 10),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        const Text(
                          '认知诊断与学习路径',
                          style: TextStyle(fontSize: 13, fontWeight: FontWeight.w600, color: Colors.indigo),
                        ),
                        const SizedBox(height: 2),
                        Text(
                          '基于最近编译历史分析认知盲区，获取针对性练习',
                          style: TextStyle(fontSize: 11, color: Colors.grey[600]),
                        ),
                      ],
                    ),
                  ),
                  const Icon(Icons.arrow_forward_ios, size: 14, color: Colors.indigo),
                ],
              ),
            ),
          ),
          const SizedBox(height: 24),
          // 重置按钮
          Center(
            child: TextButton.icon(
              onPressed: () async {
                final confirmed = await showDialog<bool>(
                  context: context,
                  builder: (ctx) => AlertDialog(
                    title: const Text('重置学习进度'),
                    content: const Text('确定要清除所有学习进度数据吗？此操作不可恢复。'),
                    actions: [
                      TextButton(onPressed: () => Navigator.pop(ctx, false), child: const Text('取消')),
                      TextButton(onPressed: () => Navigator.pop(ctx, true), child: const Text('确定')),
                    ],
                  ),
                );
                if (confirmed == true) {
                  await ref.read(ideProvider.notifier).resetProgress();
                }
              },
              icon: const Icon(Icons.restore, size: 16),
              label: const Text('重置进度', style: TextStyle(fontSize: 12)),
            ),
          ),
        ],
      ),
    );
  }
}

class ProgressCard extends StatelessWidget {
  final String title;
  final String value;
  final String subtitle;
  final IconData icon;
  final Color color;
  final double? progress;

  const ProgressCard({
    super.key,
    required this.title,
    required this.value,
    required this.subtitle,
    required this.icon,
    required this.color,
    this.progress,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.05),
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: color.withValues(alpha: 0.15)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(icon, size: 18, color: color),
              const SizedBox(width: 8),
              Text(title, style: TextStyle(fontSize: 13, color: color, fontWeight: FontWeight.w600)),
              const Spacer(),
              Text(value, style: TextStyle(fontSize: 15, color: color, fontWeight: FontWeight.bold)),
            ],
          ),
          const SizedBox(height: 6),
          Text(
            subtitle,
            style: TextStyle(fontSize: 11, color: Colors.grey[500], height: 1.4),
          ),
          if (progress != null) ...[
            const SizedBox(height: 8),
            ClipRRect(
              borderRadius: BorderRadius.circular(4),
              child: LinearProgressIndicator(
                value: progress,
                backgroundColor: color.withValues(alpha: 0.1),
                valueColor: AlwaysStoppedAnimation<Color>(color),
                minHeight: 6,
              ),
            ),
          ],
        ],
      ),
    );
  }
}
