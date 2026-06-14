import 'package:flutter_test/flutter_test.dart';
import 'package:cide/models/ide_state.dart';
import 'package:cide/src/rust/api/types.dart' as rust;

void main() {
  group('IdeState defaults', () {
    test('default values are correct', () {
      const state = IdeState();
      expect(state.files.length, 1);
      expect(state.files.first.filename, 'main.c');
      expect(state.currentFile, 'main.c');
      expect(state.source.contains('Hello, Cide!'), isTrue);
      expect(state.isCompiling, isFalse);
      expect(state.isRunning, isFalse);
      expect(state.isStepMode, isFalse);
      expect(state.output, '');
      expect(state.diagnostics, isEmpty);
      expect(state.knowledgeCards, isEmpty);
      expect(state.breakpoints, isEmpty);
      expect(state.bottomSlots, isNotEmpty);
      expect(state.floatingSlots, isNotEmpty);
      expect(state.bottomHeight, 220);
      expect(state.executionSpeed, 0);
      expect(state.showIntro, isFalse);
    });
  });

  group('IdeState copyWith', () {
    test('updates fields without changing others', () {
      const state = IdeState();
      final newState = state.copyWith(
        isCompiling: true,
        output: 'test output',
        currentLine: 7,
      );
      expect(newState.isCompiling, isTrue);
      expect(newState.output, 'test output');
      expect(newState.currentLine, 7);
      expect(newState.source, state.source);
      expect(newState.files, state.files);
    });

    test('clearError removes error', () {
      const state = IdeState(error: 'error');
      final cleared = state.copyWith(clearError: true);
      expect(cleared.error, isNull);
    });
  });

  group('CodeFile', () {
    test('copyWith updates fields', () {
      const file = CodeFile(filename: 'main.c', source: 'int main() {}');
      final updated = file.copyWith(source: 'int x;');
      expect(updated.filename, 'main.c');
      expect(updated.source, 'int x;');
    });
  });

  group('hasErrors / hasWarnings', () {
    test('hasErrors true when severity 0 present', () {
      final state = IdeState(
        diagnostics: [
          rust.Diagnostic(
            severity: 0,
            errorCode: 1001,
            message: 'error',
            filename: 'main.c',
            line: 1,
            column: 1,
            fixSuggestion: '',
            fixKind: 0,
            replaceStartLine: 0,
            replaceStartColumn: 0,
            replaceEndLine: 0,
            replaceEndColumn: 0,
            replacementText: '',
          ),
        ],
      );
      expect(state.hasErrors, isTrue);
      expect(state.hasWarnings, isFalse);
    });

    test('hasWarnings true when severity 1 present', () {
      final state = IdeState(
        diagnostics: [
          rust.Diagnostic(
            severity: 1,
            errorCode: 2001,
            message: 'warning',
            filename: 'main.c',
            line: 1,
            column: 1,
            fixSuggestion: '',
            fixKind: 0,
            replaceStartLine: 0,
            replaceStartColumn: 0,
            replaceEndLine: 0,
            replaceEndColumn: 0,
            replacementText: '',
          )
        ],
      );
      expect(state.hasErrors, isFalse);
      expect(state.hasWarnings, isTrue);
    });

    test('no errors or warnings when empty', () {
      const state = IdeState();
      expect(state.hasErrors, isFalse);
      expect(state.hasWarnings, isFalse);
    });
  });
}
