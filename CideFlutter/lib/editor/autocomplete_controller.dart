import 'dart:async';
import 'package:flutter/foundation.dart';
import 'package:cide/src/rust/api/cide.dart' as rust;

/// ---------------------------------------------------------------------------
/// AutocompleteController — 自动补全状态管理（v2 语义增强）
/// ---------------------------------------------------------------------------
/// 支持静态关键词 + 动态语义候选混合补全。
/// 语义候选通过 flutter_rust_bridge 从 Rust 后端获取，含类型、签名、字段等。
/// ---------------------------------------------------------------------------

class AutocompleteCandidate {
  final String word;
  final String? type; // 'keyword', 'function', 'macro', 'variable', 'struct', 'field', ...
  final String? signature;
  final String? detail;
  final String? insertText;

  const AutocompleteCandidate({
    required this.word,
    this.type,
    this.signature,
    this.detail,
    this.insertText,
  });
}

class AutocompleteController extends ChangeNotifier {
  String _prefix = '';
  List<AutocompleteCandidate> _candidates = [];
  int _selectedIndex = 0;
  bool _visible = false;
  bool _fetchingSemantic = false;

  Timer? _debounceTimer;

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
  bool get fetchingSemantic => _fetchingSemantic;

  /// 同步更新：基于前缀过滤静态关键词列表
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

  /// 异步获取语义补全候选（防抖 150ms）
  void fetchSemanticCandidates(
    String source,
    int line,
    int column,
    String textBeforeCursor,
  ) {
    final prefix = extractPrefix(textBeforeCursor);
    _prefix = prefix;
    _debounceTimer?.cancel();

    // 如果没有前缀且不在特定上下文，延迟显示语义候选
    if (prefix.isEmpty) {
      // 只有在成员访问等特定上下文才显示无前缀补全
      // 这里简化处理：总是获取
    }

    _debounceTimer = Timer(const Duration(milliseconds: 150), () async {
      await _doFetchSemantic(source, line, column, prefix);
    });
  }

  Future<void> _doFetchSemantic(
    String source,
    int line,
    int column,
    String prefix,
  ) async {
    _fetchingSemantic = true;
    notifyListeners();

    try {
      final results = await rust.getCompletionCandidates(
        source: source,
        line: line,
        column: column,
        prefix: prefix,
      );

      final semantic = results.map((r) {
        String? sig;
        if (r.kind == 'function' && r.detail.isNotEmpty) {
          sig = r.detail;
        }
        return AutocompleteCandidate(
          word: r.label,
          type: r.kind,
          signature: sig,
          detail: r.detail.isNotEmpty ? r.detail : null,
          insertText: r.insertText.isNotEmpty ? r.insertText : r.label,
        );
      }).toList();

      // 合并静态候选（作为 fallback，只保留未在语义候选中出现的）
      final existing = <String>{for (var s in semantic) s.word.toLowerCase()};
      final staticFallback = _allWords
          .where((w) =>
              !existing.contains(w.word.toLowerCase()) &&
              w.word.toLowerCase().startsWith(prefix.toLowerCase()))
          .take(4)
          .toList();

      _candidates = [...semantic, ...staticFallback];
      _selectedIndex = 0;
      _visible = _candidates.isNotEmpty;
      notifyListeners();
    } catch (e) {
      // 语义补全失败时静默回退到静态列表
      if (_candidates.isEmpty) {
        updateWithPrefix(prefix);
      }
    } finally {
      _fetchingSemantic = false;
      notifyListeners();
    }
  }

  /// 仅基于前缀更新静态候选（失败回退用）
  void updateWithPrefix(String prefix) {
    _prefix = prefix;
    _candidates = _allWords
        .where((w) => w.word.toLowerCase().startsWith(prefix.toLowerCase()))
        .take(8)
        .toList();
    _selectedIndex = 0;
    _visible = _candidates.isNotEmpty;
    notifyListeners();
  }

  void hide() {
    if (!_visible && !_fetchingSemantic) return;
    _debounceTimer?.cancel();
    _visible = false;
    _fetchingSemantic = false;
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
