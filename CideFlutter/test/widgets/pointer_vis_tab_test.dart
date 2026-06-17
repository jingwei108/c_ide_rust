import 'package:flutter_test/flutter_test.dart';
import 'package:cide/providers/unified_provider.dart';
import 'package:cide/src/rust/unified/types.dart' as unified;
import 'package:cide/widgets/pointer_arrow_painter.dart' as arrow;
import 'package:cide/widgets/pointer_vis_tab.dart';
import '../helpers/ide_state_factory.dart';
import '../helpers/pump_app.dart';

void main() {
  group('PointerVisTab', () {
    testWidgets('shows empty hint when no frame cache', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (_) => const PointerVisTab(isDark: false),
      );

      expect(find.text('运行程序以查看指针追踪'), findsOneWidget);
    });

    testWidgets('shows empty hint when no pointer snapshots', (tester) async {
      final container = await pumpAppWithProviders(
        tester,
        builder: (_) => const PointerVisTab(isDark: false),
      );

      container.read(unifiedProvider.notifier).state = playbackState([
        stepPayload(stepIndex: 0),
      ]);
      await tester.pump();

      expect(find.text('当前步未检测到指针变量'), findsOneWidget);
    });

    testWidgets('renders pointer snapshots from unified state', (tester) async {
      final container = await pumpAppWithProviders(
        tester,
        builder: (_) => const PointerVisTab(isDark: false),
      );

      container.read(unifiedProvider.notifier).state = playbackState([
        stepPayload(
          stepIndex: 0,
          pointerSnapshots: [
            pointerSnapshot(
              name: 'p',
              tyName: 'int*',
              targetName: 'x',
              status: unified.PointerStatus.valid,
            ),
          ],
        ),
      ]);
      await tester.pump();

      expect(find.text('p'), findsOneWidget);
      expect(find.text('int*'), findsOneWidget);
      expect(find.text('x'), findsOneWidget);
      expect(find.byType(arrow.PointerArrowWidget), findsOneWidget);
    });
  });
}
