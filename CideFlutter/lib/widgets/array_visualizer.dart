import 'package:flutter/material.dart';
import 'dart:math' as math;

class ArrayVisualizer extends StatefulWidget {
  final String name;
  final String elementTy;
  final List<String> elements;
  final Set<int> highlightedIndices;
  final Set<int> swappedIndices;
  final bool isDark;

  const ArrayVisualizer({
    super.key,
    required this.name,
    required this.elementTy,
    required this.elements,
    this.highlightedIndices = const {},
    this.swappedIndices = const {},
    required this.isDark,
  });

  @override
  State<ArrayVisualizer> createState() => _ArrayVisualizerState();
}

class _ArrayVisualizerState extends State<ArrayVisualizer>
    with SingleTickerProviderStateMixin {
  static const int _maxElements = 40;

  late AnimationController _pulseController;
  List<String> _prevElements = [];

  @override
  void initState() {
    super.initState();
    _pulseController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 600),
    )..repeat(reverse: true);
  }

  @override
  void didUpdateWidget(covariant ArrayVisualizer oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.elements.length != widget.elements.length ||
        !_listEquals(oldWidget.elements, widget.elements)) {
      _prevElements = List.from(oldWidget.elements);
    }
  }

  @override
  void dispose() {
    _pulseController.dispose();
    super.dispose();
  }

  bool _listEquals(List<String> a, List<String> b) {
    if (a.length != b.length) return false;
    for (int i = 0; i < a.length; i++) {
      if (a[i] != b[i]) return false;
    }
    return true;
  }

  @override
  Widget build(BuildContext context) {
    final displayElements = widget.elements.length > _maxElements
        ? widget.elements.sublist(0, _maxElements)
        : widget.elements;

    // 解析数值用于条形图高度
    final numbers = displayElements.map((e) {
      final clean = e.replaceAll("'", "");
      return double.tryParse(clean) ?? 0.0;
    }).toList();

    final maxVal = numbers.map((v) => v.abs()).fold(0.0, (a, b) => a > b ? a : b).clamp(1.0, double.infinity);
    const barHeight = 120.0;

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
                    widget.elementTy,
                    style: TextStyle(fontSize: 11, color: Colors.grey[600], fontFamily: 'monospace'),
                  ),
                ),
                const Spacer(),
                Text(
                  '${widget.elements.length} 个元素',
                  style: TextStyle(fontSize: 11, color: Colors.grey[500]),
                ),
              ],
            ),
            const SizedBox(height: 12),
            SizedBox(
              height: barHeight + 28,
              child: ListView.separated(
                scrollDirection: Axis.horizontal,
                itemCount: displayElements.length,
                separatorBuilder: (_, __) => const SizedBox(width: 4),
                itemBuilder: (context, index) {
                  final valStr = displayElements[index];
                  final num = numbers[index];
                  final ratio = num.abs() / maxVal;
                  final height = (ratio * barHeight).clamp(4.0, barHeight);
                  final isNegative = num < 0;
                  final isHighlighted = widget.highlightedIndices.contains(index);
                  final isSwapped = widget.swappedIndices.contains(index);
                  final changed = _prevElements.isNotEmpty &&
                      index < _prevElements.length &&
                      index < displayElements.length &&
                      _prevElements[index] != displayElements[index];

                  Color barColor;
                  if (isSwapped) {
                    barColor = Colors.amber;
                  } else if (isHighlighted) {
                    barColor = Colors.cyanAccent;
                  } else if (isNegative) {
                    barColor = Colors.redAccent.withValues(alpha: 0.7);
                  } else {
                    barColor = Colors.blueAccent.withValues(alpha: 0.7);
                  }

                  Widget bar = AnimatedContainer(
                    duration: const Duration(milliseconds: 200),
                    curve: Curves.easeInOut,
                    width: 24,
                    height: height,
                    decoration: BoxDecoration(
                      color: barColor,
                      borderRadius: const BorderRadius.vertical(top: Radius.circular(3)),
                      boxShadow: (isHighlighted || isSwapped)
                          ? null // 脉冲动画会单独处理阴影
                          : null,
                    ),
                  );

                  // 高亮/交换脉冲动画
                  if (isHighlighted || isSwapped) {
                    bar = AnimatedBuilder(
                      animation: _pulseController,
                      builder: (context, child) {
                        final t = _pulseController.value;
                        final scale = isSwapped
                            ? 1.0 + 0.10 * math.sin(t * math.pi * 4)
                            : 1.0 + 0.06 * math.sin(t * math.pi * 2);
                        final shadowOpacity = isSwapped
                            ? 0.5 + 0.5 * math.sin(t * math.pi * 4).abs()
                            : 0.3 + 0.3 * math.sin(t * math.pi * 2).abs();
                        final shadowColor = isSwapped
                            ? Colors.amber
                            : Colors.cyanAccent;
                        return Transform.scale(
                          scale: scale,
                          child: Container(
                            decoration: BoxDecoration(
                              boxShadow: [
                                BoxShadow(
                                  color: shadowColor.withValues(alpha: shadowOpacity),
                                  blurRadius: 12,
                                  spreadRadius: 3,
                                ),
                              ],
                            ),
                            child: child,
                          ),
                        );
                      },
                      child: bar,
                    );
                  }

                  // 值变化弹跳动画
                  if (changed) {
                    bar = _BounceWidget(
                      trigger: valStr,
                      child: bar,
                    );
                  }

                  return Column(
                    mainAxisAlignment: MainAxisAlignment.end,
                    children: [
                      bar,
                      const SizedBox(height: 4),
                      Text(
                        valStr.length > 6 ? '${valStr.substring(0, 6)}..' : valStr,
                        style: TextStyle(
                          fontSize: 10,
                          color: widget.isDark ? Colors.grey[400] : Colors.grey[700],
                          fontFamily: 'monospace',
                          fontWeight: (isHighlighted || isSwapped) ? FontWeight.bold : FontWeight.normal,
                        ),
                      ),
                      Text(
                        '[$index]',
                        style: TextStyle(
                          fontSize: 9,
                          color: isSwapped
                              ? Colors.amber
                              : isHighlighted
                                  ? Colors.cyanAccent
                                  : Colors.grey[600],
                          fontWeight: (isHighlighted || isSwapped) ? FontWeight.bold : FontWeight.normal,
                        ),
                      ),
                    ],
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// 值变化时的弹跳动画组件。
class _BounceWidget extends StatefulWidget {
  final String trigger;
  final Widget child;

  const _BounceWidget({required this.trigger, required this.child});

  @override
  State<_BounceWidget> createState() => _BounceWidgetState();
}

class _BounceWidgetState extends State<_BounceWidget>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 400),
    );
    _controller.forward(from: 0.0);
  }

  @override
  void didUpdateWidget(covariant _BounceWidget oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.trigger != widget.trigger) {
      _controller.forward(from: 0.0);
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: _controller,
      builder: (context, child) {
        final v = _controller.value;
        // 弹性衰减：1.2 → 1.0，带轻微震荡
        final scale = 1.0 + 0.18 * (1.0 - v) * math.sin(v * math.pi * 2.5);
        return Transform.scale(
          scale: scale.clamp(1.0, 1.2),
          child: child!,
        );
      },
      child: widget.child,
    );
  }
}
