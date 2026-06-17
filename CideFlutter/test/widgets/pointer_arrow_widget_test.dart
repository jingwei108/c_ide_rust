import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/widgets/pointer_arrow_painter.dart';
import '../helpers/pump_app.dart';

void main() {
  group('PointerArrowWidget', () {
    Future<void> pumpPointer(
      WidgetTester tester, {
      required PointerStatus status,
      int targetAddr = 0x3000,
      String targetName = '',
    }) async {
      await pumpWidget(
        tester,
        child: PointerArrowWidget(
          name: 'p',
          addr: 0x2000,
          tyName: 'int*',
          targetAddr: targetAddr,
          targetName: targetName,
          status: status,
          isDark: false,
        ),
      );
    }

    testWidgets('renders valid pointer', (tester) async {
      await pumpPointer(tester, status: PointerStatus.valid);

      expect(find.text('p'), findsOneWidget);
      expect(find.text('int*'), findsOneWidget);
      expect(find.text('0x2000'), findsOneWidget);
      expect(find.text('0x3000'), findsOneWidget);
      expect(find.text('有效'), findsOneWidget);
    });

    testWidgets('renders NULL pointer with reduced opacity', (tester) async {
      await pumpPointer(tester, status: PointerStatus.null_, targetAddr: 0);

      expect(find.text('NULL'), findsOneWidget);
      expect(find.text('0x0000'), findsOneWidget);
      final opacity = tester.widget<AnimatedOpacity>(find.byType(AnimatedOpacity));
      expect(opacity.opacity, 0.35);
    });

    testWidgets('renders freed pointer', (tester) async {
      await pumpPointer(tester, status: PointerStatus.freed);

      expect(find.text('已释放'), findsOneWidget);
    });

    testWidgets('renders dangling pointer', (tester) async {
      await pumpPointer(tester, status: PointerStatus.dangling);

      expect(find.text('悬空'), findsOneWidget);
    });
  });
}
