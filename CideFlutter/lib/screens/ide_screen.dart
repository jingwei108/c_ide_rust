import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:cide/src/rust/api/cide.dart' as rust;
import '../models/knowledge_card.dart';
import '../models/panel_item.dart';
import '../providers/ide_provider.dart';
import '../providers/theme_provider.dart';
import '../widgets/editor_panel.dart';
import '../widgets/linked_list_visualizer.dart';
import '../widgets/intro_overlay.dart';
import '../widgets/memory_map_visualizer.dart';

class IdeScreen extends ConsumerStatefulWidget {
  const IdeScreen({super.key});

  @override
  ConsumerState<IdeScreen> createState() => _IdeScreenState();
}

class _IdeScreenState extends ConsumerState<IdeScreen> {
  final _editorKey = GlobalKey<EditorPanelState>();
  final _inputController = TextEditingController();

  @override
  void dispose() {
    _inputController.dispose();
    super.dispose();
  }

  void _insertText(String text) => _editorKey.currentState?.insertText(text);
  void _insertPair(String open, String close) => _editorKey.currentState?.insertPair(open, close);
  void _undo() => _editorKey.currentState?.undo();
  void _redo() => _editorKey.currentState?.redo();
  void _moveCursor(int offset) => _editorKey.currentState?.moveCursor(offset);
  void _scrollToLine(int line) => _editorKey.currentState?.scrollToLine(line);

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(ideProvider);
    final notifier = ref.read(ideProvider.notifier);
    final isDark = ref.watch(themeProvider) == ThemeMode.dark;

    final scaffoldBg = isDark ? const Color(0xff121212) : const Color(0xfff5f5f5);

    return Scaffold(
      backgroundColor: scaffoldBg,
      body: SafeArea(
        child: Stack(
          children: [
            Column(
              children: [
                _buildToolbar(state, notifier, isDark),
                Expanded(
                  child: Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 8),
                    child: EditorPanel(key: _editorKey),
                  ),
                ),
                _buildSymbolBar(),
                _buildTemplateBar(state, notifier),
                _buildBottomPanel(state, notifier, isDark),
              ],
            ),
            if (state.showIntro)
              IntroOverlay(
                isDark: isDark,
                onDone: notifier.hideIntro,
              ),
          ],
        ),
      ),
      floatingActionButton: _buildFloatingButton(state, notifier),
      bottomSheet: state.isFloatingOpen ? _buildFloatingDrawer(state, notifier, isDark) : null,
    );
  }

  // ========== 工具栏 ==========

  Widget _buildToolbar(IdeState state, IdeNotifier notifier, bool isDark) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
      decoration: BoxDecoration(
        color: isDark ? const Color(0xff1e1e1e) : const Color(0xfff5f5f5),
        border: Border(
          bottom: BorderSide(color: Theme.of(context).dividerColor.withValues(alpha: 0.2)),
        ),
      ),
      child: Row(
        children: [
          _ToolButton(
            icon: Icons.play_arrow,
            color: Colors.green,
            onPressed: state.isRunning && !state.isStepMode ? null : notifier.run,
          ),
          if (state.isRunning)
            _ToolButton(
              icon: Icons.stop,
              color: Colors.red,
              onPressed: notifier.reset,
            ),
          _ToolButton(
            icon: Icons.skip_next,
            color: Colors.orange,
            onPressed: notifier.step,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              state.isCompiling
                  ? '编译中...'
                  : state.isRunning
                      ? state.isStepMode
                          ? '调试中 (第 ${state.currentLine} 行)'
                          : '运行中'
                      : '等待执行',
              style: TextStyle(fontSize: 13, color: Theme.of(context).textTheme.bodyMedium?.color),
              overflow: TextOverflow.ellipsis,
            ),
          ),
          // 执行速度滑块
          if (state.isStepMode)
            Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                const Icon(Icons.speed, size: 14, color: Colors.grey),
                const SizedBox(width: 4),
                SizedBox(
                  width: 80,
                  child: SliderTheme(
                    data: SliderTheme.of(context).copyWith(
                      trackHeight: 2,
                      thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 6),
                      overlayShape: SliderComponentShape.noOverlay,
                    ),
                    child: Slider(
                      value: state.executionSpeed.toDouble(),
                      min: 0,
                      max: 500,
                      divisions: 10,
                      label: '${state.executionSpeed}ms',
                      onChanged: (v) => notifier.setExecutionSpeed(v.toInt()),
                    ),
                  ),
                ),
              ],
            ),
          _ToolButton(
            icon: isDark ? Icons.light_mode : Icons.dark_mode,
            onPressed: () => ref.read(themeProvider.notifier).toggle(),
          ),
          _ToolButton(
            icon: Icons.help_outline,
            onPressed: notifier.showIntro,
          ),
          if (state.output.isNotEmpty)
            _ToolButton(
              icon: Icons.delete_outline,
              onPressed: notifier.clearOutput,
            ),
        ],
      ),
    );
  }

  // ========== 符号快捷栏 ==========

  Widget _buildSymbolBar() {
    final symbols = [
      ('{ }', () => _insertPair('{', '}')),
      ('( )', () => _insertPair('(', ')')),
      ('[ ]', () => _insertPair('[', ']')),
      ('" "', () => _insertPair('"', '"')),
      ("' '", () => _insertPair("'", "'")),
      (';', () => _insertText(';')),
      ('#', () => _insertText('#')),
      ('->', () => _insertText('->')),
      ('&', () => _insertText('&')),
      ('*', () => _insertText('*')),
      ('=', () => _insertText('=')),
      ('==', () => _insertText('==')),
      ('!=', () => _insertText('!=')),
      ('<', () => _insertText('<')),
      ('>', () => _insertText('>')),
      ('+', () => _insertText('+')),
      ('-', () => _insertText('-')),
      ('/', () => _insertText('/')),
      ('%', () => _insertText('%')),
      ('&&', () => _insertText('&&')),
      ('||', () => _insertText('||')),
      ('!', () => _insertText('!')),
      ('|', () => _insertText('|')),
      ('^', () => _insertText('^')),
      ('~', () => _insertText('~')),
      (',', () => _insertText(',')),
      ('.', () => _insertText('.')),
    ];

    final actions = [
      ('←', () => _moveCursor(-1)),
      ('→', () => _moveCursor(1)),
      ('Tab', () => _insertText('    ')),
      ('↩', _undo),
      ('↪', _redo),
    ];

    return Container(
      height: 36,
      decoration: BoxDecoration(
        border: Border(
          top: BorderSide(color: Theme.of(context).dividerColor.withValues(alpha: 0.2)),
        ),
      ),
      child: ListView(
        scrollDirection: Axis.horizontal,
        padding: const EdgeInsets.symmetric(horizontal: 4),
        children: [
          ...symbols.map((s) => _SymbolChip(label: s.$1, onTap: s.$2)),
          Container(
            margin: const EdgeInsets.symmetric(horizontal: 4, vertical: 6),
            width: 1,
            color: Theme.of(context).dividerColor,
          ),
          ...actions.map((a) => _SymbolChip(label: a.$1, onTap: a.$2, isAction: true)),
        ],
      ),
    );
  }

  // ========== 模板快捷栏 ==========

  Widget _buildTemplateBar(IdeState state, IdeNotifier notifier) {
    return Container(
      height: 36,
      decoration: BoxDecoration(
        border: Border(
          top: BorderSide(color: Theme.of(context).dividerColor.withValues(alpha: 0.2)),
        ),
      ),
      child: ListView(
        scrollDirection: Axis.horizontal,
        padding: const EdgeInsets.symmetric(horizontal: 4),
        children: CodeTemplate.defaults.map((tmpl) {
          return _TemplateChip(
            label: tmpl.displayName,
            onTap: () => _insertText(tmpl.code),
          );
        }).toList(),
      ),
    );
  }

  // ========== 底部面板 ==========

  Widget _buildBottomPanel(IdeState state, IdeNotifier notifier, bool isDark) {
    final panelBg = isDark ? const Color(0xff1e1e1e) : const Color(0xffffffff);

    return _HeightResizablePanel(
      height: state.bottomHeight,
      onHeightChanged: notifier.setBottomHeight,
      child: Container(
        decoration: BoxDecoration(
          color: panelBg,
          border: Border(
            top: BorderSide(color: Theme.of(context).dividerColor.withValues(alpha: 0.2)),
          ),
        ),
        child: Column(
          children: [
            // Tab 栏（可拖拽交换）
            Container(
              height: 36,
              padding: const EdgeInsets.symmetric(horizontal: 8),
              child: Row(
                children: [
                  ...List.generate(state.bottomSlots.length, (index) {
                    final panelId = state.bottomSlots[index];
                    final item = PanelItem.fromId(panelId);
                    if (item == null) return const SizedBox.shrink();
                    final isActive = state.bottomActiveIndex == index;
                    return Expanded(
                      child: _DraggablePanelTab(
                        item: item,
                        isActive: isActive,
                        badge: _getBadgeForPanel(panelId, state),
                        onTap: () => notifier.selectBottomTab(index),
                        onDoubleTap: () => notifier.removeBottomPanel(index),
                        data: _PanelDragData(panelId: panelId, fromLocation: PanelLocation.bottom, fromIndex: index),
                        onAccept: (dragData) {
                          if (dragData.fromLocation == PanelLocation.bottom) {
                            notifier.swapBottomPanels(index, dragData.fromIndex);
                          } else {
                            notifier.moveToBottom(dragData.panelId);
                          }
                        },
                      ),
                    );
                  }),
                  // 底部空位 DropTarget（拖拽到底部区域上方添加）
                  Expanded(
                    flex: 1,
                    child: DragTarget<_PanelDragData>(
                      onAcceptWithDetails: (details) {
                        notifier.moveToBottom(details.data.panelId);
                      },
                      builder: (context, candidateData, rejectedData) {
                        final isHovering = candidateData.isNotEmpty;
                        return Container(
                          margin: const EdgeInsets.symmetric(horizontal: 2, vertical: 4),
                          decoration: BoxDecoration(
                            color: isHovering ? Colors.blueAccent.withValues(alpha: 0.2) : null,
                            borderRadius: BorderRadius.circular(4),
                            border: isHovering ? Border.all(color: Colors.blueAccent) : null,
                          ),
                          child: const Center(
                            child: Icon(Icons.add, size: 16, color: Colors.grey),
                          ),
                        );
                      },
                    ),
                  ),
                ],
              ),
            ),
            // 内容区域（支持水平滑动切换标签）
            Expanded(
              child: GestureDetector(
                onHorizontalDragEnd: (details) {
                  const threshold = 300.0;
                  final dx = details.velocity.pixelsPerSecond.dx;
                  if (dx > threshold && state.bottomActiveIndex > 0) {
                    notifier.selectBottomTab(state.bottomActiveIndex - 1);
                  } else if (dx < -threshold &&
                      state.bottomActiveIndex < state.bottomSlots.length - 1) {
                    notifier.selectBottomTab(state.bottomActiveIndex + 1);
                  }
                },
                child: _buildBottomTabContent(state, notifier, isDark),
              ),
            ),
          ],
        ),
      ),
    );
  }

  String? _getBadgeForPanel(String panelId, IdeState state) {
    switch (panelId) {
      case 'diagnostics':
        return state.diagnostics.isNotEmpty ? '${state.diagnostics.length}' : null;
      case 'algorithm':
        return state.algorithmMatches.isNotEmpty ? '${state.algorithmMatches.length}' : null;
      default:
        return null;
    }
  }

  Widget _buildBottomTabContent(IdeState state, IdeNotifier notifier, bool isDark) {
    if (state.bottomSlots.isEmpty) return const SizedBox.shrink();
    final panelId = state.bottomSlots[state.bottomActiveIndex.clamp(0, state.bottomSlots.length - 1)];
    switch (panelId) {
      case 'output':
        return _buildOutputTab(state, notifier, isDark);
      case 'diagnostics':
        return _buildDiagnosticsTab(state, notifier, isDark);
      case 'algorithm':
        return _buildAlgorithmTab(state, isDark);
      case 'knowledge':
        return _buildKnowledgeCardTab(state, isDark);
      case 'pointer':
        return _buildPointerVisTab(state, isDark);
      case 'arrayVis':
        return _buildArrayVisTab(state, isDark);
      case 'memory':
        return _buildMemoryTab(state, isDark);
      case 'variables':
        return _buildVariablesTab(state, isDark);
      case 'watch':
        return _buildWatchTab(state, isDark);
      case 'callstack':
        return _buildCallstackTab(state, isDark);
      case 'progress':
        return _buildProgressTab(state, isDark);
      default:
        return const SizedBox.shrink();
    }
  }

  // ========== 悬浮球 ==========

  Widget _buildFloatingButton(IdeState state, IdeNotifier notifier) {
    return FloatingActionButton(
      mini: true,
      backgroundColor: state.isFloatingOpen ? Colors.redAccent : Colors.blueAccent,
      onPressed: notifier.toggleFloating,
      child: Icon(state.isFloatingOpen ? Icons.close : Icons.bug_report, size: 20),
    );
  }

  Widget _buildFloatingDrawer(IdeState state, IdeNotifier notifier, bool isDark) {
    final panelBg = isDark ? const Color(0xff1e1e1e) : const Color(0xffffffff);

    return Container(
      height: 320,
      decoration: BoxDecoration(
        color: panelBg,
        borderRadius: const BorderRadius.vertical(top: Radius.circular(12)),
        boxShadow: [BoxShadow(color: Colors.black.withValues(alpha: 0.2), blurRadius: 8)],
      ),
      child: Column(
        children: [
          // 拖拽手柄 + 关闭
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            child: Row(
              children: [
                Container(width: 40, height: 4, decoration: BoxDecoration(color: Colors.grey, borderRadius: BorderRadius.circular(2))),
                const Spacer(),
                Text('调试面板', style: TextStyle(fontSize: 12, color: Colors.grey[600])),
                const Spacer(),
                InkWell(
                  onTap: notifier.closeFloating,
                  child: const Icon(Icons.close, size: 18, color: Colors.grey),
                ),
              ],
            ),
          ),
          // Tab 栏
          Container(
            height: 40,
            padding: const EdgeInsets.symmetric(horizontal: 8),
            child: Row(
              children: [
                ...List.generate(state.floatingSlots.length, (index) {
                  final panelId = state.floatingSlots[index];
                  final item = PanelItem.fromId(panelId);
                  if (item == null) return const SizedBox.shrink();
                  final isActive = state.floatingActiveIndex == index;
                  return Expanded(
                    child: _DraggablePanelTab(
                      item: item,
                      isActive: isActive,
                      onTap: () => notifier.selectFloatingTab(index),
                      onDoubleTap: () => notifier.removeFloatingPanel(index),
                      data: _PanelDragData(panelId: panelId, fromLocation: PanelLocation.floating, fromIndex: index),
                      onAccept: (dragData) {
                        if (dragData.fromLocation == PanelLocation.floating) {
                          notifier.swapFloatingPanels(index, dragData.fromIndex);
                        } else {
                          notifier.moveToFloating(dragData.panelId);
                        }
                      },
                    ),
                  );
                }),
                // 悬浮球空位 DropTarget
                if (state.floatingSlots.length < 7)
                  Expanded(
                    child: DragTarget<_PanelDragData>(
                      onAcceptWithDetails: (details) {
                        notifier.moveToFloating(details.data.panelId);
                      },
                      builder: (context, candidateData, rejectedData) {
                        final isHovering = candidateData.isNotEmpty;
                        return Container(
                          margin: const EdgeInsets.symmetric(horizontal: 2, vertical: 4),
                          decoration: BoxDecoration(
                            color: isHovering ? Colors.blueAccent.withValues(alpha: 0.2) : null,
                            borderRadius: BorderRadius.circular(4),
                            border: isHovering ? Border.all(color: Colors.blueAccent) : null,
                          ),
                          child: const Center(
                            child: Icon(Icons.add, size: 16, color: Colors.grey),
                          ),
                        );
                      },
                    ),
                  ),
              ],
            ),
          ),
          // 内容区域（支持水平滑动切换标签）
          Expanded(
            child: GestureDetector(
              onHorizontalDragEnd: (details) {
                const threshold = 300.0;
                final dx = details.velocity.pixelsPerSecond.dx;
                if (dx > threshold && state.floatingActiveIndex > 0) {
                  notifier.selectFloatingTab(state.floatingActiveIndex - 1);
                } else if (dx < -threshold &&
                    state.floatingActiveIndex < state.floatingSlots.length - 1) {
                  notifier.selectFloatingTab(state.floatingActiveIndex + 1);
                }
              },
              child: _buildFloatingTabContent(state, notifier, isDark),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildFloatingTabContent(IdeState state, IdeNotifier notifier, bool isDark) {
    if (state.floatingSlots.isEmpty) return const SizedBox.shrink();
    final panelId = state.floatingSlots[state.floatingActiveIndex.clamp(0, state.floatingSlots.length - 1)];
    switch (panelId) {
      case 'output':
        return _buildOutputTab(state, notifier, isDark);
      case 'diagnostics':
        return _buildDiagnosticsTab(state, notifier, isDark);
      case 'algorithm':
        return _buildAlgorithmTab(state, isDark);
      case 'knowledge':
        return _buildKnowledgeCardTab(state, isDark);
      case 'pointer':
        return _buildPointerVisTab(state, isDark);
      case 'arrayVis':
        return _buildArrayVisTab(state, isDark);
      case 'memory':
        return _buildMemoryTab(state, isDark);
      case 'variables':
        return _buildVariablesTab(state, isDark);
      case 'watch':
        return _buildWatchTab(state, isDark);
      case 'callstack':
        return _buildCallstackTab(state, isDark);
      case 'progress':
        return _buildProgressTab(state, isDark);
      default:
        return const SizedBox.shrink();
    }
  }

  // ========== 各 Tab 内容 ==========

  Widget _buildOutputTab(IdeState state, IdeNotifier notifier, bool isDark) {
    return Column(
      children: [
        Expanded(
          child: Stack(
            children: [
              SingleChildScrollView(
                padding: const EdgeInsets.all(12),
                child: SelectableText(
                  state.output.isEmpty ? '等待执行...' : state.output,
                  style: TextStyle(
                    fontFamily: 'Consolas',
                    fontFamilyFallback: const ['monospace'],
                    fontSize: 13,
                    color: isDark ? const Color(0xffd4d4d4) : const Color(0xff333333),
                  ),
                ),
              ),
              Positioned(
                top: 4,
                right: 4,
                child: IconButton(
                  icon: const Icon(Icons.copy, size: 16),
                  tooltip: '复制输出',
                  onPressed: state.output.isEmpty
                      ? null
                      : () {
                          Clipboard.setData(ClipboardData(text: state.output));
                          ScaffoldMessenger.of(context).showSnackBar(
                            const SnackBar(content: Text('已复制到剪贴板'), duration: Duration(seconds: 1)),
                          );
                        },
                ),
              ),
            ],
          ),
        ),
        if (state.waitingInput)
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
            decoration: BoxDecoration(
              border: Border(
                top: BorderSide(color: Theme.of(context).dividerColor.withValues(alpha: 0.2)),
              ),
            ),
            child: Row(
              children: [
                const Text('➜', style: TextStyle(color: Colors.green)),
                const SizedBox(width: 8),
                Expanded(
                  child: TextField(
                    controller: _inputController,
                    autofocus: true,
                    style: TextStyle(
                      color: isDark ? Colors.white : Colors.black,
                      fontFamily: 'monospace',
                    ),
                    decoration: const InputDecoration(
                      isDense: true,
                      border: InputBorder.none,
                      hintText: '输入数据',
                    ),
                    onSubmitted: (value) {
                      if (value.isNotEmpty) {
                        notifier.provideInput(value);
                        _inputController.clear();
                      }
                    },
                  ),
                ),
                TextButton(
                  onPressed: () {
                    final value = _inputController.text;
                    if (value.isNotEmpty) {
                      notifier.provideInput(value);
                      _inputController.clear();
                    }
                  },
                  child: const Text('发送'),
                ),
              ],
            ),
          ),
      ],
    );
  }

  Widget _buildDiagnosticsTab(IdeState state, IdeNotifier notifier, bool isDark) {
    if (state.diagnostics.isEmpty) {
      return const Center(child: Text('无诊断信息', style: TextStyle(color: Colors.grey)));
    }
    return ListView.builder(
      itemCount: state.diagnostics.length,
      itemBuilder: (context, index) {
        final diag = state.diagnostics[index];
        final isError = diag.severity == 0;
        return InkWell(
          onTap: () {
            notifier.highlightLine(diag.line);
            _scrollToLine(diag.line);
          },
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: BoxDecoration(
              border: Border(
                bottom: BorderSide(color: Theme.of(context).dividerColor.withValues(alpha: 0.1)),
              ),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                      decoration: BoxDecoration(
                        color: isError ? Colors.redAccent.withValues(alpha: 0.2) : Colors.orangeAccent.withValues(alpha: 0.2),
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Text(
                        isError ? '错误' : '警告',
                        style: TextStyle(
                          fontSize: 11,
                          color: isError ? Colors.redAccent : Colors.orangeAccent,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                    ),
                    const SizedBox(width: 8),
                    Text('第 ${diag.line} 行', style: const TextStyle(fontSize: 12, color: Colors.grey)),
                    if (diag.errorCode > 0)
                      Text(' [${diag.errorCode}]', style: const TextStyle(fontSize: 11, color: Colors.grey)),
                  ],
                ),
                const SizedBox(height: 4),
                Text(
                  diag.message,
                  style: TextStyle(fontSize: 13, color: isDark ? const Color(0xffd4d4d4) : const Color(0xff333333)),
                ),
                if (diag.fixSuggestion.isNotEmpty) ...[
                  const SizedBox(height: 4),
                  Row(
                    children: [
                      const Text('💡 ', style: TextStyle(fontSize: 12)),
                      Expanded(
                        child: Text(
                          diag.fixSuggestion,
                          style: TextStyle(fontSize: 12, color: Colors.grey[400]),
                        ),
                      ),
                    ],
                  ),
                ],
                // 应用修复按钮
                if (diag.fixKind == 1 || diag.fixKind == 2 || diag.fixSuggestion.isNotEmpty)
                  Padding(
                    padding: const EdgeInsets.only(top: 6),
                    child: Align(
                      alignment: Alignment.centerLeft,
                      child: TextButton.icon(
                        onPressed: () async {
                          final msg = await notifier.applyFix(diag);
                          if (!context.mounted) return;
                          if (msg != null) {
                            _editorKey.currentState?.setText(state.source);
                            ScaffoldMessenger.of(context).showSnackBar(
                              SnackBar(content: Text(msg), duration: const Duration(seconds: 2)),
                            );
                            // 修复后重新编译
                            await notifier.compile();
                          } else {
                            if (!context.mounted) return;
                            ScaffoldMessenger.of(context).showSnackBar(
                              SnackBar(
                                content: Text('💡 修复提示（第${diag.line}行）：${diag.fixSuggestion}\n请手动修改代码。'),
                                duration: const Duration(seconds: 3),
                              ),
                            );
                          }
                        },
                        icon: const Icon(Icons.auto_fix_high, size: 14),
                        label: const Text('应用修复', style: TextStyle(fontSize: 12)),
                        style: TextButton.styleFrom(
                          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                          minimumSize: Size.zero,
                          tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                        ),
                      ),
                    ),
                  ),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildAlgorithmTab(IdeState state, bool isDark) {
    if (state.algorithmMatches.isEmpty) {
      return const Center(child: Text('未检测到算法模式', style: TextStyle(color: Colors.grey)));
    }
    return StatefulBuilder(
      builder: (context, setState) {
        final validationResults = <int, AlgorithmValidationResult>{};
        final validating = <int, bool>{};
        final expandedVis = <int, bool>{};
        return ListView.builder(
          itemCount: state.algorithmMatches.length,
          itemBuilder: (context, index) {
            final match = state.algorithmMatches[index];
            final result = validationResults[index];
            final isValidating = validating[index] ?? false;
            final isVisExpanded = expandedVis[index] ?? false;
            return Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              decoration: BoxDecoration(
                border: Border(
                  bottom: BorderSide(color: Theme.of(context).dividerColor.withValues(alpha: 0.1)),
                ),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      Expanded(
                        child: Text(
                          match.displayName.isEmpty ? match.name : match.displayName,
                          style: const TextStyle(fontSize: 14, fontWeight: FontWeight.bold),
                        ),
                      ),
                      Text('置信度 ${match.confidence}%', style: const TextStyle(fontSize: 12, color: Colors.grey)),
                    ],
                  ),
                  if (match.suggestion.isNotEmpty)
                    Padding(
                      padding: const EdgeInsets.only(top: 4),
                      child: Text(match.suggestion, style: TextStyle(fontSize: 12, color: Colors.grey[400])),
                    ),
                  const SizedBox(height: 8),
                  Row(
                    children: [
                      TextButton.icon(
                        onPressed: isValidating
                            ? null
                            : () async {
                                setState(() => validating[index] = true);
                                final notifier = ref.read(ideProvider.notifier);
                                final res = await notifier.validateAlgorithm(match);
                                setState(() {
                                  validating[index] = false;
                                  validationResults[index] = res;
                                });
                              },
                        icon: isValidating
                            ? const SizedBox(width: 14, height: 14, child: CircularProgressIndicator(strokeWidth: 2))
                            : const Icon(Icons.search, size: 14),
                        label: const Text('验证算法', style: TextStyle(fontSize: 12)),
                        style: TextButton.styleFrom(
                          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                          minimumSize: Size.zero,
                          tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                        ),
                      ),
                      if (match.visEvents.isNotEmpty)
                        TextButton.icon(
                          onPressed: () => setState(() => expandedVis[index] = !isVisExpanded),
                          icon: Icon(
                            isVisExpanded ? Icons.visibility_off : Icons.visibility,
                            size: 14,
                          ),
                          label: Text(
                            isVisExpanded ? '收起事件' : '可视化事件 (${match.visEvents.length})',
                            style: const TextStyle(fontSize: 12),
                          ),
                          style: TextButton.styleFrom(
                            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                            minimumSize: Size.zero,
                            tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                          ),
                        ),
                    ],
                  ),
                  if (result != null)
                    Container(
                      margin: const EdgeInsets.only(top: 4),
                      padding: const EdgeInsets.all(8),
                      decoration: BoxDecoration(
                        color: result.passed ? Colors.green.withValues(alpha: 0.1) : Colors.red.withValues(alpha: 0.1),
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Row(
                        children: [
                          Icon(
                            result.passed ? Icons.check_circle : Icons.error,
                            size: 16,
                            color: result.passed ? Colors.green : Colors.red,
                          ),
                          const SizedBox(width: 6),
                          Expanded(
                            child: Text(
                              result.message,
                              style: TextStyle(
                                fontSize: 12,
                                color: result.passed ? Colors.green[300] : Colors.red[300],
                              ),
                            ),
                          ),
                        ],
                      ),
                    ),
                  if (isVisExpanded && match.visEvents.isNotEmpty)
                    Container(
                      margin: const EdgeInsets.only(top: 6),
                      padding: const EdgeInsets.all(8),
                      decoration: BoxDecoration(
                        color: isDark ? const Color(0xFF2A2A2C) : const Color(0xFFF5F5F7),
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          const Text(
                            '关键比较事件',
                            style: TextStyle(fontSize: 11, fontWeight: FontWeight.bold, color: Colors.grey),
                          ),
                          const SizedBox(height: 4),
                          Wrap(
                            spacing: 6,
                            runSpacing: 4,
                            children: match.visEvents.asMap().entries.map((entry) {
                              final i = entry.key;
                              final ev = entry.value;
                              return Tooltip(
                                message: '第 ${ev.line} 行: ${ev.context}',
                                child: Container(
                                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                                  decoration: BoxDecoration(
                                    color: Colors.blueAccent.withValues(alpha: 0.15),
                                    borderRadius: BorderRadius.circular(4),
                                    border: Border.all(color: Colors.blueAccent.withValues(alpha: 0.3)),
                                  ),
                                  child: Text(
                                    '${i + 1}. ${ev.context}',
                                    style: const TextStyle(fontSize: 11, color: Colors.blueAccent),
                                  ),
                                ),
                              );
                            }).toList(),
                          ),
                        ],
                      ),
                    ),
                ],
              ),
            );
          },
        );
      },
    );
  }

  Widget _buildKnowledgeCardTab(IdeState state, bool isDark) {
    final cards = state.knowledgeCards;
    if (cards.isEmpty) {
      return const Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.menu_book, size: 48, color: Colors.grey),
            SizedBox(height: 12),
            Text('暂无相关知识卡片', style: TextStyle(color: Colors.grey)),
            SizedBox(height: 4),
            Text('编译出错后将自动匹配对应的知识卡片', style: TextStyle(fontSize: 12, color: Colors.grey)),
          ],
        ),
      );
    }
    return ListView.builder(
      padding: const EdgeInsets.all(12),
      itemCount: cards.length,
      itemBuilder: (context, index) {
        final card = cards[index];
        Future.microtask(() => ref.read(ideProvider.notifier).recordKnowledgeCardView(card.id));
        return _KnowledgeCardItem(card: card, isDark: isDark);
      },
    );
  }

  Widget _buildPointerVisTab(IdeState state, bool isDark) {
    return FutureBuilder<List<rust.VariableSnapshot>>(
      future: rust.getVariables(),
      builder: (context, snapshot) {
        final vars = snapshot.data ?? [];
        const nullTrapEnd = 64;
        const linearMemorySize = 256 * 1024;
        final pointers = vars.where((v) {
          final val = v.value;
          return v.tyName.contains('*') &&
              val > nullTrapEnd &&
              val < linearMemorySize;
        }).toList();

        // 查找链表头节点（struct Node* 类型）
        final headVars = pointers.where((v) {
          return v.tyName.toLowerCase().contains('struct') &&
              v.tyName.toLowerCase().contains('node');
        }).toList();

        if (pointers.isEmpty && headVars.isEmpty) {
          return const Center(child: Text('未检测到指针变量', style: TextStyle(color: Colors.grey)));
        }

        return Column(
          children: [
            // 链表可视化区域
            if (headVars.isNotEmpty)
              FutureBuilder<List<rust.VisEvent>>(
                future: rust.getVisEvents(),
                builder: (context, visSnapshot) {
                  final visEvents = visSnapshot.data ?? [];
                  return SizedBox(
                    height: 120,
                    child: ListView.builder(
                      scrollDirection: Axis.horizontal,
                      padding: const EdgeInsets.symmetric(horizontal: 12),
                      itemCount: headVars.length,
                      itemBuilder: (context, idx) {
                        final hv = headVars[idx];
                        return Container(
                          margin: const EdgeInsets.only(right: 16),
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Padding(
                                padding: const EdgeInsets.symmetric(vertical: 4),
                                child: Text(
                                  '${hv.name} (${hv.tyName})',
                                  style: const TextStyle(fontSize: 11, color: Colors.grey, fontFamily: 'monospace'),
                                ),
                              ),
                              Expanded(
                                child: LinkedListVisualizer(
                                  headAddr: hv.value,
                                  structName: 'Node',
                                  visEvents: visEvents,
                                  isDark: isDark,
                                ),
                              ),
                            ],
                          ),
                        );
                      },
                    ),
                  );
                },
              ),
            // 指针列表
            Expanded(
              child: ListView.builder(
                padding: const EdgeInsets.all(12),
                itemCount: pointers.length,
                itemBuilder: (context, index) {
                  final p = pointers[index];
                  String targetName = '';
                  final targetAddr = p.value;
                  for (final v in vars) {
                    if (v.addr == targetAddr) {
                      targetName = v.name;
                      break;
                    }
                  }
                  return Container(
                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                    decoration: BoxDecoration(
                      border: Border(
                        bottom: BorderSide(color: Theme.of(context).dividerColor.withValues(alpha: 0.1)),
                      ),
                    ),
                    child: Row(
                      children: [
                        const Icon(Icons.arrow_forward, size: 16, color: Colors.blueAccent),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(
                                p.name,
                                style: TextStyle(
                                  fontSize: 13,
                                  fontFamily: 'monospace',
                                  color: isDark ? const Color(0xffd4d4d4) : const Color(0xff333333),
                                ),
                              ),
                              const SizedBox(height: 2),
                              Text(
                                '0x${p.addr.toRadixString(16).toUpperCase().padLeft(4, '0')} → 0x${targetAddr.toRadixString(16).toUpperCase().padLeft(4, '0')} ${targetName.isNotEmpty ? '($targetName)' : ''}',
                                style: const TextStyle(fontSize: 11, color: Colors.grey, fontFamily: 'monospace'),
                              ),
                            ],
                          ),
                        ),
                        Container(
                          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                          decoration: BoxDecoration(
                            color: Colors.blueAccent.withValues(alpha: 0.1),
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Text(p.tyName, style: const TextStyle(fontSize: 10, color: Colors.blueAccent, fontFamily: 'monospace')),
                        ),
                      ],
                    ),
                  );
                },
              ),
            ),
          ],
        );
      },
    );
  }

  Widget _buildArrayVisTab(IdeState state, bool isDark) {
    return FutureBuilder<List<rust.VariableSnapshot>>(
      future: rust.getVariables(),
      builder: (context, varSnapshot) {
        final vars = varSnapshot.data ?? [];
        // 筛选可能是数组的变量（类型名包含 [ 或名字包含 arr）
        final arrayVars = vars.where((v) {
          return v.tyName.contains('[') || v.name.toLowerCase().contains('arr');
        }).toList();

        if (arrayVars.isEmpty) {
          return const Center(child: Text('未检测到数组变量', style: TextStyle(color: Colors.grey)));
        }

        return ListView.builder(
          padding: const EdgeInsets.all(12),
          itemCount: arrayVars.length,
          itemBuilder: (context, index) {
            final v = arrayVars[index];
            return _ArrayVisualizer(
              name: v.name,
              addr: v.addr,
              tyName: v.tyName,
              isDark: isDark,
            );
          },
        );
      },
    );
  }

  Widget _buildWatchTab(IdeState state, bool isDark) {
    final controller = TextEditingController();
    return Column(
      children: [
        // 输入栏
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          decoration: BoxDecoration(
            border: Border(bottom: BorderSide(color: Theme.of(context).dividerColor.withValues(alpha: 0.2))),
          ),
          child: Row(
            children: [
              Expanded(
                child: TextField(
                  controller: controller,
                  style: TextStyle(fontSize: 13, color: isDark ? Colors.white : Colors.black),
                  decoration: const InputDecoration(
                    isDense: true,
                    border: InputBorder.none,
                    hintText: '输入变量名（如 a、arr[0]）',
                    hintStyle: TextStyle(fontSize: 13),
                  ),
                  onSubmitted: (value) {
                    if (value.trim().isNotEmpty) {
                      ref.read(ideProvider.notifier).addWatchExpression(value.trim());
                      controller.clear();
                    }
                  },
                ),
              ),
              TextButton(
                onPressed: () {
                  final value = controller.text.trim();
                  if (value.isNotEmpty) {
                    ref.read(ideProvider.notifier).addWatchExpression(value);
                    controller.clear();
                  }
                },
                child: const Text('添加'),
              ),
            ],
          ),
        ),
        // 表达式列表
        Expanded(
          child: state.watchExpressions.isEmpty
              ? const Center(child: Text('暂无监视表达式', style: TextStyle(color: Colors.grey)))
              : FutureBuilder<List<rust.VariableSnapshot>>(
                  future: rust.getVariables(),
                  builder: (context, snapshot) {
                    final vars = snapshot.data ?? [];
                    return ListView.builder(
                      itemCount: state.watchExpressions.length,
                      itemBuilder: (context, index) {
                        final expr = state.watchExpressions[index];
                        final result = _evalWatchExpression(expr, vars);
                        return Container(
                          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                          decoration: BoxDecoration(
                            border: Border(bottom: BorderSide(color: Theme.of(context).dividerColor.withValues(alpha: 0.1))),
                          ),
                          child: Row(
                            children: [
                              Expanded(
                                child: Column(
                                  crossAxisAlignment: CrossAxisAlignment.start,
                                  children: [
                                    Text(
                                      expr,
                                      style: TextStyle(
                                        fontSize: 13,
                                        fontFamily: 'monospace',
                                        color: isDark ? const Color(0xffd4d4d4) : const Color(0xff333333),
                                      ),
                                    ),
                                    const SizedBox(height: 2),
                                    Text(
                                      result,
                                      style: TextStyle(
                                        fontSize: 12,
                                        color: result.startsWith('值:') ? Colors.green : Colors.orange,
                                        fontFamily: 'monospace',
                                      ),
                                    ),
                                  ],
                                ),
                              ),
                              IconButton(
                                icon: const Icon(Icons.close, size: 16, color: Colors.grey),
                                onPressed: () => ref.read(ideProvider.notifier).removeWatchExpression(expr),
                              ),
                            ],
                          ),
                        );
                      },
                    );
                  },
                ),
        ),
      ],
    );
  }

  String _evalWatchExpression(String expr, List<rust.VariableSnapshot> vars) {
    // 简单数组索引：arr[0]
    final arrMatch = RegExp(r'^(\w+)\[(\d+)\]$').firstMatch(expr);
    if (arrMatch != null) {
      final name = arrMatch.group(1)!;
      final idx = int.tryParse(arrMatch.group(2)!) ?? 0;
      final v = vars.where((x) => x.name == name).firstOrNull;
      if (v != null) {
        // 异步读取内存，这里返回提示
        return '数组 $name，地址 0x${v.addr.toRadixString(16)}，索引 $idx';
      }
      return '未找到变量: $name';
    }
    // 简单变量名匹配
    final v = vars.where((x) => x.name == expr).firstOrNull;
    if (v != null) {
      return '值: ${v.value}  (0x${v.addr.toRadixString(16).toUpperCase().padLeft(4, '0')})';
    }
    return '未找到变量: $expr';
  }

  Widget _buildMemoryTab(IdeState state, bool isDark) {
    return FutureBuilder<List<rust.MemoryRegion>>(
      future: rust.getMemoryRegions(),
      builder: (context, snapshot) {
        final regions = snapshot.data ?? [];
        if (regions.isEmpty) {
          return const Center(child: Text('无内存信息', style: TextStyle(color: Colors.grey)));
        }
        return MemoryMapVisualizer(regions: regions, isDark: isDark);
      },
    );
  }

  Widget _buildVariablesTab(IdeState state, bool isDark) {
    return FutureBuilder<List<rust.VariableSnapshot>>(
      future: rust.getVariables(),
      builder: (context, snapshot) {
        final vars = snapshot.data ?? [];
        if (vars.isEmpty) {
          return const Center(child: Text('无变量信息', style: TextStyle(color: Colors.grey)));
        }
        return ListView.builder(
          itemCount: vars.length,
          itemBuilder: (context, index) {
            final v = vars[index];
            return ListTile(
              dense: true,
              title: Row(
                children: [
                  Text(v.name, style: const TextStyle(fontFamily: 'monospace', fontSize: 13)),
                  const SizedBox(width: 8),
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                    decoration: BoxDecoration(
                      color: isDark ? const Color(0xff2a2a2a) : const Color(0xffe5e5e5),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(v.tyName, style: TextStyle(fontSize: 10, color: isDark ? Colors.grey : Colors.black54, fontFamily: 'monospace')),
                  ),
                ],
              ),
              subtitle: Text('值: ${v.value}  地址: 0x${v.addr.toRadixString(16).toUpperCase().padLeft(4, '0')}',
                  style: const TextStyle(fontSize: 11, color: Colors.grey, fontFamily: 'monospace')),
            );
          },
        );
      },
    );
  }

  Widget _buildCallstackTab(IdeState state, bool isDark) {
    return FutureBuilder<List<rust.TraceEntry>>(
      future: rust.getCallstack(),
      builder: (context, snapshot) {
        final entries = snapshot.data ?? [];
        if (entries.isEmpty) {
          return const Center(child: Text('调用栈为空', style: TextStyle(color: Colors.grey)));
        }
        return ListView.builder(
          itemCount: entries.length,
          itemBuilder: (context, index) {
            final e = entries[index];
            final isCurrent = index == 0;
            return ListTile(
              dense: true,
              title: Text(e.operation, style: const TextStyle(fontFamily: 'monospace', fontSize: 13)),
              subtitle: Text('返回行 ${e.line}', style: const TextStyle(fontSize: 11, color: Colors.grey)),
              trailing: isCurrent
                  ? Container(
                      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                      decoration: BoxDecoration(
                        color: Colors.blueAccent.withValues(alpha: 0.2),
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: const Text('当前', style: TextStyle(fontSize: 10, color: Colors.blueAccent)),
                    )
                  : null,
            );
          },
        );
      },
    );
  }

  Widget _buildProgressTab(IdeState state, bool isDark) {
    final progress = state.learningProgress;
    final totalCards = KnowledgeCard.all.length;
    final viewedCards = progress.viewedKnowledgeCards.length;
    final cardProgress = totalCards == 0 ? 0.0 : viewedCards / totalCards;

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 连续活跃天数
          _ProgressCard(
            title: '🔥 连续活跃',
            value: '${progress.streakDays} 天',
            subtitle: progress.lastActiveDate.isEmpty ? '开始你的学习之旅吧' : '最后活跃: ${progress.lastActiveDate}',
            icon: Icons.local_fire_department,
            color: Colors.orangeAccent,
          ),
          const SizedBox(height: 12),
          // 编译统计
          _ProgressCard(
            title: '📝 编译统计',
            value: '${progress.totalCompiles} 次',
            subtitle: '成功 ${progress.successfulCompiles} · 失败 ${progress.failedCompiles} · 成功率 ${(progress.successRate * 100).toStringAsFixed(1)}%',
            icon: Icons.code,
            color: Colors.blueAccent,
          ),
          const SizedBox(height: 12),
          // 错误修复
          _ProgressCard(
            title: '🛠️ 错误修复',
            value: '${progress.totalErrorsFixed} / ${progress.totalErrorsEncountered}',
            subtitle: '已修复 / 遇到',
            icon: Icons.build,
            color: Colors.green,
          ),
          const SizedBox(height: 12),
          // 知识卡片
          _ProgressCard(
            title: '📚 知识卡片',
            value: '$viewedCards / $totalCards',
            subtitle: '已阅读 / 总数',
            icon: Icons.menu_book,
            color: Colors.purpleAccent,
            progress: cardProgress,
          ),
          const SizedBox(height: 12),
          // 算法验证
          _ProgressCard(
            title: '🔍 算法验证',
            value: '${(progress.algorithmOverallPassRate * 100).toStringAsFixed(1)}%',
            subtitle: progress.algorithmValidationsTotal.isEmpty
                ? '暂无验证记录'
                : progress.algorithmValidationsTotal.entries.map((e) {
                    final passed = progress.algorithmValidationsPassed[e.key] ?? 0;
                    return '${e.key}: $passed/${e.value}';
                  }).join(' · '),
            icon: Icons.auto_fix_high,
            color: Colors.teal,
          ),
          const SizedBox(height: 24),
          // 重置按钮
          Center(
            child: TextButton.icon(
              onPressed: () async {
                final confirmed = await showDialog<bool>(
                  context: context,
                  builder: (ctx) => AlertDialog(
                    title: const Text('重置学习进度'),
                    content: const Text('确定要清除所有学习进度数据吗？此操作不可恢复。'),
                    actions: [
                      TextButton(onPressed: () => Navigator.pop(ctx, false), child: const Text('取消')),
                      TextButton(onPressed: () => Navigator.pop(ctx, true), child: const Text('确定')),
                    ],
                  ),
                );
                if (confirmed == true) {
                  await ref.read(ideProvider.notifier).resetProgress();
                }
              },
              icon: const Icon(Icons.restore, size: 16),
              label: const Text('重置进度', style: TextStyle(fontSize: 12)),
            ),
          ),
        ],
      ),
    );
  }
}

// ========== 可拖拽高度的面板包装器 ==========

class _HeightResizablePanel extends StatefulWidget {
  final double height;
  final ValueChanged<double> onHeightChanged;
  final Widget child;

  const _HeightResizablePanel({
    required this.height,
    required this.onHeightChanged,
    required this.child,
  });

  @override
  State<_HeightResizablePanel> createState() => _HeightResizablePanelState();
}

class _HeightResizablePanelState extends State<_HeightResizablePanel> {
  double? _dragStartHeight;
  double? _dragStartY;

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // 拖拽条
        GestureDetector(
          onVerticalDragStart: (details) {
            _dragStartHeight = widget.height;
            _dragStartY = details.globalPosition.dy;
          },
          onVerticalDragUpdate: (details) {
            if (_dragStartHeight == null || _dragStartY == null) return;
            final delta = _dragStartY! - details.globalPosition.dy;
            widget.onHeightChanged(_dragStartHeight! + delta);
          },
          onVerticalDragEnd: (_) {
            _dragStartHeight = null;
            _dragStartY = null;
          },
          child: Container(
            height: 8,
            color: Colors.transparent,
            child: Center(
              child: Container(
                width: 40,
                height: 4,
                decoration: BoxDecoration(
                  color: Colors.grey.withValues(alpha: 0.4),
                  borderRadius: BorderRadius.circular(2),
                ),
              ),
            ),
          ),
        ),
        // 内容
        SizedBox(height: widget.height, child: widget.child),
      ],
    );
  }
}

// ========== 可拖拽的面板 Tab ==========

class _PanelDragData {
  final String panelId;
  final PanelLocation fromLocation;
  final int fromIndex;

  const _PanelDragData({
    required this.panelId,
    required this.fromLocation,
    required this.fromIndex,
  });
}

class _DraggablePanelTab extends StatelessWidget {
  final PanelItem item;
  final bool isActive;
  final String? badge;
  final VoidCallback onTap;
  final VoidCallback onDoubleTap;
  final _PanelDragData data;
  final void Function(_PanelDragData) onAccept;

  const _DraggablePanelTab({
    required this.item,
    required this.isActive,
    this.badge,
    required this.onTap,
    required this.onDoubleTap,
    required this.data,
    required this.onAccept,
  });

  @override
  Widget build(BuildContext context) {
    final tab = InkWell(
      onTap: onTap,
      onDoubleTap: onDoubleTap,
      child: Container(
        margin: const EdgeInsets.symmetric(horizontal: 2, vertical: 4),
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        decoration: BoxDecoration(
          border: Border(
            bottom: BorderSide(
              color: isActive ? Colors.blueAccent : Colors.transparent,
              width: 2,
            ),
          ),
        ),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(item.icon, size: 14, color: isActive ? Colors.blueAccent : Colors.grey),
            const SizedBox(width: 4),
            Flexible(
              child: Text(
                item.label,
                style: TextStyle(
                  fontSize: 12,
                  color: isActive ? Colors.blueAccent : Colors.grey,
                  fontWeight: isActive ? FontWeight.bold : FontWeight.normal,
                ),
                overflow: TextOverflow.ellipsis,
              ),
            ),
            if (badge != null) ...[
              const SizedBox(width: 4),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                decoration: BoxDecoration(
                  color: isActive ? Colors.blueAccent : Colors.grey,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Text(badge!, style: const TextStyle(fontSize: 10, color: Colors.white)),
              ),
            ],
          ],
        ),
      ),
    );

    return DragTarget<_PanelDragData>(
      onAcceptWithDetails: (details) => onAccept(details.data),
      builder: (context, candidateData, rejectedData) {
        final isHovering = candidateData.isNotEmpty;
        return Draggable<_PanelDragData>(
          data: data,
          feedback: Material(
            color: Colors.transparent,
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
              decoration: BoxDecoration(
                color: Colors.blueAccent.withValues(alpha: 0.9),
                borderRadius: BorderRadius.circular(6),
                boxShadow: [BoxShadow(color: Colors.black.withValues(alpha: 0.3), blurRadius: 6)],
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(item.icon, size: 14, color: Colors.white),
                  const SizedBox(width: 4),
                  Text(item.label, style: const TextStyle(fontSize: 12, color: Colors.white)),
                ],
              ),
            ),
          ),
          childWhenDragging: Opacity(opacity: 0.3, child: tab),
          child: Container(
            decoration: BoxDecoration(
              color: isHovering ? Colors.blueAccent.withValues(alpha: 0.1) : null,
              borderRadius: BorderRadius.circular(4),
            ),
            child: tab,
          ),
        );
      },
    );
  }
}

// ========== 知识卡片组件 ==========

class _KnowledgeCardItem extends StatelessWidget {
  final KnowledgeCard card;
  final bool isDark;

  const _KnowledgeCardItem({required this.card, required this.isDark});

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.only(bottom: 12),
      color: isDark ? const Color(0xff2a2a2a) : const Color(0xfff8f8f8),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Text(card.emoji, style: const TextStyle(fontSize: 24)),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    card.title,
                    style: TextStyle(
                      fontSize: 16,
                      fontWeight: FontWeight.bold,
                      color: isDark ? Colors.white : Colors.black87,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              card.explanation,
              style: TextStyle(
                fontSize: 13,
                color: isDark ? const Color(0xffbbbbbb) : const Color(0xff555555),
                height: 1.5,
              ),
            ),
            const SizedBox(height: 12),
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(10),
              decoration: BoxDecoration(
                color: isDark ? const Color(0xff1e1e1e) : const Color(0xffeeeeee),
                borderRadius: BorderRadius.circular(6),
                border: Border.all(color: Colors.green.withValues(alpha: 0.3)),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text('✅ 正确写法', style: TextStyle(fontSize: 11, color: Colors.green)),
                  const SizedBox(height: 4),
                  Text(
                    card.correctCode,
                    style: const TextStyle(fontFamily: 'monospace', fontSize: 12, color: Colors.green),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 8),
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(10),
              decoration: BoxDecoration(
                color: isDark ? const Color(0xff1e1e1e) : const Color(0xffeeeeee),
                borderRadius: BorderRadius.circular(6),
                border: Border.all(color: Colors.redAccent.withValues(alpha: 0.3)),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text('❌ 错误写法', style: TextStyle(fontSize: 11, color: Colors.redAccent)),
                  const SizedBox(height: 4),
                  Text(
                    card.wrongCode,
                    style: const TextStyle(fontFamily: 'monospace', fontSize: 12, color: Colors.redAccent),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

// ========== 小型组件 ==========

class _ToolButton extends StatelessWidget {
  final IconData icon;
  final Color? color;
  final VoidCallback? onPressed;

  const _ToolButton({required this.icon, this.color, this.onPressed});

  @override
  Widget build(BuildContext context) {
    return IconButton(
      icon: Icon(icon, size: 20, color: color),
      padding: const EdgeInsets.all(6),
      constraints: const BoxConstraints(),
      onPressed: onPressed,
    );
  }
}

class _SymbolChip extends StatelessWidget {
  final String label;
  final VoidCallback onTap;
  final bool isAction;

  const _SymbolChip({required this.label, required this.onTap, this.isAction = false});

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      child: Container(
        margin: const EdgeInsets.symmetric(horizontal: 2, vertical: 4),
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        decoration: BoxDecoration(
          color: isAction ? Colors.blueAccent.withValues(alpha: 0.15) : Theme.of(context).dividerColor.withValues(alpha: 0.1),
          borderRadius: BorderRadius.circular(4),
        ),
        alignment: Alignment.center,
        child: Text(
          label,
          style: TextStyle(
            fontSize: 12,
            color: isAction ? Colors.blueAccent : Theme.of(context).textTheme.bodyMedium?.color,
            fontFamily: 'monospace',
          ),
        ),
      ),
    );
  }
}

class _ArrayVisualizer extends StatefulWidget {
  final String name;
  final int addr;
  final String tyName;
  final bool isDark;

  const _ArrayVisualizer({
    required this.name,
    required this.addr,
    required this.tyName,
    required this.isDark,
  });

  @override
  State<_ArrayVisualizer> createState() => _ArrayVisualizerState();
}

class _ArrayVisualizerState extends State<_ArrayVisualizer> {
  static const int _maxElements = 20;

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.only(bottom: 12),
      color: widget.isDark ? const Color(0xff2a2a2a) : const Color(0xfff8f8f8),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Text(
                  widget.name,
                  style: TextStyle(
                    fontSize: 14,
                    fontWeight: FontWeight.bold,
                    color: widget.isDark ? Colors.white : Colors.black87,
                  ),
                ),
                const SizedBox(width: 8),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                  decoration: BoxDecoration(
                    color: widget.isDark ? const Color(0xff3a3a3a) : const Color(0xffe0e0e0),
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(
                    widget.tyName,
                    style: TextStyle(fontSize: 11, color: Colors.grey[600], fontFamily: 'monospace'),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            FutureBuilder<dynamic>(
              future: rust.readMemory(addr: widget.addr, count: _maxElements),
              builder: (context, snapshot) {
                if (!snapshot.hasData) {
                  return const Center(child: SizedBox(width: 20, height: 20, child: CircularProgressIndicator(strokeWidth: 2)));
                }
                final data = (snapshot.data as List<dynamic>).cast<int>().toList();
                if (data.isEmpty) {
                  return const Text('无法读取数组数据', style: TextStyle(color: Colors.grey, fontSize: 12));
                }
                return _ArrayBarChart(data: data, isDark: widget.isDark);
              },
            ),
          ],
        ),
      ),
    );
  }
}

class _ArrayBarChart extends StatefulWidget {
  final List<int> data;
  final bool isDark;

  const _ArrayBarChart({required this.data, required this.isDark});

  @override
  State<_ArrayBarChart> createState() => _ArrayBarChartState();
}

class _ArrayBarChartState extends State<_ArrayBarChart> {
  final Set<int> _flashIndices = {};

  @override
  void didUpdateWidget(covariant _ArrayBarChart oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.data.length != oldWidget.data.length) {
      _flashIndices.clear();
      return;
    }
    final changed = <int>{};
    for (var i = 0; i < widget.data.length; i++) {
      if (i < oldWidget.data.length && widget.data[i] != oldWidget.data[i]) {
        changed.add(i);
      }
    }
    if (changed.isNotEmpty) {
      setState(() {
        _flashIndices.addAll(changed);
      });
      Future.delayed(const Duration(milliseconds: 500), () {
        if (mounted) {
          setState(() {
            for (final i in changed) {
              _flashIndices.remove(i);
            }
          });
        }
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final maxVal = widget.data.map((v) => v.abs()).reduce((a, b) => a > b ? a : b).clamp(1, 999999);
    const barHeight = 120.0;

    return SizedBox(
      height: barHeight + 24,
      child: ListView.separated(
        scrollDirection: Axis.horizontal,
        itemCount: widget.data.length,
        separatorBuilder: (_, __) => const SizedBox(width: 4),
        itemBuilder: (context, index) {
          final val = widget.data[index];
          final ratio = val.abs() / maxVal;
          final height = (ratio * barHeight).clamp(4.0, barHeight);
          final isNegative = val < 0;
          final isFlashing = _flashIndices.contains(index);

          Color barColor;
          if (isFlashing) {
            barColor = Colors.amber;
          } else if (isNegative) {
            barColor = Colors.redAccent.withValues(alpha: 0.7);
          } else {
            barColor = Colors.blueAccent.withValues(alpha: 0.7);
          }

          return Column(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              AnimatedContainer(
                duration: const Duration(milliseconds: 200),
                width: 24,
                height: height,
                decoration: BoxDecoration(
                  color: barColor,
                  borderRadius: const BorderRadius.vertical(top: Radius.circular(3)),
                  boxShadow: isFlashing
                      ? [BoxShadow(color: Colors.amber.withValues(alpha: 0.6), blurRadius: 8, spreadRadius: 2)]
                      : null,
                ),
              ),
              const SizedBox(height: 4),
              Text(
                '$val',
                style: TextStyle(
                  fontSize: 10,
                  color: widget.isDark ? Colors.grey[400] : Colors.grey[700],
                  fontFamily: 'monospace',
                  fontWeight: isFlashing ? FontWeight.bold : FontWeight.normal,
                ),
              ),
              Text(
                '[$index]',
                style: TextStyle(fontSize: 9, color: Colors.grey[600]),
              ),
            ],
          );
        },
      ),
    );
  }
}

class _TemplateChip extends StatelessWidget {
  final String label;
  final VoidCallback onTap;

  const _TemplateChip({required this.label, required this.onTap});

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      child: Container(
        margin: const EdgeInsets.symmetric(horizontal: 4, vertical: 6),
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
        decoration: BoxDecoration(
          color: Colors.green.withValues(alpha: 0.15),
          borderRadius: BorderRadius.circular(12),
        ),
        alignment: Alignment.center,
        child: Text(
          label,
          style: const TextStyle(fontSize: 12, color: Colors.green),
        ),
      ),
    );
  }
}

/// 学习进度卡片组件
class _ProgressCard extends StatelessWidget {
  final String title;
  final String value;
  final String subtitle;
  final IconData icon;
  final Color color;
  final double? progress;

  const _ProgressCard({
    required this.title,
    required this.value,
    required this.subtitle,
    required this.icon,
    required this.color,
    this.progress,
  });

  @override
  Widget build(BuildContext context) {
    final isDark = Theme.of(context).brightness == Brightness.dark;
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: isDark ? const Color(0xff2a2a2a) : const Color(0xfff8f8f8),
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: color.withValues(alpha: 0.2)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(icon, size: 18, color: color),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  title,
                  style: TextStyle(
                    fontSize: 13,
                    fontWeight: FontWeight.bold,
                    color: isDark ? Colors.white : Colors.black87,
                  ),
                ),
              ),
              Text(
                value,
                style: TextStyle(
                  fontSize: 16,
                  fontWeight: FontWeight.bold,
                  color: color,
                ),
              ),
            ],
          ),
          const SizedBox(height: 6),
          Text(
            subtitle,
            style: TextStyle(fontSize: 11, color: Colors.grey[500], height: 1.4),
          ),
          if (progress != null) ...[
            const SizedBox(height: 8),
            ClipRRect(
              borderRadius: BorderRadius.circular(4),
              child: LinearProgressIndicator(
                value: progress,
                backgroundColor: color.withValues(alpha: 0.1),
                valueColor: AlwaysStoppedAnimation<Color>(color),
                minHeight: 6,
              ),
            ),
          ],
        ],
      ),
    );
  }
}
