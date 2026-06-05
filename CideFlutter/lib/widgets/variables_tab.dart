import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../providers/unified_provider.dart';

class VariablesTab extends ConsumerWidget {
  final bool isDark;

  const VariablesTab({super.key, required this.isDark});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final unifiedState = ref.watch(unifiedProvider);
    final frameCache = unifiedState.frameCache;
    final currentStep = unifiedState.currentStep;

    if (frameCache.isEmpty || currentStep < 0 || currentStep >= frameCache.length) {
      return _buildEmpty('运行程序以查看变量');
    }

    final payload = frameCache[currentStep];
    final localVars = payload.localVars;
    final accessedMap = <String, String>{};
    for (final av in payload.accessedVars) {
      accessedMap[av.name] = av.accessType;
    }

    // 上一步变量值，用于检测变化
    final prevValues = <String, String>{};
    if (currentStep > 0) {
      final prevPayload = frameCache[currentStep - 1];
      for (final pv in prevPayload.localVars) {
        prevValues[pv.name] = pv.value;
      }
    }

    if (localVars.isEmpty) {
      return _buildEmpty('当前作用域无变量');
    }

    return ListView.builder(
      padding: const EdgeInsets.all(8),
      itemCount: localVars.length,
      itemBuilder: (context, index) {
        final v = localVars[index];
        final accessType = accessedMap[v.name];
        final prevValue = prevValues[v.name];
        return _VarTile(
          v: v,
          accessType: accessType,
          prevValue: prevValue,
          isDark: isDark,
        );
      },
    );
  }

  Widget _buildEmpty(String message) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(Icons.data_object, size: 40, color: Colors.grey[500]),
          const SizedBox(height: 12),
          Text(message, style: TextStyle(fontSize: 14, color: Colors.grey[500])),
        ],
      ),
    );
  }
}

class _VarTile extends StatefulWidget {
  final dynamic v;
  final String? accessType;
  final String? prevValue;
  final bool isDark;

  const _VarTile({
    required this.v,
    this.accessType,
    this.prevValue,
    required this.isDark,
  });

  @override
  State<_VarTile> createState() => _VarTileState();
}

class _VarTileState extends State<_VarTile> with SingleTickerProviderStateMixin {
  late AnimationController _flashController;
  String? _lastValue;

  @override
  void initState() {
    super.initState();
    _flashController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 300),
    );
    _lastValue = widget.v.value as String?;
    _triggerFlashIfChanged();
  }

  @override
  void didUpdateWidget(covariant _VarTile oldWidget) {
    super.didUpdateWidget(oldWidget);
    final currentValue = widget.v.value as String?;
    if (_lastValue != currentValue) {
      _lastValue = currentValue;
      _triggerFlashIfChanged();
    }
  }

  void _triggerFlashIfChanged() {
    if (widget.prevValue != null && widget.prevValue != widget.v.value) {
      _flashController.forward(from: 0.0);
    }
  }

  @override
  void dispose() {
    _flashController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final textColor = widget.isDark ? const Color(0xffd4d4d4) : const Color(0xff383a42);
    final subTextColor = widget.isDark ? const Color(0xff5c6370) : const Color(0xffa0a1a7);

    Color? borderColor;
    String? badgeText;
    if (widget.accessType == "Read") {
      borderColor = Colors.blueAccent;
      badgeText = "读";
    } else if (widget.accessType == "Write") {
      borderColor = Colors.orange;
      badgeText = "写";
    }

    // 值变化检测
    String? changeIndicator;
    Color? changeColor;
    final prevValue = widget.prevValue;
    final currValue = widget.v.value as String?;
    if (prevValue != null && prevValue != currValue) {
      final prevNum = double.tryParse(prevValue);
      final currNum = double.tryParse(currValue ?? '');
      if (prevNum != null && currNum != null) {
        if (currNum > prevNum) {
          changeIndicator = '↑';
          changeColor = Colors.green;
        } else {
          changeIndicator = '↓';
          changeColor = Colors.red;
        }
      } else {
        changeIndicator = '•';
        changeColor = Colors.amber;
      }
    }

    return AnimatedBuilder(
      animation: _flashController,
      builder: (context, child) {
        final flashOpacity = (1.0 - _flashController.value) * 0.25;
        return Container(
          margin: const EdgeInsets.only(bottom: 6),
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          decoration: BoxDecoration(
            color: Color.lerp(
              widget.isDark ? const Color(0xff1e1e1e) : const Color(0xfff5f5f5),
              Colors.redAccent,
              flashOpacity,
            ),
            borderRadius: BorderRadius.circular(6),
            border: borderColor != null
                ? Border.all(color: borderColor.withValues(alpha: 0.5), width: 1.5)
                : Border.all(
                    color: widget.isDark ? const Color(0xff3e4451) : const Color(0xffe5e5e5),
                  ),
          ),
          child: child,
        );
      },
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Text(
                      widget.v.name as String,
                      style: TextStyle(
                        fontFamily: 'monospace',
                        fontSize: 13,
                        color: textColor,
                        fontWeight: borderColor != null ? FontWeight.w600 : FontWeight.normal,
                      ),
                    ),
                    const SizedBox(width: 8),
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                      decoration: BoxDecoration(
                        color: widget.isDark ? const Color(0xff2a2a2a) : const Color(0xffe5e5e5),
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Text(
                        widget.v.tyName as String,
                        style: TextStyle(
                          fontSize: 10,
                          color: widget.isDark ? Colors.grey : Colors.black54,
                          fontFamily: 'monospace',
                        ),
                      ),
                    ),
                    if (badgeText != null) ...[
                      const SizedBox(width: 6),
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                        decoration: BoxDecoration(
                          color: borderColor?.withValues(alpha: 0.15),
                          borderRadius: BorderRadius.circular(4),
                        ),
                        child: Text(
                          badgeText,
                          style: TextStyle(
                            fontSize: 9,
                            color: borderColor,
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                      ),
                    ],
                    if (changeIndicator != null) ...[
                      const SizedBox(width: 6),
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 1),
                        decoration: BoxDecoration(
                          color: changeColor?.withValues(alpha: 0.15),
                          borderRadius: BorderRadius.circular(4),
                        ),
                        child: Text(
                          changeIndicator,
                          style: TextStyle(
                            fontSize: 10,
                            color: changeColor,
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                      ),
                    ],
                  ],
                ),
                const SizedBox(height: 2),
                Text(
                  '值: ${widget.v.value}  地址: 0x${(widget.v.addr as int).toRadixString(16).toUpperCase().padLeft(4, '0')}',
                  style: TextStyle(
                    fontSize: 11,
                    color: subTextColor,
                    fontFamily: 'monospace',
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
