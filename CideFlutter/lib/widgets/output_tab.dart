import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../providers/ide_provider.dart';

class OutputTab extends StatelessWidget {
  final IdeState state;
  final IdeNotifier notifier;
  final bool isDark;
  final TextEditingController inputController;

  const OutputTab({
    super.key,
    required this.state,
    required this.notifier,
    required this.isDark,
    required this.inputController,
  });

  @override
  Widget build(BuildContext context) {
    // TODO(#D09): output 较长时 SelectableText 每帧重建可能卡顿，未来应使用 ListView 或缓存 RichText。
    final emptyColor = isDark ? Colors.grey[600] : Colors.grey[400];
    return Column(
      children: [
        Expanded(
          child: Stack(
            children: [
              SingleChildScrollView(
                padding: const EdgeInsets.fromLTRB(12, 12, 44, 12),
                child: SelectableText(
                  state.output.isEmpty ? '' : state.output,
                  style: TextStyle(
                    fontFamily: 'Consolas',
                    fontFamilyFallback: const ['monospace'],
                    fontSize: 13,
                    color: isDark ? const Color(0xffd4d4d4) : const Color(0xff333333),
                  ),
                ),
              ),
              if (state.output.isEmpty)
                Center(
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(Icons.terminal_outlined, size: 40, color: emptyColor),
                      const SizedBox(height: 12),
                      Text(
                        '等待执行',
                        style: TextStyle(fontSize: 14, color: emptyColor),
                      ),
                    ],
                  ),
                )
              else
                Positioned(
                  top: 4,
                  right: 4,
                  child: Material(
                    color: isDark ? const Color(0xff2a2a2a) : const Color(0xfff0f0f0),
                    borderRadius: BorderRadius.circular(6),
                    child: InkWell(
                      borderRadius: BorderRadius.circular(6),
                      onTap: () {
                        Clipboard.setData(ClipboardData(text: state.output));
                        ScaffoldMessenger.of(context).showSnackBar(
                          const SnackBar(content: Text('已复制到剪贴板'), duration: Duration(seconds: 1)),
                        );
                      },
                      child: Container(
                        padding: const EdgeInsets.all(6),
                        child: Icon(Icons.copy, size: 16, color: isDark ? Colors.grey[400] : Colors.grey[600]),
                      ),
                    ),
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
                    controller: inputController,
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
                        inputController.clear();
                      }
                    },
                  ),
                ),
                TextButton(
                  onPressed: () {
                    final value = inputController.text;
                    if (value.isNotEmpty) {
                      notifier.provideInput(value);
                      inputController.clear();
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
}
