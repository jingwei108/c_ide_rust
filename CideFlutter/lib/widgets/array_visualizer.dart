import 'package:flutter/material.dart';
import 'package:cide/src/rust/api/cide.dart' as rust;

class ArrayVisualizer extends StatefulWidget {
  final String name;
  final int addr;
  final String tyName;
  final bool isDark;

  const ArrayVisualizer({
    super.key,
    required this.name,
    required this.addr,
    required this.tyName,
    required this.isDark,
  });

  @override
  State<ArrayVisualizer> createState() => _ArrayVisualizerState();
}

class _ArrayVisualizerState extends State<ArrayVisualizer> {
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
