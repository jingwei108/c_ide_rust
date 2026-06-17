import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/models/unified_state.dart';
import 'package:cide/providers/unified_provider.dart';
import 'package:cide/src/rust/unified/types.dart';
import 'package:cide/widgets/execution_control_panel.dart';
import '../helpers/ide_state_factory.dart';
import '../helpers/pump_app.dart';

void main() {
  group('ExecutionControlPanel', () {
    Future<ProviderContainer> pumpPanel(
      WidgetTester tester, {
      required Widget child,
    }) async {
      return pumpAppWithProviders(
        tester,
        builder: (_) => SizedBox(
          width: 1200,
          height: 300,
          child: ClipRect(child: child),
        ),
      );
    }

    testWidgets('is hidden when phase is idle', (tester) async {
      await pumpPanel(
        tester,
        child: ExecutionControlPanel(onRun: _doNothing),
      );

      expect(find.byIcon(Icons.play_arrow), findsNothing);
      expect(find.byIcon(Icons.pause), findsNothing);
    });

    testWidgets('shows pause button during collecting', (tester) async {
      final container = await pumpPanel(
        tester,
        child: const ExecutionControlPanel(onRun: _doNothing),
      );

      container.read(unifiedProvider.notifier).state = collectingState();
      await tester.pump();

      expect(find.byIcon(Icons.pause), findsOneWidget);
      expect(find.byTooltip('暂停'), findsOneWidget);
    });

    testWidgets('shows play button when paused', (tester) async {
      final container = await pumpPanel(
        tester,
        child: const ExecutionControlPanel(onRun: _doNothing),
      );

      container.read(unifiedProvider.notifier).state = pausedState([
        stepPayload(stepIndex: 0),
      ]);
      await tester.pump();

      expect(find.byIcon(Icons.play_arrow), findsOneWidget);
      expect(find.byTooltip('继续'), findsOneWidget);
    });

    testWidgets('shows trap message in error phase', (tester) async {
      final container = await pumpPanel(
        tester,
        child: const ExecutionControlPanel(onRun: _doNothing),
      );

      container.read(unifiedProvider.notifier).state = const UnifiedState(
        phase: ExecutionPhase.error,
        trapMessage: '数组越界',
      );
      await tester.pump();

      expect(find.text('数组越界'), findsOneWidget);
      expect(find.text('查看帮助'), findsOneWidget);
      expect(find.text('重置'), findsOneWidget);
    });

    testWidgets('shows algorithm step annotation when present', (tester) async {
      final container = await pumpPanel(
        tester,
        child: const ExecutionControlPanel(onRun: _doNothing),
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

      expect(find.text('比较相邻元素'), findsOneWidget);
      expect(find.text('比较'), findsOneWidget);
    });

    testWidgets('shows step counter', (tester) async {
      final container = await pumpPanel(
        tester,
        child: const ExecutionControlPanel(onRun: _doNothing),
      );

      container.read(unifiedProvider.notifier).state = playbackState([
        stepPayload(stepIndex: 0),
        stepPayload(stepIndex: 1),
      ]);
      await tester.pump();

      expect(find.text('0 / 1'), findsOneWidget);
    });
  });
}

void _doNothing() {}
