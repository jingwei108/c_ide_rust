import 'dart:math' as math;
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../models/knowledge_card.dart';
import '../providers/ide_provider.dart';
import '../providers/unified_provider.dart';
import 'knowledge_card_item.dart';
import 'root_cause_banner.dart';

class ExecutionControlPanel extends ConsumerWidget {
  final VoidCallback onRun;
  final void Function(int line)? onScrollToLine;

  const ExecutionControlPanel({
    super.key,
    required this.onRun,
    this.onScrollToLine,
  });

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(unifiedProvider);
    final controller = ref.read(unifiedProvider.notifier);

    if (state.phase == ExecutionPhase.idle) {
      return const SizedBox.shrink();
    }

    // 获取当前步的算法可视化事件上下文
    String? visContext;
    final cacheIdx = state.currentStep - state.frameCacheStartStep;
    final currentPayload = cacheIdx >= 0 && cacheIdx < state.frameCache.length
        ? state.frameCache[cacheIdx]
        : null;
    final algorithmStep = currentPayload?.algorithmStep;
    final rootCauseHint = currentPayload?.rootCauseHint;
    if (currentPayload != null && currentPayload.visEvents.isNotEmpty) {
      visContext = currentPayload.visEvents
          .map((e) => e.context)
          .where((c) => c.isNotEmpty)
          .join(' · ');
    }

    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // 算法检测信息条
        if (state.algorithmMatches.isNotEmpty)
          Container(
            width: double.infinity,
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 5),
            color: Colors.indigo.shade800,
            child: Row(
              children: [
                const Icon(Icons.psychology, color: Colors.white, size: 16),
                const SizedBox(width: 8),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        state.algorithmMatches.map((m) => m.displayName).join(' · '),
                        style: const TextStyle(
                          color: Colors.white,
                          fontSize: 12,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                      if (state.algorithmMatches.first.suggestion.isNotEmpty)
                        Text(
                          state.algorithmMatches.first.suggestion,
                          style: TextStyle(
                            color: Colors.indigo.shade100,
                            fontSize: 10,
                          ),
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        // 算法步骤语义标注条
        if (algorithmStep != null)
          Container(
            width: double.infinity,
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
            color: _phaseColor(algorithmStep.phase),
            child: Row(
              children: [
                Icon(_phaseIcon(algorithmStep.phase), color: Colors.white, size: 14),
                const SizedBox(width: 6),
                Expanded(
                  child: Text(
                    algorithmStep.description,
                    style: const TextStyle(
                      color: Colors.white,
                      fontSize: 12,
                      fontWeight: FontWeight.w500,
                    ),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 1),
                  decoration: BoxDecoration(
                    color: Colors.white.withValues(alpha: 0.2),
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(
                    algorithmStep.displayName,
                    style: const TextStyle(color: Colors.white, fontSize: 10),
                  ),
                ),
              ],
            ),
          ),
        // 根因分析提示条（认知推理 P0）
        if (rootCauseHint != null)
          RootCauseBanner(
            hint: rootCauseHint,
            onLineTap: onScrollToLine,
          ),
        // 运行时异常提示条
        if (state.trapMessage != null)
          Container(
            width: double.infinity,
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
            color: Colors.red.shade900,
            child: Row(
              children: [
                const Icon(Icons.warning_amber, color: Colors.white, size: 16),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    state.trapMessage!,
                    style: const TextStyle(color: Colors.white, fontSize: 12),
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                TextButton(
                  onPressed: () {
                    _showTrapHelp(context, state.trapMessage!);
                  },
                  style: TextButton.styleFrom(
                    foregroundColor: Colors.yellow,
                    padding: EdgeInsets.zero,
                    minimumSize: const Size(0, 0),
                    tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                  ),
                  child: const Text('查看帮助', style: TextStyle(fontSize: 12)),
                ),
                const SizedBox(width: 8),
                TextButton(
                  onPressed: () {
                    ref.read(unifiedProvider.notifier).onCodeChanged();
                  },
                  style: TextButton.styleFrom(
                    foregroundColor: Colors.white,
                    padding: EdgeInsets.zero,
                    minimumSize: const Size(0, 0),
                    tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                  ),
                  child: const Text('重置', style: TextStyle(fontSize: 12)),
                ),
              ],
            ),
          ),
        // 算法可视化事件指示条
        if (visContext != null && visContext.isNotEmpty)
          Container(
            width: double.infinity,
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
            decoration: BoxDecoration(
              gradient: LinearGradient(
                colors: [Colors.purple.shade700, Colors.pink.shade600],
              ),
            ),
            child: Row(
              children: [
                const Icon(Icons.auto_graph, color: Colors.white, size: 14),
                const SizedBox(width: 6),
                Expanded(
                  child: Text(
                    '算法事件: $visContext',
                    style: const TextStyle(color: Colors.white, fontSize: 11),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              ],
            ),
          ),
        Container(
          height: 56,
          padding: const EdgeInsets.symmetric(horizontal: 8),
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            border: Border(
              top: BorderSide(color: Theme.of(context).dividerColor),
            ),
          ),
          child: Row(
        children: [
          // 播放/暂停按钮
          _buildPlayPauseButton(state, controller, onRun),
          // 单步按钮
          if (state.canStep)
            IconButton(
              icon: const Icon(Icons.skip_next, size: 20),
              tooltip: '单步',
              onPressed: controller.stepNext,
            ),
          // 进度条（含事件标记）
          if (state.showSlider)
            Expanded(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  _buildEventMarkerStrip(state),
                  Slider(
                    min: 0,
                    max: math.max(state.maxCollectedStep.toDouble(), 1),
                    value: state.currentStep.clamp(0, state.maxCollectedStep).toDouble(),
                    divisions: state.maxCollectedStep > 0 ? state.maxCollectedStep : null,
                    label: _buildSliderLabel(state),
                    onChangeStart: (_) => controller.pause(),
                    onChanged: (v) => controller.onSliderChanged(v.round()),
                    onChangeEnd: (v) => controller.seekTo(v.round()),
                  ),
                ],
              ),
            ),
          // 播放速度
          if (state.phase == ExecutionPhase.collecting ||
              state.phase == ExecutionPhase.paused ||
              state.phase == ExecutionPhase.stepMode)
            _buildSpeedButton(state, controller),
          // 代码覆盖率
          if (state.heatmap != null && state.heatmap!.lineCounts.isNotEmpty) ...[
            Padding(
              padding: const EdgeInsets.only(left: 8),
              child: _buildCoverageText(context, state, ref),
            ),
          ],
          // 步数显示
          Padding(
            padding: const EdgeInsets.only(left: 8),
            child: Text(
              '${state.currentStep} / ${state.maxCollectedStep}',
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ),
        ],
      ),
    ),
  ],
);
  }

  Widget _buildCoverageText(BuildContext context, UnifiedState state, WidgetRef ref) {
    final source = ref.watch(ideProvider).source;
    final totalLines = source.isEmpty ? 0 : source.split('\n').length;
    // VM 已只记录用户主文件（file_id == 0）的源码行号，
    // heatmap 不再包含 Bytecode Libc / 标准库的外部行号。
    final executedLines = state.heatmap!.lineCounts
        .where((entry) => entry.$1 > 0)
        .length;
    final coverage = totalLines > 0 ? executedLines / totalLines : 0.0;
    final color = coverage >= 0.8
        ? Colors.green
        : coverage >= 0.5
            ? Colors.orange
            : Colors.red;
    return Text(
      '覆盖率 ${(coverage * 100).toStringAsFixed(1)}%',
      style: Theme.of(context).textTheme.bodySmall?.copyWith(color: color),
    );
  }

  String _buildSliderLabel(UnifiedState state) {
    final cacheIdx = state.currentStep - state.frameCacheStartStep;
    if (cacheIdx >= 0 && cacheIdx < state.frameCache.length) {
      final payload = state.frameCache[cacheIdx];
      if (payload.semanticLabel.isNotEmpty) {
        return payload.semanticLabel;
      }
    }
    return '第 ${state.currentStep} 步';
  }

  /// 在进度条上方绘制关键事件标记（交换/递归/调用/IO）。
  Widget _buildEventMarkerStrip(UnifiedState state) {
    if (state.frameCache.isEmpty || state.maxCollectedStep <= 0) {
      return const SizedBox(height: 6);
    }

    final markers = <_EventMarker>[];
    for (final payload in state.frameCache) {
      final label = payload.semanticLabel;
      if (label.isEmpty) continue;
      Color? color;
      String tooltip = label;
      if (label.contains('交换')) {
        color = Colors.amber;
      } else if (label.contains('递归')) {
        color = Colors.purpleAccent;
      } else if (label.contains('调用') || label.contains('返回')) {
        color = Colors.blueAccent;
      } else if (label.contains('IO') || label.contains('printf') || label.contains('scanf')) {
        color = Colors.greenAccent;
      } else if (label.contains('内存') || label.contains('malloc') || label.contains('free')) {
        color = Colors.cyanAccent;
      }
      if (color != null) {
        markers.add(_EventMarker(step: payload.stepIndex, color: color, tooltip: tooltip));
      }
    }

    if (markers.isEmpty) return const SizedBox(height: 6);

    return LayoutBuilder(
      builder: (context, constraints) {
        final width = constraints.maxWidth;
        final maxStep = state.maxCollectedStep.toDouble();
        return SizedBox(
          height: 6,
          child: Stack(
            clipBehavior: Clip.none,
            children: markers.map((m) {
              final left = maxStep > 0 ? (m.step / maxStep) * width : 0.0;
              return Positioned(
                left: left.clamp(0, width - 4),
                top: 1,
                child: Tooltip(
                  message: m.tooltip,
                  child: Container(
                    width: 4,
                    height: 4,
                    decoration: BoxDecoration(
                      color: m.color,
                      shape: BoxShape.circle,
                      boxShadow: [
                        BoxShadow(
                          color: m.color.withValues(alpha: 0.6),
                          blurRadius: 3,
                          spreadRadius: 0.5,
                        ),
                      ],
                    ),
                  ),
                ),
              );
            }).toList(),
          ),
        );
      },
    );
  }

  Widget _buildPlayPauseButton(
    UnifiedState state,
    UnifiedNotifier controller,
    VoidCallback onRun,
  ) {
    IconData icon;
    VoidCallback? onPressed;
    String tooltip;

    switch (state.phase) {
      case ExecutionPhase.idle:
        return const SizedBox.shrink();
      case ExecutionPhase.compiling:
        icon = Icons.hourglass_top;
        onPressed = null;
        tooltip = '编译中';
      case ExecutionPhase.collecting:
        icon = Icons.pause;
        onPressed = controller.pause;
        tooltip = '暂停';
      case ExecutionPhase.paused:
      case ExecutionPhase.stepMode:
        icon = Icons.play_arrow;
        onPressed = controller.resume;
        tooltip = '继续';
      case ExecutionPhase.playback:
        icon = Icons.play_arrow;
        onPressed = controller.resume;
        tooltip = '从当前步继续执行';
      case ExecutionPhase.seeking:
        icon = Icons.hourglass_top;
        onPressed = null;
        tooltip = '加载中';
      case ExecutionPhase.error:
        icon = Icons.play_arrow;
        onPressed = onRun;
        tooltip = '重新运行';
    }

    return IconButton(
      icon: Icon(icon, size: 24),
      tooltip: tooltip,
      onPressed: onPressed,
    );
  }

  void _showTrapHelp(BuildContext context, String trapMessage) {
    final cards = KnowledgeCard.findByTrapMessage(trapMessage);
    showModalBottomSheet(
      context: context,
      isScrollControlled: true,
      builder: (context) => DraggableScrollableSheet(
        initialChildSize: 0.5,
        minChildSize: 0.3,
        maxChildSize: 0.9,
        expand: false,
        builder: (context, scrollController) {
          return Container(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  cards.isEmpty ? '未找到相关知识卡片' : '相关帮助',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                const SizedBox(height: 8),
                Text(
                  '异常信息: $trapMessage',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Colors.grey[600],
                  ),
                ),
                const SizedBox(height: 12),
                Expanded(
                  child: cards.isEmpty
                      ? Center(
                          child: Text(
                            '暂无与该异常匹配的知识卡片',
                            style: TextStyle(color: Colors.grey[500]),
                          ),
                        )
                      : ListView.builder(
                          controller: scrollController,
                          itemCount: cards.length,
                          itemBuilder: (context, index) => KnowledgeCardItem(
                            card: cards[index],
                            isDark: Theme.of(context).brightness == Brightness.dark,
                          ),
                        ),
                ),
              ],
            ),
          );
        },
      ),
    );
  }

  Widget _buildSpeedButton(UnifiedState state, UnifiedNotifier controller) {
    return PopupMenuButton<double>(
      tooltip: '播放速度',
      initialValue: state.playbackSpeed,
      onSelected: controller.setPlaybackSpeed,
      itemBuilder: (context) => [
        const PopupMenuItem(value: 0.5, child: Text('0.5x')),
        const PopupMenuItem(value: 1.0, child: Text('1.0x')),
        const PopupMenuItem(value: 2.0, child: Text('2.0x')),
        const PopupMenuItem(value: 4.0, child: Text('4.0x')),
      ],
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 8),
        child: Text(
          '${state.playbackSpeed}x',
          style: const TextStyle(fontSize: 12),
        ),
      ),
    );
  }

  Color _phaseColor(String phase) {
    switch (phase) {
      case 'outer_loop':
        return Colors.blue.shade700;
      case 'inner_loop':
        return Colors.indigo.shade600;
      case 'compare':
        return Colors.purple.shade600;
      case 'swap':
        return Colors.amber.shade700;
      case 'insert':
        return Colors.orange.shade700;
      case 'partition_init':
        return Colors.pink.shade700;
      case 'partition_scan':
        return Colors.pink.shade600;
      case 'partition_swap':
        return Colors.pink.shade800;
      case 'recursive':
      case 'recursive_split':
        return Colors.teal.shade700;
      case 'merge':
        return Colors.green.shade700;
      case 'loop':
        return Colors.blueGrey.shade700;
      case 'mid_calc':
        return Colors.cyan.shade700;
      case 'narrow_left':
      case 'narrow_right':
        return Colors.lightBlue.shade700;
      case 'found':
        return Colors.green.shade800;
      case 'not_found':
        return Colors.red.shade700;
      case 'finish':
        return Colors.green.shade600;
      default:
        return Colors.grey.shade700;
    }
  }

  IconData _phaseIcon(String phase) {
    switch (phase) {
      case 'outer_loop':
      case 'inner_loop':
      case 'loop':
        return Icons.loop;
      case 'compare':
        return Icons.compare_arrows;
      case 'swap':
      case 'partition_swap':
        return Icons.swap_horiz;
      case 'insert':
        return Icons.input;
      case 'partition_init':
        return Icons.adjust;
      case 'partition_scan':
        return Icons.search;
      case 'recursive':
      case 'recursive_split':
        return Icons.call_split;
      case 'merge':
        return Icons.merge_type;
      case 'mid_calc':
        return Icons.calculate;
      case 'narrow_left':
      case 'narrow_right':
        return Icons.zoom_in;
      case 'found':
        return Icons.check_circle;
      case 'not_found':
        return Icons.cancel;
      case 'finish':
        return Icons.done_all;
      default:
        return Icons.auto_graph;
    }
  }
}

class _EventMarker {
  final int step;
  final Color color;
  final String tooltip;

  const _EventMarker({required this.step, required this.color, required this.tooltip});
}
