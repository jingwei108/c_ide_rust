import 'package:flutter/material.dart';
import '../models/panel_item.dart';
import 'panel_drag_data.dart';

class DraggablePanelTab extends StatelessWidget {
  final PanelItem item;
  final bool isActive;
  final String? badge;
  final VoidCallback onTap;
  final VoidCallback? onDoubleTap;
  final PanelDragData data;
  final void Function(PanelDragData) onAccept;

  const DraggablePanelTab({
    super.key,
    required this.item,
    required this.isActive,
    this.badge,
    required this.onTap,
    this.onDoubleTap,
    required this.data,
    required this.onAccept,
  });

  @override
  Widget build(BuildContext context) {
    final tab = InkWell(
      onTap: onTap,
      onDoubleTap: onDoubleTap,
      child: Container(
        margin: const EdgeInsets.symmetric(horizontal: 2, vertical: 4),
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        decoration: BoxDecoration(
          border: Border(
            bottom: BorderSide(
              color: isActive ? Colors.blueAccent : Colors.transparent,
              width: 2,
            ),
          ),
        ),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(item.icon, size: 14, color: isActive ? Colors.blueAccent : Colors.grey),
            const SizedBox(width: 4),
            Flexible(
              child: Text(
                item.label,
                style: TextStyle(
                  fontSize: 12,
                  color: isActive ? Colors.blueAccent : Colors.grey,
                  fontWeight: isActive ? FontWeight.bold : FontWeight.normal,
                ),
                overflow: TextOverflow.ellipsis,
              ),
            ),
            if (badge != null) ...[
              const SizedBox(width: 4),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                decoration: BoxDecoration(
                  color: isActive ? Colors.blueAccent : Colors.grey,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Text(badge!, style: const TextStyle(fontSize: 10, color: Colors.white)),
              ),
            ],
          ],
        ),
      ),
    );

    return DragTarget<PanelDragData>(
      onWillAcceptWithDetails: (details) {
        debugPrint('[BottomDragTarget] onWillAccept: ${details.data.panelId}, from=${details.data.fromLocation}');
        return true;
      },
      onAcceptWithDetails: (details) {
        debugPrint('[BottomDragTarget] onAccept: ${details.data.panelId}');
        onAccept(details.data);
      },
      onLeave: (data) {
        debugPrint('[BottomDragTarget] onLeave: ${data?.panelId}');
      },
      builder: (context, candidateData, rejectedData) {
        final isHovering = candidateData.isNotEmpty;
        if (isHovering) debugPrint('[BottomDragTarget] isHovering=true, count=${candidateData.length}');
        return Draggable<PanelDragData>(
          data: data,
          dragAnchorStrategy: (draggable, context, position) {
            final renderBox = context.findRenderObject() as RenderBox?;
            if (renderBox != null) {
              final size = renderBox.size;
              debugPrint('[BottomTabDrag] size=$size, offset=${Offset(size.width / 2, size.height / 2)}');
              return Offset(size.width / 2, size.height / 2);
            }
            debugPrint('[BottomTabDrag] renderBox is null');
            return Offset.zero;
          },
          feedback: Material(
            color: Colors.transparent,
            elevation: 8,
            borderRadius: BorderRadius.circular(8),
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
              decoration: BoxDecoration(
                color: Colors.blueAccent.withValues(alpha: 0.95),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: Colors.white.withValues(alpha: 0.4), width: 1),
                boxShadow: [
                  BoxShadow(color: Colors.black.withValues(alpha: 0.5), blurRadius: 12, spreadRadius: 2),
                ],
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(item.icon, size: 15, color: Colors.white),
                  const SizedBox(width: 6),
                  Text(item.label, style: const TextStyle(fontSize: 13, color: Colors.white, fontWeight: FontWeight.w600)),
                ],
              ),
            ),
          ),
          childWhenDragging: Opacity(opacity: 0.5, child: tab),
          child: Container(
            decoration: BoxDecoration(
              color: isHovering ? Colors.blueAccent.withValues(alpha: 0.15) : null,
              borderRadius: BorderRadius.circular(4),
              border: isHovering ? Border.all(color: Colors.blueAccent.withValues(alpha: 0.5), width: 1.5) : null,
              boxShadow: isHovering
                  ? [BoxShadow(color: Colors.blueAccent.withValues(alpha: 0.2), blurRadius: 8, spreadRadius: 1)]
                  : null,
            ),
            child: tab,
          ),
        );
      },
    );
  }
}
