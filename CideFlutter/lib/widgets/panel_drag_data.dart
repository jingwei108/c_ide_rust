import '../models/panel_item.dart';

class PanelDragData {
  final String panelId;
  final PanelLocation fromLocation;
  final int fromIndex;

  const PanelDragData({
    required this.panelId,
    required this.fromLocation,
    required this.fromIndex,
  });
}
