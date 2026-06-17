import 'package:flutter_test/flutter_test.dart';
import 'package:cide/editor/find_replace_controller.dart';

void main() {
  group('FindReplaceController', () {
    test('show/hide toggles visibility and resets matches', () {
      final controller = FindReplaceController();

      expect(controller.visible, isFalse);

      controller.show();
      expect(controller.visible, isTrue);

      controller.setQuery('foo');
      controller.search('foo bar');
      expect(controller.hasMatches, isTrue);

      controller.hide();
      expect(controller.visible, isFalse);
      expect(controller.hasMatches, isFalse);
    });

    test('search finds all literal matches', () {
      final controller = FindReplaceController();
      controller.setQuery('ab');
      controller.search('abab');

      expect(controller.matches.length, 2);
      expect(controller.matches[0].start, 0);
      expect(controller.matches[0].end, 2);
      expect(controller.matches[1].start, 2);
      expect(controller.matches[1].end, 4);
      expect(controller.currentMatchIndex, 0);
    });

    test('search is case insensitive by default', () {
      final controller = FindReplaceController();
      controller.setQuery('Hello');
      controller.search('hello HELLO');

      expect(controller.matches.length, 2);
    });

    test('case sensitive search respects case', () {
      final controller = FindReplaceController();
      controller.setQuery('Hello');
      controller.toggleCaseSensitive();
      controller.search('hello HELLO Hello');

      expect(controller.matches.length, 1);
      expect(controller.matches.first.start, 12);
    });

    test('regex search uses pattern', () {
      final controller = FindReplaceController();
      controller.setQuery(r'\d+');
      controller.toggleRegex();
      controller.search('a 12 b 345');

      expect(controller.matches.length, 2);
      expect(controller.matches[0].start, 2);
      expect(controller.matches[0].end, 4);
      expect(controller.matches[1].start, 7);
      expect(controller.matches[1].end, 10);
    });

    test('invalid regex is handled gracefully', () {
      final controller = FindReplaceController();
      controller.setQuery('[');
      controller.toggleRegex();
      controller.search('abc');

      expect(controller.matches, isEmpty);
      expect(controller.hasMatches, isFalse);
    });

    test('nextMatch cycles forward', () {
      final controller = FindReplaceController();
      controller.setQuery('a');
      controller.search('aba');

      expect(controller.currentMatchIndex, 0);
      controller.nextMatch();
      expect(controller.currentMatchIndex, 1);
      controller.nextMatch();
      expect(controller.currentMatchIndex, 0);
    });

    test('previousMatch cycles backward', () {
      final controller = FindReplaceController();
      controller.setQuery('a');
      controller.search('aba');

      controller.previousMatch();
      expect(controller.currentMatchIndex, 1);
      controller.previousMatch();
      expect(controller.currentMatchIndex, 0);
    });

    test('currentMatch returns null when no matches', () {
      final controller = FindReplaceController();
      controller.setQuery('xyz');
      controller.search('abc');

      expect(controller.currentMatch, isNull);
    });

    test('setReplacement updates replacement text', () {
      final controller = FindReplaceController();
      controller.setReplacement('X');

      expect(controller.replacement, 'X');
    });
  });
}
