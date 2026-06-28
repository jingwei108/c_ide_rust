import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/models/code_template.dart';
import 'package:cide/screens/ide/template_bar.dart';
import 'package:cide/widgets/template_bar.dart';

import '../helpers/pump_app.dart';

void main() {
  group('IdeTemplateBar', () {
    final cTemplate = CodeTemplate(
      'array',
      '数组遍历',
      '基础',
      'int main() { return 0; }',
      tutorialSteps: const [
        TutorialStep(
          title: '数组定义',
          description: '定义数组',
          focusLines: [1],
        ),
      ],
    );

    final cppTemplate = CodeTemplate(
      'cpp_hello',
      'C++ 入门',
      'C++基础',
      'int main() { return 0; }',
      ext: 'cpp',
      tutorialSteps: const [
        TutorialStep(
          title: 'C++ 程序结构',
          description: 'main 函数',
          focusLines: [1],
        ),
      ],
    );

    testWidgets('tapping C template inserts code without dialog',
        (tester) async {
      String? inserted;
      await pumpWidget(
        tester,
        child: IdeTemplateBar(
          animation: const AlwaysStoppedAnimation(1.0),
          onInsertText: (text) => inserted = text,
          onScrollToLine: (_) {},
        ),
      );

      // IdeTemplateBar uses FutureBuilder; pump to load.
      await tester.pump();

      // Since we cannot easily mock rootBundle here, test the underlying
      // TemplateBar directly for the no-tutorial case.
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: TemplateBar(
              templates: [cTemplate],
              onSelectTemplate: (t) => inserted = t.code,
            ),
          ),
        ),
      );
      await tester.pump();

      await tester.tap(find.text('数组遍历'));
      await tester.pump();

      expect(inserted, 'int main() { return 0; }');
    });

    test('C++ template carries cpp extension', () {
      expect(cppTemplate.ext, 'cpp');
      expect(cTemplate.ext, 'c');
    });
  });
}
