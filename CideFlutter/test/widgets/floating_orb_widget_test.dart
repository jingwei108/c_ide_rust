import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/widgets/floating_orb_widget.dart';
import '../helpers/pump_app.dart';

void main() {
  group('FloatingOrbWidget', () {
    testWidgets('renders orb and does not show menu when closed', (tester) async {
      await pumpWidget(
        tester,
        child: FloatingOrbWidget(
          isMenuOpen: false,
          menuItems: const ['memory', 'variables'],
          onToggleMenu: () {},
          onSelectPanel: (_) {},
          onCloseMenu: () {},
        ),
        wrapScaffold: false,
      );

      // CustomPaint is used by the breathing orb.
      expect(find.byType(CustomPaint), findsWidgets);
      expect(find.text('内存区域'), findsNothing);
    });

    testWidgets('renders menu items when open', (tester) async {
      await pumpWidget(
        tester,
        child: FloatingOrbWidget(
          isMenuOpen: true,
          menuItems: const ['memory', 'variables'],
          onToggleMenu: () {},
          onSelectPanel: (_) {},
          onCloseMenu: () {},
        ),
        wrapScaffold: false,
      );

      expect(find.text('内存区域'), findsOneWidget);
      expect(find.text('局部变量'), findsOneWidget);
    });
  });
}
