import 'package:flutter/material.dart';
import '../models/knowledge_card.dart';

class KnowledgeCardItem extends StatelessWidget {
  final KnowledgeCard card;
  final bool isDark;

  const KnowledgeCardItem({super.key, required this.card, required this.isDark});

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.only(bottom: 12),
      color: isDark ? const Color(0xff2a2a2a) : const Color(0xfff8f8f8),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Text(card.emoji, style: const TextStyle(fontSize: 24)),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    card.title,
                    style: TextStyle(
                      fontSize: 16,
                      fontWeight: FontWeight.bold,
                      color: isDark ? Colors.white : Colors.black87,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              card.explanation,
              style: TextStyle(
                fontSize: 13,
                color: isDark ? const Color(0xffbbbbbb) : const Color(0xff555555),
                height: 1.5,
              ),
            ),
            const SizedBox(height: 12),
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(10),
              decoration: BoxDecoration(
                color: isDark ? const Color(0xff1e1e1e) : const Color(0xffeeeeee),
                borderRadius: BorderRadius.circular(6),
                border: Border.all(color: Colors.green.withValues(alpha: 0.3)),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text('✅ 正确写法', style: TextStyle(fontSize: 11, color: Colors.green)),
                  const SizedBox(height: 4),
                  Text(
                    card.correctCode,
                    style: const TextStyle(fontFamily: 'monospace', fontSize: 12, color: Colors.green),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 8),
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(10),
              decoration: BoxDecoration(
                color: isDark ? const Color(0xff1e1e1e) : const Color(0xffeeeeee),
                borderRadius: BorderRadius.circular(6),
                border: Border.all(color: Colors.redAccent.withValues(alpha: 0.3)),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text('❌ 错误写法', style: TextStyle(fontSize: 11, color: Colors.redAccent)),
                  const SizedBox(height: 4),
                  Text(
                    card.wrongCode,
                    style: const TextStyle(fontFamily: 'monospace', fontSize: 12, color: Colors.redAccent),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
