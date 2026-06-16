import 'dart:math' as math;
import 'package:flutter/material.dart';
import '../models/panel_item.dart';
import 'panel_drag_data.dart';

/// 可拖动发光悬浮球组件
///
/// 特性：
/// - 发光球体样式（多层渐变+阴影光晕）
/// - 支持拖动，松手后贴边吸附（带动画）
/// - 点击展开/收起垂直菜单
/// - 菜单展开方向根据悬浮球在屏幕的上下位置自动决定（向上/向下）
/// - 点击菜单项触发 [onSelectPanel] 回调
class FloatingOrbWidget extends StatefulWidget {
  final bool isMenuOpen;
  final List<String> menuItems;
  final VoidCallback onToggleMenu;
  final ValueChanged<String> onSelectPanel;
  final VoidCallback onCloseMenu;
  final ValueChanged<PanelDragData>? onDragAccept;
  final void Function(PanelDragData data, int targetIndex)? onSwapWithFloatingItem;

  const FloatingOrbWidget({
    super.key,
    required this.isMenuOpen,
    required this.menuItems,
    required this.onToggleMenu,
    required this.onSelectPanel,
    required this.onCloseMenu,
    this.onDragAccept,
    this.onSwapWithFloatingItem,
  });

  @override
  State<FloatingOrbWidget> createState() => _FloatingOrbWidgetState();
}

class _FloatingOrbWidgetState extends State<FloatingOrbWidget>
    with TickerProviderStateMixin {
  late final AnimationController _snapController;
  late final AnimationController _menuController;
  late final AnimationController _breathController;

  /// 球体直径
  static const double _orbSize = 64;

  /// 菜单项高度（估算，用于边界计算）
  static const double _menuItemHeight = 44;

  /// 当前球体左上角坐标
  Offset _pos = Offset.zero;

  /// 吸附动画中的目标坐标
  Offset _snapTarget = Offset.zero;

  /// 吸附动画起始坐标（缓存，避免多 listener 竞争）
  Offset _snapBegin = Offset.zero;

  /// 指针追踪：是否按下
  bool _isPointerDown = false;

  /// 指针追踪：是否已判定为拖动
  bool _hasDragged = false;

  /// 指针追踪：按下时的全局位置
  Offset _pointerDownPos = Offset.zero;

  /// 指针追踪：按下时间戳
  DateTime? _pointerDownTime;

  /// 是否已初始化位置
  bool _initialized = false;

  @override
  void initState() {
    super.initState();
    _snapController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 350),
    );
    _snapController.addListener(_onSnapTick);

    _menuController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 250),
    );
    _breathController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 3500),
    )..repeat();
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    // 在 didChangeDependencies 中初始化位置（context 已完全挂载，MediaQuery 数据可靠）
    if (!_initialized) {
      final size = MediaQuery.of(context).size;
      // 移动端首次构建时 size 可能尚未就绪，需校验有效性
      if (size.width > 0 && size.height > 0) {
        _pos = Offset(size.width - _orbSize - 16, size.height * 0.62);
        _initialized = true;
      }
    }
  }

  @override
  void didUpdateWidget(covariant FloatingOrbWidget oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.isMenuOpen) {
      _menuController.forward();
    } else {
      _menuController.reverse();
    }
  }

  @override
  void dispose() {
    _snapController.removeListener(_onSnapTick);
    _snapController.dispose();
    _menuController.dispose();
    _breathController.dispose();
    super.dispose();
  }

  // ========== 指针事件（绕过手势竞技场，不会被其他 GestureDetector 打断） ==========

  void _onPointerDown(PointerDownEvent event) {
    _isPointerDown = true;
    _hasDragged = false;
    _pointerDownPos = event.position;
    _pointerDownTime = DateTime.now();
  }

  void _onPointerMove(PointerMoveEvent event) {
    if (!_isPointerDown) return;

    // 使用局部坐标判定拖动（相对于 Listener 的位移）
    final moveDist = (event.position - _pointerDownPos).distance;
    if (!_hasDragged && moveDist > 8.0) {
      _hasDragged = true;
      if (widget.isMenuOpen) widget.onCloseMenu();
    }

    // 使用 delta（全局位移）更新位置，不受 Listener 坐标系影响
    if (_hasDragged) {
      debugPrint('[OrbDrag] delta=${event.delta}, pos before=$_pos');
      setState(() {
        _pos += event.delta;
      });
      debugPrint('[OrbDrag] pos after=$_pos');
    }
  }

  void _onPointerUp(PointerUpEvent event) {
    if (!_isPointerDown) return;

    if (!_hasDragged) {
      // 未触发拖动：根据按下时间判定是点击还是长按
      final duration = _pointerDownTime != null
          ? DateTime.now().difference(_pointerDownTime!)
          : Duration.zero;
      if (duration < const Duration(milliseconds: 250)) {
        widget.onToggleMenu();
      }
    } else {
      // 拖动结束，贴边吸附
      _snapToEdge();
    }

    _isPointerDown = false;
    _hasDragged = false;
  }

  void _onPointerCancel(PointerCancelEvent event) {
    if (_hasDragged) {
      _snapToEdge();
    }
    _isPointerDown = false;
    _hasDragged = false;
  }

  /// 吸附动画 tick（单一 listener，避免多 listener 竞争导致位置跳变）
  void _onSnapTick() {
    final t = Curves.easeOutBack.transform(_snapController.value);
    final newPos = Offset.lerp(_snapBegin, _snapTarget, t)!;
    // 动画结束后 clamp 到屏幕内，防止停留在 overshoot 位置
    final size = MediaQuery.of(context).size;
    final clamped = Offset(
      newPos.dx.clamp(0.0, size.width - _orbSize),
      newPos.dy.clamp(0.0, size.height - _orbSize),
    );
    setState(() => _pos = clamped);
  }

  /// 贴边吸附动画
  void _snapToEdge() {
    final size = MediaQuery.of(context).size;
    final safeTop = MediaQuery.of(context).padding.top + 80;
    final safeBottom = size.height - MediaQuery.of(context).padding.bottom - 180;

    final leftDist = _pos.dx.abs();
    final rightDist = (size.width - _pos.dx - _orbSize).abs();
    final targetX = leftDist < rightDist ? 8.0 : size.width - _orbSize - 8.0;
    final targetY = _pos.dy.clamp(safeTop, safeBottom.clamp(safeTop, double.infinity));

    _snapBegin = _pos;
    _snapTarget = Offset(targetX, targetY);
    debugPrint('[OrbSnap] start _pos=$_pos, targetX=$targetX, targetY=$targetY, leftDist=$leftDist, rightDist=$rightDist');
    _snapController.forward(from: 0);
  }

  // ========== 菜单方向 ==========

  /// 根据菜单项文字长度动态计算菜单宽度
  double _calcMenuWidth(List<PanelItem> items) {
    const textStyle = TextStyle(
      color: Color(0xFFE0E0F0),
      fontSize: 13,
      fontWeight: FontWeight.w500,
    );
    double maxTextWidth = 0;
    for (final item in items) {
      final tp = TextPainter(
        text: TextSpan(text: item.label, style: textStyle),
        textDirection: TextDirection.ltr,
      );
      tp.layout();
      if (tp.width > maxTextWidth) maxTextWidth = tp.width;
    }
    // icon(16) + SizedBox(10) + 左右padding(14*2) + 余量(8)
    return 16 + 10 + maxTextWidth + 28 + 8;
  }

  bool get _menuGoesUp {
    final items = widget.menuItems
        .map((id) => PanelItem.fromId(id))
        .whereType<PanelItem>()
        .toList();
    final menuHeight = items.length * _menuItemHeight + 16;
    // 优先向上展开，上方空间足够（菜单高度 + 间距8 + 安全余量20）才向上
    return _pos.dy >= menuHeight + 28;
  }

  // ========== 构建 ==========

  @override
  Widget build(BuildContext context) {
    final size = MediaQuery.of(context).size;

    // 保护：只在未初始化时修正 _pos（吸附动画 easeOutBack overshoot 时 _pos 可能短暂为负，
    // 如果此时重置会导致视觉跳变，故不再检查 _pos.dx/dy < 0）
    if (!_initialized) {
      // 移动端首次构建时 size 可能尚未就绪，需校验有效性
      if (size.width > 0 && size.height > 0) {
        _pos = Offset(size.width - _orbSize - 16, size.height * 0.62);
        _initialized = true;
      }
    }

    // 计算菜单项列表
    final items = widget.menuItems
        .map((id) => PanelItem.fromId(id))
        .whereType<PanelItem>()
        .toList();
    final menuHeight = items.length * _menuItemHeight + 16;

    // 菜单在水平方向尽量居中于球体，但不出界
    final menuWidth = _calcMenuWidth(items);
    double menuLeft = _pos.dx + (_orbSize - menuWidth) / 2;
    final maxMenuLeft = (size.width - menuWidth - 8.0).clamp(8.0, double.infinity);
    menuLeft = menuLeft.clamp(8.0, maxMenuLeft);

    // 菜单方向判断：优先向上展开，空间不够才向下
    final menuGoesUp = _menuGoesUp;

    // 菜单垂直位置
    double? menuTop;
    double? menuBottom;
    if (menuGoesUp) {
      menuBottom = size.height - _pos.dy + 8;
    } else {
      menuTop = _pos.dy + _orbSize + 8;
    }

    // 边界保护：向下展开时如果下方空间也不够（极端情况），强制向上
    if (!menuGoesUp && _pos.dy + _orbSize + menuHeight + 20 > size.height) {
      menuTop = null;
      menuBottom = size.height - _pos.dy + 8;
    }

    // 用 RepaintBoundary 隔离发光球体动画，避免其重绘扩散到整个 IDE。
    return RepaintBoundary(
      child: Stack(
        clipBehavior: Clip.none,
        children: [
          // 菜单（条件构建：关闭时不渲染，避免退出动画叠加闪烁）
        if (widget.isMenuOpen) ...[
          // 点击遮罩（translucent 让拖拽事件穿透到底层 DragTarget）
          Positioned.fill(
            child: GestureDetector(
              behavior: HitTestBehavior.translucent,
              onTap: widget.onCloseMenu,
              child: const SizedBox.expand(),
            ),
          ),
          // 菜单面板
          AnimatedPositioned(
            duration: const Duration(milliseconds: 250),
            curve: Curves.easeOutBack,
            left: menuLeft,
            top: menuTop,
            bottom: menuBottom,
            child: DragTarget<PanelDragData>(
              onWillAcceptWithDetails: (details) {
                // 仅接受来自底部 Tab 的跨区域拖拽（padding 区域兜底）
                final accept = details.data.fromLocation == PanelLocation.bottom;
                debugPrint('[MenuDragTarget] onWillAccept: ${details.data.panelId}, from=${details.data.fromLocation}, accept=$accept');
                return accept;
              },
              onAcceptWithDetails: (details) {
                debugPrint('[MenuDragTarget] onAccept: ${details.data.panelId}');
                widget.onDragAccept?.call(details.data);
              },
              onLeave: (data) {
                debugPrint('[MenuDragTarget] onLeave: ${data?.panelId}');
              },
              builder: (context, candidateData, rejectedData) {
                final isHovering = candidateData.isNotEmpty;
                if (isHovering) debugPrint('[MenuDragTarget] isHovering=true, count=${candidateData.length}');
                return _buildMenu(items, isHovering: isHovering);
              },
            ),
          ),
        ],

        // 发光球体（Listener 绕过手势竞技场，不会被编辑器/ScrollView 打断）
        Positioned(
          left: _pos.dx - 16,
          top: _pos.dy - 16,
          child: DragTarget<PanelDragData>(
            onWillAcceptWithDetails: (details) {
              // 仅接受来自底部 Tab 的跨区域拖拽
              final accept = details.data.fromLocation == PanelLocation.bottom;
              debugPrint('[OrbDragTarget] onWillAccept: ${details.data.panelId}, accept=$accept');
              return accept;
            },
            onAcceptWithDetails: (details) {
              debugPrint('[OrbDragTarget] onAccept: ${details.data.panelId}');
              widget.onDragAccept?.call(details.data);
            },
            onLeave: (data) {
              debugPrint('[OrbDragTarget] onLeave: ${data?.panelId}');
            },
            builder: (context, candidateData, rejectedData) {
              final isHovering = candidateData.isNotEmpty;
              if (isHovering) debugPrint('[OrbDragTarget] isHovering=true');
              return SizedBox(
                width: 96,
                height: 96,
                child: Center(
                  child: Container(
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      border: isHovering ? Border.all(color: Colors.blueAccent, width: 3) : null,
                    ),
                    child: Listener(
                      onPointerDown: _onPointerDown,
                      onPointerMove: _onPointerMove,
                      onPointerUp: _onPointerUp,
                      onPointerCancel: _onPointerCancel,
                      behavior: HitTestBehavior.translucent,
                      child: _buildOrb(),
                    ),
                  ),
                ),
              );
            },
          ),
        ),
        ],
      ),
    );
  }

  /// 构建呼吸水泡球体
  Widget _buildOrb() {
    return AnimatedBuilder(
      animation: _breathController,
      builder: (context, child) {
        return CustomPaint(
          size: const Size(_orbSize, _orbSize),
          painter: _BreathingOrbPainter(_breathController.value),
        );
      },
    );
  }

  /// 构建菜单面板
  Widget _buildMenu(List<PanelItem> items, {bool isHovering = false}) {
    return Material(
      color: Colors.transparent,
      child: Container(
        padding: const EdgeInsets.symmetric(vertical: 8),
        decoration: BoxDecoration(
          color: isHovering
              ? const Color(0xFF3D4D68).withValues(alpha: 0.98)
              : const Color(0xFF2D2D48).withValues(alpha: 0.96),
          borderRadius: BorderRadius.circular(14),
          border: Border.all(
            color: isHovering
                ? Colors.blueAccent.withValues(alpha: 0.8)
                : const Color(0xFF88AAFF).withValues(alpha: 0.15),
            width: isHovering ? 2 : 1,
          ),
          boxShadow: isHovering
              ? [BoxShadow(color: Colors.blueAccent.withValues(alpha: 0.3), blurRadius: 12, spreadRadius: 3)]
              : [
                  BoxShadow(
                    color: Colors.black.withValues(alpha: 0.35),
                    blurRadius: 16,
                    spreadRadius: 2,
                    offset: const Offset(0, 4),
                  ),
                ],
        ),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: items.asMap().entries.map((entry) {
            final index = entry.key;
            final item = entry.value;
            return _buildMenuItem(item, index, items.length);
          }).toList(),
        ),
      ),
    );
  }

  Widget _buildMenuItem(PanelItem item, int index, int total) {
    final menuItem = AnimatedBuilder(
      animation: _menuController,
      builder: (context, child) {
        final stagger = index * 0.05;
        final value = (_menuController.value - stagger).clamp(0.0, 1.0) / (1 - stagger);
        final slide = Curves.easeOut.transform(value);
        final direction = _menuGoesUp ? 1.0 : -1.0;
        return Transform.translate(
          offset: Offset(0, (1 - slide) * 20 * direction),
          child: Opacity(
            opacity: slide,
            child: child,
          ),
        );
      },
      child: InkWell(
        onTap: () => widget.onSelectPanel(item.id),
        borderRadius: BorderRadius.circular(8),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
          child: Row(
            children: [
              Icon(item.icon, size: 16, color: const Color(0xFFAAAADD)),
              const SizedBox(width: 10),
              Text(
                item.label,
                style: const TextStyle(
                  color: Color(0xFFE0E0F0),
                  fontSize: 13,
                  fontWeight: FontWeight.w500,
                ),
              ),
            ],
          ),
        ),
      ),
    );

    return Draggable<PanelDragData>(
      data: PanelDragData(
        panelId: item.id,
        fromLocation: PanelLocation.floating,
        fromIndex: index,
      ),
      dragAnchorStrategy: (draggable, context, position) {
        final renderBox = context.findRenderObject() as RenderBox?;
        if (renderBox != null) {
          final size = renderBox.size;
          debugPrint('[FloatingDrag] size=$size, offset=${Offset(size.width / 2, size.height / 2)}, position=$position');
          return Offset(size.width / 2, size.height / 2);
        }
        debugPrint('[FloatingDrag] renderBox is null');
        return Offset.zero;
      },
      feedback: Material(
        color: Colors.transparent,
        elevation: 8,
        borderRadius: BorderRadius.circular(8),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
          decoration: BoxDecoration(
            color: const Color(0xFF2D2D48).withValues(alpha: 0.95),
            borderRadius: BorderRadius.circular(8),
            border: Border.all(color: Colors.blueAccent.withValues(alpha: 0.6), width: 1),
            boxShadow: [
              BoxShadow(color: Colors.black.withValues(alpha: 0.5), blurRadius: 12, spreadRadius: 2),
            ],
          ),
          child: Row(
            children: [
              Icon(item.icon, size: 16, color: const Color(0xFFAAAADD)),
              const SizedBox(width: 10),
              Text(
                item.label,
                style: const TextStyle(
                  color: Color(0xFFE0E0F0),
                  fontSize: 13,
                  fontWeight: FontWeight.w500,
                ),
              ),
            ],
          ),
        ),
      ),
      childWhenDragging: Opacity(opacity: 0.5, child: menuItem),
      child: DragTarget<PanelDragData>(
        onWillAcceptWithDetails: (details) {
          // 仅接受来自底部 Tab 的跨区域拖拽
          final accept = details.data.fromLocation == PanelLocation.bottom;
          debugPrint('[MenuItemDragTarget#$index] onWillAccept: ${details.data.panelId}, accept=$accept');
          return accept;
        },
        onAcceptWithDetails: (details) {
          debugPrint('[MenuItemDragTarget#$index] onAccept: ${details.data.panelId}');
          widget.onSwapWithFloatingItem?.call(details.data, index);
        },
        onLeave: (data) {
          debugPrint('[MenuItemDragTarget#$index] onLeave: ${data?.panelId}');
        },
        builder: (context, candidateData, rejectedData) {
          final isHovering = candidateData.isNotEmpty;
          if (isHovering) debugPrint('[MenuItemDragTarget#$index] isHovering=true');
          return Container(
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(8),
              border: isHovering
                  ? Border.all(color: Colors.blueAccent.withValues(alpha: 0.6), width: 1.5)
                  : null,
              boxShadow: isHovering
                  ? [
                      BoxShadow(
                        color: Colors.blueAccent.withValues(alpha: 0.2),
                        blurRadius: 6,
                        spreadRadius: 1,
                      ),
                    ]
                  : null,
            ),
            child: menuItem,
          );
        },
      ),
    );
  }
}

// ========== 呼吸水泡绘制器（顶层类）==========

class _BreathingOrbPainter extends CustomPainter {
  final double progress;

  _BreathingOrbPainter(this.progress);

  @override
  void paint(Canvas canvas, Size size) {
    // TODO(#D09): 动画帧内重复创建 RadialGradient/MaskFilter，应提升为类级缓存或 const。
    final center = Offset(size.width / 2, size.height / 2);
    final baseR = size.width / 2;

    final breath = 0.92 + math.sin(progress * math.pi * 2) * 0.08;
    final r = baseR * breath;
    final t = progress * math.pi * 2;

    // ========== 1. 外层弥散光晕（bloom） ==========
    final bloom1 = Paint()
      ..color = const Color(0xFF8877FF).withValues(alpha: 0.12)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 65);
    canvas.drawCircle(center, r * 3.2, bloom1);

    final bloom2 = Paint()
      ..color = const Color(0xFFAA99FF).withValues(alpha: 0.18)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 42);
    canvas.drawCircle(center, r * 2.3, bloom2);

    final bloom3 = Paint()
      ..color = const Color(0xFFBBAAFF).withValues(alpha: 0.26)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 26);
    canvas.drawCircle(center, r * 1.55, bloom3);

    // ========== 2. 球体主体（冷暖交织自发光） ==========
    // 底层：淡紫发光基底
    final bodyBase = Paint()
      ..shader = RadialGradient(
        colors: [
          const Color(0xFFF2ECFF).withValues(alpha: 0.55),
          const Color(0xFFD0B8FF).withValues(alpha: 0.50),
          const Color(0xFFAA88EE).withValues(alpha: 0.42),
          const Color(0xFF8866DD).withValues(alpha: 0.35),
          const Color(0xFF6644BB).withValues(alpha: 0.25),
        ],
        stops: const [0.0, 0.22, 0.50, 0.78, 1.0],
      ).createShader(Rect.fromCircle(center: center, radius: r));
    canvas.drawCircle(center, r, bodyBase);

    // 暖色上层叠加（左上暖，增加立体感）
    final warmLayer = Paint()
      ..shader = RadialGradient(
        colors: [
          const Color(0xFFFFF0DD).withValues(alpha: 0.35),
          const Color(0xFFFFE0CC).withValues(alpha: 0.15),
          Colors.transparent,
        ],
        stops: const [0.0, 0.45, 1.0],
      ).createShader(Rect.fromCircle(
        center: Offset(center.dx - r * 0.12, center.dy - r * 0.18),
        radius: r * 0.65,
      ));
    canvas.drawCircle(
      Offset(center.dx - r * 0.12, center.dy - r * 0.18),
      r * 0.65,
      warmLayer,
    );

    // 冷色下层叠加（右下冷，增加立体感）
    final coolLayer = Paint()
      ..shader = RadialGradient(
        colors: [
          const Color(0xFFCCDDFF).withValues(alpha: 0.30),
          const Color(0xFF99BBFF).withValues(alpha: 0.12),
          Colors.transparent,
        ],
        stops: const [0.0, 0.50, 1.0],
      ).createShader(Rect.fromCircle(
        center: Offset(center.dx + r * 0.15, center.dy + r * 0.20),
        radius: r * 0.70,
      ));
    canvas.drawCircle(
      Offset(center.dx + r * 0.15, center.dy + r * 0.20),
      r * 0.70,
      coolLayer,
    );

    // ========== 3. 内部流动光斑（液体感） ==========
    // 主暖光（大光斑，带变形拖尾）
    _drawFlowOval(
      canvas, center, r, t * 0.55, 0.28, 0.24,
      const Color(0xFFFFF8EE), 0.58, 24,
      baseW: 0.55, baseH: 0.26,
      deformSpeed: 1.3, deformAmount: 0.15,
    );
    // 拖尾
    _drawFlowOval(
      canvas, center, r, t * 0.55 - 0.4, 0.28, 0.24,
      const Color(0xFFFFF0DD), 0.22, 30,
      baseW: 0.75, baseH: 0.18,
      deformSpeed: 1.3, deformAmount: 0.10,
    );

    // 蓝色冷光（带变形）
    _drawFlowOval(
      canvas, center, r, t * 0.42 + 2.8, 0.34, 0.26,
      const Color(0xFFE8F5FF), 0.52, 20,
      baseW: 0.48, baseH: 0.22,
      deformSpeed: 1.1, deformAmount: 0.12,
    );

    // 淡粉微光
    _drawFlowOval(
      canvas, center, r, t * 0.70 + 4.8, 0.18, 0.14,
      const Color(0xFFFFF0F8), 0.45, 16,
      baseW: 0.32, baseH: 0.14,
      deformSpeed: 1.5, deformAmount: 0.18,
    );

    // 流动小光斑（带轨迹摆动）
    _drawFlowCircle(
      canvas, center, r, t * 0.75, 0.72,
      const Color(0xFFD5E8FF), 0.40, 12, 0.13,
      wobbleAmp: 0.08, wobbleFreq: 2.5,
    );
    _drawFlowCircle(
      canvas, center, r, t * 0.95 + 2.8, 0.68,
      const Color(0xFFFFF8E0), 0.38, 10, 0.11,
      wobbleAmp: 0.06, wobbleFreq: 3.0,
    );

    // 微小气泡（快速闪烁）
    _drawFlowCircle(
      canvas, center, r, t * 1.4 + 0.5, 0.55,
      const Color(0xFFFFFFFF), 0.30, 5, 0.045,
      wobbleAmp: 0.12, wobbleFreq: 4.0,
    );
    _drawFlowCircle(
      canvas, center, r, t * 1.6 + 3.2, 0.48,
      const Color(0xFFFFE8CC), 0.25, 4, 0.038,
      wobbleAmp: 0.10, wobbleFreq: 3.5,
    );
    _drawFlowCircle(
      canvas, center, r, t * 1.3 + 5.5, 0.42,
      const Color(0xFFCCE8FF), 0.22, 4, 0.032,
      wobbleAmp: 0.14, wobbleFreq: 4.5,
    );
    _drawFlowCircle(
      canvas, center, r, t * 1.7 + 1.8, 0.60,
      const Color(0xFFD5FFEE), 0.20, 3, 0.028,
      wobbleAmp: 0.09, wobbleFreq: 5.0,
    );

    // 中心亮芯（轻微脉动）
    final corePulse = 0.90 + math.sin(t * 2.5) * 0.10;
    final core = Paint()
      ..color = const Color(0xFFFFFFFF).withValues(alpha: 0.42)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 14);
    canvas.drawCircle(center, r * 0.18 * corePulse, core);

    // ========== 4. 顶部弥散高光（代替白点） ==========
    // 大的柔和高光区
    final topGlow = Paint()
      ..shader = RadialGradient(
        colors: [
          Colors.white.withValues(alpha: 0.45),
          Colors.white.withValues(alpha: 0.15),
          Colors.transparent,
        ],
        stops: const [0.0, 0.45, 1.0],
      ).createShader(Rect.fromCircle(
        center: Offset(center.dx - r * 0.08, center.dy - r * 0.22),
        radius: r * 0.32,
      ));
    canvas.drawCircle(
      Offset(center.dx - r * 0.08, center.dy - r * 0.22),
      r * 0.32,
      topGlow,
    );

    // 小亮点（更自然）
    final topDot = Paint()
      ..color = Colors.white.withValues(alpha: 0.60)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4);
    canvas.drawCircle(
      Offset(center.dx - r * 0.04, center.dy - r * 0.28),
      r * 0.06,
      topDot,
    );

    // ========== 5. 底部暖反光 ==========
    final botGlow = Paint()
      ..shader = RadialGradient(
        colors: [
          const Color(0xFFFFEEDD).withValues(alpha: 0.25),
          const Color(0xFFFFDDBB).withValues(alpha: 0.08),
          Colors.transparent,
        ],
        stops: const [0.0, 0.5, 1.0],
      ).createShader(Rect.fromCircle(
        center: Offset(center.dx + r * 0.12, center.dy + r * 0.46),
        radius: r * 0.38,
      ));
    canvas.drawCircle(
      Offset(center.dx + r * 0.12, center.dy + r * 0.46),
      r * 0.38,
      botGlow,
    );
  }

  /// 绘制流动椭圆光斑（带变形）
  void _drawFlowOval(
    Canvas canvas, Offset center, double r, double angle,
    double distX, double distY, Color color, double alpha, double blur, {
    required double baseW,
    required double baseH,
    double deformSpeed = 1.0,
    double deformAmount = 0.1,
  }) {
    // 位置：基础圆周 + 微小摆动
    final wobble = math.sin(angle * 3.0) * 0.03;
    final pos = Offset(
      center.dx + math.cos(angle + wobble) * r * distX,
      center.dy + math.sin(angle + wobble) * r * distY,
    );
    // 变形：长短轴随时间交替伸缩
    final deform = math.sin(angle * deformSpeed) * deformAmount;
    final w = r * baseW * (1.0 + deform);
    final h = r * baseH * (1.0 - deform);
    canvas.save();
    canvas.translate(pos.dx, pos.dy);
    canvas.rotate(angle * 0.2);
    final paint = Paint()
      ..color = color.withValues(alpha: alpha)
      ..maskFilter = MaskFilter.blur(BlurStyle.normal, blur);
    canvas.drawOval(
      Rect.fromCenter(center: Offset.zero, width: w, height: h),
      paint,
    );
    canvas.restore();
  }

  /// 绘制流动圆形光斑（带轨迹摆动）
  void _drawFlowCircle(
    Canvas canvas, Offset center, double r, double angle,
    double dist, Color color, double alpha, double blur, double radius, {
    double wobbleAmp = 0.0,
    double wobbleFreq = 1.0,
  }) {
    // 轨迹摆动：半径轻微变化
    final wobble = math.sin(angle * wobbleFreq) * wobbleAmp;
    final d = dist + wobble;
    final pos = Offset(
      center.dx + math.cos(angle) * r * d,
      center.dy + math.sin(angle) * r * d,
    );
    final paint = Paint()
      ..color = color.withValues(alpha: alpha)
      ..maskFilter = MaskFilter.blur(BlurStyle.normal, blur);
    canvas.drawCircle(pos, r * radius, paint);
  }

  @override
  bool shouldRepaint(covariant _BreathingOrbPainter old) {
    return old.progress != progress;
  }
}
