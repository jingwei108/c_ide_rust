import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:cide/providers/ide_provider.dart';
import 'package:cide/widgets/toolbar.dart';
import '../helpers/ide_state_factory.dart';
import '../helpers/pump_app.dart';
import '../helpers/rust_api_stubs.dart';
import '../mocks/rust_api_service_mock.dart';

void main() {
  group('Toolbar', () {
    testWidgets('shows play, step and theme buttons', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (container) => Toolbar(
          state: idleIdeState(),
          notifier: container.read(ideProvider.notifier),
          isDark: false,
          onToggleTheme: () {},
        ),
      );

      expect(find.byIcon(Icons.play_arrow), findsOneWidget);
      expect(find.byIcon(Icons.skip_next), findsOneWidget);
      expect(find.byIcon(Icons.dark_mode), findsOneWidget);
      expect(find.byIcon(Icons.help_outline), findsOneWidget);
    });

    testWidgets('displays compiling state', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (container) => Toolbar(
          state: const IdeState(isCompiling: true),
          notifier: container.read(ideProvider.notifier),
          isDark: false,
          onToggleTheme: () {},
        ),
      );

      expect(find.text('编译中...'), findsOneWidget);
    });

    testWidgets('tapping run triggers compile/run pipeline', (tester) async {
      final mock = MockRustApiService();
      stubCompileSuccess(mock);
      stubRunSuccess(mock, output: 'hello');
      await pumpAppWithProviders(
        tester,
        mock: mock,
        builder: (container) => Toolbar(
          state: idleIdeState(),
          notifier: container.read(ideProvider.notifier),
          isDark: false,
          onToggleTheme: () {},
        ),
      );

      await tester.tap(find.byIcon(Icons.play_arrow));
      await tester.pump();

      verify(() => mock.compileMulti(files: any(named: 'files'))).called(1);
    });

    testWidgets('tapping theme toggle calls callback', (tester) async {
      var toggled = false;
      await pumpAppWithProviders(
        tester,
        builder: (container) => Toolbar(
          state: idleIdeState(),
          notifier: container.read(ideProvider.notifier),
          isDark: false,
          onToggleTheme: () => toggled = true,
        ),
      );

      await tester.tap(find.byIcon(Icons.dark_mode));
      await tester.pump();

      expect(toggled, isTrue);
    });

    testWidgets('shows execution speed slider in step mode', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (container) => Toolbar(
          state: const IdeState(
            isRunning: true,
            isStepMode: true,
            executionSpeed: 100,
          ),
          notifier: container.read(ideProvider.notifier),
          isDark: false,
          onToggleTheme: () {},
        ),
      );

      expect(find.byIcon(Icons.speed), findsOneWidget);
      expect(find.byType(Slider), findsOneWidget);
    });
  });
}
