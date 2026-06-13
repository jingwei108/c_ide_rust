import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../models/template_registry.dart';
//  // CompileSnapshot from ide_provider state
import '../providers/ide_provider.dart';
import '../src/rust/api/cide.dart' as rust;
import '../src/rust/diagnostics/learning_path.dart' as rust_lp;
import '../src/rust/diagnostics/misconception_patterns.dart' as rust_mp;

/// Displays detected misconceptions and recommended learning paths.
///
/// Typically shown as a bottom sheet triggered from the Progress tab.
class LearningPathPanel extends ConsumerStatefulWidget {
  const LearningPathPanel({super.key});

  @override
  ConsumerState<LearningPathPanel> createState() => _LearningPathPanelState();
}

class _LearningPathPanelState extends ConsumerState<LearningPathPanel> {
  List<rust_lp.LearningPath>? _paths;
  bool _loading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadPaths();
  }

  Future<void> _loadPaths() async {
    try {
      final progress = ref.read(ideProvider).learningProgress;
      final records = progress.recentCompileRecords.map((s) => rust_mp.CompileRecord(
        timestampMs: s.timestampMs,
        success: s.success,
        errorCodes: Int32List.fromList(s.errorCodes),
        trapMessage: s.trapMessage,
      )).toList();

      final detected = await rust.detectMisconceptions(history: records);
      final paths = await rust.recommendLearningPaths(detected: detected);

      if (mounted) {
        setState(() {
          _paths = paths;
          _loading = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _error = '加载学习路径失败: $e';
          _loading = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Container(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Row(
            children: [
              Icon(Icons.psychology, color: theme.colorScheme.primary),
              const SizedBox(width: 8),
              Text(
                '认知诊断与学习路径',
                style: theme.textTheme.titleMedium?.copyWith(fontWeight: FontWeight.w600),
              ),
              const Spacer(),
              IconButton(
                icon: const Icon(Icons.close, size: 20),
                onPressed: () => Navigator.pop(context),
              ),
            ],
          ),
          const Divider(),
          if (_loading)
            const Center(child: Padding(padding: EdgeInsets.all(24), child: CircularProgressIndicator()))
          else if (_error != null)
            Center(child: Padding(padding: const EdgeInsets.all(24), child: Text(_error!, style: TextStyle(color: Colors.red))))
          else if (_paths == null || _paths!.isEmpty)
            const Center(
              child: Padding(
                padding: EdgeInsets.all(24),
                child: Text('暂无检测到的认知盲区。继续保持！'),
              ),
            )
          else
            Expanded(
              child: ListView.builder(
                itemCount: _paths!.length,
                itemBuilder: (context, index) => _buildPathCard(_paths![index]),
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildPathCard(rust_lp.LearningPath path) {
    final theme = Theme.of(context);
    final color = _misconceptionColor(path.targetMisconceptionId);

    return Card(
      margin: const EdgeInsets.only(bottom: 12),
      elevation: 0,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(10),
        side: BorderSide(color: color.withValues(alpha: 0.3)),
      ),
      child: Padding(
        padding: const EdgeInsets.all(14),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(_misconceptionIcon(path.targetMisconceptionId), color: color, size: 20),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    path.targetMisconceptionName,
                    style: theme.textTheme.bodyLarge?.copyWith(fontWeight: FontWeight.w600, color: color),
                  ),
                ),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                  decoration: BoxDecoration(
                    color: color.withValues(alpha: 0.1),
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: Text(
                    '约 ${path.estimatedTimeMinutes} 分钟',
                    style: TextStyle(fontSize: 11, color: color, fontWeight: FontWeight.w500),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 10),
            ...path.steps.asMap().entries.map((entry) {
              final i = entry.key;
              final step = entry.value;
              return _buildStepTile(i + 1, step);
            }),
          ],
        ),
      ),
    );
  }

  Widget _buildStepTile(int index, rust_lp.PathStep step) {
    final icon = _stepTypeIcon(step.stepType);
    final notifier = ref.read(ideProvider.notifier);

    return InkWell(
      onTap: () => _executeStep(step, notifier),
      borderRadius: BorderRadius.circular(8),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 4),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Container(
              width: 24,
              height: 24,
              alignment: Alignment.center,
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.primary.withValues(alpha: 0.1),
                shape: BoxShape.circle,
              ),
              child: Text('$index', style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600, color: Theme.of(context).colorScheme.primary)),
            ),
            const SizedBox(width: 10),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(step.title, style: const TextStyle(fontSize: 13, fontWeight: FontWeight.w600)),
                  const SizedBox(height: 2),
                  Text(step.detail, style: TextStyle(fontSize: 11, color: Colors.grey[600], height: 1.4)),
                ],
              ),
            ),
            Icon(icon, size: 16, color: Colors.grey[400]),
          ],
        ),
      ),
    );
  }

  void _executeStep(rust_lp.PathStep step, IdeNotifier notifier) async {
    switch (step.stepType) {
      case 'ReadKnowledgeCard':
        Navigator.pop(context);
        // Switch to knowledge card tab would be handled by parent or state
        break;
      case 'StudyTemplate':
        final navigator = Navigator.of(context);
        final templates = await getDynamicTemplates();
        final template = templates.firstWhere(
          (t) => t.key == step.targetId,
          orElse: () => templates.first,
        );
        final generated = template.params.isEmpty
            ? template.code
            : template.buildCode({for (var p in template.params) p.key: p.defaultValue});
        notifier.updateSource(generated);
        if (template.tutorialSteps.isNotEmpty) {
          notifier.startTutorial(template, generated);
        }
        navigator.pop();
        break;
      case 'CompleteExercise':
        // Show a snackbar with exercise description for now
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('练习: ${step.detail}')),
        );
        break;
      case 'ReviewOwnCode':
        Navigator.pop(context);
        // Jump to line if provided
        if (step.highlightLines.isNotEmpty) {
          // Parent should handle scrollToLine via state or callback
        }
        break;
      default:
        break;
    }
  }

  IconData _stepTypeIcon(String type) {
    switch (type) {
      case 'ReadKnowledgeCard':
        return Icons.menu_book;
      case 'StudyTemplate':
        return Icons.school;
      case 'CompleteExercise':
        return Icons.edit_note;
      case 'ReviewOwnCode':
        return Icons.code;
      default:
        return Icons.arrow_forward;
    }
  }

  Color _misconceptionColor(String id) {
    switch (id) {
      case 'M01':
        return Colors.amber.shade800;
      case 'M02':
        return Colors.deepOrange.shade700;
      case 'M03':
        return Colors.blue.shade700;
      case 'M04':
        return Colors.purple.shade700;
      case 'M05':
        return Colors.teal.shade700;
      case 'M06':
        return Colors.cyan.shade700;
      default:
        return Colors.grey.shade700;
    }
  }

  IconData _misconceptionIcon(String id) {
    switch (id) {
      case 'M01':
        return Icons.exposure_plus_1;
      case 'M02':
        return Icons.memory;
      case 'M03':
        return Icons.compare;
      case 'M04':
        return Icons.alt_route;
      case 'M05':
        return Icons.repeat;
      case 'M06':
        return Icons.print;
      default:
        return Icons.psychology;
    }
  }
}
