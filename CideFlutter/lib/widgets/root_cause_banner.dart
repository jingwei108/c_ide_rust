import 'package:flutter/material.dart';
import '../src/rust/unified/root_cause.dart' as rust;

/// Displays a root-cause analysis banner when a runtime trap occurs.
///
/// Shown in the [ExecutionControlPanel] between the algorithm-step bar
/// and the trap-error bar. Tapping a related line number requests the
/// IDE to jump to that line.
class RootCauseBanner extends StatelessWidget {
  final rust.RootCauseHint? hint;
  final void Function(int line)? onLineTap;

  const RootCauseBanner({
    super.key,
    required this.hint,
    this.onLineTap,
  });

  @override
  Widget build(BuildContext context) {
    if (hint == null) return const SizedBox.shrink();

    final h = hint!;
    final bgColor = _categoryColor(h.category);
    final icon = _categoryIcon(h.category);

    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      color: bgColor,
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(icon, color: Colors.white, size: 16),
          const SizedBox(width: 8),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  h.oneLiner,
                  style: const TextStyle(
                    color: Colors.white,
                    fontSize: 12,
                    fontWeight: FontWeight.w500,
                  ),
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                ),
                if (h.relatedLines.isNotEmpty) ...[
                  const SizedBox(height: 4),
                  Wrap(
                    spacing: 6,
                    children: h.relatedLines.map((line) {
                      return InkWell(
                        onTap: onLineTap != null ? () => onLineTap!(line) : null,
                        borderRadius: BorderRadius.circular(4),
                        child: Container(
                          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                          decoration: BoxDecoration(
                            color: Colors.white.withValues(alpha: 0.15),
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Text(
                            '第 $line 行',
                            style: const TextStyle(
                              color: Colors.white,
                              fontSize: 10,
                              decoration: TextDecoration.underline,
                              decorationColor: Colors.white70,
                            ),
                          ),
                        ),
                      );
                    }).toList(),
                  ),
                ],
              ],
            ),
          ),
          // Suggested fix chip (if any)
          if (h.suggestedFixKind.isNotEmpty && h.suggestedFixKind != 'None')
            Container(
              margin: const EdgeInsets.only(left: 8),
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
              decoration: BoxDecoration(
                color: Colors.white.withValues(alpha: 0.2),
                borderRadius: BorderRadius.circular(4),
              ),
              child: Text(
                _fixKindLabel(h.suggestedFixKind),
                style: const TextStyle(color: Colors.white, fontSize: 10),
              ),
            ),
        ],
      ),
    );
  }

  Color _categoryColor(String category) {
    switch (category) {
      case 'OffByOne':
        return Colors.amber.shade800;
      case 'UseAfterFree':
      case 'DoubleFree':
        return Colors.deepOrange.shade800;
      case 'NullDeref':
        return Colors.purple.shade700;
      case 'DivZero':
        return Colors.teal.shade700;
      case 'UninitializedIndex':
      case 'InitVariable':
        return Colors.lightBlue.shade700;
      default:
        return Colors.blueGrey.shade700;
    }
  }

  IconData _categoryIcon(String category) {
    switch (category) {
      case 'OffByOne':
        return Icons.exposure_plus_1;
      case 'UseAfterFree':
      case 'DoubleFree':
        return Icons.memory;
      case 'NullDeref':
        return Icons.block;
      case 'DivZero':
        return Icons.calculate;
      case 'UninitializedIndex':
      case 'InitVariable':
        return Icons.help_outline;
      default:
        return Icons.lightbulb;
    }
  }

  String _fixKindLabel(String kind) {
    switch (kind) {
      case 'ChangeLeToLt':
        return '<= → <';
      case 'AddNullCheck':
        return '加 NULL 检查';
      case 'InitVariable':
        return '初始化变量';
      case 'FixLoopStart':
        return '修正循环起点';
      case 'SetNullAfterFree':
        return 'free 后置 NULL';
      case 'AvoidDivZero':
        return '避免除零';
      default:
        return kind;
    }
  }
}
