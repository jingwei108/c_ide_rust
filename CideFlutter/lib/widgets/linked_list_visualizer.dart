import 'package:flutter/material.dart';
import 'package:cide/src/rust/api/types.dart' as rust;

/// 链表节点数据
class _NodeData {
  final int address;
  final int data;
  final int? nextAddress;
  final Color? flashColor;

  _NodeData({
    required this.address,
    required this.data,
    this.nextAddress,
    this.flashColor,
  });
}

/// 链表图可视化组件
class LinkedListVisualizer extends StatefulWidget {
  final int headAddr;
  final String structName;
  final List<rust.VisEvent> visEvents;
  final bool isDark;

  const LinkedListVisualizer({
    super.key,
    required this.headAddr,
    required this.structName,
    this.visEvents = const [],
    this.isDark = false,
  });

  @override
  State<LinkedListVisualizer> createState() => _LinkedListVisualizerState();
}

class _LinkedListVisualizerState extends State<LinkedListVisualizer>
    with SingleTickerProviderStateMixin {
  List<_NodeData> _nodes = [];
  bool _loading = true;
  String? _error;
  late AnimationController _entranceController;

  @override
  void initState() {
    super.initState();
    _entranceController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 500),
    );
    _loadNodes();
  }

  @override
  void didUpdateWidget(covariant LinkedListVisualizer oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.headAddr != widget.headAddr ||
        oldWidget.visEvents.length != widget.visEvents.length) {
      _loadNodes();
    }
  }

  @override
  void dispose() {
    _entranceController.dispose();
    super.dispose();
  }

  Future<void> _loadNodes() async {
    setState(() {
      _loading = true;
      _error = null;
    });

    try {
      final fields = await rust.getStructFields(name: widget.structName);
      int dataOffset = 0;
      int nextOffset = 4;
      for (final f in fields) {
        final name = f.name.toLowerCase();
        if (name == 'data' || name == 'val' || name == 'value') {
          dataOffset = f.offset;
        } else if (name == 'next') {
          nextOffset = f.offset;
        }
      }

      // 收集 vis event 的闪色（按地址，最后的事件优先）
      final flashColors = <int, Color>{};
      for (final ev in widget.visEvents) {
        final addr = ev.extra0;
        switch (ev.ty) {
          case 4: // NodeCreate
            flashColors[addr] = const Color(0xFF32D74B);
            break;
          case 6: // NodeAccess
            flashColors[addr] = const Color(0xFF0A84FF);
            break;
          case 7: // NodeDelete
            flashColors[addr] = const Color(0xFFFF453A);
            break;
        }
      }

      final nodes = <_NodeData>[];
      final visited = <int>{};
      var currentAddr = widget.headAddr;
      const nullTrapEnd = 64;
      final linearMemorySize = await rust.getMemorySize();

      while (currentAddr != 0 &&
          currentAddr >= nullTrapEnd &&
          currentAddr < linearMemorySize &&
          !visited.contains(currentAddr)) {
        visited.add(currentAddr);

        final dataVals = await rust.readMemory(addr: currentAddr + dataOffset, count: 1);
        final nextVals = await rust.readMemory(addr: currentAddr + nextOffset, count: 1);
        final dataValue = dataVals.isNotEmpty ? dataVals[0] : 0;
        final nextValue = nextVals.isNotEmpty ? nextVals[0] : 0;

        nodes.add(_NodeData(
          address: currentAddr,
          data: dataValue,
          nextAddress: nextValue != 0 ? nextValue : null,
          flashColor: flashColors[currentAddr],
        ));

        currentAddr = nextValue;
      }

      if (mounted) {
        setState(() {
          _nodes = nodes;
          _loading = false;
        });
        _entranceController.forward(from: 0.0);
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _error = e.toString();
          _loading = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator(strokeWidth: 2));
    }
    if (_error != null) {
      return Center(child: Text('加载失败: $_error', style: const TextStyle(color: Colors.grey)));
    }
    if (_nodes.isEmpty) {
      return const Center(child: Text('链表为空或无法遍历', style: TextStyle(color: Colors.grey)));
    }

    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      padding: const EdgeInsets.all(16),
      child: AnimatedBuilder(
        animation: _entranceController,
        builder: (context, child) {
          // RepaintBoundary 隔离入场动画重绘，避免影响父级面板。
          return RepaintBoundary(
            child: CustomPaint(
              size: Size(_nodes.length * 100.0 + 40, 80),
              painter: _LinkedListPainter(
                nodes: _nodes,
                isDark: widget.isDark,
                progress: _entranceController.value,
              ),
            ),
          );
        },
      ),
    );
  }
}

class _LinkedListPainter extends CustomPainter {
  final List<_NodeData> nodes;
  final bool isDark;
  final double progress;

  // 复用 Paint 对象，避免每节点每帧重建。
  final Paint _nodePaint = Paint()..style = PaintingStyle.fill;
  final Paint _borderPaint = Paint()
    ..style = PaintingStyle.stroke
    ..strokeWidth = 1.5;
  final Paint _arrowPaint = Paint()
    ..style = PaintingStyle.stroke
    ..strokeWidth = 1.5;

  _LinkedListPainter({
    required this.nodes,
    required this.isDark,
    this.progress = 1.0,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final baseNodeColor = isDark ? const Color(0xFF3E4451) : const Color(0xFFE5E5E5);
    final baseBorderColor = isDark ? const Color(0xFF5C6370) : const Color(0xFFB0B0B0);
    final baseArrowColor = isDark ? const Color(0xFFABB2BF) : const Color(0xFF383A42);

    final textStyle = TextStyle(
      color: isDark ? const Color(0xFFABB2BF) : const Color(0xFF383A42),
      fontSize: 12,
      fontFamily: 'monospace',
    );

    const nodeWidth = 60.0;
    const nodeHeight = 40.0;
    const spacing = 40.0;

    for (var i = 0; i < nodes.length; i++) {
      // 渐进式入场：每个节点延迟 0.1 的进度
      final nodeProgress = ((progress - i * 0.08) / 0.5).clamp(0.0, 1.0);
      if (nodeProgress <= 0) continue;

      final x = 20.0 + i * (nodeWidth + spacing);
      const y = 20.0;
      final node = nodes[i];

      // 入场动画：从下方滑入 + 淡入
      final slideY = y + (1.0 - nodeProgress) * 20.0;
      final alpha = nodeProgress;

      final rect = RRect.fromRectAndRadius(
        Rect.fromLTWH(x, slideY, nodeWidth, nodeHeight),
        const Radius.circular(4),
      );

      // 背景（带透明度）
      _nodePaint.color = baseNodeColor.withValues(alpha: alpha);
      canvas.drawRRect(rect, _nodePaint);

      // 闪色边框或普通边框
      if (node.flashColor != null) {
        final flashPaint = Paint()
          ..color = node.flashColor!.withValues(alpha: alpha)
          ..style = PaintingStyle.stroke
          ..strokeWidth = 2;
        canvas.drawRRect(rect, flashPaint);
      } else {
        _borderPaint.color = baseBorderColor.withValues(alpha: alpha);
        canvas.drawRRect(rect, _borderPaint);
      }

      // TODO(#D09): 每个节点每帧新建两个 TextPainter，应缓存文本布局。
      // 数据文本（带透明度）
      final textSpan = TextSpan(
        text: '${node.data}',
        style: textStyle.copyWith(
          color: textStyle.color?.withValues(alpha: alpha),
        ),
      );
      final textPainter = TextPainter(
        text: textSpan,
        textDirection: TextDirection.ltr,
        textAlign: TextAlign.center,
      );
      textPainter.layout(minWidth: nodeWidth, maxWidth: nodeWidth);
      textPainter.paint(
        canvas,
        Offset(x, slideY + (nodeHeight - textPainter.height) / 2),
      );

      // 地址标签
      final addrSpan = TextSpan(
        text: '0x${node.address.toRadixString(16).toUpperCase()}',
        style: TextStyle(
          color: (isDark ? Colors.grey[600] : Colors.grey[400])?.withValues(alpha: alpha),
          fontSize: 8,
          fontFamily: 'monospace',
        ),
      );
      final addrPainter = TextPainter(text: addrSpan, textDirection: TextDirection.ltr);
      addrPainter.layout();
      addrPainter.paint(
        canvas,
        Offset(x + (nodeWidth - addrPainter.width) / 2, slideY - 14),
      );

      // 绘制箭头到下一个节点（带渐进动画）
      if (node.nextAddress != null && i < nodes.length - 1) {
        final arrowProgress = ((progress - i * 0.08 - 0.3) / 0.3).clamp(0.0, 1.0);
        if (arrowProgress > 0) {
          final startX = x + nodeWidth;
          final startY = slideY + nodeHeight / 2;
          final endX = x + nodeWidth + spacing;
          final endY = startY;
          final currentEndX = startX + (endX - startX - 8) * arrowProgress;

          _arrowPaint.color = baseArrowColor.withValues(alpha: alpha * arrowProgress);

          final path = Path()
            ..moveTo(startX, startY)
            ..lineTo(currentEndX, endY);
          canvas.drawPath(path, _arrowPaint);

          if (arrowProgress > 0.8) {
            // 箭头头部
            final headProgress = (arrowProgress - 0.8) / 0.2;
            final headAlpha = alpha * headProgress;
            _arrowPaint.color = baseArrowColor.withValues(alpha: headAlpha);
            final arrowHead = Path()
              ..moveTo(endX - 8, endY - 4)
              ..lineTo(endX, endY)
              ..lineTo(endX - 8, endY + 4);
            canvas.drawPath(arrowHead, _arrowPaint);
          }
        }
      } else if (node.nextAddress == null) {
        // NULL 终止符
        final nullSpan = TextSpan(
          text: 'NULL',
          style: TextStyle(
            color: (isDark ? Colors.grey[600] : Colors.grey[400])?.withValues(alpha: alpha),
            fontSize: 10,
            fontFamily: 'monospace',
          ),
        );
        final nullPainter = TextPainter(text: nullSpan, textDirection: TextDirection.ltr);
        nullPainter.layout();
        nullPainter.paint(
          canvas,
          Offset(x + nodeWidth + 8, slideY + (nodeHeight - nullPainter.height) / 2),
        );
      }
    }
  }

  @override
  bool shouldRepaint(covariant _LinkedListPainter oldDelegate) {
    return oldDelegate.nodes != nodes ||
        oldDelegate.isDark != isDark ||
        oldDelegate.progress != progress;
  }
}
