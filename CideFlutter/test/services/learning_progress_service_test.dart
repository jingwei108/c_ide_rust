import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:cide/models/learning_progress.dart';
import 'package:cide/services/learning_progress_service.dart';

void main() {
  setUp(() {
    SharedPreferences.setMockInitialValues({});
  });

  group('LearningProgressService.load', () {
    test('returns default progress when no data', () async {
      final progress = await LearningProgressService.load();
      expect(progress, const LearningProgress());
    });

    test('returns default progress when JSON is invalid', () async {
      SharedPreferences.setMockInitialValues({
        'cide_learning_progress': 'not-json',
      });
      final progress = await LearningProgressService.load();
      expect(progress, const LearningProgress());
    });

    test('deserializes valid JSON', () async {
      const progress = LearningProgress(
        totalCompiles: 5,
        successfulCompiles: 3,
        streakDays: 2,
      );
      SharedPreferences.setMockInitialValues({
        'cide_learning_progress': progress.toJsonString(),
      });

      final loaded = await LearningProgressService.load();
      expect(loaded.totalCompiles, 5);
      expect(loaded.successfulCompiles, 3);
      expect(loaded.streakDays, 2);
    });
  });

  group('LearningProgressService.save', () {
    test('writes JSON to SharedPreferences', () async {
      const progress = LearningProgress(totalCompiles: 7);
      await LearningProgressService.save(progress);

      final prefs = await SharedPreferences.getInstance();
      expect(prefs.getString('cide_learning_progress'), progress.toJsonString());
    });
  });

  group('LearningProgressService.clear', () {
    test('removes stored progress', () async {
      await LearningProgressService.save(const LearningProgress(totalCompiles: 1));
      await LearningProgressService.clear();

      final prefs = await SharedPreferences.getInstance();
      expect(prefs.containsKey('cide_learning_progress'), isFalse);
    });
  });
}
