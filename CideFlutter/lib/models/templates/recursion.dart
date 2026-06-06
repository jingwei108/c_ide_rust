import '../code_template.dart';

const List<CodeTemplate> recursionTemplates = [
    CodeTemplate(
      'hanoi',
      '汉诺塔',
      '递归',
      '#include <stdio.h>\n'
      '\n'
      'void hanoi(int n, char from, char to, char aux) {\n'
      '    if (n == 1) {\n'
      '        printf("Move disk 1 from %c to %c\\n", from, to);\n'
      '        return;\n'
      '    }\n'
      '    hanoi(n - 1, from, aux, to);\n'
      '    printf("Move disk %d from %c to %c\\n", n, from, to);\n'
      '    hanoi(n - 1, aux, to, from);\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int n = {{n:3}};\n'
      '    hanoi(n, \'A\', \'C\', \'B\');\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'n', label: '盘子数量', defaultValue: '3', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '递归思想',
          description: '汉诺塔的核心是分治：要把 n 个盘子从 A 移到 C，只需先把上面 n-1 个移到 B，再把最底下的移到 C，最后把 n-1 个从 B 移到 C。',
          focusLines: [3, 4, 5, 6, 7, 8, 9, 10],
          explanations: [
            LineExplanation(line: 3, short: '参数', detail: 'n 是盘子数，from 是起始柱，to 是目标柱，aux 是辅助柱。'),
            LineExplanation(line: 4, short: '基准情况', detail: 'n == 1 时直接把盘子从 from 移到 to。'),
            LineExplanation(line: 7, short: '第一步', detail: 'hanoi(n-1, from, aux, to)：把上面 n-1 个盘子从起始柱移到辅助柱。'),
            LineExplanation(line: 8, short: '第二步', detail: '把第 n 个（最大的）盘子从起始柱直接移到目标柱。'),
            LineExplanation(line: 9, short: '第三步', detail: 'hanoi(n-1, aux, to, from)：把辅助柱上的 n-1 个盘子移到目标柱。'),
          ],
        ),
        TutorialStep(
          title: '移动次数',
          description: 'n 个盘子的最少移动次数为 2^n - 1。每增加一个盘子，移动次数翻倍再加 1。',
          focusLines: [3, 4, 5, 6, 7, 8, 9, 10],
          explanations: [
            LineExplanation(line: 4, short: '基准', detail: 'T(1) = 1，一个盘子只需移动一次。'),
            LineExplanation(line: 7, short: '递推', detail: 'T(n) = 2*T(n-1) + 1，两次 n-1 的移动加一次最底下的移动。'),
          ],
        ),
      ],
    ),
    // ========== 数据结构（经典教材案例） ==========

    CodeTemplate(
      'factorial',
      '递归阶乘',
      '递归',
      '#include <stdio.h>\n'
      '\n'
      'int factorial(int n) {\n'
      '    if (n <= 1) return 1;\n'
      '    return n * factorial(n - 1);\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int result = factorial(5);\n'
      '    printf("%d\\n", result);\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'n', label: '求 n 的阶乘', defaultValue: '5', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '递归终止条件',
          description: '阶乘的递归定义：n! = n * (n-1)!，终止条件是 0! = 1! = 1。',
          focusLines: [4, 5],
          explanations: [
            LineExplanation(line: 4, short: '基准情况', detail: 'n <= 1 时直接返回 1，这是递归能结束的关键。'),
            LineExplanation(line: 5, short: '递归调用', detail: 'n * factorial(n-1)，把问题分解为更小的子问题。'),
          ],
        ),
        TutorialStep(
          title: '调用栈展开',
          description: '计算 5! 时，调用栈依次展开为 5*4! → 4*3! → 3*2! → 2*1! → 1，然后逐层返回结果。',
          focusLines: [9, 10],
          explanations: [
            LineExplanation(line: 9, short: '触发递归', detail: 'factorial(5) 开始第一次调用，函数内部会继续调用 factorial(4)。'),
            LineExplanation(line: 10, short: '输出结果', detail: '所有递归调用返回后，result = 120。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'fib',
      '斐波那契',
      '递归',
      '#include <stdio.h>\n'
      '\n'
      'int fibonacci(int n) {\n'
      '    if (n <= 1) return n;\n'
      '    return fibonacci(n - 1) + fibonacci(n - 2);\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int result = fibonacci(7);\n'
      '    printf("%d\\n", result);\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'n', label: '求第 n 项', defaultValue: '7', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '递归定义',
          description: '斐波那契数列：F(0)=0, F(1)=1, F(n)=F(n-1)+F(n-2)。有两个递归出口。',
          focusLines: [4, 5, 6],
          explanations: [
            LineExplanation(line: 4, short: '基准情况', detail: 'n <= 1 时返回 n 本身，F(0)=0, F(1)=1。'),
            LineExplanation(line: 5, short: '双重递归', detail: 'fibonacci(n-1) + fibonacci(n-2)，每次分裂为两个子问题。'),
          ],
        ),
        TutorialStep(
          title: '重复计算',
          description: '纯递归实现会重复计算大量子问题，时间复杂度为 O(2^n)。对比 DP 版本可以显著优化。',
          focusLines: [9, 10],
          explanations: [
            LineExplanation(line: 9, short: '触发计算', detail: 'fibonacci(7) 会递归调用 fibonacci(6) 和 fibonacci(5)。'),
            LineExplanation(line: 10, short: '结果', detail: '第 7 项的值为 13。'),
          ],
        ),
      ],
    ),
];
