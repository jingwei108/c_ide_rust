import 'package:flutter_test/flutter_test.dart';
import 'package:cide/models/code_template.dart';

void main() {
  group('CodeTemplate.buildCode', () {
    const template = CodeTemplate(
      'bubble_sort',
      'Bubble Sort',
      'sort',
      'void sort(int /*__PARAM_n__*/ 5) { /*__PARAM_name__*/ x; }',
    );

    test('replaces all placeholders with provided values', () {
      final code = template.buildCode({'n': '10', 'name': 'arr'});
      expect(code, 'void sort(int 10) { arr; }');
    });

    test('keeps default value when parameter is missing', () {
      final code = template.buildCode({'name': 'arr'});
      expect(code, 'void sort(int 5) { arr; }');
    });

    test('ignores extra values', () {
      final code = template.buildCode({'n': '10', 'name': 'arr', 'extra': 'ignored'});
      expect(code, 'void sort(int 10) { arr; }');
    });

    test('returns original code when no placeholders', () {
      const plain = CodeTemplate('plain', 'Plain', 'basic', 'int main() { return 0; }');
      expect(plain.buildCode({}), 'int main() { return 0; }');
    });
  });

  group('TemplateParam', () {
    test('holds default values', () {
      const param = TemplateParam(
        key: 'n',
        label: 'Count',
        defaultValue: '5',
      );
      expect(param.key, 'n');
      expect(param.label, 'Count');
      expect(param.defaultValue, '5');
      expect(param.type, ParamType.int);
    });
  });

  group('TutorialStep', () {
    test('holds fields', () {
      const step = TutorialStep(
        title: 'Step 1',
        description: 'Description',
        focusLines: [1, 2],
      );
      expect(step.title, 'Step 1');
      expect(step.description, 'Description');
      expect(step.focusLines, [1, 2]);
      expect(step.explanations, isEmpty);
    });
  });
}
