import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/providers/ide_provider.dart';
import 'package:cide/providers/unified_provider.dart';
import 'package:cide/services/rust_api_service.dart';
import '../mocks/rust_api_service_mock.dart';

/// Pump a plain widget inside a [MaterialApp] with a [Scaffold].
///
/// Use this for stateless/stateful widgets that receive all their data via
/// constructor parameters and do not read Riverpod providers directly.
Future<void> pumpWidget(
  WidgetTester tester, {
  required Widget child,
  bool wrapScaffold = true,
}) async {
  await tester.pumpWidget(
    MaterialApp(
      home: wrapScaffold ? Scaffold(body: child) : child,
    ),
  );
}

/// Pump a widget tree inside [ProviderScope] with a mocked Rust API service.
///
/// This is the default helper for widgets that read [ideProvider],
/// [unifiedProvider] or any other Riverpod provider that eventually calls Rust.
/// The returned [ProviderContainer] is kept alive for the duration of the test
/// via an explicit listener.
///
/// Use the [builder] callback to access the created [ProviderContainer] so you
/// can read notifiers (e.g. `container.read(ideProvider.notifier)`) and pass
/// them to widgets that require an [IdeNotifier] instance.
Future<ProviderContainer> pumpAppWithProviders(
  WidgetTester tester, {
  required Widget Function(ProviderContainer container) builder,
  MockRustApiService? mock,
}) async {
  final service = mock ?? MockRustApiService();
  final container = ProviderContainer(
    overrides: [rustApiServiceProvider.overrideWithValue(service)],
  );
  // Keep auto-dispose notifiers alive while the test runs.
  container.listen(ideProvider, (prev, next) {});
  container.listen(unifiedProvider, (prev, next) {});

  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: MaterialApp(
        home: Scaffold(body: builder(container)),
      ),
    ),
  );

  return container;
}
