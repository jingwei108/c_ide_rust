import 'package:flutter/material.dart';
import 'package:cide/src/rust/api/cide.dart' as rust;

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

class _LinkedListVisualizerState extends State<LinkedListVisualizer> {
  List<_NodeData> _nodes = [];
  bool _loading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
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
      const linearMemorySize = 256 * 1024;

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

      setState(() {
        _nodes = nodes;
        _loading = false;
      });
    } catch (e) {
      setState(() {
        _error = e.toString();
        _loading = false;
      });
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
      child: CustomPaint(
        size: Size(_nodes.length * 100.0 + 40, 80),
        painter: _LinkedListPainter(
          nodes: _nodes,
          isDark: widget.isDark,
        ),
      ),
    );
  }
}

class _LinkedListPainter extends CustomPainter {
  final List<_NodeData> nodes;
  final bool isDark;

  _LinkedListPainter({required this.nodes, required this.isDark});

  @override
  void paint(Canvas canvas, Size size) {
    final nodePaint = Paint()
      ..color = isDark ? const Color(0xFF3E4451) : const Color(0xFFE5E5E5)
      ..style = PaintingStyle.fill;

    final borderPaint = Paint()
      ..color = isDark ? const Color(0xFF5C6370) : const Color(0xFFB0B0B0)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5;

    final arrowPaint = Paint()
      ..color = isDark ? const Color(0xFFABB2BF) : const Color(0xFF383A42)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5;

    final textStyle = TextStyle(
      color: isDark ? const Color(0xFFABB2BF) : const Color(0xFF383A42),
      fontSize: 12,
      fontFamily: 'monospace',
    );

    const nodeWidth = 60.0;
    const nodeHeight = 40.0;
    const spacing = 40.0;

    for (var i = 0; i < nodes.length; i++) {
      final x = 20.0 + i * (nodeWidth + spacing);
      const y = 20.0;
      final node = nodes[i];

      final rect = RRect.fromRectAndRadius(
        Rect.fromLTWH(x, y, nodeWidth, nodeHeight),
        const Radius.circular(4),
      );

      // 背景
      canvas.drawRRect(rect, nodePaint);

      // 闪色边框
      if (node.flashColor != null) {
        final flashPaint = Paint()
          ..color = node.flashColor!
          ..style = PaintingStyle.stroke
          ..strokeWidth = 2;
        canvas.drawRRect(rect, flashPaint);
      } else {
        canvas.drawRRect(rect, borderPaint);
      }

      // 数据文本
      final textSpan = TextSpan(text: '${node.data}', style: textStyle);
      final textPainter = TextPainter(
        text: textSpan,
        textDirection: TextDirection.ltr,
        textAlign: TextAlign.center,
      );
      textPainter.layout(minWidth: nodeWidth, maxWidth: nodeWidth);
      textPainter.paint(canvas, Offset(x, y + (nodeHeight - textPainter.height) / 2));

      // 地址标签
      final addrSpan = TextSpan(
        text: '0x${node.address.toRadixString(16).toUpperCase()}',
        style: TextStyle(
          color: isDark ? Colors.grey[600] : Colors.grey[400],
          fontSize: 8,
          fontFamily: 'monospace',
        ),
      );
      final addrPainter = TextPainter(text: addrSpan, textDirection: TextDirection.ltr);
      addrPainter.layout();
      addrPainter.paint(canvas, Offset(x + (nodeWidth - addrPainter.width) / 2, y - 14));

      // 绘制箭头到下一个节点
      if (node.nextAddress != null && i < nodes.length - 1) {
        final startX = x + nodeWidth;
        final startY = y + nodeHeight / 2;
        final endX = x + nodeWidth + spacing;
        final endY = startY;

        final path = Path()
          ..moveTo(startX, startY)
          ..lineTo(endX - 8, endY);
        canvas.drawPath(path, arrowPaint);

        // 箭头头部
        final arrowHead = Path()
          ..moveTo(endX - 8, endY - 4)
          ..lineTo(endX, endY)
          ..lineTo(endX - 8, endY + 4);
        canvas.drawPath(arrowHead, arrowPaint);
      } else if (node.nextAddress == null) {
        // NULL 终止符
        final nullSpan = TextSpan(
          text: 'NULL',
          style: TextStyle(
            color: isDark ? Colors.grey[600] : Colors.grey[400],
            fontSize: 10,
            fontFamily: 'monospace',
          ),
        );
        final nullPainter = TextPainter(text: nullSpan, textDirection: TextDirection.ltr);
        nullPainter.layout();
        nullPainter.paint(canvas, Offset(x + nodeWidth + 8, y + (nodeHeight - nullPainter.height) / 2));
      }
    }
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => true;
}
