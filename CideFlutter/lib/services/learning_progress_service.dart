import 'package:shared_preferences/shared_preferences.dart';
import '../models/learning_progress.dart';

/// 学习进度持久化服务
///
/// 使用 `SharedPreferences` 存储 JSON，所有操作均为异步但轻量。
class LearningProgressService {
  static const _key = 'cide_learning_progress';

  static Future<LearningProgress> load() async {
    final prefs = await SharedPreferences.getInstance();
    final jsonStr = prefs.getString(_key);
    if (jsonStr == null || jsonStr.isEmpty) {
      return const LearningProgress();
    }
    try {
      return LearningProgress.fromJsonString(jsonStr);
    } catch (_) {
      return const LearningProgress();
    }
  }

  static Future<void> save(LearningProgress progress) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_key, progress.toJsonString());
  }

  /// 清除所有学习进度（用于重置/调试）
  static Future<void> clear() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_key);
  }
}
