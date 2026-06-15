part of '../ide_notifier.dart';

/// 运行与调试控制：运行、单步、输入、断点、输出清理。
mixin RunNotifierMixin on AutoDisposeNotifier<IdeState>, CompileNotifierMixin {
  RustApiService get _rustApi => ref.read(rustApiServiceProvider);
  Future<void> run() async {
    if (!state.isRunning) {
      await compileOnly();
      if (state.hasErrors) {
        state = state.copyWith(error: '请先修复编译错误');
        return;
      }
    }
    state = state.copyWith(
      isRunning: true,
      isStepMode: false,
      output: '',
      clearError: true,
    );
    try {
      final result = await _rustApi.runCode();
      state = state.copyWith(
        isRunning: result.waitingInput,
        output: result.output,
        waitingInput: result.waitingInput,
        error: result.error,
      );
    } catch (e) {
      state = state.copyWith(isRunning: false, error: '运行异常: $e');
    }
  }

  Future<void> step() async {
    if (!state.isRunning) {
      await compileOnly();
      if (state.hasErrors) {
        state = state.copyWith(error: '请先修复编译错误');
        return;
      }
    }
    state = state.copyWith(isStepMode: true, isRunning: true, clearError: true);
    try {
      final result = await _rustApi.stepNext();
      if (state.executionSpeed > 0) {
        await Future.delayed(Duration(milliseconds: state.executionSpeed));
      }
      state = state.copyWith(
        isRunning:
            result.status != rust.StepStatus.finished &&
            result.status != rust.StepStatus.trap,
        isStepMode:
            result.status != rust.StepStatus.finished &&
            result.status != rust.StepStatus.trap,
        currentLine: result.currentLine,
        output: result.output,
        waitingInput: result.waitingInput,
        error: result.status == rust.StepStatus.trap ? '运行出错' : null,
      );
    } catch (e) {
      state = state.copyWith(
        isRunning: false,
        isStepMode: false,
        error: '单步异常: $e',
      );
    }
  }

  Future<void> provideInput(String line) async {
    try {
      await _rustApi.provideInputLine(line: line);
      if (state.isStepMode) {
        await step();
      } else {
        await run();
      }
    } catch (e) {
      state = state.copyWith(error: '输入异常: $e');
    }
  }

  Future<void> reset() async {
    await _rustApi.resetSession();
    state = const IdeState();
  }

  void clearOutput() {
    state = state.copyWith(output: '');
  }

  void clearError() {
    state = state.copyWith(clearError: true);
  }

  Future<void> toggleBreakpoint(int line) async {
    final newBreakpoints = Set<int>.from(state.breakpoints);
    if (newBreakpoints.contains(line)) {
      newBreakpoints.remove(line);
    } else {
      newBreakpoints.add(line);
    }
    await _rustApi.setBreakpoints(lines: newBreakpoints.toList());
    state = state.copyWith(breakpoints: newBreakpoints);
  }
}
