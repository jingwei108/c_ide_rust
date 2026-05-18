import 'package:flutter/material.dart';
import '../models/code_template.dart';

/// 模板交互式教程面板
///
/// 固定在 IDE 底部（编辑器与底部面板之间），引导学生逐行理解代码。
class TemplateTutorialPanel extends StatelessWidget {
  final String templateName;
  final int currentStep;
  final int totalSteps;
  final TutorialStep step;
  final bool isDark;
  final VoidCallback onNext;
  final VoidCallback onPrev;
  final VoidCallback onSkip;
  final VoidCallback onRun;

  const TemplateTutorialPanel({
    super.key,
    required this.templateName,
    required this.currentStep,
    required this.totalSteps,
    required this.step,
    required this.isDark,
    required this.onNext,
    required this.onPrev,
    required this.onSkip,
    required this.onRun,
  });

  @override
  Widget build(BuildContext context) {
    final bgColor = isDark ? const Color(0xFF1E1E1E) : const Color(0xFFFFFFFF);
    final textColor = isDark ? const Color(0xFFD4D4D4) : const Color(0xFF333333);
    final secondaryColor = textColor.withValues(alpha: 0.6);
    final isLastStep = currentStep >= totalSteps - 1;

    return Container(
      decoration: BoxDecoration(
        color: bgColor,
        border: Border(
          top: BorderSide(
            color: isDark ? const Color(0xFF3E4451) : const Color(0xFFE5E5E5),
          ),
        ),
      ),
      child: SafeArea(
        top: false,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // 顶部标题栏 + 步骤指示器
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 12, 16, 0),
              child: Row(
                children: [
                  Container(
                    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                    decoration: BoxDecoration(
                      color: Colors.blueAccent.withValues(alpha: 0.15),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: const Text(
                      '教程模式',
                      style: TextStyle(
                        fontSize: 11,
                        color: Colors.blueAccent,
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      '${currentStep + 1} / $totalSteps  ${step.title}',
                      style: TextStyle(
                        fontSize: 14,
                        fontWeight: FontWeight.w600,
                        color: textColor,
                      ),
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                  TextButton(
                    onPressed: onSkip,
                    style: TextButton.styleFrom(
                      padding: const EdgeInsets.symmetric(horizontal: 8),
                      minimumSize: Size.zero,
                      tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                    ),
                    child: Text(
                      '跳过',
                      style: TextStyle(
                        fontSize: 12,
                        color: secondaryColor,
                      ),
                    ),
                  ),
                ],
              ),
            ),
            // 步骤圆点指示器
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              child: Row(
                children: List.generate(totalSteps, (index) {
                  final isActive = index == currentStep;
                  final isPast = index < currentStep;
                  return Expanded(
                    child: Container(
                      height: 3,
                      margin: const EdgeInsets.symmetric(horizontal: 2),
                      decoration: BoxDecoration(
                        color: isActive
                            ? Colors.blueAccent
                            : isPast
                                ? Colors.blueAccent.withValues(alpha: 0.4)
                                : isDark
                                    ? const Color(0xFF3E4451)
                                    : const Color(0xFFE5E5E5),
                        borderRadius: BorderRadius.circular(2),
                      ),
                    ),
                  );
                }),
              ),
            ),
            // 步骤描述
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Text(
                step.description,
                style: TextStyle(
                  fontSize: 13,
                  color: secondaryColor,
                  height: 1.5,
                ),
              ),
            ),
            const SizedBox(height: 8),
            // 关键行解释列表
            if (step.explanations.isNotEmpty)
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 12),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: step.explanations.map((exp) {
                    return _LineExplanationTile(
                      explanation: exp,
                      isDark: isDark,
                    );
                  }).toList(),
                ),
              ),
            // 按钮栏
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 8, 16, 12),
              child: Row(
                children: [
                  // 上一步
                  OutlinedButton(
                    onPressed: currentStep > 0 ? onPrev : null,
                    style: OutlinedButton.styleFrom(
                      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                      minimumSize: Size.zero,
                      tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(6),
                      ),
                    ),
                    child: const Text('上一步', style: TextStyle(fontSize: 12)),
                  ),
                  const Spacer(),
                  // 下一步 / 运行
                  ElevatedButton.icon(
                    onPressed: isLastStep ? onRun : onNext,
                    icon: Icon(
                      isLastStep ? Icons.play_arrow : Icons.arrow_forward,
                      size: 16,
                    ),
                    label: Text(
                      isLastStep ? '运行代码' : '下一步',
                      style: const TextStyle(fontSize: 12),
                    ),
                    style: ElevatedButton.styleFrom(
                      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 8),
                      backgroundColor: isLastStep ? Colors.green : Colors.blueAccent,
                      foregroundColor: Colors.white,
                      minimumSize: Size.zero,
                      tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(6),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// 单行代码解释展开项
class _LineExplanationTile extends StatefulWidget {
  final LineExplanation explanation;
  final bool isDark;

  const _LineExplanationTile({
    required this.explanation,
    required this.isDark,
  });

  @override
  State<_LineExplanationTile> createState() => _LineExplanationTileState();
}

class _LineExplanationTileState extends State<_LineExplanationTile> {
  bool _expanded = false;

  @override
  Widget build(BuildContext context) {
    final textColor = widget.isDark ? const Color(0xFFD4D4D4) : const Color(0xFF333333);

    return InkWell(
      onTap: () => setState(() => _expanded = !_expanded),
      borderRadius: BorderRadius.circular(6),
      child: Container(
        margin: const EdgeInsets.symmetric(vertical: 2),
        padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
        decoration: BoxDecoration(
          color: widget.isDark
              ? const Color(0xFF2A2D3E)
              : const Color(0xFFF5F7FA),
          borderRadius: BorderRadius.circular(6),
          border: Border.all(
            color: widget.isDark
                ? const Color(0xFF3E4451)
                : const Color(0xFFE8ECF0),
          ),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(
                  Icons.lightbulb_outline,
                  size: 14,
                  color: Colors.amber,
                ),
                const SizedBox(width: 6),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 1),
                  decoration: BoxDecoration(
                    color: Colors.blueAccent.withValues(alpha: 0.1),
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(
                    '第 ${widget.explanation.line} 行',
                    style: const TextStyle(
                      fontSize: 11,
                      color: Colors.blueAccent,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    widget.explanation.short,
                    style: TextStyle(
                      fontSize: 12,
                      color: textColor,
                      fontWeight: FontWeight.w500,
                    ),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                Icon(
                  _expanded ? Icons.expand_less : Icons.expand_more,
                  size: 16,
                  color: textColor.withValues(alpha: 0.5),
                ),
              ],
            ),
            if (_expanded) ...[
              const SizedBox(height: 6),
              Padding(
                padding: const EdgeInsets.only(left: 20),
                child: Text(
                  widget.explanation.detail,
                  style: TextStyle(
                    fontSize: 12,
                    color: textColor.withValues(alpha: 0.75),
                    height: 1.5,
                  ),
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
