part of '../custom_keyboard.dart';


// ========== 子组件 ==========

/// 带顶部小数字提示的字母按键
class _LetterKey extends StatelessWidget {
  final String letter;
  final String? number;
  final VoidCallback onTap;
  final Color backgroundColor;
  final Color textColor;

  const _LetterKey({
    required this.letter,
    this.number,
    required this.onTap,
    required this.backgroundColor,
    required this.textColor,
  });

  @override
  Widget build(BuildContext context) {
    return _KeyButton(
      label: letter,
      onTap: onTap,
      backgroundColor: backgroundColor,
      textColor: textColor,
      fontSize: 18,
      topLabel: number,
      topLabelColor: textColor.withValues(alpha: 0.45),
    );
  }
}

/// 单个按键按钮 —— 带按压动画、触觉反馈、长按连发、水平拖拽
class _KeyButton extends StatefulWidget {
  final String label;
  final VoidCallback onTap;
  final GestureDragUpdateCallback? onHorizontalDragUpdate;
  final bool repeatOnLongPress;
  final Color backgroundColor;
  final Color textColor;
  final double fontSize;
  final EdgeInsetsGeometry padding;
  final double? height;
  final String? topLabel;
  final Color? topLabelColor;

  const _KeyButton({
    required this.label,
    required this.onTap,
    this.onHorizontalDragUpdate,
    this.repeatOnLongPress = false,
    required this.backgroundColor,
    required this.textColor,
    this.fontSize = 16,
    this.padding = EdgeInsets.zero,
    this.height = 48,
    this.topLabel,
    this.topLabelColor,
  });

  @override
  State<_KeyButton> createState() => _KeyButtonState();
}

class _KeyButtonState extends State<_KeyButton>
    with SingleTickerProviderStateMixin {
  bool _pressed = false;
  Timer? _longPressTimer;
  Timer? _repeatTimer;
  late final AnimationController _controller;
  late final Animation<double> _scaleAnim;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 80),
    );
    _scaleAnim = Tween<double>(begin: 1.0, end: 0.92).animate(
      CurvedAnimation(parent: _controller, curve: Curves.easeOut),
    );
  }

  @override
  void dispose() {
    _cancelRepeat();
    _controller.dispose();
    super.dispose();
  }

  void _cancelRepeat() {
    _longPressTimer?.cancel();
    _longPressTimer = null;
    _repeatTimer?.cancel();
    _repeatTimer = null;
  }

  void _onTapDown(TapDownDetails details) {
    if (!mounted) return;
    setState(() => _pressed = true);
    _controller.forward();
    HapticFeedback.selectionClick();

    if (widget.repeatOnLongPress) {
      _longPressTimer = Timer(const Duration(milliseconds: 400), () {
        if (!mounted) return;
        widget.onTap();
        _repeatTimer = Timer.periodic(
          const Duration(milliseconds: 80),
          (_) {
            if (mounted) widget.onTap();
          },
        );
      });
    }
  }

  void _onTapUp(TapUpDetails details) {
    if (!mounted) return;
    final wasRepeating = _repeatTimer != null;
    _cancelRepeat();
    setState(() => _pressed = false);
    _controller.reverse();
    if (!wasRepeating) {
      widget.onTap();
    }
  }

  void _onTapCancel() {
    if (!mounted) return;
    _cancelRepeat();
    setState(() => _pressed = false);
    _controller.reverse();
  }

  void _onHorizontalDragStart(DragStartDetails details) {
    _cancelRepeat();
    if (_pressed) {
      setState(() => _pressed = false);
      _controller.reverse();
    }
  }

  void _onHorizontalDragEnd(DragEndDetails details) {
    if (_pressed) {
      setState(() => _pressed = false);
      _controller.reverse();
    }
  }

  Color get _bgColor {
    if (!_pressed) return widget.backgroundColor;
    final hsl = HSLColor.fromColor(widget.backgroundColor);
    return hsl
        .withLightness((hsl.lightness - 0.12).clamp(0.0, 1.0))
        .toColor();
  }

  bool get _isDark => Theme.of(context).brightness == Brightness.dark;

  List<BoxShadow> get _shadows {
    if (_pressed) return const [];
    return [
      BoxShadow(
        color: Colors.black.withValues(alpha: _isDark ? 0.35 : 0.12),
        blurRadius: 2,
        offset: const Offset(0, 1.5),
      ),
    ];
  }

  @override
  Widget build(BuildContext context) {
    final gestures = <Type, GestureRecognizerFactory>{};

    gestures[TapGestureRecognizer] =
        GestureRecognizerFactoryWithHandlers<TapGestureRecognizer>(
      () => TapGestureRecognizer(),
      (instance) {
        instance.onTapDown = _onTapDown;
        instance.onTapUp = _onTapUp;
        instance.onTapCancel = _onTapCancel;
      },
    );

    if (widget.onHorizontalDragUpdate != null) {
      gestures[HorizontalDragGestureRecognizer] =
          GestureRecognizerFactoryWithHandlers<HorizontalDragGestureRecognizer>(
        () => HorizontalDragGestureRecognizer(),
        (instance) {
          instance.onStart = _onHorizontalDragStart;
          instance.onUpdate = widget.onHorizontalDragUpdate;
          instance.onEnd = _onHorizontalDragEnd;
        },
      );
    }

    Widget content;
    if (widget.topLabel != null) {
      content = Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Text(
            widget.topLabel!,
            style: TextStyle(
              fontSize: widget.fontSize * 0.55,
              color: widget.topLabelColor ?? widget.textColor.withValues(alpha: 0.5),
              fontWeight: FontWeight.w500,
              height: 1.0,
            ),
          ),
          Text(
            widget.label,
            style: TextStyle(
              fontSize: widget.fontSize,
              color: widget.textColor,
              fontWeight: FontWeight.w500,
              height: 1.2,
            ),
          ),
        ],
      );
    } else {
      content = Text(
        widget.label,
        style: TextStyle(
          fontSize: widget.fontSize,
          color: widget.textColor,
          fontWeight: FontWeight.w500,
        ),
      );
    }

    return RawGestureDetector(
      gestures: gestures,
      behavior: HitTestBehavior.opaque,
      child: AnimatedBuilder(
        animation: _scaleAnim,
        builder: (context, child) {
          return Transform.scale(
            scale: _scaleAnim.value,
            child: AnimatedContainer(
              duration: const Duration(milliseconds: 60),
              height: widget.height,
              padding: widget.padding,
              decoration: BoxDecoration(
                color: _bgColor,
                borderRadius: BorderRadius.circular(8),
                boxShadow: _shadows,
              ),
              alignment: Alignment.center,
              child: content,
            ),
          );
        },
      ),
    );
  }
}
