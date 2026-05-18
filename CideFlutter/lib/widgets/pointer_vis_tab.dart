import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:cide/src/rust/api/cide.dart' as rust;
import 'package:cide/src/rust/unified/types.dart' as rust_types;
import '../providers/unified_provider.dart';
import 'pointer_arrow_painter.dart';

class PointerVisTab extends ConsumerWidget {
  final bool isDark;

  const PointerVisTab({super.key, required this.isDark});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final unifiedState = ref.watch(unifiedProvider);
    final frameCache = unifiedState.frameCache;
    final currentStep = unifiedState.currentStep;

    // 统一模式：从 frameCache 读取指针快照
    if (frameCache.isNotEmpty && currentStep >= 0 && currentStep < frameCache.length) {
      final payload = frameCache[currentStep];
      final pointers = payload.pointerSnapshots;

      if (pointers.isEmpty) {
        return _buildEmpty('当前步未检测到指针变量');
      }

      return ListView.builder(
        padding: const EdgeInsets.symmetric(vertical: 8),
        itemCount: pointers.length,
        itemBuilder: (context, index) {
          final p = pointers[index];
          return PointerArrowWidget(
            name: p.name,
            addr: p.addr,
            tyName: p.tyName,
            targetAddr: p.targetAddr,
            targetName: p.targetName,
            status: _mapStatus(p.status),
            isDark: isDark,
          );
        },
      );
    }

    // 非统一模式（单步调试/直接运行）：回退到实时读取变量
    return _buildLegacyView();
  }

  Widget _buildLegacyView() {
    return FutureBuilder<List<_LegacyPointer>>(
      future: _fetchLegacyPointers(),
      builder: (context, snapshot) {
        if (!snapshot.hasData || snapshot.data!.isEmpty) {
          return _buildEmpty('运行程序以查看指针追踪');
        }
        final pointers = snapshot.data!;
        return ListView.builder(
          padding: const EdgeInsets.symmetric(vertical: 8),
          itemCount: pointers.length,
          itemBuilder: (context, index) {
            final p = pointers[index];
            return PointerArrowWidget(
              name: p.name,
              addr: p.addr,
              tyName: p.tyName,
              targetAddr: p.targetAddr,
              targetName: p.targetName,
              status: p.status,
              isDark: isDark,
            );
          },
        );
      },
    );
  }

  Future<List<_LegacyPointer>> _fetchLegacyPointers() async {
    try {
      final vars = await rust.getVariables();
      const nullTrapEnd = 64;
      const linearMemorySize = 256 * 1024;
      final pointers = <_LegacyPointer>[];

      for (final v in vars) {
        if (!v.tyName.contains('*')) continue;
        final val = int.tryParse(v.value) ?? 0;
        if (val < 0 || val > linearMemorySize) continue;

        PointerStatus status;
        if (val == 0) {
          status = PointerStatus.null_;
        } else if (val < nullTrapEnd) {
          status = PointerStatus.dangling;
        } else {
          status = PointerStatus.valid;
        }

        String targetName = '';
        for (final other in vars) {
          if (other.addr == val) {
            targetName = other.name;
            break;
          }
        }

        pointers.add(_LegacyPointer(
          name: v.name,
          addr: v.addr,
          tyName: v.tyName,
          targetAddr: val,
          targetName: targetName,
          status: status,
        ));
      }
      return pointers;
    } catch (_) {
      return [];
    }
  }

  Widget _buildEmpty(String message) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(Icons.polyline, size: 40, color: Colors.grey[500]),
          const SizedBox(height: 12),
          Text(message, style: TextStyle(fontSize: 14, color: Colors.grey[500])),
        ],
      ),
    );
  }

  PointerStatus _mapStatus(rust_types.PointerStatus s) {
    switch (s) {
      case rust_types.PointerStatus.valid:
        return PointerStatus.valid;
      case rust_types.PointerStatus.freed:
        return PointerStatus.freed;
      case rust_types.PointerStatus.null_:
        return PointerStatus.null_;
      case rust_types.PointerStatus.dangling:
        return PointerStatus.dangling;
    }
  }
}

class _LegacyPointer {
  final String name;
  final int addr;
  final String tyName;
  final int targetAddr;
  final String targetName;
  final PointerStatus status;

  _LegacyPointer({
    required this.name,
    required this.addr,
    required this.tyName,
    required this.targetAddr,
    required this.targetName,
    required this.status,
  });
}
