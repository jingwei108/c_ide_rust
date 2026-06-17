import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/providers/unified_provider.dart';
import 'package:cide/src/rust/unified/types.dart';
import 'package:cide/widgets/variables_tab.dart';
import '../helpers/ide_state_factory.dart';
import '../helpers/pump_app.dart';

void main() {
  group('VariablesTab', () {
    testWidgets('shows empty state when no frames', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (_) => const VariablesTab(isDark: false),
      );

      expect(find.text('运行程序以查看变量'), findsOneWidget);
    });

    testWidgets('shows empty scope message when no local vars', (tester) async {
      final container = await pumpAppWithProviders(
        tester,
        builder: (_) => const VariablesTab(isDark: false),
      );

      container.read(unifiedProvider.notifier).state = playbackState([
        stepPayload(stepIndex: 0),
      ]);
      await tester.pump();

      expect(find.text('当前作用域无变量'), findsOneWidget);
    });

    testWidgets('renders local variables with types and values', (tester) async {
      final container = await pumpAppWithProviders(
        tester,
        builder: (_) => const VariablesTab(isDark: false),
      );

      container.read(unifiedProvider.notifier).state = playbackState([
        stepPayload(
          stepIndex: 0,
          localVars: [
            variable(name: 'a', value: '10', tyName: 'int'),
            variable(name: 'b', value: '20', tyName: 'int'),
          ],
        ),
      ]);
      await tester.pump();

      expect(find.text('a'), findsOneWidget);
      expect(find.text('b'), findsOneWidget);
      expect(find.text('int'), findsNWidgets(2));
      expect(find.textContaining('值: 10'), findsOneWidget);
      expect(find.textContaining('值: 20'), findsOneWidget);
    });

    testWidgets('shows read/write badges for accessed vars', (tester) async {
      final container = await pumpAppWithProviders(
        tester,
        builder: (_) => const VariablesTab(isDark: false),
      );

      container.read(unifiedProvider.notifier).state = playbackState([
        stepPayload(
          stepIndex: 0,
          localVars: [variable(name: 'x', value: '1')],
          accessedVars: [
            const AccessedVar(name: 'x', accessType: 'Write'),
          ],
        ),
      ]);
      await tester.pump();

      expect(find.text('写'), findsOneWidget);
    });
  });
}
