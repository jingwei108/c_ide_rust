import 'package:flutter/material.dart';

class ArrayVisualizer extends StatefulWidget {
  final String name;
  final String elementTy;
  final List<String> elements;
  final Set<int> highlightedIndices;
  final bool isDark;

  const ArrayVisualizer({
    super.key,
    required this.name,
    required this.elementTy,
    required this.elements,
    this.highlightedIndices = const {},
    required this.isDark,
  });

  @override
  State<ArrayVisualizer> createState() => _ArrayVisualizerState();
}

class _ArrayVisualizerState extends State<ArrayVisualizer> {
  static const int _maxElements = 40;

  @override
  Widget build(BuildContext context) {
    final displayElements = widget.elements.length > _maxElements
        ? widget.elements.sublist(0, _maxElements)
        : widget.elements;

    // 解析数值用于条形图高度
    final numbers = displayElements.map((e) {
      // 去掉引号，如 "'5'" → "5"
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

                  Color barColor;
                  if (isHighlighted) {
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
                          boxShadow: isHighlighted
                              ? [BoxShadow(color: Colors.amber.withValues(alpha: 0.6), blurRadius: 8, spreadRadius: 2)]
                              : null,
                        ),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        valStr.length > 6 ? '${valStr.substring(0, 6)}..' : valStr,
                        style: TextStyle(
                          fontSize: 10,
                          color: widget.isDark ? Colors.grey[400] : Colors.grey[700],
                          fontFamily: 'monospace',
                          fontWeight: isHighlighted ? FontWeight.bold : FontWeight.normal,
                        ),
                      ),
                      Text(
                        '[$index]',
                        style: TextStyle(
                          fontSize: 9,
                          color: isHighlighted ? Colors.amber : Colors.grey[600],
                          fontWeight: isHighlighted ? FontWeight.bold : FontWeight.normal,
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
