import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../providers/ide_provider.dart';
import '../../models/panel_item.dart';
import '../../providers/theme_provider.dart';
import '../../widgets/floating_orb_widget.dart';
import '../../widgets/floating_panel_popup.dart';
import 'bottom_panel.dart';

/// IDE 悬浮球与悬浮面板弹窗区域组件。
///
/// 通过 [OverlayEntry] 渲染悬浮球菜单与面板弹窗，管理自身的 overlay 生命周期。
class FloatingOrbArea extends ConsumerStatefulWidget {
  final TextEditingController inputController;
  final void Function(int line) onScrollToLine;
  final void Function(String source) onUpdateSource;

  const FloatingOrbArea({
    super.key,
    required this.inputController,
    required this.onScrollToLine,
    required this.onUpdateSource,
  });

  @override
  ConsumerState<FloatingOrbArea> createState() => _FloatingOrbAreaState();
}

class _FloatingOrbAreaState extends ConsumerState<FloatingOrbArea> {
  OverlayEntry? _orbOverlayEntry;
  OverlayEntry? _panelOverlayEntry;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _insertOrbOverlay();
    });
  }

  @override
  void dispose() {
    _orbOverlayEntry?.remove();
    _panelOverlayEntry?.remove();
    super.dispose();
  }

  void _insertOrbOverlay() {
    if (!mounted) return;
    _orbOverlayEntry = OverlayEntry(builder: (context) => _buildOrbOverlay());
    Overlay.of(context).insert(_orbOverlayEntry!);
  }

  void _insertPanelOverlay(String panelId) {
    _removePanelOverlay();
    if (!mounted) return;
    _panelOverlayEntry = OverlayEntry(
      builder: (context) => _buildPanelOverlay(panelId),
    );
    Overlay.of(context).insert(_panelOverlayEntry!);
  }

  void _removePanelOverlay() {
    _panelOverlayEntry?.remove();
    _panelOverlayEntry = null;
  }

  Widget _buildOrbOverlay() {
    return Consumer(
      builder: (context, ref, child) {
        final state = ref.watch(ideProvider);
        final notifier = ref.read(ideProvider.notifier);
        return FloatingOrbWidget(
          isMenuOpen: state.isFloatingOpen,
          menuItems: state.floatingSlots,
          onToggleMenu: notifier.toggleFloating,
          onSelectPanel: (panelId) {
            notifier.openFloatingPanel(panelId);
            _insertPanelOverlay(panelId);
          },
          onCloseMenu: notifier.closeFloating,
          onDragAccept: (dragData) {
            if (dragData.fromLocation == PanelLocation.bottom) {
              // 拖到边缘/padding 区域，未落在具体菜单项上
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(
                  content: Text('未识别到可交换的目标位置'),
                  duration: Duration(seconds: 1),
                ),
              );
            }
          },
          onSwapWithFloatingItem: (dragData, targetIndex) {
            if (dragData.fromLocation == PanelLocation.bottom) {
              // 底部 → 悬浮球具体项，交换
              notifier.swapBottomWithFloatingItem(
                dragData.panelId,
                targetIndex,
              );
            }
          },
        );
      },
    );
  }

  Widget _buildPanelOverlay(String panelId) {
    return Consumer(
      builder: (context, ref, child) {
        final state = ref.watch(ideProvider);
        final notifier = ref.read(ideProvider.notifier);
        final isDark = ref.watch(themeProvider) == ThemeMode.dark;
        return FloatingPanelPopup(
          panelId: panelId,
          isDark: isDark,
          onClose: () {
            notifier.closeFloatingPanel();
            _removePanelOverlay();
          },
          child: PanelContent(
            panelId: panelId,
            state: state,
            notifier: notifier,
            isDark: isDark,
            inputController: widget.inputController,
            onScrollToLine: widget.onScrollToLine,
            onUpdateSource: widget.onUpdateSource,
          ),
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) => const SizedBox.shrink();
}
