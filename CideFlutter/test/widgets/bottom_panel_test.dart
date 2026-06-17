import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/providers/ide_provider.dart';
import 'package:cide/screens/ide/bottom_panel.dart';
import 'package:cide/widgets/output_tab.dart';
import 'package:cide/widgets/diagnostics_tab.dart';
import 'package:cide/widgets/algorithm_tab.dart';
import 'package:cide/widgets/intent_inference_panel.dart';
import '../helpers/ide_state_factory.dart';
import '../helpers/pump_app.dart';

void main() {
  group('IdeBottomPanel', () {
    testWidgets('renders default tabs and active output panel', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (container) => IdeBottomPanel(
          animation: const AlwaysStoppedAnimation(1.0),
          inputController: TextEditingController(),
          onScrollToLine: (_) {},
          onUpdateSource: (_) {},
        ),
      );

      expect(find.text('输出'), findsOneWidget);
      expect(find.text('诊断'), findsOneWidget);
      expect(find.text('算法'), findsOneWidget);
      expect(find.text('意图'), findsOneWidget);
      expect(find.byType(OutputTab), findsOneWidget);
    });

    testWidgets('switching tabs updates active panel', (tester) async {
      final container = await pumpAppWithProviders(
        tester,
        builder: (container) => IdeBottomPanel(
          animation: const AlwaysStoppedAnimation(1.0),
          inputController: TextEditingController(),
          onScrollToLine: (_) {},
          onUpdateSource: (_) {},
        ),
      );

      await tester.tap(find.text('诊断'));
      await tester.pump();

      expect(container.read(ideProvider).bottomActiveIndex, 1);
      expect(find.byType(DiagnosticsTab), findsOneWidget);

      await tester.tap(find.text('算法'));
      await tester.pump();

      expect(container.read(ideProvider).bottomActiveIndex, 2);
      expect(find.byType(AlgorithmTab), findsOneWidget);

      await tester.tap(find.text('意图'));
      await tester.pump();

      expect(container.read(ideProvider).bottomActiveIndex, 3);
      expect(find.byType(IntentInferencePanel), findsOneWidget);
    });

    testWidgets('shows diagnostic badge count', (tester) async {
      final container = await pumpAppWithProviders(
        tester,
        builder: (container) => IdeBottomPanel(
          animation: const AlwaysStoppedAnimation(1.0),
          inputController: TextEditingController(),
          onScrollToLine: (_) {},
          onUpdateSource: (_) {},
        ),
      );

      container.read(ideProvider.notifier).state = ideStateWithDiagnostic(
        diagnostic(message: 'err'),
      );
      await tester.pump();

      expect(find.text('1'), findsOneWidget);
    });
  });
}
