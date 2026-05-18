import 'package:flutter/material.dart';
import '../models/ide_state.dart';

class FileTabBar extends StatelessWidget {
  final List<CodeFile> files;
  final String currentFile;
  final ValueChanged<String> onSwitch;
  final ValueChanged<String> onClose;
  final VoidCallback onAdd;

  const FileTabBar({
    super.key,
    required this.files,
    required this.currentFile,
    required this.onSwitch,
    required this.onClose,
    required this.onAdd,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isDark = theme.brightness == Brightness.dark;

    return Container(
      height: 40,
      decoration: BoxDecoration(
        color: isDark ? const Color(0xFF1E1E1E) : const Color(0xFFF5F5F5),
        border: Border(
          bottom: BorderSide(
            color: isDark ? const Color(0xFF333333) : const Color(0xFFE0E0E0),
          ),
        ),
      ),
      child: Row(
        children: [
          Expanded(
            child: ListView.builder(
              scrollDirection: Axis.horizontal,
              itemCount: files.length,
              itemBuilder: (context, index) {
                final file = files[index];
                final isActive = file.filename == currentFile;
                return _buildTab(file, isActive, isDark);
              },
            ),
          ),
          IconButton(
            icon: const Icon(Icons.add, size: 18),
            tooltip: '新建文件',
            onPressed: onAdd,
            padding: const EdgeInsets.symmetric(horizontal: 8),
            constraints: const BoxConstraints(minWidth: 32, minHeight: 32),
          ),
        ],
      ),
    );
  }

  Widget _buildTab(CodeFile file, bool isActive, bool isDark) {
    return GestureDetector(
      onTap: () => onSwitch(file.filename),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 12),
        decoration: BoxDecoration(
          color: isActive
              ? (isDark ? const Color(0xFF2D2D2D) : Colors.white)
              : Colors.transparent,
          border: Border(
            bottom: BorderSide(
              color: isActive
                  ? const Color(0xFF007ACC)
                  : Colors.transparent,
              width: 2,
            ),
          ),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              file.filename,
              style: TextStyle(
                fontSize: 13,
                color: isActive
                    ? (isDark ? Colors.white : Colors.black87)
                    : (isDark ? Colors.grey : Colors.grey.shade700),
                fontWeight: isActive ? FontWeight.w500 : FontWeight.normal,
              ),
            ),
            if (files.length > 1) ...[
              const SizedBox(width: 6),
              GestureDetector(
                onTap: () => onClose(file.filename),
                child: Icon(
                  Icons.close,
                  size: 14,
                  color: isActive
                      ? (isDark ? Colors.white70 : Colors.black54)
                      : (isDark ? Colors.grey.shade600 : Colors.grey.shade500),
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
