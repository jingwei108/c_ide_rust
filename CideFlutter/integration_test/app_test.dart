import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:cide/main.dart' as app;
import 'package:cide/screens/ide_screen.dart';
import 'package:cide/widgets/editor_panel_v2.dart';
import 'package:cide/widgets/output_tab.dart';
import 'package:cide/widgets/diagnostics_tab.dart';
import 'package:cide/widgets/algorithm_tab.dart';

/// Pump a fixed number of frames to allow async initialization and one-off
/// animations to complete, without waiting for continuous animations (such as
/// the floating orb) to settle.
Future<void> _pumpFrames(WidgetTester tester, {int count = 60}) async {
  for (var i = 0; i < count; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  group('Cide App Integration Smoke Tests', () {
    testWidgets('app launches and shows IDE core UI', (tester) async {
      app.main();
      await _pumpFrames(tester);

      expect(find.byType(IdeScreen), findsOneWidget);
      expect(find.byIcon(Icons.play_arrow), findsOneWidget);
      expect(find.byIcon(Icons.skip_next), findsOneWidget);
      expect(find.text('main.c'), findsOneWidget);
      expect(find.byIcon(Icons.add), findsOneWidget);
      expect(find.text('输出'), findsOneWidget);
      expect(find.text('诊断'), findsOneWidget);
      expect(find.text('算法'), findsOneWidget);
      expect(find.text('意图'), findsOneWidget);
      expect(find.byType(EditorPanelV2), findsOneWidget);
    });

    testWidgets('theme toggle switches icon', (tester) async {
      app.main();
      await _pumpFrames(tester);

      final darkModeIcon = find.byIcon(Icons.dark_mode);
      final lightModeIcon = find.byIcon(Icons.light_mode);

      final hasDark = darkModeIcon.evaluate().isNotEmpty;
      final hasLight = lightModeIcon.evaluate().isNotEmpty;
      expect(hasDark || hasLight, isTrue);

      if (hasDark) {
        await tester.tap(darkModeIcon);
        await _pumpFrames(tester);
        expect(lightModeIcon, findsOneWidget);
      } else {
        await tester.tap(lightModeIcon);
        await _pumpFrames(tester);
        expect(darkModeIcon, findsOneWidget);
      }
    });

    testWidgets('bottom panel tab switches content', (tester) async {
      app.main();
      await _pumpFrames(tester);

      expect(find.byType(OutputTab), findsOneWidget);

      await tester.tap(find.text('诊断'));
      await _pumpFrames(tester);
      expect(find.byType(DiagnosticsTab), findsOneWidget);

      await tester.tap(find.text('算法'));
      await _pumpFrames(tester);
      expect(find.byType(AlgorithmTab), findsOneWidget);
    });

    testWidgets('add new file via file tab bar', (tester) async {
      app.main();
      await _pumpFrames(tester);

      await tester.tap(find.byIcon(Icons.add));
      await _pumpFrames(tester);

      expect(find.byType(AlertDialog), findsOneWidget);
      expect(find.byType(TextField), findsOneWidget);

      await tester.enterText(find.byType(TextField), 'test.c');
      await tester.tap(find.text('确定'));
      await _pumpFrames(tester);

      expect(find.text('main.c'), findsOneWidget);
      expect(find.text('test.c'), findsOneWidget);
    });
  });
}
