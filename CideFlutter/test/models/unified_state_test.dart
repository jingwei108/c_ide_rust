import 'package:flutter_test/flutter_test.dart';
import 'package:cide/models/unified_state.dart';

void main() {
  group('UnifiedState defaults', () {
    test('default values are correct', () {
      const state = UnifiedState();
      expect(state.phase, ExecutionPhase.idle);
      expect(state.currentStep, 0);
      expect(state.maxCollectedStep, 0);
      expect(state.totalSteps, 0);
      expect(state.frameCache, isEmpty);
      expect(state.isPlaying, false);
      expect(state.playbackSpeed, 1.0);
      expect(state.errorMessage, isNull);
      expect(state.isVmRestored, false);
      expect(state.heatmap, isNull);
      expect(state.currentLine, 0);
      expect(state.trapMessage, isNull);
      expect(state.algorithmMatches, isEmpty);
    });
  });

  group('UnifiedState copyWith', () {
    test('updates fields without changing others', () {
      const state = UnifiedState();
      final newState = state.copyWith(
        phase: ExecutionPhase.paused,
        currentStep: 5,
        playbackSpeed: 2.0,
      );
      expect(newState.phase, ExecutionPhase.paused);
      expect(newState.currentStep, 5);
      expect(newState.playbackSpeed, 2.0);
      expect(newState.maxCollectedStep, state.maxCollectedStep);
      expect(newState.totalSteps, state.totalSteps);
    });

    test('clearError removes errorMessage', () {
      const state = UnifiedState(errorMessage: 'oops');
      final cleared = state.copyWith(clearError: true);
      expect(cleared.errorMessage, isNull);
    });

    test('clearTrap removes trapMessage', () {
      const state = UnifiedState(trapMessage: 'trap');
      final cleared = state.copyWith(clearTrap: true);
      expect(cleared.trapMessage, isNull);
    });
  });

  group('ExecutionPhase getters', () {
    const idle = UnifiedState(phase: ExecutionPhase.idle);
    const compiling = UnifiedState(phase: ExecutionPhase.compiling);
    const collecting = UnifiedState(phase: ExecutionPhase.collecting);
    const paused = UnifiedState(phase: ExecutionPhase.paused);
    const playback = UnifiedState(phase: ExecutionPhase.playback);
    const seeking = UnifiedState(phase: ExecutionPhase.seeking);
    const stepMode = UnifiedState(phase: ExecutionPhase.stepMode);
    const error = UnifiedState(phase: ExecutionPhase.error);

    test('canPlay', () {
      expect(idle.canPlay, isTrue);
      expect(paused.canPlay, isTrue);
      expect(playback.canPlay, isTrue);
      expect(stepMode.canPlay, isTrue);
      expect(compiling.canPlay, isFalse);
      expect(collecting.canPlay, isFalse);
      expect(seeking.canPlay, isFalse);
      expect(error.canPlay, isFalse);
    });

    test('canPause', () {
      expect(collecting.canPause, isTrue);
      expect(idle.canPause, isFalse);
      expect(paused.canPause, isFalse);
      expect(playback.canPause, isFalse);
    });

    test('canStep', () {
      expect(paused.canStep, isTrue);
      expect(stepMode.canStep, isTrue);
      expect(idle.canStep, isFalse);
      expect(playback.canStep, isFalse);
      expect(collecting.canStep, isFalse);
    });

    test('canSeek', () {
      expect(playback.canSeek, isTrue);
      expect(paused.canSeek, isTrue);
      expect(stepMode.canSeek, isTrue);
      expect(collecting.canSeek, isTrue);
      expect(idle.canSeek, isFalse);
      expect(compiling.canSeek, isFalse);
      expect(error.canSeek, isFalse);
    });

    test('showSlider', () {
      expect(idle.showSlider, isFalse);
      expect(compiling.showSlider, isFalse);
      expect(error.showSlider, isFalse);
      expect(collecting.showSlider, isTrue);
      expect(paused.showSlider, isTrue);
      expect(playback.showSlider, isTrue);
      expect(seeking.showSlider, isTrue);
      expect(stepMode.showSlider, isTrue);
    });
  });
}
