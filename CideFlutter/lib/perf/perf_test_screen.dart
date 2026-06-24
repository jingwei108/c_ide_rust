import 'dart:async';
import 'dart:developer' as developer;
import 'dart:io';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/ide_state.dart';
import '../providers/unified_provider.dart';

/// 前端帧率性能测试入口。
///
/// 启动后自动编译并运行冒泡排序可视化用例，在统一模式播放阶段通过
/// [SchedulerBinding.addTimingsCallback] 收集真实帧时间，持续 [sampleDuration]
/// 后输出 FPS 统计并退出应用。
class PerfTestScreen extends ConsumerStatefulWidget {
  const PerfTestScreen({super.key});

  @override
  ConsumerState<PerfTestScreen> createState() => _PerfTestScreenState();
}

class _PerfTestScreenState extends ConsumerState<PerfTestScreen> {
  static const sampleDuration = Duration(seconds: 5);

  final List<FrameTiming> _timings = [];
  bool _startedSampling = false;
  bool _finished = false;

  static const _source = r'''
#include <stdio.h>

void bubbleSort(int arr[], int n) {
    for (int i = 0; i < n - 1; i++) {
        for (int j = 0; j < n - i - 1; j++) {
            if (arr[j] > arr[j + 1]) {
                int temp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
        }
    }
}

int main() {
    int arr[40];
    int n = 40;
    for (int i = 0; i < n; i++) {
        arr[i] = n - i;
    }
    bubbleSort(arr, n);
    for (int i = 0; i < n; i++) {
        printf("%d ", arr[i]);
    }
    printf("\n");
    return 0;
}
''';

  @override
  void initState() {
    super.initState();

    SchedulerBinding.instance.addTimingsCallback(_onTimings);

    WidgetsBinding.instance.addPostFrameCallback((_) async {
      final notifier = ref.read(unifiedProvider.notifier);
      await notifier.compileAndRunMulti([
        const CodeFile(filename: 'perf_baseline.c', source: _source),
      ]);
    });
  }

  @override
  void dispose() {
    SchedulerBinding.instance.removeTimingsCallback(_onTimings);
    super.dispose();
  }

  void _onTimings(List<FrameTiming> timings) {
    if (!_startedSampling) return;
    _timings.addAll(timings);
  }

  void _startSampling() {
    if (_startedSampling) return;
    setState(() {
      _startedSampling = true;
    });

    // 延迟一帧后开始正式采样，避免初始化阶段的异常帧时间污染数据。
    Timer(sampleDuration, _finishSampling);
  }

  void _finishSampling() {
    if (_finished) return;
    _finished = true;
    SchedulerBinding.instance.removeTimingsCallback(_onTimings);

    final report = _buildReport();
    // 同时输出到 dart:developer 日志与 stdout，便于 CI/脚本捕获。
    developer.log(report);
    // ignore: avoid_print
    print(report);

    // 给日志/stdout 刷新留出时间后退出。
    Timer(const Duration(milliseconds: 500), () {
      exit(0);
    });
  }

  String _buildReport() {
    if (_timings.isEmpty) {
      return '[CidePerf] 未收集到帧时间数据，可能处于 headless 环境或 UI 未实际渲染。';
    }

    final buildTimes = _timings.map((t) => t.buildDuration.inMicroseconds.toDouble()).toList();
    final rasterTimes = _timings.map((t) => t.rasterDuration.inMicroseconds.toDouble()).toList();
    final totalTimes = _timings.map((t) => t.totalSpan.inMicroseconds.toDouble()).toList();

    double avg(List<double> values) => values.reduce((a, b) => a + b) / values.length;
    double minValue(List<double> values) => values.reduce(math.min);
    double maxValue(List<double> values) => values.reduce(math.max);
    double percentile(List<double> values, double p) {
      final sorted = List<double>.from(values)..sort();
      final index = (sorted.length * p).ceil() - 1;
      return sorted[index.clamp(0, sorted.length - 1)];
    }

    double fps(double avgUs) => avgUs <= 0 ? 0 : 1_000_000.0 / avgUs;

    final buffer = StringBuffer();
    buffer.writeln('[CidePerf] 前端帧率实测报告');
    buffer.writeln('[CidePerf] 采样帧数: ${_timings.length}');
    buffer.writeln('[CidePerf] 平均 build 帧时间: ${avg(buildTimes).toStringAsFixed(1)} μs -> 等效 FPS: ${fps(avg(buildTimes)).toStringAsFixed(1)}');
    buffer.writeln('[CidePerf] 平均 raster 帧时间: ${avg(rasterTimes).toStringAsFixed(1)} μs -> 等效 FPS: ${fps(avg(rasterTimes)).toStringAsFixed(1)}');
    buffer.writeln('[CidePerf] 平均总帧时间: ${avg(totalTimes).toStringAsFixed(1)} μs -> 等效 FPS: ${fps(avg(totalTimes)).toStringAsFixed(1)}');
    buffer.writeln('[CidePerf] 总帧时间 min/max/p95: ${minValue(totalTimes).toStringAsFixed(0)} / ${maxValue(totalTimes).toStringAsFixed(0)} / ${percentile(totalTimes, 0.95).toStringAsFixed(0)} μs');
    buffer.writeln('[CidePerf] 结论: ${fps(avg(totalTimes)) >= 55 ? "满足 >=55fps 基线" : "未满足 >=55fps 基线"}');
    return buffer.toString();
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(unifiedProvider);

    if (state.phase == ExecutionPhase.playback && state.totalSteps > 0 && !_startedSampling && !_finished) {
      // 使用 addPostFrameCallback 避免在 build 中触发 setState 导致异常。
      WidgetsBinding.instance.addPostFrameCallback((_) => _startSampling());
    }

    if (state.phase == ExecutionPhase.error && !_finished) {
      _finished = true;
      final msg = '[CidePerf] 统一模式启动失败: ${state.errorMessage}';
      developer.log(msg);
      // ignore: avoid_print
      print(msg);
      Timer(const Duration(milliseconds: 500), () => exit(1));
    }

    return Scaffold(
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const CircularProgressIndicator(),
            const SizedBox(height: 16),
            Text('性能测试中... 阶段: ${state.phase.name}'),
            if (_startedSampling) const Text('正在采集帧时间数据（5 秒）'),
          ],
        ),
      ),
    );
  }
}
