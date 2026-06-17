import 'package:flutter_test/flutter_test.dart';
import 'package:cide/providers/unified_provider.dart';
import 'package:cide/widgets/memory_map_visualizer.dart';
import 'package:cide/widgets/memory_tab.dart';
import '../helpers/ide_state_factory.dart';
import '../helpers/pump_app.dart';

void main() {
  group('MemoryTab', () {
    testWidgets('shows empty hint when no memory regions', (tester) async {
      await pumpAppWithProviders(
        tester,
        builder: (_) => const MemoryTab(isDark: false),
      );

      expect(find.text('无内存信息'), findsOneWidget);
    });

    testWidgets('renders MemoryMapVisualizer when regions exist', (tester) async {
      final container = await pumpAppWithProviders(
        tester,
        builder: (_) => const MemoryTab(isDark: false),
      );

      container.read(unifiedProvider.notifier).state = playbackStateWithMemory(
        cache: [stepPayload(stepIndex: 0)],
        regions: [
          memoryRegion(name: 'x', isHeap: true, addr: 0x2000, size: 64, allocBy: 'malloc'),
        ],
      );
      await tester.pump();

      expect(find.byType(MemoryMapVisualizer), findsOneWidget);
      expect(find.text('堆'), findsWidgets);
    });
  });
}
