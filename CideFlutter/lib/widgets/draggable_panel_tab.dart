import 'package:flutter/material.dart';
import '../models/panel_item.dart';
import 'panel_drag_data.dart';

class DraggablePanelTab extends StatelessWidget {
  final PanelItem item;
  final bool isActive;
  final String? badge;
  final VoidCallback onTap;
  final VoidCallback onDoubleTap;
  final PanelDragData data;
  final void Function(PanelDragData) onAccept;

  const DraggablePanelTab({
    super.key,
    required this.item,
    required this.isActive,
    this.badge,
    required this.onTap,
    required this.onDoubleTap,
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
      onAcceptWithDetails: (details) => onAccept(details.data),
      builder: (context, candidateData, rejectedData) {
        final isHovering = candidateData.isNotEmpty;
        return Draggable<PanelDragData>(
          data: data,
          feedback: Material(
            color: Colors.transparent,
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
              decoration: BoxDecoration(
                color: Colors.blueAccent.withValues(alpha: 0.9),
                borderRadius: BorderRadius.circular(6),
                boxShadow: [BoxShadow(color: Colors.black.withValues(alpha: 0.3), blurRadius: 6)],
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(item.icon, size: 14, color: Colors.white),
                  const SizedBox(width: 4),
                  Text(item.label, style: const TextStyle(fontSize: 12, color: Colors.white)),
                ],
              ),
            ),
          ),
          childWhenDragging: Opacity(opacity: 0.3, child: tab),
          child: Container(
            decoration: BoxDecoration(
              color: isHovering ? Colors.blueAccent.withValues(alpha: 0.1) : null,
              borderRadius: BorderRadius.circular(4),
            ),
            child: tab,
          ),
        );
      },
    );
  }
}
