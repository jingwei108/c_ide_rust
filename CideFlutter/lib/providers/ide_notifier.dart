import 'dart:math' as math;
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:cide/src/rust/api/types.dart' as rust;
import 'package:cide/src/rust/compiler/intent.dart';
import '../models/algorithm_validation.dart';
import '../models/code_template.dart';
import '../models/ide_state.dart';
import '../models/knowledge_card.dart';
import '../models/learning_progress.dart';
import '../services/learning_progress_service.dart';
import 'unified_provider.dart';

part 'notifiers/progress_notifier.dart';
part 'notifiers/file_notifier.dart';
part 'notifiers/compile_notifier.dart';
part 'notifiers/run_notifier.dart';
part 'notifiers/panel_notifier.dart';
part 'notifiers/learning_notifier.dart';

/// ---------------------------------------------------------------------------
/// IdeNotifier —— IDE 主控 Notifier
///
/// 本类仅作为入口与 facade，具体职责已拆分到以下 mixin：
///   - [FileNotifierMixin]     文件增删改、当前文件切换、源码同步
///   - [CompileNotifierMixin]  编译、诊断、意图推断、学习进度 streak
///   - [RunNotifierMixin]      运行/单步/输入/断点/输出清理
///   - [PanelNotifierMixin]    底部面板、悬浮球、高亮线布局管理
///   - [LearningNotifierMixin] 学习进度、教程、算法验证、Watch/Intro
/// ---------------------------------------------------------------------------
class IdeNotifier extends AutoDisposeNotifier<IdeState>
    with
        ProgressMixin,
        FileNotifierMixin,
        CompileNotifierMixin,
        RunNotifierMixin,
        PanelNotifierMixin,
        LearningNotifierMixin {
  final _outputController = TextEditingController();
  TextEditingController get outputController => _outputController;

  @override
  IdeState build() {
    ref.onDispose(() {
      _outputController.dispose();
    });
    // 延迟加载持久化进度
    Future.microtask(_loadProgress);
    return const IdeState();
  }
}
