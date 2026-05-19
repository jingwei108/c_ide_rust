import 'package:flutter/foundation.dart';

/// ---------------------------------------------------------------------------
/// FindReplaceController — 查找替换状态管理
/// ---------------------------------------------------------------------------

class SearchMatch {
  final int start;
  final int end;
  const SearchMatch({required this.start, required this.end});
}

class FindReplaceController extends ChangeNotifier {
  String _query = '';
  String _replacement = '';
  bool _caseSensitive = false;
  bool _useRegex = false;
  List<SearchMatch> _matches = [];
  int _currentMatchIndex = -1;
  bool _visible = false;

  String get query => _query;
  String get replacement => _replacement;
  bool get caseSensitive => _caseSensitive;
  bool get useRegex => _useRegex;
  List<SearchMatch> get matches => List.unmodifiable(_matches);
  int get currentMatchIndex => _currentMatchIndex;
  bool get visible => _visible;
  bool get hasMatches => _matches.isNotEmpty;

  void show() {
    if (_visible) return;
    _visible = true;
    notifyListeners();
  }

  void hide() {
    if (!_visible) return;
    _visible = false;
    _matches = [];
    _currentMatchIndex = -1;
    notifyListeners();
  }

  void setQuery(String value) {
    if (_query == value) return;
    _query = value;
    notifyListeners();
  }

  void setReplacement(String value) {
    if (_replacement == value) return;
    _replacement = value;
    notifyListeners();
  }

  void toggleCaseSensitive() {
    _caseSensitive = !_caseSensitive;
    notifyListeners();
  }

  void toggleRegex() {
    _useRegex = !_useRegex;
    notifyListeners();
  }

  /// 在文本中搜索匹配项
  void search(String text) {
    _matches = [];
    _currentMatchIndex = -1;

    if (_query.isEmpty) {
      notifyListeners();
      return;
    }

    if (_useRegex) {
      try {
        final pattern = RegExp(
          _query,
          caseSensitive: _caseSensitive,
          multiLine: true,
        );
        for (final match in pattern.allMatches(text)) {
          _matches.add(SearchMatch(start: match.start, end: match.end));
        }
      } catch (_) {
        // 非法正则表达式
      }
    } else {
      final target = _caseSensitive ? _query : _query.toLowerCase();
      final source = _caseSensitive ? text : text.toLowerCase();
      int start = 0;
      while (true) {
        final idx = source.indexOf(target, start);
        if (idx < 0) break;
        _matches.add(SearchMatch(start: idx, end: idx + _query.length));
        start = idx + 1;
      }
    }

    if (_matches.isNotEmpty) {
      _currentMatchIndex = 0;
    }

    notifyListeners();
  }

  void nextMatch() {
    if (_matches.isEmpty) return;
    _currentMatchIndex = (_currentMatchIndex + 1) % _matches.length;
    notifyListeners();
  }

  void previousMatch() {
    if (_matches.isEmpty) return;
    _currentMatchIndex =
        (_currentMatchIndex - 1 + _matches.length) % _matches.length;
    notifyListeners();
  }

  SearchMatch? get currentMatch {
    if (_currentMatchIndex < 0 || _currentMatchIndex >= _matches.length) {
      return null;
    }
    return _matches[_currentMatchIndex];
  }
}
