import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../models/ide_state.dart';
import 'ide_notifier.dart';

export 'ide_notifier.dart';
export '../models/ide_state.dart';
export '../services/rust_api_service.dart' show RustApiService, rustApiServiceProvider;

final ideProvider = AutoDisposeNotifierProvider<IdeNotifier, IdeState>(IdeNotifier.new);
