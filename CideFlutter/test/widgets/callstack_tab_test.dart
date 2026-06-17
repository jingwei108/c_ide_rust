import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/providers/unified_provider.dart';
import 'package:cide/widgets/callstack_tab.dart';
import '../helpers/ide_state_factory.dart';
import '../helpers/pump_app.dart';

void main() {
  group('CallstackTab', () {
    testWidgets('shows empty state when no frames', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (_) => const CallstackTab(isDark: false),
      );

      expect(find.text('运行程序以查看调用栈'), findsOneWidget);
    });

    testWidgets('shows empty callstack message', (tester) async {
      final container = await pumpAppWithProviders(
        tester,
        builder: (_) => const CallstackTab(isDark: false),
      );

      container.read(unifiedProvider.notifier).state = playbackState([
        stepPayload(stepIndex: 0),
      ]);
      await tester.pump();

      expect(find.text('调用栈为空'), findsOneWidget);
    });

    testWidgets('renders call stack frames with current badge', (tester) async {
      final container = await pumpAppWithProviders(
        tester,
        builder: (_) => const CallstackTab(isDark: false),
      );

      container.read(unifiedProvider.notifier).state = playbackState([
        stepPayload(
          stepIndex: 0,
          callStack: [
            frame(funcName: 'main', returnLine: 0),
            frame(funcName: 'foo', returnLine: 5),
          ],
        ),
      ]);
      await tester.pump();

      expect(find.text('main'), findsOneWidget);
      expect(find.text('foo'), findsOneWidget);
      expect(find.text('当前'), findsOneWidget);
      expect(find.text('入口'), findsOneWidget);
      expect(find.text('第 5 行 →'), findsOneWidget);
    });

    testWidgets('tapping return line scrolls to line', (tester) async {
      int? scrolledLine;
      final container = await pumpAppWithProviders(
        tester,
        builder: (_) => CallstackTab(
          isDark: false,
          onScrollToLine: (line) => scrolledLine = line,
        ),
      );

      container.read(unifiedProvider.notifier).state = playbackState([
        stepPayload(
          stepIndex: 0,
          callStack: [
            frame(funcName: 'main', returnLine: 0),
            frame(funcName: 'foo', returnLine: 8),
          ],
        ),
      ]);
      await tester.pump();

      await tester.tap(find.text('第 8 行 →'));
      await tester.pump();

      expect(scrolledLine, 8);
    });
  });
}
