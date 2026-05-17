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
        return _buildVarTile(v, accessType, prevValue, isDark);
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

  Widget _buildVarTile(
    dynamic v, // ApiVariableSnapshot
    String? accessType,
    String? prevValue,
    bool isDark,
  ) {
    final textColor = isDark ? const Color(0xffd4d4d4) : const Color(0xff383a42);
    final subTextColor = isDark ? const Color(0xff5c6370) : const Color(0xffa0a1a7);

    Color? borderColor;
    String? badgeText;
    if (accessType == "Read") {
      borderColor = Colors.blueAccent;
      badgeText = "读";
    } else if (accessType == "Write") {
      borderColor = Colors.orange;
      badgeText = "写";
    }

    // 值变化检测
    String? changeIndicator;
    Color? changeColor;
    if (prevValue != null && prevValue != v.value) {
      final prevNum = double.tryParse(prevValue);
      final currNum = double.tryParse(v.value);
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

    return Container(
      margin: const EdgeInsets.only(bottom: 6),
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: isDark ? const Color(0xff1e1e1e) : const Color(0xfff5f5f5),
        borderRadius: BorderRadius.circular(6),
        border: borderColor != null
            ? Border.all(color: borderColor.withValues(alpha: 0.5), width: 1.5)
            : Border.all(
                color: isDark ? const Color(0xff3e4451) : const Color(0xffe5e5e5),
              ),
      ),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Text(
                      v.name,
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
                        color: isDark ? const Color(0xff2a2a2a) : const Color(0xffe5e5e5),
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Text(
                        v.tyName,
                        style: TextStyle(
                          fontSize: 10,
                          color: isDark ? Colors.grey : Colors.black54,
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
                  '值: ${v.value}  地址: 0x${v.addr.toRadixString(16).toUpperCase().padLeft(4, '0')}',
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
