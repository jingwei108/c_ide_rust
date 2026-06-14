import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/providers/unified_provider.dart';
import 'package:cide/src/rust/unified/types.dart' as types;

types.StepPayload _dummyStepPayload({required int stepIndex, required int codeLine}) {
  return types.StepPayload(
    stepIndex: stepIndex,
    codeLine: codeLine,
    funcName: 'main',
    semanticLabel: '',
    localVars: const [],
    callStack: const [],
    visEvents: const [],
    heatmapLine: 0,
    heatmapCount: BigInt.zero,
    accessedVars: const [],
    arraySnapshots: const [],
    pointerSnapshots: const [],
  );
}

void main() {
  group('UnifiedNotifier build', () {
    test('default state is idle', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final state = container.read(unifiedProvider);
      expect(state.phase, ExecutionPhase.idle);
      expect(state.currentStep, 0);
      expect(state.frameCache, isEmpty);
      expect(state.isPlaying, isFalse);
    });
  });

  group('UnifiedNotifier playback controls without Rust', () {
    test('setPlaybackSpeed updates speed', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(unifiedProvider.notifier);
      notifier.setPlaybackSpeed(2.5);

      expect(container.read(unifiedProvider).playbackSpeed, 2.5);
    });

    test('onSliderChanged updates current step and line from cache', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(unifiedProvider.notifier);
      // Build a cache with 3 frames.
      final cache = [
        _dummyStepPayload(stepIndex: 0, codeLine: 1),
        _dummyStepPayload(stepIndex: 1, codeLine: 2),
        _dummyStepPayload(stepIndex: 2, codeLine: 3),
      ];
      notifier.state = container.read(unifiedProvider).copyWith(
        phase: ExecutionPhase.playback,
        frameCache: cache,
        maxCollectedStep: 2,
      );

      notifier.onSliderChanged(1);

      final state = container.read(unifiedProvider);
      expect(state.currentStep, 1);
      expect(state.currentLine, 2);
    });

    test('onSliderChanged ignores out of bounds', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(unifiedProvider.notifier);
      notifier.state = container.read(unifiedProvider).copyWith(
        phase: ExecutionPhase.playback,
        frameCache: [_dummyStepPayload(stepIndex: 0, codeLine: 1)],
      );

      notifier.onSliderChanged(10);

      final state = container.read(unifiedProvider);
      expect(state.currentStep, 0);
    });

    test('isPaused getter', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(unifiedProvider.notifier);
      expect(notifier.isPaused, isFalse);

      notifier.state = container.read(unifiedProvider).copyWith(
        phase: ExecutionPhase.paused,
      );
      expect(notifier.isPaused, isTrue);
    });
  });

  group('UnifiedNotifier onCodeChanged', () {
    test('resets to idle when not idle', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(unifiedProvider.notifier);
      notifier.state = container.read(unifiedProvider).copyWith(
        phase: ExecutionPhase.playback,
        isPlaying: true,
        frameCache: [_dummyStepPayload(stepIndex: 0, codeLine: 1)],
        currentStep: 5,
        maxCollectedStep: 5,
        totalSteps: 10,
        algorithmMatches: const [],
      );

      notifier.onCodeChanged();

      final state = container.read(unifiedProvider);
      expect(state.phase, ExecutionPhase.idle);
      expect(state.isPlaying, isFalse);
      expect(state.frameCache, isEmpty);
      expect(state.currentStep, 0);
      expect(state.maxCollectedStep, 0);
      expect(state.totalSteps, 0);
    });

    test('does nothing when already idle', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(unifiedProvider.notifier);
      notifier.onCodeChanged();

      final state = container.read(unifiedProvider);
      expect(state.phase, ExecutionPhase.idle);
    });
  });
}
