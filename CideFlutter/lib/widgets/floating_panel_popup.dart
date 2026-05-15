import 'package:flutter/material.dart';
import '../models/panel_item.dart';

/// 带动画的面板弹窗
///
/// 从屏幕中央缩放淡入，点击遮罩或关闭按钮可关闭。
class FloatingPanelPopup extends StatefulWidget {
  final String panelId;
  final Widget child;
  final VoidCallback onClose;
  final bool isDark;

  const FloatingPanelPopup({
    super.key,
    required this.panelId,
    required this.child,
    required this.onClose,
    required this.isDark,
  });

  @override
  State<FloatingPanelPopup> createState() => _FloatingPanelPopupState();
}

class _FloatingPanelPopupState extends State<FloatingPanelPopup>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller;
  late final Animation<double> _scaleAnim;
  late final Animation<double> _opacityAnim;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 280),
    );

    _scaleAnim = CurvedAnimation(
      parent: _controller,
      curve: Curves.easeOutBack,
    );

    _opacityAnim = Tween<double>(begin: 0.0, end: 1.0).animate(
      CurvedAnimation(parent: _controller, curve: const Interval(0.0, 0.6, curve: Curves.easeOut)),
    );

    _controller.forward();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _close() async {
    await _controller.reverse();
    widget.onClose();
  }

  @override
  Widget build(BuildContext context) {
    final panelItem = PanelItem.fromId(widget.panelId);
    final title = panelItem?.label ?? widget.panelId;

    final panelBg = widget.isDark ? const Color(0xFF1E1E2E) : const Color(0xFFFFFFFF);
    final headerBg = widget.isDark ? const Color(0xFF2A2A3E) : const Color(0xFFF0F0F5);
    final borderColor = widget.isDark ? const Color(0xFF3E3E55) : const Color(0xFFE0E0E8);

    return Positioned.fill(
      child: GestureDetector(
        onTap: _close,
        child: AnimatedBuilder(
          animation: _controller,
          builder: (context, child) {
            return Container(
              color: Colors.black.withValues(alpha: 0.5 * _opacityAnim.value),
              child: Center(
                child: Opacity(
                  opacity: _opacityAnim.value,
                  child: Transform.scale(
                    scale: _scaleAnim.value,
                    child: child,
                  ),
                ),
              ),
            );
          },
          child: GestureDetector(
            // 阻止点击内容区域关闭弹窗
            onTap: () {},
            child: Container(
              margin: const EdgeInsets.symmetric(horizontal: 24, vertical: 48),
              constraints: const BoxConstraints(maxWidth: 520, maxHeight: 640),
              decoration: BoxDecoration(
                color: panelBg,
                borderRadius: BorderRadius.circular(16),
                border: Border.all(color: borderColor, width: 1),
                boxShadow: [
                  BoxShadow(
                    color: Colors.black.withValues(alpha: 0.3),
                    blurRadius: 24,
                    spreadRadius: 4,
                  ),
                ],
              ),
              child: ClipRRect(
                borderRadius: BorderRadius.circular(16),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    // 标题栏
                    Container(
                      height: 44,
                      padding: const EdgeInsets.symmetric(horizontal: 16),
                      decoration: BoxDecoration(
                        color: headerBg,
                        border: Border(
                          bottom: BorderSide(color: borderColor, width: 1),
                        ),
                      ),
                      child: Row(
                        children: [
                          if (panelItem != null)
                            Icon(panelItem.icon, size: 16, color: Colors.blueAccent),
                          if (panelItem != null) const SizedBox(width: 8),
                          Text(
                            title,
                            style: TextStyle(
                              fontSize: 14,
                              fontWeight: FontWeight.w600,
                              color: widget.isDark ? const Color(0xFFE0E0F0) : const Color(0xFF333333),
                            ),
                          ),
                          const Spacer(),
                          InkWell(
                            onTap: _close,
                            borderRadius: BorderRadius.circular(12),
                            child: const Padding(
                              padding: EdgeInsets.all(4),
                              child: Icon(Icons.close, size: 18, color: Colors.grey),
                            ),
                          ),
                        ],
                      ),
                    ),
                    // 内容区域
                    Flexible(
                      child: widget.child,
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
