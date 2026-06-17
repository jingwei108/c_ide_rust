import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/src/rust/compiler/intent.dart';
import 'package:cide/widgets/intent_inference_panel.dart';
import '../helpers/pump_app.dart';

void main() {
  group('IntentInferencePanel', () {
    testWidgets('shows empty state when no scores', (tester) async {
      await pumpWidget(
        tester,
        child: const IntentInferencePanel(scores: []),
      );

      expect(find.text('暂无代码意图分析'), findsOneWidget);
    });

    testWidgets('renders intent scores with badges', (tester) async {
      await pumpWidget(
        tester,
        child: IntentInferencePanel(
          scores: [
            IntentScore(
              intent: CodeIntent.sort,
              score: 85,
              reasons: const ['包含嵌套循环', '存在交换操作'],
            ),
            IntentScore(
              intent: CodeIntent.search,
              score: 30,
              reasons: const ['范围缩小模式'],
            ),
          ],
        ),
      );

      expect(find.text('排序算法'), findsOneWidget);
      expect(find.text('查找算法'), findsOneWidget);
      expect(find.text('高'), findsOneWidget);
      expect(find.text('低'), findsOneWidget);
      expect(find.text('包含嵌套循环'), findsOneWidget);
      expect(find.text('存在交换操作'), findsOneWidget);
    });
  });
}
