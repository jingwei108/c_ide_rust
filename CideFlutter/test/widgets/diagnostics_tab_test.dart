import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:cide/providers/ide_provider.dart';
import 'package:cide/src/rust/session.dart';
import 'package:cide/widgets/diagnostics_tab.dart';
import '../helpers/ide_state_factory.dart';
import '../helpers/pump_app.dart';
import '../mocks/rust_api_service_mock.dart';

class _DiagnosticFake extends Fake implements Diagnostic {}

void main() {
  setUpAll(() {
    registerFallbackValue(_DiagnosticFake());
  });

  group('DiagnosticsTab', () {
    testWidgets('shows empty state when no diagnostics', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (container) => DiagnosticsTab(
          state: idleIdeState(),
          notifier: container.read(ideProvider.notifier),
          isDark: false,
          onScrollToLine: (_) {},
          onUpdateSource: (_) {},
        ),
      );

      expect(find.text('无诊断信息'), findsOneWidget);
    });

    testWidgets('renders error diagnostic with line and code', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (container) => DiagnosticsTab(
          state: ideStateWithDiagnostic(
            diagnostic(message: '缺少分号', line: 3, errorCode: 1001),
          ),
          notifier: container.read(ideProvider.notifier),
          isDark: false,
          onScrollToLine: (_) {},
          onUpdateSource: (_) {},
        ),
      );

      expect(find.text('错误'), findsOneWidget);
      expect(find.text('第 3 行'), findsOneWidget);
      expect(find.text(' [1001]'), findsOneWidget);
      expect(find.text('缺少分号'), findsOneWidget);
    });

    testWidgets('renders warning diagnostic', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (container) => DiagnosticsTab(
          state: ideStateWithDiagnostic(
            diagnostic(
              message: '未使用变量',
              line: 5,
              errorCode: 2001,
              severity: 1,
            ),
          ),
          notifier: container.read(ideProvider.notifier),
          isDark: false,
          onScrollToLine: (_) {},
          onUpdateSource: (_) {},
        ),
      );

      expect(find.text('警告'), findsOneWidget);
      expect(find.text('未使用变量'), findsOneWidget);
    });

    testWidgets('tapping diagnostic highlights line and scrolls', (tester) async {
      int? scrolledLine;
      late ProviderContainer container;
      container = await pumpAppWithProviders(
        tester,
        builder: (c) {
          container = c;
          return DiagnosticsTab(
            state: ideStateWithDiagnostic(
              diagnostic(message: '错误', line: 7),
            ),
            notifier: c.read(ideProvider.notifier),
            isDark: false,
            onScrollToLine: (line) => scrolledLine = line,
            onUpdateSource: (_) {},
          );
        },
      );

      // Tap the InkWell wrapping the diagnostic row.
      await tester.tap(find.byType(InkWell).first);
      await tester.pump();

      expect(scrolledLine, 7);
      expect(container.read(ideProvider).highlightedLine, 7);
    });

    testWidgets('shows apply fix button when fixKind supports it', (tester) async {
      final mock = MockRustApiService();
      when(
        () => mock.applyFix(
          source: any(named: 'source'),
          diag: any(named: 'diag'),
        ),
      ).thenAnswer((_) async => null);
      await pumpAppWithProviders(
        tester,
        mock: mock,
        builder: (container) => DiagnosticsTab(
          state: ideStateWithDiagnostic(
            diagnostic(
              message: '错误',
              line: 2,
              fixSuggestion: '添加分号',
              fixKind: 1,
            ),
          ),
          notifier: container.read(ideProvider.notifier),
          isDark: false,
          onScrollToLine: (_) {},
          onUpdateSource: (_) {},
        ),
      );

      expect(find.text('应用修复'), findsOneWidget);
      await tester.tap(find.text('应用修复'));
      await tester.pump();

      verify(
        () => mock.applyFix(
          source: any(named: 'source'),
          diag: any(named: 'diag'),
        ),
      ).called(1);
    });
  });
}
