import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/models/knowledge_card.dart';
import 'package:cide/widgets/knowledge_card_item.dart';
import '../helpers/pump_app.dart';

void main() {
  group('KnowledgeCardItem', () {
    final card = KnowledgeCard(
      id: 'E1001',
      emoji: '🐛',
      title: '数组越界',
      explanation: '访问了数组有效范围之外的元素。',
      correctCode: 'int arr[5]; arr[4] = 0;',
      wrongCode: 'int arr[5]; arr[5] = 0;',
      relatedTrapKeywords: const ['bounds'],
    );

    testWidgets('renders emoji, title and explanation', (tester) async {
      await pumpWidget(
        tester,
        child: KnowledgeCardItem(card: card, isDark: false),
      );

      expect(find.text('🐛'), findsOneWidget);
      expect(find.text('数组越界'), findsOneWidget);
      expect(find.text('访问了数组有效范围之外的元素。'), findsOneWidget);
    });

    testWidgets('renders correct and wrong code blocks', (tester) async {
      await pumpWidget(
        tester,
        child: KnowledgeCardItem(card: card, isDark: false),
      );

      expect(find.text('✅ 正确写法'), findsOneWidget);
      expect(find.text('❌ 错误写法'), findsOneWidget);
      expect(find.text('int arr[5]; arr[4] = 0;'), findsOneWidget);
      expect(find.text('int arr[5]; arr[5] = 0;'), findsOneWidget);
    });
  });
}
