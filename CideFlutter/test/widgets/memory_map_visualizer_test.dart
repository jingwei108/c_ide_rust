import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/widgets/memory_map_visualizer.dart';
import '../helpers/ide_state_factory.dart';
import '../helpers/pump_app.dart';

void main() {
  group('MemoryMapVisualizer', () {
    testWidgets('renders empty grid without regions', (tester) async {
      await pumpWidget(
        tester,
        child: const SizedBox(
          width: 400,
          height: 400,
          child: MemoryMapVisualizer(regions: [], isDark: false),
        ),
      );

      expect(find.text('空闲/已释放'), findsOneWidget);
      expect(find.text('1'), findsOneWidget);
    });

    testWidgets('renders heap stats and legend', (tester) async {
      await pumpWidget(
        tester,
        child: SizedBox(
          width: 400,
          height: 400,
          child: MemoryMapVisualizer(
            regions: [
              memoryRegion(name: 'h', isHeap: true, size: 256, allocBy: 'malloc', allocLine: 5),
            ],
            heapStats: heapStats(totalHeap: 1024, allocated: 256, fragmented: 128, fragmentationRate: 37),
            isDark: false,
          ),
        ),
      );

      expect(find.text('堆内存统计'), findsOneWidget);
      expect(find.text('1024B'), findsOneWidget);
      expect(find.text('256B'), findsOneWidget);
      expect(find.text('128B'), findsOneWidget);
      expect(find.text('37%'), findsOneWidget);
      expect(find.text('堆'), findsWidgets);
    });

    testWidgets('shows block details on tap', (tester) async {
      await pumpWidget(
        tester,
        child: SizedBox(
          width: 400,
          height: 400,
          child: MemoryMapVisualizer(
            regions: [
              memoryRegion(name: 'heapBlock', isHeap: true, addr: 0x2000, size: 256, allocBy: 'malloc', allocLine: 7),
            ],
            fragments: [
              memoryFragment(addr: 0x2100, size: 64),
            ],
            isDark: false,
          ),
        ),
      );

      // Block at 0x2000 is block index 2 (4KB per block).
      await tester.tap(find.text('3'));
      await tester.pumpAndSettle();

      expect(find.textContaining('heapBlock'), findsOneWidget);
      expect(find.textContaining('分配于第 7 行'), findsOneWidget);
      expect(find.text('碎片区（外部碎片）'), findsOneWidget);
    });
  });
}
