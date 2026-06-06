export 'code_template.dart';
export 'template_loader.dart';

import 'code_template.dart';
import 'template_loader.dart';

/// 运行时从 assets 加载模板的 Future（单例，只加载一次）。
final _templatesFuture = TemplateLoader.load();

Future<List<CodeTemplate>> getDynamicTemplates() => _templatesFuture;
