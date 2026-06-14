import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/models/ide_state.dart';
import 'package:cide/widgets/file_tab_bar.dart';

void main() {
  group('FileTabBar', () {
    testWidgets('renders all file names', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: FileTabBar(
              files: const [
                CodeFile(filename: 'main.c', source: ''),
                CodeFile(filename: 'helper.c', source: ''),
              ],
              currentFile: 'main.c',
              onSwitch: (_) {},
              onClose: (_) {},
              onAdd: () {},
            ),
          ),
        ),
      );

      expect(find.text('main.c'), findsOneWidget);
      expect(find.text('helper.c'), findsOneWidget);
    });

    testWidgets('tapping tab triggers onSwitch', (tester) async {
      String? switched;
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: FileTabBar(
              files: const [
                CodeFile(filename: 'main.c', source: ''),
                CodeFile(filename: 'helper.c', source: ''),
              ],
              currentFile: 'main.c',
              onSwitch: (file) => switched = file,
              onClose: (_) {},
              onAdd: () {},
            ),
          ),
        ),
      );

      await tester.tap(find.text('helper.c'));
      await tester.pump();
      expect(switched, 'helper.c');
    });

    testWidgets('shows close buttons when more than one file', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: FileTabBar(
              files: const [
                CodeFile(filename: 'main.c', source: ''),
                CodeFile(filename: 'helper.c', source: ''),
              ],
              currentFile: 'main.c',
              onSwitch: (_) {},
              onClose: (_) {},
              onAdd: () {},
            ),
          ),
        ),
      );

      expect(find.byIcon(Icons.close), findsNWidgets(2));
    });

    testWidgets('hides close button when only one file', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: FileTabBar(
              files: const [CodeFile(filename: 'main.c', source: '')],
              currentFile: 'main.c',
              onSwitch: (_) {},
              onClose: (_) {},
              onAdd: () {},
            ),
          ),
        ),
      );

      expect(find.byIcon(Icons.close), findsNothing);
    });

    testWidgets('tapping close triggers onClose', (tester) async {
      String? closed;
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: FileTabBar(
              files: const [
                CodeFile(filename: 'main.c', source: ''),
                CodeFile(filename: 'helper.c', source: ''),
              ],
              currentFile: 'main.c',
              onSwitch: (_) {},
              onClose: (file) => closed = file,
              onAdd: () {},
            ),
          ),
        ),
      );

      // Tap the close icon inside the helper.c tab.
      final helperTab = find.widgetWithText(GestureDetector, 'helper.c');
      final closeIcon = find.descendant(
        of: helperTab,
        matching: find.byIcon(Icons.close),
      );
      await tester.tap(closeIcon);
      await tester.pump();
      expect(closed, 'helper.c');
    });

    testWidgets('tapping add button triggers onAdd', (tester) async {
      var added = false;
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: FileTabBar(
              files: const [CodeFile(filename: 'main.c', source: '')],
              currentFile: 'main.c',
              onSwitch: (_) {},
              onClose: (_) {},
              onAdd: () => added = true,
            ),
          ),
        ),
      );

      await tester.tap(find.byIcon(Icons.add));
      await tester.pump();
      expect(added, isTrue);
    });
  });
}
