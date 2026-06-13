import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../models/ide_state.dart';
import 'ide_notifier.dart';

export 'ide_notifier.dart';
export '../models/ide_state.dart';

final ideProvider = AutoDisposeNotifierProvider<IdeNotifier, IdeState>(IdeNotifier.new);
