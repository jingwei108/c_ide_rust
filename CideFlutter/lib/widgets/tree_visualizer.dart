import 'dart:math' as math;
import 'package:flutter/material.dart';
import 'package:cide/src/rust/api/types.dart' as rust;

/// 树节点数据
class _TreeNodeData {
  final int address;
  final int val;
  final int? leftAddr;
  final int? rightAddr;
  final Color? flashColor;
  double x = 0;
  double y = 0;

  _TreeNodeData({
    required this.address,
    required this.val,
    this.leftAddr,
    this.rightAddr,
    this.flashColor,
  });
}

/// 二叉树可视化组件
class TreeVisualizer extends StatefulWidget {
  final int rootAddr;
  final String structName;
  final List<rust.VisEvent> visEvents;
  final bool isDark;

  const TreeVisualizer({
    super.key,
    required this.rootAddr,
    required this.structName,
    this.visEvents = const [],
    this.isDark = false,
  });

  @override
  State<TreeVisualizer> createState() => _TreeVisualizerState();
}

class _TreeVisualizerState extends State<TreeVisualizer>
    with SingleTickerProviderStateMixin {
  List<_TreeNodeData> _nodes = [];
  bool _loading = true;
  String? _error;
  late AnimationController _entranceController;

  @override
  void initState() {
    super.initState();
    _entranceController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 600),
    );
    _loadNodes();
  }

  @override
  void didUpdateWidget(covariant TreeVisualizer oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.rootAddr != widget.rootAddr ||
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
      int valOffset = 0;
      int leftOffset = 4;
      int rightOffset = 8;
      for (final f in fields) {
        final name = f.name.toLowerCase();
        if (name == 'val' || name == 'value' || name == 'data') {
          valOffset = f.offset;
        } else if (name == 'left') {
          leftOffset = f.offset;
        } else if (name == 'right') {
          rightOffset = f.offset;
        }
      }

      // 收集 vis event 的闪色
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

      final nodeMap = <int, _TreeNodeData>{};
      final queue = <int>[widget.rootAddr];
      final visited = <int>{};
      const maxDepth = 6; // 限制最大深度，避免画布过宽
      int currentDepth = 0;
      int nodesAtCurrentDepth = 1;
      int nodesInNextDepth = 0;

      while (queue.isNotEmpty && currentDepth < maxDepth) {
        final addr = queue.removeAt(0);
        nodesAtCurrentDepth--;

        if (addr == 0 || addr < 64 || visited.contains(addr)) {
          if (nodesAtCurrentDepth == 0) {
            currentDepth++;
            nodesAtCurrentDepth = nodesInNextDepth;
            nodesInNextDepth = 0;
          }
          continue;
        }
        visited.add(addr);

        final valVals = await rust.readMemory(addr: addr + valOffset, count: 1);
        final leftVals = await rust.readMemory(addr: addr + leftOffset, count: 1);
        final rightVals = await rust.readMemory(addr: addr + rightOffset, count: 1);

        final val = valVals.isNotEmpty ? valVals[0] : 0;
        final left = leftVals.isNotEmpty ? leftVals[0] : 0;
        final right = rightVals.isNotEmpty ? rightVals[0] : 0;

        nodeMap[addr] = _TreeNodeData(
          address: addr,
          val: val,
          leftAddr: left != 0 ? left : null,
          rightAddr: right != 0 ? right : null,
          flashColor: flashColors[addr],
        );

        if (left != 0) {
          queue.add(left);
          nodesInNextDepth++;
        }
        if (right != 0) {
          queue.add(right);
          nodesInNextDepth++;
        }

        if (nodesAtCurrentDepth == 0) {
          currentDepth++;
          nodesAtCurrentDepth = nodesInNextDepth;
          nodesInNextDepth = 0;
        }
      }

      if (nodeMap.isNotEmpty) {
        _layoutNodes(nodeMap, widget.rootAddr);
      }

      if (mounted) {
        setState(() {
          _nodes = nodeMap.values.toList();
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

  /// 基于满二叉树位置的层级布局算法。
  void _layoutNodes(Map<int, _TreeNodeData> nodeMap, int rootAddr) {
    // 给每个节点分配 (level, pos)，其中 pos 是满二叉树中的位置索引
    final info = <int, (int, int)>{};
    final queue = <(int, int, int)>[]; // (addr, level, pos)
    queue.add((rootAddr, 0, 0));
    final visited = <int>{};
    int maxLevel = 0;

    while (queue.isNotEmpty) {
      final (addr, level, pos) = queue.removeAt(0);
      if (addr == 0 || !nodeMap.containsKey(addr) || visited.contains(addr)) continue;
      visited.add(addr);
      info[addr] = (level, pos);
      maxLevel = math.max(maxLevel, level);
      final node = nodeMap[addr]!;
      if (node.leftAddr != null) queue.add((node.leftAddr!, level + 1, pos * 2));
      if (node.rightAddr != null) queue.add((node.rightAddr!, level + 1, pos * 2 + 1));
    }

    const baseCellWidth = 56.0;
    const levelHeight = 72.0;
    const marginY = 16.0;

    for (final entry in info.entries) {
      final node = nodeMap[entry.key]!;
      final (level, pos) = entry.value;
      final cellWidth = baseCellWidth * math.pow(2, maxLevel - level).toInt();
      node.x = pos * cellWidth + cellWidth / 2;
      node.y = marginY + level * levelHeight;
    }
  }

  double get _canvasWidth {
    if (_nodes.isEmpty) return 200;
    final maxX = _nodes.map((n) => n.x).reduce(math.max);
    return maxX + 40;
  }

  double get _canvasHeight {
    if (_nodes.isEmpty) return 120;
    final maxY = _nodes.map((n) => n.y).reduce(math.max);
    return maxY + 60;
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
      return const Center(child: Text('树为空或无法遍历', style: TextStyle(color: Colors.grey)));
    }

    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      padding: const EdgeInsets.all(12),
      child: AnimatedBuilder(
        animation: _entranceController,
        builder: (context, child) {
          // RepaintBoundary 隔离入场动画重绘，避免影响父级面板。
          return RepaintBoundary(
            child: CustomPaint(
              size: Size(_canvasWidth, _canvasHeight),
              painter: _TreePainter(
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

class _TreePainter extends CustomPainter {
  final List<_TreeNodeData> nodes;
  final bool isDark;
  final double progress;

  // 复用 Paint 对象，避免每节点每帧重建。
  final Paint _nodePaint = Paint()..style = PaintingStyle.fill;
  final Paint _borderPaint = Paint()
    ..style = PaintingStyle.stroke
    ..strokeWidth = 1.5;
  final Paint _edgePaint = Paint()
    ..style = PaintingStyle.stroke
    ..strokeWidth = 1.5;

  _TreePainter({
    required this.nodes,
    required this.isDark,
    this.progress = 1.0,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final baseNodeColor = isDark ? const Color(0xFF3E4451) : const Color(0xFFE5E5E5);
    final baseBorderColor = isDark ? const Color(0xFF5C6370) : const Color(0xFFB0B0B0);
    _edgePaint.color = baseBorderColor;

    final textStyle = TextStyle(
      color: isDark ? const Color(0xFFABB2BF) : const Color(0xFF383A42),
      fontSize: 12,
      fontFamily: 'monospace',
    );

    const nodeWidth = 48.0;
    const nodeHeight = 36.0;

    // 构建地址到节点的映射，用于绘制边
    final nodeMap = <int, _TreeNodeData>{};
    for (final node in nodes) {
      nodeMap[node.address] = node;
    }

    // 绘制边（先绘制边，后绘制节点，使节点覆盖边）
    for (final node in nodes) {
      final parentProgress = ((progress - node.y * 0.002)).clamp(0.0, 1.0);
      if (parentProgress <= 0) continue;

      if (node.leftAddr != null && nodeMap.containsKey(node.leftAddr!)) {
        final child = nodeMap[node.leftAddr!]!;
        final childProgress = ((progress - child.y * 0.002)).clamp(0.0, 1.0);
        final currentEndX = node.x + (child.x - node.x) * math.min(parentProgress, childProgress);
        final currentEndY = node.y + (child.y - node.y) * math.min(parentProgress, childProgress);

        final path = Path()
          ..moveTo(node.x, node.y + nodeHeight / 2)
          ..lineTo(currentEndX, currentEndY - nodeHeight / 2);
        canvas.drawPath(path, _edgePaint);
      }
      if (node.rightAddr != null && nodeMap.containsKey(node.rightAddr!)) {
        final child = nodeMap[node.rightAddr!]!;
        final childProgress = ((progress - child.y * 0.002)).clamp(0.0, 1.0);
        final currentEndX = node.x + (child.x - node.x) * math.min(parentProgress, childProgress);
        final currentEndY = node.y + (child.y - node.y) * math.min(parentProgress, childProgress);

        final path = Path()
          ..moveTo(node.x, node.y + nodeHeight / 2)
          ..lineTo(currentEndX, currentEndY - nodeHeight / 2);
        canvas.drawPath(path, _edgePaint);
      }
    }

    // 绘制节点
    for (final node in nodes) {
      final nodeProgress = ((progress - node.y * 0.002)).clamp(0.0, 1.0);
      if (nodeProgress <= 0) continue;

      final slideY = node.y + (1.0 - nodeProgress) * 15.0;
      final alpha = nodeProgress;

      final rect = RRect.fromRectAndRadius(
        Rect.fromCenter(center: Offset(node.x, slideY), width: nodeWidth, height: nodeHeight),
        const Radius.circular(6),
      );

      // 背景
      _nodePaint.color = baseNodeColor.withValues(alpha: alpha);
      canvas.drawRRect(rect, _nodePaint);

      // 边框 / 闪色
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
      // 值文本
      final textSpan = TextSpan(
        text: '${node.val}',
        style: textStyle.copyWith(
          color: textStyle.color?.withValues(alpha: alpha),
        ),
      );
      final textPainter = TextPainter(
        text: textSpan,
        textDirection: TextDirection.ltr,
        textAlign: TextAlign.center,
      );
      textPainter.layout();
      textPainter.paint(
        canvas,
        Offset(node.x - textPainter.width / 2, slideY - textPainter.height / 2),
      );

      // 地址标签（底部）
      final addrSpan = TextSpan(
        text: '0x${node.address.toRadixString(16).toUpperCase()}',
        style: TextStyle(
          color: (isDark ? Colors.grey[600] : Colors.grey[400])?.withValues(alpha: alpha),
          fontSize: 7,
          fontFamily: 'monospace',
        ),
      );
      final addrPainter = TextPainter(text: addrSpan, textDirection: TextDirection.ltr);
      addrPainter.layout();
      addrPainter.paint(
        canvas,
        Offset(node.x - addrPainter.width / 2, slideY + nodeHeight / 2 + 2),
      );
    }
  }

  @override
  bool shouldRepaint(covariant _TreePainter oldDelegate) {
    return oldDelegate.nodes != nodes ||
        oldDelegate.isDark != isDark ||
        oldDelegate.progress != progress;
  }
}
