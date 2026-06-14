import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/providers/theme_provider.dart';

void main() {
  group('ThemeNotifier', () {
    test('default theme is dark', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);
      expect(container.read(themeProvider), ThemeMode.dark);
    });

    test('toggle switches between dark and light', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(themeProvider.notifier);
      expect(container.read(themeProvider), ThemeMode.dark);

      notifier.toggle();
      expect(container.read(themeProvider), ThemeMode.light);

      notifier.toggle();
      expect(container.read(themeProvider), ThemeMode.dark);
    });

    test('setDark and setLight work', () {
      final container = ProviderContainer();
      addTearDown(container.dispose);

      final notifier = container.read(themeProvider.notifier);
      notifier.setLight();
      expect(container.read(themeProvider), ThemeMode.light);

      notifier.setDark();
      expect(container.read(themeProvider), ThemeMode.dark);
    });
  });
}
