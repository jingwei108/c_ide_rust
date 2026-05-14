import 'package:flutter/material.dart';

class SymbolChip extends StatelessWidget {
  final String label;
  final VoidCallback onTap;
  final bool isAction;

  const SymbolChip({super.key, required this.label, required this.onTap, this.isAction = false});

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      child: Container(
        margin: const EdgeInsets.symmetric(horizontal: 2, vertical: 4),
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        decoration: BoxDecoration(
          color: isAction ? Colors.blueAccent.withValues(alpha: 0.15) : Theme.of(context).dividerColor.withValues(alpha: 0.1),
          borderRadius: BorderRadius.circular(4),
        ),
        alignment: Alignment.center,
        child: Text(
          label,
          style: TextStyle(
            fontSize: 12,
            color: isAction ? Colors.blueAccent : Theme.of(context).textTheme.bodyMedium?.color,
            fontFamily: 'monospace',
          ),
        ),
      ),
    );
  }
}
