part of '../ide_notifier.dart';

/// 进度持久化：被编译、学习等多个 mixin 共享。
mixin ProgressMixin on AutoDisposeNotifier<IdeState> {
  Future<void> _loadProgress() async {
    final progress = await LearningProgressService.load();
    state = state.copyWith(learningProgress: progress);
  }

  Future<void> _saveProgress() async {
    await LearningProgressService.save(state.learningProgress);
  }
}
