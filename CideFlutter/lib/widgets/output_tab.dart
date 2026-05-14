import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../models/ide_state.dart';
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
