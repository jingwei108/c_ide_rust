import 'package:flutter_test/flutter_test.dart';
import 'package:cide/providers/unified_provider.dart';
import 'package:cide/widgets/array_vis_tab.dart';
import '../helpers/ide_state_factory.dart';
import '../helpers/pump_app.dart';

void main() {
  group('ArrayVisTab', () {
    testWidgets('shows empty hint when no frame cache', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (_) => const ArrayVisTab(isDark: false),
      );

      expect(find.text('运行程序以查看数组'), findsOneWidget);
    });

    testWidgets('shows empty hint when no array snapshots', (tester) async {
      final container = await pumpAppWithProviders(
        tester,
        builder: (_) => const ArrayVisTab(isDark: false),
      );

      container.read(unifiedProvider.notifier).state = playbackState([
        stepPayload(stepIndex: 0),
      ]);
      await tester.pump();

      expect(find.text('未检测到数组变量'), findsOneWidget);
    });

    testWidgets('renders array snapshots from unified state', (tester) async {
      final container = await pumpAppWithProviders(
        tester,
        builder: (_) => const ArrayVisTab(isDark: false),
      );

      container.read(unifiedProvider.notifier).state = playbackState([
        stepPayload(
          stepIndex: 0,
          arraySnapshots: [
            arraySnapshot(name: 'arr', elementTy: 'int', elements: ['3', '1', '4']),
          ],
        ),
      ]);
      await tester.pump();

      expect(find.text('arr'), findsOneWidget);
      expect(find.text('3'), findsOneWidget);
      expect(find.text('1'), findsOneWidget);
      expect(find.text('4'), findsOneWidget);
    });

    testWidgets('highlights indices from vis events and swap label', (tester) async {
      final container = await pumpAppWithProviders(
        tester,
        builder: (_) => const ArrayVisTab(isDark: false),
      );

      container.read(unifiedProvider.notifier).state = playbackState([
        stepPayload(
          stepIndex: 0,
          semanticLabel: '交换 arr[1]↔arr[3]',
          arraySnapshots: [
            arraySnapshot(name: 'arr', elementTy: 'int', elements: ['3', '1', '4', '2']),
          ],
          visEvents: [visEvent(context: 'arr[1]:arr[3]')],
        ),
      ]);
      await tester.pump();

      // The labels [1] and [3] are rendered with swapped/highlighted styling.
      expect(find.text('[1]'), findsOneWidget);
      expect(find.text('[3]'), findsOneWidget);
    });
  });
}
