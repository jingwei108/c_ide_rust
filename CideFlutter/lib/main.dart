import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:cide/src/rust/frb_generated.dart';
import 'package:cide/src/rust/api/cide.dart' as rust;
import 'providers/theme_provider.dart';
import 'screens/ide_screen.dart';
import 'perf/perf_test_screen.dart';

// Guard to make `RustLib.init()` idempotent across multiple `app.main()` calls
// in integration tests.
bool _rustLibInitialized = false;

bool get _perfTestEnabled {
  // 支持环境变量 CIDE_PERF_TEST=1 或命令行参数 --perf-test
  if (Platform.environment['CIDE_PERF_TEST']?.trim() == '1') return true;
  return Platform.executableArguments.contains('--perf-test');
}

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  if (!_rustLibInitialized) {
    await RustLib.init();
    _rustLibInitialized = true;
  }

  SystemChannels.lifecycle.setMessageHandler((msg) async {
    if (msg == AppLifecycleState.detached.toString()) {
      final sessionId = await rust.getCurrentSessionId();
      if (sessionId != BigInt.zero) {
        await rust.destroySession(sessionId: sessionId);
      }
    }
    return msg;
  });

  if (_perfTestEnabled) {
    runApp(const ProviderScope(child: PerfTestApp()));
  } else {
    runApp(const ProviderScope(child: MyApp()));
  }
}

class PerfTestApp extends StatelessWidget {
  const PerfTestApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Cide Perf Test',
      debugShowCheckedModeBanner: false,
      theme: ThemeData.light(useMaterial3: true),
      home: const PerfTestScreen(),
    );
  }
}

class MyApp extends ConsumerWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final themeMode = ref.watch(themeProvider);

    return MaterialApp(
      title: 'Cide',
      debugShowCheckedModeBanner: false,
      themeMode: themeMode,
      theme: _buildLightTheme(),
      darkTheme: _buildDarkTheme(),
      // Phase 0 POC 入口：临时切换到 EditorPocScreen 验证 Gesture Proxy
      // home: const EditorPocScreen(),
      home: const IdeScreen(),
    );
  }

  ThemeData _buildLightTheme() {
    return ThemeData.light(useMaterial3: true).copyWith(
      colorScheme: ColorScheme.fromSeed(
        seedColor: Colors.blue,
        brightness: Brightness.light,
      ),
      appBarTheme: const AppBarTheme(
        backgroundColor: Color(0xfff3f3f3),
        foregroundColor: Colors.black87,
        elevation: 0,
      ),
    );
  }

  ThemeData _buildDarkTheme() {
    return ThemeData.dark(useMaterial3: true).copyWith(
      colorScheme: ColorScheme.fromSeed(
        seedColor: Colors.blue,
        brightness: Brightness.dark,
      ),
      appBarTheme: const AppBarTheme(
        backgroundColor: Color(0xff1e1e1e),
        foregroundColor: Colors.white,
        elevation: 0,
      ),
    );
  }
}
