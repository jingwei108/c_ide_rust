import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:cide/src/rust/unified/types.dart' as rust;
import '../providers/unified_provider.dart';

/// 变量历史面板：显示每个局部变量在执行过程中的变化趋势。
class VarHistoryTab extends ConsumerWidget {
  final bool isDark;

  const VarHistoryTab({super.key, required this.isDark});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(unifiedProvider);
    final frameCache = state.frameCache;

    if (frameCache.isEmpty) {
      return _buildEmpty('运行程序以查看变量历史');
    }

    // 从 frame_cache 中提取所有变量名及其历史值
    final varHistory = _extractVarHistory(frameCache);
    if (varHistory.isEmpty) {
      return _buildEmpty('未检测到变量');
    }

    final currentStep = state.currentStep;

    return ListView.builder(
      padding: const EdgeInsets.all(12),
      itemCount: varHistory.length,
      itemBuilder: (context, index) {
        final entry = varHistory[index];
        return _buildVarCard(entry, currentStep, isDark);
      },
    );
  }

  Widget _buildEmpty(String message) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(Icons.show_chart, size: 40, color: Colors.grey[500]),
          const SizedBox(height: 12),
          Text(message, style: TextStyle(fontSize: 14, color: Colors.grey[500])),
        ],
      ),
    );
  }

  Widget _buildVarCard(_VarHistoryEntry entry, int currentStep, bool isDark) {
    final textColor = isDark ? const Color(0xffd4d4d4) : const Color(0xff383a42);
    final subTextColor = isDark ? const Color(0xff5c6370) : const Color(0xffa0a1a7);

    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: isDark ? const Color(0xff1e1e1e) : const Color(0xfff5f5f5),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: isDark ? const Color(0xff3e4451) : const Color(0xffe5e5e5),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                decoration: BoxDecoration(
                  color: Colors.blueAccent.withValues(alpha: 0.1),
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Text(
                  entry.tyName,
                  style: const TextStyle(
                    fontSize: 10,
                    color: Colors.blueAccent,
                    fontFamily: 'monospace',
                  ),
                ),
              ),
              const SizedBox(width: 8),
              Text(
                entry.name,
                style: TextStyle(
                  fontSize: 14,
                  fontWeight: FontWeight.w600,
                  color: textColor,
                  fontFamily: 'monospace',
                ),
              ),
              const Spacer(),
              Text(
                entry.values.isNotEmpty ? entry.values.last.value : '-',
                style: TextStyle(
                  fontSize: 14,
                  color: textColor,
                  fontFamily: 'monospace',
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          SizedBox(
            height: 24,
            child: CustomPaint(
              painter: _VarTrendPainter(
                values: entry.values,
                currentStep: currentStep,
                isDark: isDark,
              ),
              size: const Size(double.infinity, 24),
            ),
          ),
          const SizedBox(height: 4),
          Text(
            '${entry.values.length} 次变化',
            style: TextStyle(fontSize: 11, color: subTextColor),
          ),
        ],
      ),
    );
  }

  /// 从 frame_cache 中提取变量历史。
  List<_VarHistoryEntry> _extractVarHistory(List<rust.StepPayload> frameCache) {
    final Map<String, _VarHistoryEntry> map = {};

    for (final payload in frameCache) {
      for (final varSnap in payload.localVars) {
        final name = varSnap.name;
        final entry = map.putIfAbsent(
          name,
          () => _VarHistoryEntry(
            name: name,
            tyName: varSnap.tyName,
            values: [],
          ),
        );
        // 只在值变化时记录
        if (entry.values.isEmpty || entry.values.last.value != varSnap.value) {
          entry.values.add(_VarValue(
            stepIndex: payload.stepIndex,
            value: varSnap.value,
          ));
        }
      }
    }

    return map.values.toList();
  }
}

class _VarHistoryEntry {
  final String name;
  final String tyName;
  final List<_VarValue> values;

  _VarHistoryEntry({required this.name, required this.tyName, required this.values});
}

class _VarValue {
  final int stepIndex;
  final String value;

  _VarValue({required this.stepIndex, required this.value});
}

/// 迷你趋势图绘制器。
class _VarTrendPainter extends CustomPainter {
  final List<_VarValue> values;
  final int currentStep;
  final bool isDark;

  _VarTrendPainter({
    required this.values,
    required this.currentStep,
    required this.isDark,
  });

  @override
  void paint(Canvas canvas, Size size) {
    if (values.isEmpty) return;

    // 尝试将所有值解析为数字
    final numericValues = <double>[];
    for (final v in values) {
      final parsed = double.tryParse(v.value);
      if (parsed != null) {
        numericValues.add(parsed);
      }
    }

    if (numericValues.length < 2) {
      // 非数值变量：绘制离散点
      _drawDiscreteDots(canvas, size);
      return;
    }

    // 数值变量：绘制折线趋势图
    final minVal = numericValues.reduce((a, b) => a < b ? a : b);
    final maxVal = numericValues.reduce((a, b) => a > b ? a : b);
    final range = (maxVal - minVal).abs();
    final padding = 4.0;

    final paint = Paint()
      ..color = Colors.blueAccent
      ..strokeWidth = 2
      ..style = PaintingStyle.stroke;

    final pointPaint = Paint()
      ..color = Colors.blueAccent
      ..style = PaintingStyle.fill;

    final currentPaint = Paint()
      ..color = Colors.orange
      ..style = PaintingStyle.fill;

    final path = Path();
    final stepWidth = (size.width - padding * 2) / (values.length - 1);

    for (int i = 0; i < values.length; i++) {
      final x = padding + i * stepWidth;
      final y = range == 0
          ? size.height / 2
          : padding + (size.height - padding * 2) *
              (1 - (numericValues[i] - minVal) / range);

      if (i == 0) {
        path.moveTo(x, y);
      } else {
        path.lineTo(x, y);
      }

      // 绘制当前步指示点
      if (values[i].stepIndex == currentStep) {
        canvas.drawCircle(Offset(x, y), 4, currentPaint);
      } else {
        canvas.drawCircle(Offset(x, y), 2, pointPaint);
      }
    }

    canvas.drawPath(path, paint);
  }

  void _drawDiscreteDots(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = Colors.grey
      ..style = PaintingStyle.fill;

    final stepWidth = size.width / values.length;
    for (int i = 0; i < values.length; i++) {
      final x = i * stepWidth + stepWidth / 2;
      final y = size.height / 2;
      canvas.drawCircle(Offset(x, y), 3, paint);
    }
  }

  @override
  bool shouldRepaint(covariant _VarTrendPainter old) {
    return old.currentStep != currentStep || old.values.length != values.length;
  }
}
