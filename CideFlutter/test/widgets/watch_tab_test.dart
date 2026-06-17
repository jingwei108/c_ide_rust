import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/providers/ide_provider.dart';
import 'package:cide/providers/unified_provider.dart';
import 'package:cide/widgets/watch_tab.dart';
import '../helpers/ide_state_factory.dart';
import '../helpers/pump_app.dart';

void main() {
  group('WatchTab', () {
    testWidgets('shows empty state when no watch expressions', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (_) => const WatchTab(
          watchExpressions: [],
          isDark: false,
        ),
      );

      expect(find.text('暂无监视表达式'), findsOneWidget);
      expect(find.byType(TextField), findsOneWidget);
    });

    testWidgets('adds watch expression via text field', (tester) async {
      final container = await pumpAppWithProviders(
        tester,
        builder: (_) => const WatchTab(
          watchExpressions: [],
          isDark: false,
        ),
      );

      await tester.enterText(find.byType(TextField), 'x');
      await tester.tap(find.text('添加'));
      await tester.pump();

      expect(container.read(ideProvider).watchExpressions, ['x']);
    });

    testWidgets('renders remove buttons for each expression', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (_) => const WatchTab(
          watchExpressions: ['x', 'y'],
          isDark: false,
        ),
      );

      expect(find.text('x'), findsOneWidget);
      expect(find.text('y'), findsOneWidget);
      expect(find.byIcon(Icons.close), findsNWidgets(2));
    });

    testWidgets('evaluates simple variable expression', (tester) async {
      final container = await pumpAppWithProviders(
        tester,
        builder: (_) => const WatchTab(
          watchExpressions: ['a'],
          isDark: false,
        ),
      );

      container.read(unifiedProvider.notifier).state = playbackState([
        stepPayload(
          stepIndex: 0,
          localVars: [variable(name: 'a', value: '42')],
        ),
      ]);
      await tester.pump();

      expect(find.textContaining('值: 42'), findsOneWidget);
    });

    testWidgets('evaluates array index expression', (tester) async {
      final container = await pumpAppWithProviders(
        tester,
        builder: (_) => const WatchTab(
          watchExpressions: ['arr[0]'],
          isDark: false,
        ),
      );

      container.read(unifiedProvider.notifier).state = playbackState([
        stepPayload(
          stepIndex: 0,
          localVars: [variable(name: 'arr', value: '[...]')],
        ),
      ]);
      await tester.pump();

      expect(find.textContaining('数组 arr'), findsOneWidget);
    });
  });
}
