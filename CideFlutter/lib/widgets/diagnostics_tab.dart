import 'package:flutter/material.dart';
import '../providers/ide_provider.dart';
import '../models/knowledge_card.dart';
import 'knowledge_card_item.dart';

class DiagnosticsTab extends StatelessWidget {
  final IdeState state;
  final IdeNotifier notifier;
  final bool isDark;
  final void Function(int line) onScrollToLine;
  final void Function(String source) onUpdateSource;

  const DiagnosticsTab({
    super.key,
    required this.state,
    required this.notifier,
    required this.isDark,
    required this.onScrollToLine,
    required this.onUpdateSource,
  });

  @override
  Widget build(BuildContext context) {
    if (state.diagnostics.isEmpty) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.check_circle_outline, size: 40, color: Colors.grey[500]),
            const SizedBox(height: 12),
            Text(
              '无诊断信息',
              style: TextStyle(fontSize: 14, color: Colors.grey[500]),
            ),
          ],
        ),
      );
    }
    return ListView.builder(
      itemCount: state.diagnostics.length,
      itemBuilder: (context, index) {
        final diag = state.diagnostics[index];
        final isError = diag.severity == 0;
        // TODO(#D09): KnowledgeCard.findByErrorCode 每 item 调用，可缓存 errorCode -> cards 映射。
        final relatedCards = diag.errorCode > 0
            ? KnowledgeCard.findByErrorCode(diag.errorCode)
            : <KnowledgeCard>[];
        return InkWell(
          onTap: () {
            notifier.highlightLine(diag.line);
            onScrollToLine(diag.line);
          },
          child: Container(
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
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                      decoration: BoxDecoration(
                        color: isError ? Colors.redAccent.withValues(alpha: 0.2) : Colors.orangeAccent.withValues(alpha: 0.2),
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Text(
                        isError ? '错误' : '警告',
                        style: TextStyle(
                          fontSize: 11,
                          color: isError ? Colors.redAccent : Colors.orangeAccent,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                    ),
                    const SizedBox(width: 8),
                    Text('第 ${diag.line} 行', style: const TextStyle(fontSize: 12, color: Colors.grey)),
                    if (diag.errorCode > 0)
                      Text(' [${diag.errorCode}]', style: const TextStyle(fontSize: 11, color: Colors.grey)),
                  ],
                ),
                const SizedBox(height: 4),
                Text(
                  diag.message,
                  style: TextStyle(fontSize: 13, color: isDark ? const Color(0xffd4d4d4) : const Color(0xff333333)),
                ),
                if (diag.fixSuggestion.isNotEmpty) ...[
                  const SizedBox(height: 4),
                  Row(
                    children: [
                      const Text('💡 ', style: TextStyle(fontSize: 12)),
                      Expanded(
                        child: Text(
                          diag.fixSuggestion,
                          style: TextStyle(fontSize: 12, color: Colors.grey[400]),
                        ),
                      ),
                    ],
                  ),
                ],
                // 应用修复 / 查看建议按钮
                if (diag.fixKind == 1 || diag.fixKind == 2 || diag.fixKind == 4 || diag.fixSuggestion.isNotEmpty)
                  Padding(
                    padding: const EdgeInsets.only(top: 6),
                    child: Align(
                      alignment: Alignment.centerLeft,
                      child: TextButton.icon(
                        onPressed: () async {
                          if (diag.fixKind == 4) {
                            // ManualHint: 只显示建议，不尝试自动替换
                            if (!context.mounted) return;
                            ScaffoldMessenger.of(context).showSnackBar(
                              SnackBar(
                                content: Text('💡 ${diag.fixSuggestion}'),
                                duration: const Duration(seconds: 4),
                              ),
                            );
                            return;
                          }
                          final msg = await notifier.applyFix(diag);
                          if (!context.mounted) return;
                          if (msg != null) {
                            onUpdateSource(state.source);
                            ScaffoldMessenger.of(context).showSnackBar(
                              SnackBar(content: Text(msg), duration: const Duration(seconds: 2)),
                            );
                            // 修复后重新编译
                            await notifier.compile();
                          } else {
                            if (!context.mounted) return;
                            ScaffoldMessenger.of(context).showSnackBar(
                              SnackBar(
                                content: Text('💡 修复提示（第${diag.line}行）：${diag.fixSuggestion}\n请手动修改代码。'),
                                duration: const Duration(seconds: 3),
                              ),
                            );
                          }
                        },
                        icon: Icon(
                          diag.fixKind == 4 ? Icons.lightbulb_outline : Icons.auto_fix_high,
                          size: 14,
                        ),
                        label: Text(
                          diag.fixKind == 4 ? '查看建议' : '应用修复',
                          style: const TextStyle(fontSize: 12),
                        ),
                        style: TextButton.styleFrom(
                          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                          minimumSize: Size.zero,
                          tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                        ),
                      ),
                    ),
                  ),
                // 关联知识卡片
                if (relatedCards.isNotEmpty) ...[
                  const SizedBox(height: 8),
                  Container(
                    padding: const EdgeInsets.all(8),
                    decoration: BoxDecoration(
                      color: isDark ? const Color(0xFF2A2A2C) : const Color(0xFFF5F5F7),
                      borderRadius: BorderRadius.circular(6),
                    ),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        const Text(
                          '相关知识',
                          style: TextStyle(fontSize: 11, fontWeight: FontWeight.bold, color: Colors.grey),
                        ),
                        const SizedBox(height: 6),
                        ...relatedCards.map((card) => KnowledgeCardItem(
                          card: card,
                          isDark: isDark,
                        )),
                      ],
                    ),
                  ),
                ],
              ],
            ),
          ),
        );
      },
    );
  }
}
