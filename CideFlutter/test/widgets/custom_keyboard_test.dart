import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/widgets/custom_keyboard.dart';

void main() {
  group('CustomKeyboard letters mode', () {
    testWidgets('tapping a letter calls onInsertText', (tester) async {
      final inserted = <String>[];
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: CustomKeyboard(
              onInsertText: (text) => inserted.add(text),
              onInsertPair: (l, r) {},
              onMoveCursor: (_) {},
              onBackspace: () {},
              onEnter: () {},
              onTab: () {},
            ),
          ),
        ),
      );

      await tester.tap(find.text('q'));
      await tester.pump();
      expect(inserted, ['q']);
    });

    testWidgets('tapping { } calls onInsertPair', (tester) async {
      String? left;
      String? right;
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: CustomKeyboard(
              onInsertText: (_) {},
              onInsertPair: (l, r) {
                left = l;
                right = r;
              },
              onMoveCursor: (_) {},
              onBackspace: () {},
              onEnter: () {},
              onTab: () {},
            ),
          ),
        ),
      );

      await tester.tap(find.text('{ }'));
      await tester.pump();
      expect(left, '{');
      expect(right, '}');
    });

    testWidgets('tapping semicolon calls onInsertText', (tester) async {
      final inserted = <String>[];
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: CustomKeyboard(
              onInsertText: (text) => inserted.add(text),
              onInsertPair: (l, r) {},
              onMoveCursor: (_) {},
              onBackspace: () {},
              onEnter: () {},
              onTab: () {},
            ),
          ),
        ),
      );

      await tester.tap(find.text(';'));
      await tester.pump();
      expect(inserted, [';']);
    });

    testWidgets('shift toggles uppercase and returns to lowercase after letter', (tester) async {
      final inserted = <String>[];
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: CustomKeyboard(
              onInsertText: (text) => inserted.add(text),
              onInsertPair: (l, r) {},
              onMoveCursor: (_) {},
              onBackspace: () {},
              onEnter: () {},
              onTab: () {},
            ),
          ),
        ),
      );

      // Lowercase initially.
      expect(find.text('q'), findsOneWidget);
      expect(find.text('Q'), findsNothing);

      await tester.tap(find.text('⇧'));
      await tester.pump();

      // Uppercase after shift.
      expect(find.text('Q'), findsOneWidget);
      expect(find.text('q'), findsNothing);

      await tester.tap(find.text('Q'));
      await tester.pump();

      // Returns to lowercase after typing a letter.
      expect(find.text('q'), findsOneWidget);
      expect(find.text('Q'), findsNothing);
      expect(inserted, ['Q']);
    });

    testWidgets('space inserts a space', (tester) async {
      final inserted = <String>[];
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: CustomKeyboard(
              onInsertText: (text) => inserted.add(text),
              onInsertPair: (l, r) {},
              onMoveCursor: (_) {},
              onBackspace: () {},
              onEnter: () {},
              onTab: () {},
            ),
          ),
        ),
      );

      await tester.tap(find.text('␣'));
      await tester.pump();
      expect(inserted, [' ']);
    });

    testWidgets('backspace calls onBackspace', (tester) async {
      var backspaceCount = 0;
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: CustomKeyboard(
              onInsertText: (_) {},
              onInsertPair: (l, r) {},
              onMoveCursor: (_) {},
              onBackspace: () => backspaceCount++,
              onEnter: () {},
              onTab: () {},
            ),
          ),
        ),
      );

      await tester.tap(find.text('⌫'));
      await tester.pump();
      expect(backspaceCount, 1);
    });

    testWidgets('enter and tab call respective callbacks', (tester) async {
      var enterCalled = false;
      var tabCalled = false;
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: CustomKeyboard(
              onInsertText: (_) {},
              onInsertPair: (l, r) {},
              onMoveCursor: (_) {},
              onBackspace: () {},
              onEnter: () => enterCalled = true,
              onTab: () => tabCalled = true,
            ),
          ),
        ),
      );

      await tester.tap(find.text('↵'));
      await tester.tap(find.text('Tab'));
      await tester.pump();
      expect(enterCalled, isTrue);
      expect(tabCalled, isTrue);
    });
  });

  group('CustomKeyboard mode switching', () {
    testWidgets('switching to numbers mode shows number pad', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: CustomKeyboard(
              onInsertText: (_) {},
              onInsertPair: (l, r) {},
              onMoveCursor: (_) {},
              onBackspace: () {},
              onEnter: () {},
              onTab: () {},
            ),
          ),
        ),
      );

      expect(find.text('1'), findsNothing);
      await tester.tap(find.text('123'));
      await tester.pump();
      expect(find.text('1'), findsOneWidget);
    });

    testWidgets('switching to symbols mode shows symbol grid', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: CustomKeyboard(
              onInsertText: (_) {},
              onInsertPair: (l, r) {},
              onMoveCursor: (_) {},
              onBackspace: () {},
              onEnter: () {},
              onTab: () {},
            ),
          ),
        ),
      );

      expect(find.text('{'), findsNothing);
      await tester.tap(find.text('符'));
      await tester.pump();
      // Default category "常用" shows common symbols like '{'.
      expect(find.text('{'), findsOneWidget);

      // Switch to the "其他" category.
      await tester.tap(find.text('其他'));
      await tester.pump();
      // Scroll the symbol grid to reveal 'sizeof' at the end of the list.
      await tester.scrollUntilVisible(
        find.text('sizeof'),
        50,
        scrollable: find.descendant(
          of: find.byType(GridView),
          matching: find.byType(Scrollable),
        ),
      );
      expect(find.text('sizeof'), findsOneWidget);
    });
  });
}
