import 'package:flutter/material.dart';

class HeightResizablePanel extends StatefulWidget {
  final double height;
  final ValueChanged<double> onHeightChanged;
  final Widget child;

  const HeightResizablePanel({
    super.key,
    required this.height,
    required this.onHeightChanged,
    required this.child,
  });

  @override
  State<HeightResizablePanel> createState() => _HeightResizablePanelState();
}

class _HeightResizablePanelState extends State<HeightResizablePanel> {
  double? _dragStartHeight;
  double? _dragStartY;

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // 拖拽条
        GestureDetector(
          onVerticalDragStart: (details) {
            _dragStartHeight = widget.height;
            _dragStartY = details.globalPosition.dy;
          },
          onVerticalDragUpdate: (details) {
            if (_dragStartHeight == null || _dragStartY == null) return;
            final delta = _dragStartY! - details.globalPosition.dy;
            widget.onHeightChanged(_dragStartHeight! + delta);
          },
          onVerticalDragEnd: (_) {
            _dragStartHeight = null;
            _dragStartY = null;
          },
          child: Container(
            height: 8,
            color: Colors.transparent,
            child: Center(
              child: Container(
                width: 40,
                height: 4,
                decoration: BoxDecoration(
                  color: Colors.grey.withValues(alpha: 0.4),
                  borderRadius: BorderRadius.circular(2),
                ),
              ),
            ),
          ),
        ),
        // 内容
        SizedBox(height: widget.height, child: widget.child),
      ],
    );
  }
}
