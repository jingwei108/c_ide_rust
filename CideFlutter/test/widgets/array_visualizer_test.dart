import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/widgets/array_visualizer.dart';
import '../helpers/pump_app.dart';

void main() {
  group('ArrayVisualizer', () {
    Future<void> pumpArray(
      WidgetTester tester, {
      required String name,
      required List<String> elements,
      Set<int> highlightedIndices = const {},
      Set<int> swappedIndices = const {},
    }) async {
      await pumpWidget(
        tester,
        child: SizedBox(
          width: 1600,
          height: 300,
          child: ArrayVisualizer(
            name: name,
            elementTy: 'int',
            elements: elements,
            highlightedIndices: highlightedIndices,
            swappedIndices: swappedIndices,
            isDark: false,
          ),
        ),
      );
    }

    testWidgets('renders array name, type and element count', (tester) async {
      await pumpArray(tester, name: 'arr', elements: ['3', '1', '4']);

      expect(find.text('arr'), findsOneWidget);
      expect(find.text('int'), findsOneWidget);
      expect(find.text('3 个元素'), findsOneWidget);
      expect(find.text('3'), findsOneWidget);
      expect(find.text('1'), findsOneWidget);
      expect(find.text('4'), findsOneWidget);
    });

    testWidgets('truncates to 40 elements', (tester) async {
      tester.view.physicalSize = const Size(1800, 600);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      final elements = List.generate(50, (i) => '$i');
      await pumpArray(tester, name: 'big', elements: elements);

      expect(find.text('50 个元素'), findsOneWidget);
      // Only the first 40 elements are rendered.
      expect(find.text('39'), findsOneWidget);
      expect(find.text('40'), findsNothing);
    });

    testWidgets('highlights and swaps indices', (tester) async {
      await pumpArray(
        tester,
        name: 'arr',
        elements: ['3', '1', '4'],
        highlightedIndices: {0},
        swappedIndices: {1, 2},
      );

      // Labels remain visible even when highlighted/swapped.
      expect(find.text('[0]'), findsOneWidget);
      expect(find.text('[1]'), findsOneWidget);
      expect(find.text('[2]'), findsOneWidget);
    });

    testWidgets('renders negative values', (tester) async {
      await pumpArray(tester, name: 'arr', elements: ['-5', '0', '5']);

      expect(find.text('-5'), findsOneWidget);
      expect(find.text('0'), findsOneWidget);
      expect(find.text('5'), findsOneWidget);
    });
  });
}
