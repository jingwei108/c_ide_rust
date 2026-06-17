import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/providers/unified_provider.dart';
import 'package:cide/widgets/var_history_tab.dart';
import '../helpers/ide_state_factory.dart';
import '../helpers/pump_app.dart';

void main() {
  group('VarHistoryTab', () {
    testWidgets('shows empty hint when no frame cache', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (_) => const VarHistoryTab(isDark: false),
      );

      expect(find.text('运行程序以查看变量历史'), findsOneWidget);
    });

    testWidgets('shows empty hint when no variables', (tester) async {
      final container = await pumpAppWithProviders(
        tester,
        builder: (_) => const VarHistoryTab(isDark: false),
      );

      container.read(unifiedProvider.notifier).state = playbackState([
        stepPayload(stepIndex: 0),
      ]);
      await tester.pump();

      expect(find.text('未检测到变量'), findsOneWidget);
    });

    testWidgets('renders variable names, current values and change count', (tester) async {
      final container = await pumpAppWithProviders(
        tester,
        builder: (_) => const VarHistoryTab(isDark: false),
      );

      container.read(unifiedProvider.notifier).state = playbackState([
        stepPayload(stepIndex: 0, localVars: [variable(name: 'i', value: '0')]),
        stepPayload(stepIndex: 1, localVars: [variable(name: 'i', value: '1')]),
        stepPayload(stepIndex: 2, localVars: [variable(name: 'i', value: '2')]),
      ]);
      await tester.pump();

      expect(find.text('i'), findsOneWidget);
      expect(find.text('2'), findsOneWidget);
      expect(find.text('3 次变化（当前窗口）'), findsOneWidget);
      expect(find.byType(CustomPaint), findsWidgets);
    });

    testWidgets('renders discrete dots for non-numeric variables', (tester) async {
      final container = await pumpAppWithProviders(
        tester,
        builder: (_) => const VarHistoryTab(isDark: false),
      );

      container.read(unifiedProvider.notifier).state = playbackState([
        stepPayload(stepIndex: 0, localVars: [variable(name: 's', value: 'hello', tyName: 'char*')]),
        stepPayload(stepIndex: 1, localVars: [variable(name: 's', value: 'world', tyName: 'char*')]),
      ]);
      await tester.pump();

      expect(find.text('s'), findsOneWidget);
      expect(find.text('world'), findsOneWidget);
      expect(find.byType(CustomPaint), findsWidgets);
    });
  });
}
