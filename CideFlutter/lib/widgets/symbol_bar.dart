import 'package:flutter/material.dart';
import 'symbol_chip.dart';

class SymbolBar extends StatelessWidget {
  final void Function(String left, String right) onInsertPair;
  final void Function(String text) onInsertText;
  final void Function(int delta) onMoveCursor;
  final VoidCallback onUndo;
  final VoidCallback onRedo;

  const SymbolBar({
    super.key,
    required this.onInsertPair,
    required this.onInsertText,
    required this.onMoveCursor,
    required this.onUndo,
    required this.onRedo,
  });

  @override
  Widget build(BuildContext context) {
    final symbols = [
      ('{ }', () => onInsertPair('{', '}')),
      ('( )', () => onInsertPair('(', ')')),
      ('[ ]', () => onInsertPair('[', ']')),
      ('" "', () => onInsertPair('"', '"')),
      ("' '", () => onInsertPair("'", "'")),
      (';', () => onInsertText(';')),
      ('#', () => onInsertText('#')),
      ('->', () => onInsertText('->')),
      ('&', () => onInsertText('&')),
      ('*', () => onInsertText('*')),
      ('=', () => onInsertText('=')),
      ('==', () => onInsertText('==')),
      ('!=', () => onInsertText('!=')),
      ('<', () => onInsertText('<')),
      ('>', () => onInsertText('>')),
      ('+', () => onInsertText('+')),
      ('-', () => onInsertText('-')),
      ('/', () => onInsertText('/')),
      ('%', () => onInsertText('%')),
      ('&&', () => onInsertText('&&')),
      ('||', () => onInsertText('||')),
      ('!', () => onInsertText('!')),
      ('|', () => onInsertText('|')),
      ('^', () => onInsertText('^')),
      ('~', () => onInsertText('~')),
      (',', () => onInsertText(',')),
      ('.', () => onInsertText('.')),
    ];

    final actions = [
      ('←', () => onMoveCursor(-1)),
      ('→', () => onMoveCursor(1)),
      ('Tab', () => onInsertText('    ')),
      ('↩', onUndo),
      ('↪', onRedo),
    ];

    return Container(
      height: 36,
      decoration: BoxDecoration(
        border: Border(
          top: BorderSide(color: Theme.of(context).dividerColor.withValues(alpha: 0.2)),
        ),
      ),
      child: ListView(
        scrollDirection: Axis.horizontal,
        padding: const EdgeInsets.symmetric(horizontal: 4),
        children: [
          ...symbols.map((s) => SymbolChip(label: s.$1, onTap: s.$2)),
          Container(
            margin: const EdgeInsets.symmetric(horizontal: 4, vertical: 6),
            width: 1,
            color: Theme.of(context).dividerColor,
          ),
          ...actions.map((a) => SymbolChip(label: a.$1, onTap: a.$2, isAction: true)),
        ],
      ),
    );
  }
}
