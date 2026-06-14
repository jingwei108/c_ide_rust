part of '../ide_notifier.dart';

/// 文件管理：增删改、当前文件切换、源码同步。
mixin FileNotifierMixin on AutoDisposeNotifier<IdeState> {
  void updateSource(String value) {
    final newFiles =
        state.files.map((f) {
          if (f.filename == state.currentFile) {
            return f.copyWith(source: value);
          }
          return f;
        }).toList();
    state = state.copyWith(source: value, files: newFiles);
  }

  void addFile(String filename) {
    if (state.files.any((f) => f.filename == filename)) return;
    final newFiles = List<CodeFile>.from(state.files)
      ..add(CodeFile(filename: filename, source: ''));
    state = state.copyWith(files: newFiles, currentFile: filename, source: '');
  }

  void removeFile(String filename) {
    if (state.files.length <= 1) return;
    final newFiles = state.files.where((f) => f.filename != filename).toList();
    String newCurrent = state.currentFile;
    String newSource = state.source;
    if (state.currentFile == filename) {
      newCurrent = newFiles.first.filename;
      newSource = newFiles.first.source;
    }
    state = state.copyWith(
      files: newFiles,
      currentFile: newCurrent,
      source: newSource,
    );
  }

  void switchFile(String filename) {
    final file = state.files.firstWhere(
      (f) => f.filename == filename,
      orElse: () => state.files.first,
    );
    state = state.copyWith(currentFile: filename, source: file.source);
  }

  /// 设置指定文件的源码，并同步当前编辑器内容（如正在显示该文件）。
  void setFileSource(String filename, String source) {
    final newFiles =
        state.files.map((f) {
          if (f.filename == filename) {
            return f.copyWith(source: source);
          }
          return f;
        }).toList();
    if (state.currentFile == filename) {
      state = state.copyWith(files: newFiles, source: source);
    } else {
      state = state.copyWith(files: newFiles);
    }
  }
}
