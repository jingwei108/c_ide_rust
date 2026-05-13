/// 知识卡片数据模型
/// 对应高频编译错误的教学解释
class KnowledgeCard {
  final String id;
  final String emoji;
  final String title;
  final String explanation;
  final String correctCode;
  final String wrongCode;
  final List<int> relatedErrorCodes;

  const KnowledgeCard({
    required this.id,
    required this.emoji,
    required this.title,
    required this.explanation,
    required this.correctCode,
    required this.wrongCode,
    required this.relatedErrorCodes,
  });

  /// 全部知识卡片库
  static const List<KnowledgeCard> all = [
    KnowledgeCard(
      id: 'missing_semicolon',
      emoji: '🛑',
      title: '缺少分号',
      explanation: 'C 语言中每条语句必须以分号 ; 结尾。如果忘记写分号，编译器会在下一行报错。',
      correctCode: 'int main() {\n    printf("Hello");\n    return 0;\n}',
      wrongCode: 'int main() {\n    printf("Hello")\n    return 0;\n}',
      relatedErrorCodes: [2001, 2002, 2003],
    ),
    KnowledgeCard(
      id: 'missing_brace',
      emoji: '{ }',
      title: '缺少括号',
      explanation: '大括号 {} 用于定义代码块。忘记闭合大括号会导致编译器无法正确识别代码范围。',
      correctCode: 'int main() {\n    if (1) {\n        return 0;\n    }\n}',
      wrongCode: 'int main() {\n    if (1) {\n        return 0;\n}',
      relatedErrorCodes: [2006],
    ),
    KnowledgeCard(
      id: 'missing_paren',
      emoji: '( )',
      title: '缺少圆括号',
      explanation: '函数调用、条件表达式、类型转换等都需要圆括号。忘记闭合圆括号会导致解析错误。',
      correctCode: 'int main() {\n    printf("Hello");\n}',
      wrongCode: 'int main() {\n    printf("Hello";\n}',
      relatedErrorCodes: [2007],
    ),
    KnowledgeCard(
      id: 'missing_quote',
      emoji: '" "',
      title: '缺少引号',
      explanation: '字符串字面量必须用双引号 "" 包裹，字符字面量用单引号 \'\'。忘记闭合引号会导致编译器将后续代码误认为字符串的一部分。',
      correctCode: 'char s[] = "hello";',
      wrongCode: 'char s[] = "hello;',
      relatedErrorCodes: [1002],
    ),
    KnowledgeCard(
      id: 'undeclared_var',
      emoji: '❓',
      title: '变量未声明',
      explanation: '使用变量前必须先声明其类型。C 语言不允许隐式声明变量（与 Python/JavaScript 不同）。',
      correctCode: 'int main() {\n    int x = 10;\n    printf("%d", x);\n}',
      wrongCode: 'int main() {\n    x = 10;\n    printf("%d", x);\n}',
      relatedErrorCodes: [3001, 3002],
    ),
    KnowledgeCard(
      id: 'scanf_address',
      emoji: '&',
      title: 'scanf 忘记取地址',
      explanation: 'scanf 需要传入变量的地址才能将输入值写入变量。必须使用 & 取地址运算符（数组名除外）。',
      correctCode: 'int a;\nscanf("%d", &a);',
      wrongCode: 'int a;\nscanf("%d", a);',
      relatedErrorCodes: [3040],
    ),
    KnowledgeCard(
      id: 'struct_member',
      emoji: '→',
      title: '结构体成员访问',
      explanation: '结构体变量用 . 访问成员，结构体指针用 -> 访问成员。-> 是 (*p).member 的简写。',
      correctCode: 'struct Point p;\np.x = 10;\nstruct Point* q = &p;\nq->x = 20;',
      wrongCode: 'struct Point p;\np->x = 10;',
      relatedErrorCodes: [3041],
    ),
    KnowledgeCard(
      id: 'rvalue_assign',
      emoji: '⚠️',
      title: '给右值赋值',
      explanation: '赋值运算符 = 左边必须是可修改的左值（变量），不能是常量或表达式结果。',
      correctCode: 'int a;\na = 10;',
      wrongCode: 'int a;\n10 = a;',
      relatedErrorCodes: [3042],
    ),
    KnowledgeCard(
      id: 'missing_return',
      emoji: '↩️',
      title: '缺少返回值',
      explanation: '如果函数声明了返回类型（如 int main()），则必须通过 return 语句返回对应类型的值。',
      correctCode: 'int main() {\n    return 0;\n}',
      wrongCode: 'int main() {\n    printf("Hello");\n}',
      relatedErrorCodes: [3043],
    ),
    KnowledgeCard(
      id: 'logic_vs_bitwise',
      emoji: '&&',
      title: '逻辑与位运算符混淆',
      explanation: '&& 是逻辑与（条件判断），& 是按位与（二进制运算）。在 if 条件中几乎总是应该用 && 而不是 &。',
      correctCode: 'if (a > 0 && b > 0) { ... }',
      wrongCode: 'if (a > 0 & b > 0) { ... }',
      relatedErrorCodes: [1004],
    ),
    KnowledgeCard(
      id: 'assignment_in_condition',
      emoji: '==',
      title: '条件内使用 = 而非 ==',
      explanation: '在 if/while 条件中，== 是比较相等，= 是赋值。误用 = 会导致条件恒为真（非零值）并意外修改变量。',
      correctCode: 'if (a == 10) { ... }',
      wrongCode: 'if (a = 10) { ... }',
      relatedErrorCodes: [3050],
    ),
  ];

  /// 根据错误码查找匹配的知识卡片
  static List<KnowledgeCard> findByErrorCode(int code) {
    return all.where((c) => c.relatedErrorCodes.contains(code)).toList();
  }

  /// 根据错误码列表查找所有相关卡片（去重）
  static List<KnowledgeCard> findByErrorCodes(List<int> codes) {
    final seen = <String>{};
    final result = <KnowledgeCard>[];
    for (final code in codes) {
      for (final card in findByErrorCode(code)) {
        if (seen.add(card.id)) {
          result.add(card);
        }
      }
    }
    return result;
  }
}
