import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import '../editor/editor.dart';

/// ---------------------------------------------------------------------------
/// Phase 0 POC：Gesture Proxy 模式独立验证
/// ---------------------------------------------------------------------------
/// 验证清单：
/// 1. EditableText 完全透明，无可见残留
/// 2. CustomPaint 叠加绘制文本 + 选区
/// 3. 中文输入法 composing 下划线自绘（含跨行）
/// 4. Listener.onPointerDown + EditableText.onTap 坐标正确
/// 5. 滚动时文本、选区、光标同步无漂移
/// 6. 500 行代码性能 Profile
/// ---------------------------------------------------------------------------

class EditorPocScreen extends StatefulWidget {
  const EditorPocScreen({super.key});

  @override
  State<EditorPocScreen> createState() => _EditorPocScreenState();
}

class _EditorPocScreenState extends State<EditorPocScreen> {
  late final CideDocument _document;
  late final TextStyle _textStyle;
  final GlobalKey<CideEditorState> _editorKey = GlobalKey();

  bool _isDark = false;
  bool _showPerfOverlay = false;

  // 示例代码（含中文注释，用于测试 composing）
  static const String _sampleCode = '''#include <stdio.h>

// 中文注释测试：这是一个冒泡排序
void bubbleSort(int arr[], int n) {
    for (int i = 0; i < n - 1; i++) {
        for (int j = 0; j < n - i - 1; j++) {
            if (arr[j] > arr[j + 1]) {
                int temp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
        }
    }
}

int main() {
    int arr[] = {64, 34, 25, 12, 22, 11, 90};
    int n = sizeof(arr) / sizeof(arr[0]);
    bubbleSort(arr, n);
    printf("Sorted array: ");
    for (int i = 0; i < n; i++) {
        printf("%d ", arr[i]);
    }
    printf("\\n");
    return 0;
}
''';

  @override
  void initState() {
    super.initState();
    _document = CideDocument();
    _document.setText(_sampleCode);
    _document.addListener(_onDocChanged);
  }

  @override
  void dispose() {
    _document.removeListener(_onDocChanged);
    super.dispose();
  }

  void _onDocChanged() {
    if (mounted) setState(() {});
  }

  void _generate500Lines() {
    final buffer = StringBuffer();
    buffer.writeln('#include <stdio.h>');
    buffer.writeln();
    for (int i = 0; i < 100; i++) {
      buffer.writeln('void func$i() {');
      buffer.writeln('    int x = $i;');
      buffer.writeln('    int y = x * 2;');
      buffer.writeln('    printf("%d\\n", y);');
      buffer.writeln('    // 中文注释测试行 $i');
      buffer.writeln('}');
      buffer.writeln();
    }
    buffer.writeln('int main() {');
    for (int i = 0; i < 100; i++) {
      buffer.writeln('    func$i();');
    }
    buffer.writeln('    return 0;');
    buffer.writeln('}');
    _document.setText(buffer.toString());
  }

  void _insertSnippet() {
    _editorKey.currentState?.insertText('    printf("Hello Cide!\\n");');
  }

  void _toggleDarkMode() {
    setState(() => _isDark = !_isDark);
  }

  void _togglePerfOverlay() {
    setState(() => _showPerfOverlay = !_showPerfOverlay);
  }

  @override
  Widget build(BuildContext context) {
    final bgColor = _isDark ? const Color(0xff282c34) : const Color(0xfffafafa);
    final textColor = _isDark ? const Color(0xffabb2bf) : const Color(0xff383a42);
    final cursorColor = _isDark ? Colors.white : Colors.black;

    _textStyle = TextStyle(
      fontSize: 14,
      height: 1.5,
      fontFamily: 'Consolas',
      fontFamilyFallback: const ['monospace'],
      color: textColor,
    );

    final layers = <EditorLayer>[
      TextLayer(baseStyle: _textStyle),
      SelectionLayer(cursorColor: cursorColor),
      ComposingLayer(),
    ];

    return Scaffold(
      backgroundColor: bgColor,
      appBar: AppBar(
        backgroundColor: _isDark ? const Color(0xff1e1e1e) : const Color(0xfff3f3f3),
        foregroundColor: textColor,
        elevation: 0,
        title: const Text('Phase 0 POC — Gesture Proxy'),
        actions: [
          // 性能叠加层开关
          IconButton(
            icon: Icon(_showPerfOverlay ? Icons.speed : Icons.speed_outlined),
            tooltip: '性能叠加层',
            onPressed: _togglePerfOverlay,
          ),
          // 暗黑模式
          IconButton(
            icon: Icon(_isDark ? Icons.dark_mode : Icons.light_mode),
            tooltip: '切换主题',
            onPressed: _toggleDarkMode,
          ),
        ],
      ),
      body: Column(
        children: [
          // 控制栏
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: BoxDecoration(
              color: _isDark ? const Color(0xff1e1e1e) : const Color(0xfff3f3f3),
              border: Border(
                bottom: BorderSide(
                  color: _isDark ? const Color(0xff3e4451) : const Color(0xffe5e5e5),
                ),
              ),
            ),
            child: SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: Row(
                children: [
                  _ActionChip(
                    icon: Icons.keyboard,
                    label: '系统键盘',
                    onTap: () => _editorKey.currentState?.showSystemKeyboard(),
                  ),
                  _ActionChip(
                    icon: Icons.smart_button,
                    label: '自绘键盘',
                    onTap: () => _editorKey.currentState?.showCustomKeyboard(),
                  ),
                  _ActionChip(
                    icon: Icons.undo,
                    label: '撤销',
                    onTap: () => _editorKey.currentState?.undo(),
                  ),
                  _ActionChip(
                    icon: Icons.redo,
                    label: '重做',
                    onTap: () => _editorKey.currentState?.redo(),
                  ),
                  _ActionChip(
                    icon: Icons.code,
                    label: '插入片段',
                    onTap: _insertSnippet,
                  ),
                  _ActionChip(
                    icon: Icons.format_list_numbered,
                    label: '500行测试',
                    onTap: _generate500Lines,
                  ),
                  const SizedBox(width: 16),
                  // 状态显示
                  _StatusBadge('行: ${_document.lineCount}'),
                  _StatusBadge('字符: ${_document.text.length}'),
                  _StatusBadge('Undo: ${_document.undoStack.length}'),
                  _StatusBadge(
                    '光标: ${_document.selection.base.line + 1}:${_document.selection.base.col}',
                  ),
                ],
              ),
            ),
          ),
          // 编辑器主体
          Expanded(
            child: Stack(
              children: [
                Padding(
                  padding: const EdgeInsets.all(12),
                  child: CideEditor(
                    key: _editorKey,
                    document: _document,
                    style: _textStyle,
                    layers: layers,
                    onTap: () => debugPrint('Editor tapped'),
                    onPointerDown: (pos) => debugPrint('Pointer down at $pos'),
                  ),
                ),
                if (_showPerfOverlay)
                  Positioned(
                    top: 8,
                    right: 8,
                    child: _PerformanceOverlay(document: _document),
                  ),
              ],
            ),
          ),
          // 底部自绘键盘占位（演示用）
          Container(
            height: 48,
            color: _isDark ? const Color(0xff1e1e1e) : const Color(0xfff3f3f3),
            child: Center(
              child: Text(
                '自绘键盘区域占位（Phase 1 接入）',
                style: TextStyle(
                  fontSize: 12,
                  color: textColor.withValues(alpha: 0.5),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

// ---------------------------------------------------------------------------
// 辅助组件
// ---------------------------------------------------------------------------

class _ActionChip extends StatelessWidget {
  final IconData icon;
  final String label;
  final VoidCallback onTap;

  const _ActionChip({
    required this.icon,
    required this.label,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(right: 8),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          onTap: onTap,
          borderRadius: BorderRadius.circular(16),
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
            decoration: BoxDecoration(
              border: Border.all(color: Colors.grey.withValues(alpha: 0.3)),
              borderRadius: BorderRadius.circular(16),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(icon, size: 14),
                const SizedBox(width: 4),
                Text(label, style: const TextStyle(fontSize: 12)),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _StatusBadge extends StatelessWidget {
  final String text;
  const _StatusBadge(this.text);

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.only(right: 8),
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: Colors.grey.withValues(alpha: 0.15),
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text(text, style: const TextStyle(fontSize: 11)),
    );
  }
}

/// 简易性能叠加层（FPS + 帧构建耗时）
class _PerformanceOverlay extends StatefulWidget {
  final CideDocument document;
  const _PerformanceOverlay({required this.document});

  @override
  State<_PerformanceOverlay> createState() => _PerformanceOverlayState();
}

class _PerformanceOverlayState extends State<_PerformanceOverlay>
    with SingleTickerProviderStateMixin {
  late final Ticker _ticker;
  int _frameCount = 0;
  double _fps = 0;

  @override
  void initState() {
    super.initState();
    _ticker = createTicker((elapsed) {
      _frameCount++;
      if (elapsed.inMilliseconds > 0 && elapsed.inMilliseconds % 1000 < 17) {
        setState(() {
          _fps = _frameCount * 1000.0 / elapsed.inMilliseconds;
        });
      }
    });
    _ticker.start();
  }

  @override
  void dispose() {
    _ticker.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: Colors.black.withValues(alpha: 0.7),
        borderRadius: BorderRadius.circular(4),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Text('FPS: ${_fps.toStringAsFixed(1)}',
              style: const TextStyle(color: Colors.green, fontSize: 12)),
          Text('Lines: ${widget.document.lineCount}',
              style: const TextStyle(color: Colors.white70, fontSize: 12)),
        ],
      ),
    );
  }
}
