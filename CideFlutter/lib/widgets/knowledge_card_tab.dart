import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../models/knowledge_card.dart';
import '../providers/ide_provider.dart';
import 'knowledge_card_item.dart';

class KnowledgeCardTab extends ConsumerWidget {
  final List<KnowledgeCard> cards;
  final bool isDark;

  const KnowledgeCardTab({super.key, required this.cards, required this.isDark});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    if (cards.isEmpty) {
      return const Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.menu_book, size: 48, color: Colors.grey),
            SizedBox(height: 12),
            Text('暂无相关知识卡片', style: TextStyle(color: Colors.grey)),
            SizedBox(height: 4),
            Text('编译出错后将自动匹配对应的知识卡片', style: TextStyle(fontSize: 12, color: Colors.grey)),
          ],
        ),
      );
    }
    return ListView.builder(
      padding: const EdgeInsets.all(12),
      itemCount: cards.length,
      itemBuilder: (context, index) {
        final card = cards[index];
        Future.microtask(() => ref.read(ideProvider.notifier).recordKnowledgeCardView(card.id));
        return KnowledgeCardItem(card: card, isDark: isDark);
      },
    );
  }
}
