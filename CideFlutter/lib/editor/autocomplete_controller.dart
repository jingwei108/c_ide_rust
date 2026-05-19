import 'package:flutter/foundation.dart';

/// ---------------------------------------------------------------------------
/// AutocompleteController — 自动补全状态管理
/// ---------------------------------------------------------------------------

class AutocompleteCandidate {
  final String word;
  final String? type; // 'keyword', 'function', 'macro'
  final String? signature;

  const AutocompleteCandidate({
    required this.word,
    this.type,
    this.signature,
  });
}

class AutocompleteController extends ChangeNotifier {
  String _prefix = '';
  List<AutocompleteCandidate> _candidates = [];
  int _selectedIndex = 0;
  bool _visible = false;

  final List<AutocompleteCandidate> _allWords;

  AutocompleteController({List<AutocompleteCandidate>? words})
      : _allWords = words ?? _defaultWords;

  String get prefix => _prefix;
  List<AutocompleteCandidate> get candidates => List.unmodifiable(_candidates);
  int get selectedIndex => _selectedIndex;
  set selectedIndex(int value) {
    if (_selectedIndex == value) return;
    _selectedIndex = value;
    notifyListeners();
  }
  bool get visible => _visible;

  /// 根据当前光标前的文本更新候选
  void update(String textBeforeCursor) {
    final prefix = extractPrefix(textBeforeCursor);
    if (prefix.isEmpty) {
      hide();
      return;
    }
    _prefix = prefix;
    _candidates = _allWords
        .where((w) => w.word.toLowerCase().startsWith(prefix.toLowerCase()))
        .take(8)
        .toList();
    if (_candidates.isEmpty) {
      hide();
      return;
    }
    _selectedIndex = 0;
    _visible = true;
    notifyListeners();
  }

  void hide() {
    if (!_visible) return;
    _visible = false;
    _candidates = [];
    notifyListeners();
  }

  void selectNext() {
    if (_candidates.isEmpty) return;
    _selectedIndex = (_selectedIndex + 1) % _candidates.length;
    notifyListeners();
  }

  void selectPrevious() {
    if (_candidates.isEmpty) return;
    _selectedIndex =
        (_selectedIndex - 1 + _candidates.length) % _candidates.length;
    notifyListeners();
  }

  AutocompleteCandidate? confirm() {
    if (!_visible || _candidates.isEmpty) return null;
    final selected = _candidates[_selectedIndex];
    hide();
    return selected;
  }

  static String extractPrefix(String text) {
    // 从后往前找到非标识符字符
    int i = text.length - 1;
    while (i >= 0) {
      final c = text[i];
      if (c == '_' ||
          (c.codeUnitAt(0) >= 65 && c.codeUnitAt(0) <= 90) ||
          (c.codeUnitAt(0) >= 97 && c.codeUnitAt(0) <= 122) ||
          (c.codeUnitAt(0) >= 48 && c.codeUnitAt(0) <= 57)) {
        i--;
      } else {
        break;
      }
    }
    return text.substring(i + 1);
  }

  static final List<AutocompleteCandidate> _defaultWords = const [
    // 关键字
    AutocompleteCandidate(word: 'auto', type: 'keyword'),
    AutocompleteCandidate(word: 'break', type: 'keyword'),
    AutocompleteCandidate(word: 'case', type: 'keyword'),
    AutocompleteCandidate(word: 'char', type: 'keyword'),
    AutocompleteCandidate(word: 'const', type: 'keyword'),
    AutocompleteCandidate(word: 'continue', type: 'keyword'),
    AutocompleteCandidate(word: 'default', type: 'keyword'),
    AutocompleteCandidate(word: 'do', type: 'keyword'),
    AutocompleteCandidate(word: 'double', type: 'keyword'),
    AutocompleteCandidate(word: 'else', type: 'keyword'),
    AutocompleteCandidate(word: 'enum', type: 'keyword'),
    AutocompleteCandidate(word: 'extern', type: 'keyword'),
    AutocompleteCandidate(word: 'float', type: 'keyword'),
    AutocompleteCandidate(word: 'for', type: 'keyword'),
    AutocompleteCandidate(word: 'goto', type: 'keyword'),
    AutocompleteCandidate(word: 'if', type: 'keyword'),
    AutocompleteCandidate(word: 'int', type: 'keyword'),
    AutocompleteCandidate(word: 'long', type: 'keyword'),
    AutocompleteCandidate(word: 'register', type: 'keyword'),
    AutocompleteCandidate(word: 'return', type: 'keyword'),
    AutocompleteCandidate(word: 'short', type: 'keyword'),
    AutocompleteCandidate(word: 'signed', type: 'keyword'),
    AutocompleteCandidate(word: 'sizeof', type: 'keyword'),
    AutocompleteCandidate(word: 'static', type: 'keyword'),
    AutocompleteCandidate(word: 'struct', type: 'keyword'),
    AutocompleteCandidate(word: 'switch', type: 'keyword'),
    AutocompleteCandidate(word: 'typedef', type: 'keyword'),
    AutocompleteCandidate(word: 'union', type: 'keyword'),
    AutocompleteCandidate(word: 'unsigned', type: 'keyword'),
    AutocompleteCandidate(word: 'void', type: 'keyword'),
    AutocompleteCandidate(word: 'volatile', type: 'keyword'),
    AutocompleteCandidate(word: 'while', type: 'keyword'),
    // 标准库函数
    AutocompleteCandidate(
        word: 'printf', type: 'function', signature: 'printf(const char* format, ...)'),
    AutocompleteCandidate(
        word: 'scanf', type: 'function', signature: 'scanf(const char* format, ...)'),
    AutocompleteCandidate(
        word: 'malloc', type: 'function', signature: 'malloc(size_t size)'),
    AutocompleteCandidate(word: 'free', type: 'function', signature: 'free(void* ptr)'),
    AutocompleteCandidate(
        word: 'memset', type: 'function', signature: 'memset(void* s, int c, size_t n)'),
    AutocompleteCandidate(
        word: 'memcpy', type: 'function', signature: 'memcpy(void* dest, const void* src, size_t n)'),
    AutocompleteCandidate(
        word: 'strlen', type: 'function', signature: 'strlen(const char* s)'),
    AutocompleteCandidate(
        word: 'strcpy', type: 'function', signature: 'strcpy(char* dest, const char* src)'),
    AutocompleteCandidate(
        word: 'strcmp', type: 'function', signature: 'strcmp(const char* s1, const char* s2)'),
    AutocompleteCandidate(word: 'atoi', type: 'function', signature: 'atoi(const char* str)'),
    AutocompleteCandidate(word: 'rand', type: 'function', signature: 'rand()'),
    AutocompleteCandidate(word: 'srand', type: 'function', signature: 'srand(unsigned int seed)'),
    AutocompleteCandidate(word: 'exit', type: 'function', signature: 'exit(int status)'),
    AutocompleteCandidate(word: 'getchar', type: 'function', signature: 'getchar()'),
    AutocompleteCandidate(word: 'putchar', type: 'function', signature: 'putchar(int c)'),
    AutocompleteCandidate(
        word: 'fprintf', type: 'function', signature: 'fprintf(FILE* stream, const char* format, ...)'),
    AutocompleteCandidate(
        word: 'qsort', type: 'function', signature: 'qsort(void* base, size_t nmemb, size_t size, compar)'),
    AutocompleteCandidate(
        word: 'realloc', type: 'function', signature: 'realloc(void* ptr, size_t size)'),
    // 常用宏
    AutocompleteCandidate(word: 'NULL', type: 'macro'),
    AutocompleteCandidate(word: 'EOF', type: 'macro'),
    AutocompleteCandidate(word: 'stdout', type: 'macro'),
    AutocompleteCandidate(word: 'stderr', type: 'macro'),
    AutocompleteCandidate(word: 'true', type: 'macro'),
    AutocompleteCandidate(word: 'false', type: 'macro'),
    // 控制结构
    AutocompleteCandidate(word: 'main', type: 'keyword'),
    AutocompleteCandidate(word: 'include', type: 'keyword'),
    AutocompleteCandidate(word: 'define', type: 'keyword'),
  ];
}
