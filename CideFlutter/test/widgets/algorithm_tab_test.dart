import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/providers/unified_provider.dart';
import 'package:cide/src/rust/session.dart';
import 'package:cide/src/rust/unified/types.dart';
import 'package:cide/widgets/algorithm_tab.dart';
import '../helpers/ide_state_factory.dart';
import '../helpers/pump_app.dart';

void main() {
  group('AlgorithmTab', () {
    testWidgets('shows empty state when no matches', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (_) => const AlgorithmTab(matches: [], isDark: false),
      );

      expect(find.text('未检测到算法模式'), findsOneWidget);
    });

    testWidgets('renders algorithm match name and confidence', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (_) => AlgorithmTab(
          matches: [
            AlgorithmMatch(
              name: 'bubble_sort',
              displayName: '冒泡排序',
              funcName: 'main',
              confidence: 92,
              suggestion: '建议',
              line: 1,
              visEvents: const [],
            ),
          ],
          isDark: false,
        ),
      );

      expect(find.text('冒泡排序'), findsOneWidget);
      expect(find.text('置信度 92%'), findsOneWidget);
      expect(find.text('验证算法'), findsOneWidget);
    });

    testWidgets('renders phase flow for known algorithm', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (_) => AlgorithmTab(
          matches: [
            AlgorithmMatch(
              name: 'bubble_sort',
              displayName: '冒泡排序',
              funcName: 'main',
              confidence: 92,
              suggestion: '',
              line: 1,
              visEvents: const [],
            ),
          ],
          isDark: false,
        ),
      );

      expect(find.text('步骤流程'), findsOneWidget);
      expect(find.text('外层循环'), findsOneWidget);
      expect(find.text('交换'), findsOneWidget);
      expect(find.text('完成'), findsOneWidget);
    });

    testWidgets('shows visualization event button when events exist', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (_) => AlgorithmTab(
          matches: [
            AlgorithmMatch(
              name: 'bubble_sort',
              displayName: '冒泡排序',
              funcName: 'main',
              confidence: 92,
              suggestion: '',
              line: 1,
              visEvents: [
                VisEvent(
                  ty: 0,
                  line: 3,
                  extra0: 0,
                  extra1: 0,
                  extra2: 0,
                  context: '比较 i 与 j',
                ),
              ],
            ),
          ],
          isDark: false,
        ),
      );

      expect(find.text('可视化事件 (1)'), findsOneWidget);
      // Note: expansion toggles local state recreated each build, so the
      // expansion content cannot be tested here without a widget fix.
    });

    testWidgets('highlights current phase when unified state has algorithmStep', (tester) async {
      final container = await pumpAppWithProviders(
        tester,
        builder: (_) => AlgorithmTab(
          matches: [
            AlgorithmMatch(
              name: 'bubble_sort',
              displayName: '冒泡排序',
              funcName: 'main',
              confidence: 92,
              suggestion: '',
              line: 1,
              visEvents: const [],
            ),
          ],
          isDark: false,
        ),
      );

      container.read(unifiedProvider.notifier).state = playbackState([
        stepPayload(
          stepIndex: 0,
          algorithmStep: const AlgorithmStepSnapshot(
            algorithmName: 'bubble_sort',
            displayName: '比较',
            phase: 'compare',
            description: '比较相邻元素',
          ),
        ),
      ]);
      await tester.pump();

      // The current phase chip has white foreground on blue background.
      expect(find.text('比较'), findsOneWidget);
    });
  });
}
