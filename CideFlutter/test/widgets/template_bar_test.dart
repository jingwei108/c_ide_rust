import 'package:flutter_test/flutter_test.dart';
import 'package:cide/models/code_template.dart';
import 'package:cide/widgets/template_bar.dart';
import '../helpers/pump_app.dart';

void main() {
  group('TemplateBar', () {
    final templates = [
      CodeTemplate('bubble_sort', '冒泡排序', 'sort', 'code'),
      CodeTemplate('binary_search', '二分查找', 'search', 'code'),
    ];

    testWidgets('renders all template names', (tester) async {
      await pumpWidget(
        tester,
        child: TemplateBar(
          templates: templates,
          onSelectTemplate: (_) {},
        ),
      );

      expect(find.text('冒泡排序'), findsOneWidget);
      expect(find.text('二分查找'), findsOneWidget);
    });

    testWidgets('calls onSelectTemplate when chip tapped', (tester) async {
      CodeTemplate? selected;
      await pumpWidget(
        tester,
        child: TemplateBar(
          templates: templates,
          onSelectTemplate: (t) => selected = t,
        ),
      );

      await tester.tap(find.text('冒泡排序'));
      await tester.pump();

      expect(selected, isNotNull);
      expect(selected!.key, 'bubble_sort');
    });
  });
}
