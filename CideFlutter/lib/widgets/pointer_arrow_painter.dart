import 'package:flutter/material.dart';

/// 指针状态颜色与样式定义。
class _PointerStyle {
  final Color arrowColor;
  final Color cardColor;
  final Color textColor;
  final String label;
  final bool isDashed;

  const _PointerStyle({
    required this.arrowColor,
    required this.cardColor,
    required this.textColor,
    required this.label,
    this.isDashed = false,
  });
}

_PointerStyle _styleForStatus(PointerStatus status, bool isDark) {
  switch (status) {
    case PointerStatus.valid:
      return _PointerStyle(
        arrowColor: const Color(0xFF0A84FF),
        cardColor: isDark ? const Color(0xFF0A84FF).withValues(alpha: 0.15) : const Color(0xFF0A84FF).withValues(alpha: 0.08),
        textColor: const Color(0xFF0A84FF),
        label: '有效',
      );
    case PointerStatus.freed:
      return _PointerStyle(
        arrowColor: Colors.grey,
        cardColor: isDark ? Colors.grey.withValues(alpha: 0.15) : Colors.grey.withValues(alpha: 0.08),
        textColor: Colors.grey,
        label: '已释放',
        isDashed: true,
      );
    case PointerStatus.null_:
      return _PointerStyle(
        arrowColor: Colors.grey,
        cardColor: isDark ? Colors.grey.withValues(alpha: 0.15) : Colors.grey.withValues(alpha: 0.08),
        textColor: Colors.grey,
        label: 'NULL',
        isDashed: true,
      );
    case PointerStatus.dangling:
      return _PointerStyle(
        arrowColor: const Color(0xFFFF453A),
        cardColor: isDark ? const Color(0xFFFF453A).withValues(alpha: 0.15) : const Color(0xFFFF453A).withValues(alpha: 0.08),
        textColor: const Color(0xFFFF453A),
        label: '悬空',
        isDashed: true,
      );
  }
}

enum PointerStatus { valid, freed, null_, dangling }

/// 单条指针箭头可视化组件。
class PointerArrowWidget extends StatelessWidget {
  final String name;
  final int addr;
  final String tyName;
  final int targetAddr;
  final String targetName;
  final PointerStatus status;
  final bool isDark;

  const PointerArrowWidget({
    super.key,
    required this.name,
    required this.addr,
    required this.tyName,
    required this.targetAddr,
    required this.targetName,
    required this.status,
    required this.isDark,
  });

  @override
  Widget build(BuildContext context) {
    final style = _styleForStatus(status, isDark);
    final addrText = '0x${addr.toRadixString(16).toUpperCase().padLeft(4, '0')}';
    final targetText = targetAddr == 0
        ? '0x0000'
        : '0x${targetAddr.toRadixString(16).toUpperCase().padLeft(4, '0')}';

    Widget content = Padding(
      padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 12),
      child: Row(
        children: [
          // 左侧：指针变量卡片
          _buildVarCard(
            title: name,
            subtitle: tyName,
            footer: addrText,
            style: style,
          ),
          // 中间：箭头
          Expanded(
            child: SizedBox(
              height: 40,
              child: CustomPaint(
                painter: _ArrowPainter(
                  color: style.arrowColor,
                  isDashed: style.isDashed,
                  isNull: status == PointerStatus.null_,
                  showBreak: status == PointerStatus.freed,
                ),
              ),
            ),
          ),
          // 右侧：目标卡片
          _buildTargetCard(
            addr: targetText,
            name: targetName.isNotEmpty ? targetName : style.label,
            style: style,
          ),
        ],
      ),
    );

    // NULL 指针渐隐动画
    if (status == PointerStatus.null_) {
      content = AnimatedOpacity(
        opacity: 0.35,
        duration: const Duration(milliseconds: 400),
        curve: Curves.easeOut,
        child: content,
      );
    }

    return content;
  }

  Widget _buildVarCard({
    required String title,
    required String subtitle,
    required String footer,
    required _PointerStyle style,
  }) {
    return Container(
      width: 110,
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
      decoration: BoxDecoration(
        color: style.cardColor,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: style.arrowColor.withValues(alpha: 0.3)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            title,
            style: TextStyle(
              fontSize: 12,
              fontWeight: FontWeight.w600,
              color: isDark ? const Color(0xffd4d4d4) : const Color(0xff333333),
              fontFamily: 'monospace',
            ),
          ),
          const SizedBox(height: 2),
          Text(
            subtitle,
            style: TextStyle(
              fontSize: 10,
              color: style.textColor,
              fontFamily: 'monospace',
            ),
          ),
          const SizedBox(height: 2),
          Text(
            footer,
            style: TextStyle(
              fontSize: 9,
              color: isDark ? Colors.grey : Colors.black54,
              fontFamily: 'monospace',
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildTargetCard({
    required String addr,
    required String name,
    required _PointerStyle style,
  }) {
    return Container(
      width: 110,
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
      decoration: BoxDecoration(
        color: style.cardColor,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: style.arrowColor.withValues(alpha: 0.3)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            addr,
            style: TextStyle(
              fontSize: 11,
              fontWeight: FontWeight.w600,
              color: style.textColor,
              fontFamily: 'monospace',
            ),
          ),
          const SizedBox(height: 2),
          Text(
            name,
            style: TextStyle(
              fontSize: 10,
              color: isDark ? Colors.grey : Colors.black54,
              fontFamily: 'monospace',
            ),
            overflow: TextOverflow.ellipsis,
          ),
        ],
      ),
    );
  }
}

/// 箭头绘制器。
class _ArrowPainter extends CustomPainter {
  final Color color;
  final bool isDashed;
  final bool isNull;
  final bool showBreak;

  _ArrowPainter({
    required this.color,
    this.isDashed = false,
    this.isNull = false,
    this.showBreak = false,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = color
      ..strokeWidth = 2
      ..style = PaintingStyle.stroke;

    final start = Offset(0, size.height / 2);
    final end = Offset(size.width, size.height / 2);

    if (isNull) {
      // NULL 指针：画一条向下的折线，末端接地
      final midX = size.width / 2;
      final groundY = size.height - 4;
      final path = Path()
        ..moveTo(start.dx, start.dy)
        ..lineTo(midX, start.dy)
        ..lineTo(midX, groundY);
      if (isDashed) {
        paint.shader = null;
        canvas.drawPath(
          _dashPath(path, dashLength: 5, dashSpace: 3),
          paint,
        );
      } else {
        canvas.drawPath(path, paint);
      }
      // 接地线
      canvas.drawLine(
        Offset(midX - 8, groundY),
        Offset(midX + 8, groundY),
        paint..strokeWidth = 2,
      );
      return;
    }

    // 普通箭头（断裂时分为两段）
    if (showBreak) {
      final breakCenter = Offset(size.width / 2, size.height / 2);
      final breakSize = 6.0;
      // 左段
      final leftPath = Path()
        ..moveTo(start.dx, start.dy)
        ..lineTo(breakCenter.dx - breakSize, start.dy);
      // 右段
      final rightPath = Path()
        ..moveTo(breakCenter.dx + breakSize, start.dy)
        ..lineTo(end.dx, start.dy);
      if (isDashed) {
        canvas.drawPath(_dashPath(leftPath, dashLength: 5, dashSpace: 3), paint);
        canvas.drawPath(_dashPath(rightPath, dashLength: 5, dashSpace: 3), paint);
      } else {
        canvas.drawPath(leftPath, paint);
        canvas.drawPath(rightPath, paint);
      }
      // 红叉
      final xPaint = Paint()
        ..color = const Color(0xFFFF453A)
        ..strokeWidth = 2.5
        ..style = PaintingStyle.stroke;
      canvas.drawLine(
        Offset(breakCenter.dx - breakSize, breakCenter.dy - breakSize),
        Offset(breakCenter.dx + breakSize, breakCenter.dy + breakSize),
        xPaint,
      );
      canvas.drawLine(
        Offset(breakCenter.dx + breakSize, breakCenter.dy - breakSize),
        Offset(breakCenter.dx - breakSize, breakCenter.dy + breakSize),
        xPaint,
      );
    } else {
      if (isDashed) {
        final path = Path()
          ..moveTo(start.dx, start.dy)
          ..lineTo(end.dx, start.dy);
        canvas.drawPath(
          _dashPath(path, dashLength: 5, dashSpace: 3),
          paint,
        );
      } else {
        canvas.drawLine(start, end, paint);
      }
    }

    // 箭头头部（断裂时不画）
    if (!showBreak) {
      const arrowSize = 8.0;
      final arrowPath = Path()
        ..moveTo(end.dx - arrowSize, end.dy - arrowSize / 2)
        ..lineTo(end.dx, end.dy)
        ..lineTo(end.dx - arrowSize, end.dy + arrowSize / 2);
      canvas.drawPath(arrowPath, paint..strokeWidth = 2);
    }
  }

  Path _dashPath(Path source, {required double dashLength, required double dashSpace}) {
    final dashed = Path();
    final metrics = source.computeMetrics();
    for (final metric in metrics) {
      var distance = 0.0;
      while (distance < metric.length) {
        dashed.addPath(
          metric.extractPath(distance, distance + dashLength),
          Offset.zero,
        );
        distance += dashLength + dashSpace;
      }
    }
    return dashed;
  }

  @override
  bool shouldRepaint(covariant _ArrowPainter oldDelegate) {
    return oldDelegate.color != color ||
        oldDelegate.isDashed != isDashed ||
        oldDelegate.isNull != isNull ||
        oldDelegate.showBreak != showBreak;
  }
}
