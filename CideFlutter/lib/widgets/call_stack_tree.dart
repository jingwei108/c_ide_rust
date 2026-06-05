import 'package:flutter/material.dart';
import 'package:cide/src/rust/unified/types.dart' as rust;

/// 调用栈树形图组件。
///
/// 以横向缩进树形结构展示递归调用链，点击帧可跳转到对应代码行。
class CallStackTree extends StatelessWidget {
  final List<rust.ApiFrameInfo> frames;
  final void Function(int line)? onLineTap;

  const CallStackTree({
    super.key,
    required this.frames,
    this.onLineTap,
  });

  @override
  Widget build(BuildContext context) {
    if (frames.isEmpty) {
      return const Center(
        child: Text('无调用栈信息', style: TextStyle(fontSize: 12, color: Colors.grey)),
      );
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Padding(
          padding: EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          child: Text(
            '调用栈',
            style: TextStyle(fontSize: 12, fontWeight: FontWeight.bold),
          ),
        ),
        Expanded(
          child: ListView.builder(
            itemCount: frames.length,
            itemBuilder: (context, index) {
              final frame = frames[index];
              final depth = frames.length - 1 - index; // 栈顶最深
              final isTop = index == 0;

              return InkWell(
                onTap: frame.returnLine > 0 && onLineTap != null
                    ? () => onLineTap!(frame.returnLine)
                    : null,
                child: Padding(
                  padding: EdgeInsets.only(
                    left: 12 + depth * 16.0,
                    right: 12,
                    top: 4,
                    bottom: 4,
                  ),
                  child: Row(
                    children: [
                      Container(
                        width: 8,
                        height: 8,
                        decoration: BoxDecoration(
                          color: isTop ? Colors.orangeAccent : Colors.blueAccent,
                          shape: BoxShape.circle,
                        ),
                      ),
                      const SizedBox(width: 8),
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              frame.funcName,
                              style: TextStyle(
                                fontSize: 12,
                                fontWeight: isTop ? FontWeight.bold : FontWeight.normal,
                                color: isTop ? Colors.orangeAccent : null,
                              ),
                            ),
                            if (frame.returnLine > 0)
                              Text(
                                '第 ${frame.returnLine} 行',
                                style: const TextStyle(
                                  fontSize: 10,
                                  color: Colors.grey,
                                  decoration: TextDecoration.underline,
                                ),
                              ),
                          ],
                        ),
                      ),
                      if (isTop)
                        Container(
                          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                          decoration: BoxDecoration(
                            color: Colors.orangeAccent.withValues(alpha: 0.15),
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: const Text(
                            '当前',
                            style: TextStyle(fontSize: 9, color: Colors.orangeAccent),
                          ),
                        ),
                    ],
                  ),
                ),
              );
            },
          ),
        ),
      ],
    );
  }
}
