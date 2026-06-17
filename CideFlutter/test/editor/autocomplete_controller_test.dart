import 'package:flutter_test/flutter_test.dart';
import 'package:cide/editor/autocomplete_controller.dart';

void main() {
  group('AutocompleteController', () {
    test('extractPrefix returns identifier suffix', () {
      expect(AutocompleteController.extractPrefix('int fo'), 'fo');
      expect(AutocompleteController.extractPrefix('a.b_c'), 'b_c');
      expect(AutocompleteController.extractPrefix('   '), '');
      expect(AutocompleteController.extractPrefix('printf("'), '');
    });

    test('update filters candidates by prefix', () {
      final controller = AutocompleteController();
      controller.update('int pr');

      expect(controller.visible, isTrue);
      expect(controller.prefix, 'pr');
      expect(controller.candidates.any((c) => c.word == 'printf'), isTrue);
      expect(controller.candidates.any((c) => c.word == 'int'), isFalse);
    });

    test('update hides when prefix is empty', () {
      final controller = AutocompleteController();
      controller.update('   ');

      expect(controller.visible, isFalse);
      expect(controller.candidates, isEmpty);
    });

    test('update hides when no candidates match', () {
      final controller = AutocompleteController();
      controller.update('zzz');

      expect(controller.visible, isFalse);
    });

    test('selectNext cycles through candidates', () {
      final controller = AutocompleteController();
      controller.update('in');
      final count = controller.candidates.length;
      expect(count, greaterThan(1));

      expect(controller.selectedIndex, 0);
      controller.selectNext();
      expect(controller.selectedIndex, 1);
      for (var i = 0; i < count - 1; i++) {
        controller.selectNext();
      }
      expect(controller.selectedIndex, 0);
    });

    test('selectPrevious cycles backward', () {
      final controller = AutocompleteController();
      controller.update('in');
      final count = controller.candidates.length;

      controller.selectPrevious();
      expect(controller.selectedIndex, count - 1);
      controller.selectPrevious();
      expect(controller.selectedIndex, count - 2);
    });

    test('confirm returns selected candidate and hides', () {
      final controller = AutocompleteController();
      controller.update('pr');
      final selected = controller.confirm();

      expect(selected, isNotNull);
      expect(selected!.word, 'printf');
      expect(controller.visible, isFalse);
    });

    test('confirm returns null when not visible', () {
      final controller = AutocompleteController();
      controller.update('zzz');

      expect(controller.confirm(), isNull);
    });

    test('hide clears state', () {
      final controller = AutocompleteController();
      controller.update('pr');
      expect(controller.visible, isTrue);

      controller.hide();
      expect(controller.visible, isFalse);
      expect(controller.candidates, isEmpty);
      expect(controller.selectedIndex, 0);
    });
  });
}
