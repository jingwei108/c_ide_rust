import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../models/unified_state.dart';
import 'unified_notifier.dart';

export 'unified_notifier.dart';
export '../models/unified_state.dart';

final unifiedProvider = NotifierProvider<UnifiedNotifier, UnifiedState>(UnifiedNotifier.new);
