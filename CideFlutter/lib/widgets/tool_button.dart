import 'package:flutter/material.dart';

class ToolButton extends StatelessWidget {
  final IconData icon;
  final Color? color;
  final VoidCallback? onPressed;

  const ToolButton({super.key, required this.icon, this.color, this.onPressed});

  @override
  Widget build(BuildContext context) {
    return IconButton(
      icon: Icon(icon, size: 20, color: color),
      padding: const EdgeInsets.all(6),
      constraints: const BoxConstraints(),
      onPressed: onPressed,
    );
  }
}
