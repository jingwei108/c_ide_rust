import 'dart:convert';
import 'dart:typed_data';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:cide/models/template_loader.dart';

/// 内存 AssetBundle，用于测试时完全隔离 rootBundle 缓存。
class _TestAssetBundle extends AssetBundle {
  final Map<String, String> _assets;

  _TestAssetBundle(this._assets);

  @override
  Future<ByteData> load(String key) async {
    final value = _assets[key];
    if (value == null) {
      throw Exception('Unable to load asset: $key');
    }
    return ByteData.view(Uint8List.fromList(utf8.encode(value)).buffer);
  }

  @override
  Future<String> loadString(String key, {bool cache = true}) async {
    final value = _assets[key];
    if (value == null) {
      throw Exception('Unable to load asset: $key');
    }
    return value;
  }

  @override
  Future<T> loadStructuredData<T>(
    String key,
    Future<T> Function(String value) parser,
  ) async {
    return parser(await loadString(key));
  }
}

void main() {
  group('TemplateLoader.load', () {
    const cTemplateCode = '''#include <stdio.h>
int main() {
    int n = /*__PARAM_n__*/ 5;
    printf("%d\\n", n);
    return 0;
}
''';

    const cppTemplateCode = '''#include <stdio.h>
int main() {
    int n = /*__PARAM_n__*/ 5;
    printf("%d\\n", n);
    return 0;
}
''';

    late Map<String, String> assets;

    setUp(() {
      assets = {
        'assets/templates/index.json': jsonEncode({
          'templates': [
            {
              'key': 'hello_c',
              'name': 'Hello C',
              'category': '基础',
              'ext': 'c',
              'params': {
                'n': {'label': 'Count', 'type': 'int', 'default': '5'},
              },
              'tutorialSteps': [
                {
                  'title': 'Step 1',
                  'description': 'Define n',
                  'focusLines': [4],
                  'explanations': [
                    {'line': 4, 'short': '变量定义', 'detail': '定义整数变量 n'},
                  ],
                },
              ],
              'knowledgeNodes': ['Algorithm'],
            },
            {
              'key': 'hello_cpp',
              'name': 'Hello C++',
              'category': 'C++基础',
              'ext': 'cpp',
              'params': {},
              'tutorialSteps': [],
              'knowledgeNodes': ['CppBasic'],
            },
          ],
        }),
        'assets/templates/hello_c.c': cTemplateCode,
        'assets/templates/hello_cpp.cpp': cppTemplateCode,
      };
    });

    test('loads C and C++ templates with correct extension', () async {
      final templates = await TemplateLoader.load(
        bundle: _TestAssetBundle(assets),
      );
      expect(templates.length, 2);

      final cTemplate = templates.firstWhere((t) => t.key == 'hello_c');
      expect(cTemplate.ext, 'c');
      expect(cTemplate.displayName, 'Hello C');
      expect(cTemplate.params.length, 1);
      expect(cTemplate.params.first.key, 'n');
      expect(cTemplate.tutorialSteps.length, 1);
      expect(cTemplate.tutorialSteps.first.focusLines, [4]);

      final cppTemplate = templates.firstWhere((t) => t.key == 'hello_cpp');
      expect(cppTemplate.ext, 'cpp');
      expect(cppTemplate.displayName, 'Hello C++');
    });

    test('throws TemplateLoadException when source asset is missing', () async {
      assets['assets/templates/index.json'] = jsonEncode({
        'templates': [
          {
            'key': 'missing_template',
            'name': 'Missing',
            'category': '测试',
            'ext': 'c',
            'params': {},
            'tutorialSteps': [],
            'knowledgeNodes': [],
          },
        ],
      });
      expect(
        () => TemplateLoader.load(bundle: _TestAssetBundle(assets)),
        throwsA(isA<TemplateLoadException>()),
      );
    });

    test('buildCode replaces placeholders using provided values', () async {
      final templates = await TemplateLoader.load(
        bundle: _TestAssetBundle(assets),
      );
      final cTemplate = templates.firstWhere((t) => t.key == 'hello_c');
      final code = cTemplate.buildCode({'n': '10'});
      expect(code, contains('int n = 10;'));
    });
  });
}
