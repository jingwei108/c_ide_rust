import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:cide/providers/ide_provider.dart';
import 'package:cide/widgets/output_tab.dart';
import '../helpers/ide_state_factory.dart';
import '../helpers/pump_app.dart';
import '../mocks/rust_api_service_mock.dart';

void main() {
  group('OutputTab', () {
    testWidgets('shows empty placeholder when output is empty', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (container) => OutputTab(
          state: idleIdeState(),
          notifier: container.read(ideProvider.notifier),
          isDark: false,
          inputController: TextEditingController(),
        ),
      );

      expect(find.text('等待执行'), findsOneWidget);
      expect(find.byType(SelectableText), findsOneWidget);
    });

    testWidgets('displays output text', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (container) => OutputTab(
          state: ideStateWithOutput('hello\nworld'),
          notifier: container.read(ideProvider.notifier),
          isDark: false,
          inputController: TextEditingController(),
        ),
      );

      expect(find.text('hello\nworld'), findsOneWidget);
      expect(find.byIcon(Icons.copy), findsOneWidget);
    });

    testWidgets('shows input field when waiting for input', (tester) async {
      final mock = MockRustApiService();
      when(() => mock.provideInputLine(line: any(named: 'line'))).thenAnswer(
        (_) async {},
      );
      await pumpAppWithProviders(
        tester,
        mock: mock,
        builder: (container) => OutputTab(
          state: ideStateWaitingInput(),
          notifier: container.read(ideProvider.notifier),
          isDark: false,
          inputController: TextEditingController(),
        ),
      );

      expect(find.text('➜'), findsOneWidget);
      expect(find.byType(TextField), findsOneWidget);
      expect(find.text('发送'), findsOneWidget);

      await tester.enterText(find.byType(TextField), '42');
      await tester.tap(find.text('发送'));
      await tester.pump();

      verify(() => mock.provideInputLine(line: '42')).called(1);
    });

    testWidgets('copies output to clipboard when copy icon tapped', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (container) => OutputTab(
          state: ideStateWithOutput('copied text'),
          notifier: container.read(ideProvider.notifier),
          isDark: false,
          inputController: TextEditingController(),
        ),
      );

      expect(find.byIcon(Icons.copy), findsOneWidget);
      await tester.tap(find.byIcon(Icons.copy));
      await tester.pump();

      // Clipboard operation succeeds without exception; SnackBar is shown.
      expect(find.byType(SnackBar), findsOneWidget);
      expect(find.text('已复制到剪贴板'), findsOneWidget);
    });
  });
}
