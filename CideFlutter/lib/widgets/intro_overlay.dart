import 'package:flutter/material.dart';

class IntroStep {
  final String title;
  final String description;
  final IconData icon;

  const IntroStep({
    required this.title,
    required this.description,
    required this.icon,
  });
}

const List<IntroStep> _introSteps = [
  IntroStep(
    title: '欢迎使用 Cide',
    description: 'Cide 是一款跨平台 C 语言 IDE，支持编译、运行、单步调试和可视化。',
    icon: Icons.code,
  ),
  IntroStep(
    title: '编写代码',
    description: '在编辑器中编写 C 代码，支持语法高亮、自动补全和错误提示。',
    icon: Icons.edit,
  ),
  IntroStep(
    title: '编译与运行',
    description: '点击工具栏的 ▶ 运行代码，或点击 ⏭ 进入单步调试模式。',
    icon: Icons.play_arrow,
  ),
  IntroStep(
    title: '调试面板',
    description: '点击右下角悬浮球查看变量、内存、调用栈和链表可视化。',
    icon: Icons.bug_report,
  ),
  IntroStep(
    title: '算法验证',
    description: '在算法面板中点击"验证算法"，自动测试你的排序/查找实现。',
    icon: Icons.search,
  ),
];

class IntroOverlay extends StatefulWidget {
  final VoidCallback onDone;
  final bool isDark;

  const IntroOverlay({
    super.key,
    required this.onDone,
    this.isDark = false,
  });

  @override
  State<IntroOverlay> createState() => _IntroOverlayState();
}

class _IntroOverlayState extends State<IntroOverlay> {
  int _currentIndex = 0;

  void _next() {
    if (_currentIndex < _introSteps.length - 1) {
      setState(() => _currentIndex++);
    } else {
      widget.onDone();
    }
  }

  void _skip() {
    widget.onDone();
  }

  @override
  Widget build(BuildContext context) {
    final step = _introSteps[_currentIndex];
    final bgColor = widget.isDark ? const Color(0xE6121212) : const Color(0xE6F5F5F5);
    final textColor = widget.isDark ? const Color(0xFFD4D4D4) : const Color(0xFF333333);

    return Material(
      color: Colors.transparent,
      child: Container(
        color: bgColor,
        child: SafeArea(
          child: Column(
            children: [
              Align(
                alignment: Alignment.topRight,
                child: TextButton(
                  onPressed: _skip,
                  child: Text('跳过', style: TextStyle(color: textColor.withValues(alpha: 0.6))),
                ),
              ),
              const Spacer(),
              Icon(step.icon, size: 64, color: Colors.blueAccent),
              const SizedBox(height: 24),
              Text(
                step.title,
                style: TextStyle(
                  fontSize: 22,
                  fontWeight: FontWeight.bold,
                  color: textColor,
                ),
              ),
              const SizedBox(height: 12),
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 32),
                child: Text(
                  step.description,
                  textAlign: TextAlign.center,
                  style: TextStyle(fontSize: 14, color: textColor.withValues(alpha: 0.8), height: 1.5),
                ),
              ),
              const SizedBox(height: 32),
              // 步骤指示器
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: List.generate(_introSteps.length, (index) {
                  final isActive = index == _currentIndex;
                  return Container(
                    width: isActive ? 20 : 8,
                    height: 8,
                    margin: const EdgeInsets.symmetric(horizontal: 4),
                    decoration: BoxDecoration(
                      color: isActive ? Colors.blueAccent : Colors.grey.withValues(alpha: 0.4),
                      borderRadius: BorderRadius.circular(4),
                    ),
                  );
                }),
              ),
              const SizedBox(height: 32),
              ElevatedButton(
                onPressed: _next,
                style: ElevatedButton.styleFrom(
                  padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 12),
                  shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
                ),
                child: Text(_currentIndex < _introSteps.length - 1 ? '下一步' : '开始使用'),
              ),
              const Spacer(),
            ],
          ),
        ),
      ),
    );
  }
}
