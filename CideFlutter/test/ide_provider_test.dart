import 'package:flutter_test/flutter_test.dart';
import 'package:cide/providers/ide_provider.dart';

void main() {
  test('IdeState default values', () {
    const state = IdeState();
    expect(state.isCompiling, false);
    expect(state.isRunning, false);
    expect(state.output, '');
    expect(state.diagnostics.isEmpty, true);
    expect(state.source.contains('Hello, Cide!'), true);
  });

  test('IdeState copyWith', () {
    const state = IdeState();
    final newState = state.copyWith(isCompiling: true, output: 'test');
    expect(newState.isCompiling, true);
    expect(newState.output, 'test');
    expect(newState.source, state.source);
  });
}
