import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../providers/unified_provider.dart';

/// 调用栈面板：集成统一模式，从 frameCache 读取历史调用栈。
class CallstackTab extends ConsumerWidget {
  final bool isDark;

  const CallstackTab({super.key, required this.isDark});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final unifiedState = ref.watch(unifiedProvider);
    final frameCache = unifiedState.frameCache;
    final currentStep = unifiedState.currentStep;

    if (frameCache.isEmpty || currentStep < 0 || currentStep >= frameCache.length) {
      return _buildEmpty('运行程序以查看调用栈');
    }

    final payload = frameCache[currentStep];
    final callStack = payload.callStack;

    if (callStack.isEmpty) {
      return _buildEmpty('调用栈为空');
    }

    final textColor = isDark ? const Color(0xffd4d4d4) : const Color(0xff383a42);
    final subTextColor = isDark ? const Color(0xff5c6370) : const Color(0xffa0a1a7);

    return ListView.builder(
      padding: const EdgeInsets.all(12),
      itemCount: callStack.length,
      itemBuilder: (context, index) {
        final frame = callStack[index];
        final isCurrent = index == 0;
        final indent = index * 12.0;

        return AnimatedContainer(
          duration: const Duration(milliseconds: 250),
          curve: Curves.easeInOut,
          margin: const EdgeInsets.only(bottom: 6),
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          decoration: BoxDecoration(
            color: isCurrent
                ? (isDark ? const Color(0xff1a3a5c) : const Color(0xffe3f2fd))
                : (isDark ? const Color(0xff1e1e1e) : const Color(0xfff5f5f5)),
            borderRadius: BorderRadius.circular(6),
            border: Border.all(
              color: isCurrent
                  ? Colors.blueAccent.withValues(alpha: 0.5)
                  : (isDark ? const Color(0xff3e4451) : const Color(0xffe5e5e5)),
              width: isCurrent ? 1.5 : 1,
            ),
          ),
          child: Row(
            children: [
              SizedBox(width: indent),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Text(
                          frame.funcName.isEmpty ? '(匿名)' : frame.funcName,
                          style: TextStyle(
                            fontFamily: 'monospace',
                            fontSize: 13,
                            color: textColor,
                            fontWeight: isCurrent ? FontWeight.w600 : FontWeight.normal,
                          ),
                        ),
                        if (isCurrent) ...[
                          const SizedBox(width: 8),
                          Container(
                            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                            decoration: BoxDecoration(
                              color: Colors.blueAccent.withValues(alpha: 0.2),
                              borderRadius: BorderRadius.circular(4),
                            ),
                            child: const Text(
                              '当前',
                              style: TextStyle(fontSize: 10, color: Colors.blueAccent),
                            ),
                          ),
                        ],
                      ],
                    ),
                    const SizedBox(height: 2),
                    Text(
                      frame.returnLine > 0 ? '返回行 ${frame.returnLine}' : '入口',
                      style: TextStyle(fontSize: 11, color: subTextColor, fontFamily: 'monospace'),
                    ),
                  ],
                ),
              ),
              if (index < callStack.length - 1)
                Icon(
                  Icons.arrow_upward,
                  size: 14,
                  color: subTextColor.withValues(alpha: 0.5),
                ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildEmpty(String message) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(Icons.account_tree, size: 40, color: Colors.grey[500]),
          const SizedBox(height: 12),
          Text(message, style: TextStyle(fontSize: 14, color: Colors.grey[500])),
        ],
      ),
    );
  }
}
