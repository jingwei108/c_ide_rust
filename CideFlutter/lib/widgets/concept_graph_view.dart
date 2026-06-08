import 'dart:math' as math;
import 'package:flutter/material.dart';
import '../src/rust/api/cide.dart' as rust;
import '../src/rust/diagnostics/knowledge_graph.dart' as rust_kg;

/// A simplified concept-graph canvas that arranges nodes in three domain columns
/// and draws edges between them. Activated concepts are highlighted.
///
/// Tapping a node shows a BottomSheet with its description and related knowledge cards.
class ConceptGraphView extends StatefulWidget {
  final List<rust_kg.ActivatedConcept> activated;

  const ConceptGraphView({super.key, required this.activated});

  @override
  State<ConceptGraphView> createState() => _ConceptGraphViewState();
}

class _ConceptGraphViewState extends State<ConceptGraphView> {
  List<rust_kg.ConceptNode> _allNodes = [];
  List<rust_kg.ConceptEdge> _allEdges = [];
  final Map<String, _NodeLayout> _layout = {};
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _loadGraph();
  }

  Future<void> _loadGraph() async {
    final nodes = await rust.getAllConceptNodes();
    final edges = await rust.getAllConceptEdges();
    setState(() {
      _allNodes = nodes;
      _allEdges = edges;
      _loading = false;
    });
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator());
    }

    return LayoutBuilder(
      builder: (context, constraints) {
        final size = Size(constraints.maxWidth, constraints.maxHeight);
        _computeLayout(size);

        final activatedIds = widget.activated.map((a) => a.node.id).toSet();
        final neighborIds = <String>{};
        for (final a in widget.activated) {
          for (final n in a.neighbors) {
            neighborIds.add(n.node.id);
          }
        }

        return GestureDetector(
          onTapUp: (details) => _handleTap(details.localPosition),
          child: CustomPaint(
            size: size,
            painter: _ConceptGraphPainter(
              nodes: _allNodes,
              edges: _allEdges,
              layout: _layout,
              activatedIds: activatedIds,
              neighborIds: neighborIds,
            ),
          ),
        );
      },
    );
  }

  void _computeLayout(Size size) {
    _layout.clear();
    final domains = <String, List<rust_kg.ConceptNode>>{};
    for (final n in _allNodes) {
      domains.putIfAbsent(n.domain, () => []).add(n);
    }

    final domainKeys = domains.keys.toList();
    final colWidth = size.width / domainKeys.length;

    for (int col = 0; col < domainKeys.length; col++) {
      final domain = domainKeys[col];
      final list = domains[domain]!;
      final gap = list.length > 1 ? size.height / (list.length + 1) : size.height / 2;
      for (int i = 0; i < list.length; i++) {
        final node = list[i];
        final x = col * colWidth + colWidth / 2;
        final y = list.length > 1 ? (i + 1) * gap : size.height / 2;
        _layout[node.id] = _NodeLayout(
          offset: Offset(x, y),
          domain: domain,
        );
      }
    }
  }

  void _handleTap(Offset position) {
    for (final entry in _layout.entries) {
      final center = entry.value.offset;
      if ((position - center).distance < 28) {
        final node = _allNodes.firstWhere((n) => n.id == entry.key);
        _showNodeDetail(node);
        return;
      }
    }
  }

  void _showNodeDetail(rust_kg.ConceptNode node) {
    final theme = Theme.of(context);
    final color = _domainColor(node.domain);
    showModalBottomSheet(
      context: context,
      builder: (context) => Container(
        padding: const EdgeInsets.all(20),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Container(
                  width: 12,
                  height: 12,
                  decoration: BoxDecoration(color: color, shape: BoxShape.circle),
                ),
                const SizedBox(width: 10),
                Expanded(
                  child: Text(
                    node.title,
                    style: theme.textTheme.titleMedium?.copyWith(fontWeight: FontWeight.w600),
                  ),
                ),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                  decoration: BoxDecoration(
                    color: color.withValues(alpha: 0.1),
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: Text(
                    '难度 ${node.difficulty}/5',
                    style: TextStyle(fontSize: 11, color: color, fontWeight: FontWeight.w500),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(node.description, style: theme.textTheme.bodyMedium),
            const SizedBox(height: 12),
            if (node.relatedCardIds.isNotEmpty)
              Text(
                '相关知识卡片: ${node.relatedCardIds.join(", ")}',
                style: TextStyle(fontSize: 12, color: Colors.grey[600]),
              ),
            const SizedBox(height: 16),
            Align(
              alignment: Alignment.centerRight,
              child: TextButton(
                onPressed: () => Navigator.pop(context),
                child: const Text('关闭'),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _NodeLayout {
  final Offset offset;
  final String domain;
  _NodeLayout({required this.offset, required this.domain});
}

class _ConceptGraphPainter extends CustomPainter {
  final List<rust_kg.ConceptNode> nodes;
  final List<rust_kg.ConceptEdge> edges;
  final Map<String, _NodeLayout> layout;
  final Set<String> activatedIds;
  final Set<String> neighborIds;

  // 预创建的 Paint 对象，避免 paint() 中每帧重复创建
  final Paint _glowPaint = Paint()
    ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 8);
  final Paint _borderPaint = Paint()
    ..style = PaintingStyle.stroke;
  final Paint _nodePaint = Paint();
  final Paint _edgePaint = Paint()..strokeCap = StrokeCap.round;
  final Paint _dashPaint = Paint()
    ..style = PaintingStyle.stroke;
  final Paint _legendPaint = Paint();

  // 缓存节点 label 的 TextPainter，避免重复 layout
  final Map<String, TextPainter> _labelPainterCache = {};

  // 缓存 legend 的 TextPainter（内容固定）
  final List<_LegendItem> _legendItems;

  _ConceptGraphPainter({
    required this.nodes,
    required this.edges,
    required this.layout,
    required this.activatedIds,
    required this.neighborIds,
  }) : _legendItems = _buildLegendCache();

  static List<_LegendItem> _buildLegendCache() {
    final raw = [
      ("编译概念", _domainColor("Compile")),
      ("内存概念", _domainColor("Memory")),
      ("控制流", _domainColor("ControlFlow")),
    ];
    double x = 12;
    const y = 12.0;
    final List<_LegendItem> result = [];
    for (final (label, color) in raw) {
      final tp = TextPainter(
        text: TextSpan(
          text: label,
          style: TextStyle(fontSize: 10, color: Colors.grey[600]),
        ),
        textDirection: TextDirection.ltr,
      );
      tp.layout();
      result.add(_LegendItem(label: label, color: color, painter: tp, x: x));
      x += 14 + tp.width + 20;
    }
    return result;
  }

  TextPainter _getLabelPainter(String text, bool isActivated) {
    final key = '${text}_${isActivated ? 'a' : 'n'}';
    final cached = _labelPainterCache[key];
    if (cached != null) return cached;

    final tp = TextPainter(
      text: TextSpan(
        text: text,
        style: TextStyle(
          color: isActivated
              ? Colors.white
              : Colors.white.withValues(alpha: 0.7),
          fontSize: 11,
          fontWeight: FontWeight.w600,
        ),
      ),
      textDirection: TextDirection.ltr,
    );
    tp.layout();
    _labelPainterCache[key] = tp;
    return tp;
  }

  @override
  void paint(Canvas canvas, Size size) {
    // Draw edges first (behind nodes).
    for (final edge in edges) {
      final fromLayout = layout[edge.from];
      final toLayout = layout[edge.to];
      if (fromLayout == null || toLayout == null) continue;

      final isActive = activatedIds.contains(edge.from) || activatedIds.contains(edge.to);
      _edgePaint.strokeWidth = isActive ? 2.0 : 0.8;
      _edgePaint.color = isActive
          ? _relationColor(edge.relation).withValues(alpha: 0.6)
          : Colors.grey.withValues(alpha: 0.2);

      if (edge.relation == "CommonMistake") {
        _dashPaint.strokeWidth = _edgePaint.strokeWidth;
        _dashPaint.color = _edgePaint.color;
        final path = Path();
        path.moveTo(fromLayout.offset.dx, fromLayout.offset.dy);
        path.lineTo(toLayout.offset.dx, toLayout.offset.dy);
        canvas.drawPath(
          _dashPath(path, dashLength: 6, dashGap: 4),
          _dashPaint,
        );
      } else {
        canvas.drawLine(fromLayout.offset, toLayout.offset, _edgePaint);
      }
    }

    // Draw nodes.
    for (final node in nodes) {
      final l = layout[node.id];
      if (l == null) continue;

      final isActivated = activatedIds.contains(node.id);
      final isNeighbor = neighborIds.contains(node.id);
      final baseColor = _domainColor(node.domain);
      final color = isActivated
          ? baseColor
          : isNeighbor
              ? baseColor.withValues(alpha: 0.5)
              : baseColor.withValues(alpha: 0.2);

      // Glow for activated.
      if (isActivated) {
        _glowPaint.color = baseColor.withValues(alpha: 0.15);
        canvas.drawCircle(l.offset, 32, _glowPaint);
      }

      // Node circle.
      _nodePaint.color = color;
      canvas.drawCircle(l.offset, 24, _nodePaint);

      // Border.
      _borderPaint.color = isActivated ? Colors.white : Colors.transparent;
      _borderPaint.strokeWidth = isActivated ? 3 : 0;
      canvas.drawCircle(l.offset, 24, _borderPaint);

      // Label (first 2 chars).
      final text = node.title.length > 2 ? node.title.substring(0, 2) : node.title;
      final textPainter = _getLabelPainter(text, isActivated);
      textPainter.paint(
        canvas,
        l.offset - Offset(textPainter.width / 2, textPainter.height / 2),
      );
    }

    // Draw legend.
    _drawLegend(canvas);
  }

  Path _dashPath(Path source, {required double dashLength, required double dashGap}) {
    final dashed = Path();
    final metrics = source.computeMetrics().toList();
    for (final metric in metrics) {
      var distance = 0.0;
      while (distance < metric.length) {
        final start = distance;
        final end = math.min(distance + dashLength, metric.length);
        dashed.addPath(metric.extractPath(start, end), Offset.zero);
        distance += dashLength + dashGap;
      }
    }
    return dashed;
  }

  void _drawLegend(Canvas canvas) {
    const y = 12.0;
    for (final item in _legendItems) {
      final r = RRect.fromRectAndRadius(
        Rect.fromLTWH(item.x, y, 10, 10),
        const Radius.circular(2),
      );
      _legendPaint.color = item.color;
      canvas.drawRRect(r, _legendPaint);
      item.painter.paint(canvas, Offset(item.x + 14, y - 1));
    }
  }

  @override
  bool shouldRepaint(covariant _ConceptGraphPainter oldDelegate) {
    return oldDelegate.activatedIds != activatedIds || oldDelegate.neighborIds != neighborIds;
  }
}

class _LegendItem {
  final String label;
  final Color color;
  final TextPainter painter;
  final double x;

  _LegendItem({
    required this.label,
    required this.color,
    required this.painter,
    required this.x,
  });
}

Color _domainColor(String domain) {
  switch (domain) {
    case "Compile":
      return Colors.blue.shade600;
    case "Memory":
      return Colors.orange.shade600;
    case "ControlFlow":
      return Colors.green.shade600;
    default:
      return Colors.grey.shade600;
  }
}

Color _relationColor(String relation) {
  switch (relation) {
    case "Prerequisite":
      return Colors.orange;
    case "LeadsTo":
      return Colors.blue;
    case "CommonMistake":
      return Colors.red;
    case "UsedTogether":
      return Colors.purple;
    case "Contradicts":
      return Colors.teal;
    default:
      return Colors.grey;
  }
}
