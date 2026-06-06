import '../code_template.dart';

const List<CodeTemplate> basicTemplates = [
    CodeTemplate(
      'gcd',
      '辗转相除',
      '基础',
      '#include <stdio.h>\n'
      '\n'
      'int gcd(int a, int b) {\n'
      '    while (b != 0) {\n'
      '        int temp = b;\n'
      '        b = a % b;\n'
      '        a = temp;\n'
      '    }\n'
      '    return a;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int a = {{a:48}}, b = {{b:18}};\n'
      '    printf("%d\\n", gcd(a, b));\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'a', label: '第一个数', defaultValue: '48', type: ParamType.int),
        TemplateParam(key: 'b', label: '第二个数', defaultValue: '18', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '欧几里得算法',
          description: '辗转相除法的核心：gcd(a,b) = gcd(b, a mod b)。当 b 变为 0 时，a 就是最大公约数。',
          focusLines: [3, 4, 5, 6, 7, 8, 9],
          explanations: [
            LineExplanation(line: 4, short: '循环条件', detail: 'b != 0 时继续，因为 gcd(a,0) = a。'),
            LineExplanation(line: 5, short: '保存 b', detail: 'temp = b，因为下一步 b 会被覆盖。'),
            LineExplanation(line: 6, short: '取模', detail: 'b = a % b，这是算法的核心递推关系。'),
            LineExplanation(line: 7, short: '更新 a', detail: 'a = temp，原来的 b 变成新的 a。'),
            LineExplanation(line: 9, short: '结果', detail: '循环结束时 b 为 0，a 就是最大公约数。'),
          ],
        ),
        TutorialStep(
          title: '算法原理',
          description: 'gcd(a,b) = gcd(b, a mod b) 的证明：a 和 b 的公约数集合与 b 和 a mod b 的公约数集合完全相同。',
          focusLines: [3, 4, 5, 6, 7, 8, 9],
          explanations: [
            LineExplanation(line: 6, short: '核心递推', detail: 'a % b 是 a 除以 b 的余数，设 a = kb + r，则 gcd(a,b) = gcd(b,r)。'),
            LineExplanation(line: 9, short: '终止', detail: '由于余数严格递减，算法一定在有限步内终止。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'isPrime',
      '判断素数',
      '基础',
      '#include <stdio.h>\n'
      '\n'
      'int isPrime(int n) {\n'
      '    if (n <= 1) return 0;\n'
      '    for (int i = 2; i * i <= n; i++) {\n'
      '        if (n % i == 0) return 0;\n'
      '    }\n'
      '    return 1;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int n = {{n:17}};\n'
      '    if (isPrime(n))\n'
      '        printf("%d is prime\\n", n);\n'
      '    else\n'
      '        printf("%d is not prime\\n", n);\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'n', label: '待判断的数', defaultValue: '17', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '特殊情况',
          description: '小于等于 1 的数都不是素数，直接返回 0。',
          focusLines: [4],
          explanations: [
            LineExplanation(line: 4, short: '排除', detail: 'n <= 1 时直接返回 0（假）。'),
          ],
        ),
        TutorialStep(
          title: '试除法',
          description: '只需要检查 2 到 sqrt(n) 之间的数是否能整除 n。如果都没有，则 n 是素数。',
          focusLines: [5, 6, 7, 8],
          explanations: [
            LineExplanation(line: 5, short: '优化上界', detail: 'i * i <= n 等价于 i <= sqrt(n)，避免浮点运算。'),
            LineExplanation(line: 6, short: '整除判断', detail: 'n % i == 0 说明 i 是 n 的因子，n 不是素数。'),
            LineExplanation(line: 8, short: '是素数', detail: '循环正常结束说明没有因子，返回 1（真）。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'josephus',
      '约瑟夫环',
      '基础',
      '#include <stdio.h>\n'
      '#define N 10\n'
      '\n'
      'int main() {\n'
      '    int alive[N];\n'
      '    for (int i = 0; i < N; i++) alive[i] = 1;\n'
      '    int count = 0, i = 0, remain = N;\n'
      '    int m = {{m:3}};\n'
      '    while (remain > 0) {\n'
      '        if (alive[i]) {\n'
      '            count++;\n'
      '            if (count == m) {\n'
      '                alive[i] = 0;\n'
      '                printf("%d ", i);\n'
      '                count = 0;\n'
      '                remain--;\n'
      '            }\n'
      '        }\n'
      '        i = (i + 1) % N;\n'
      '    }\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'm', label: '报数上限', defaultValue: '3', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '初始化',
          description: '用数组 alive 标记每个人是否存活，1 表示存活。N 个人围成一圈。',
          focusLines: [4, 5, 6, 7, 8],
          explanations: [
            LineExplanation(line: 4, short: '状态数组', detail: 'alive[N] 记录每个人的存活状态。'),
            LineExplanation(line: 5, short: '全部存活', detail: '初始时所有人都 alive[i] = 1。'),
            LineExplanation(line: 7, short: '计数器', detail: 'count 记录当前报数，i 是当前位置，remain 是剩余人数。'),
          ],
        ),
        TutorialStep(
          title: '报数与淘汰',
          description: '从第 0 个人开始，每遇到存活的人就报数。报到 m 的人淘汰（alive 置 0），然后从下一个人重新报数。',
          focusLines: [9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19],
          explanations: [
            LineExplanation(line: 10, short: '存活才报数', detail: 'if (alive[i]) 跳过已淘汰的人。'),
            LineExplanation(line: 12, short: '报到 m', detail: 'count == m 时当前人淘汰。'),
            LineExplanation(line: 13, short: '淘汰', detail: 'alive[i] = 0，标记为已淘汰。'),
            LineExplanation(line: 18, short: '循环移动', detail: 'i = (i + 1) % N，下标绕回数组开头，模拟圆圈。'),
          ],
        ),
      ],
    ),
    // ========== 基础（无参数、无教程） ==========

    CodeTemplate(
      'array',
      '数组遍历',
      '基础',
      '#include <stdio.h>\n'
      '\n'
      'int main() {\n'
      '    int arr[5] = {1, 2, 3, 4, 5};\n'
      '    int sum = 0;\n'
      '    for (int i = 0; i < 5; i++) {\n'
      '        sum = sum + arr[i];\n'
      '    }\n'
      '    printf("%d\\n", sum);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '数组定义与初始化',
          description: '定义一个长度为 5 的整型数组，并用初始化列表直接赋值。',
          focusLines: [4, 5],
          explanations: [
            LineExplanation(line: 4, short: '数组定义', detail: 'int arr[5] = {1,2,3,4,5}，编译器自动把 5 个值依次放入数组。'),
          ],
        ),
        TutorialStep(
          title: '遍历求和',
          description: '用 for 循环从下标 0 扫描到下标 4，逐个累加数组元素到 sum 中。',
          focusLines: [6, 7, 8, 9],
          explanations: [
            LineExplanation(line: 6, short: '循环条件', detail: 'i < 5，数组下标从 0 开始，最后一个有效下标是 4。'),
            LineExplanation(line: 7, short: '累加', detail: 'sum = sum + arr[i]，把当前元素加到累加器中。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'pointer',
      '指针交换',
      '指针',
      '#include <stdio.h>\n'
      '\n'
      'void swap(int* a, int* b) {\n'
      '    int temp = *a;\n'
      '    *a = *b;\n'
      '    *b = temp;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int x = 3, y = 5;\n'
      '    swap(&x, &y);\n'
      '    printf("x=%d y=%d\\n", x, y);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '传址调用',
          description: 'swap 函数的参数是 int* 类型（指向整型的指针），调用时传入 &x 和 &y，即变量的地址。',
          focusLines: [4, 5, 6, 7, 8],
          explanations: [
            LineExplanation(line: 4, short: '形参为指针', detail: 'int* a 表示 a 是一个指针，存储的是某个 int 变量的地址。'),
            LineExplanation(line: 5, short: '解引用取值', detail: '*a 表示取 a 指向地址中的值，也就是 x 的值 3。'),
            LineExplanation(line: 6, short: '修改原变量', detail: '*a = *b 把 b 指向的值赋给 a 指向的地址，即 x = y。'),
          ],
        ),
        TutorialStep(
          title: '取地址运算符',
          description: 'main 函数中用 &x 获取 x 的地址，传递给 swap。这样 swap 内部就能直接修改 main 中的变量。',
          focusLines: [12, 13],
          explanations: [
            LineExplanation(line: 12, short: '取地址', detail: '&x 表示变量 x 在内存中的地址。'),
            LineExplanation(line: 13, short: '验证结果', detail: '交换后 x=5, y=3，说明 swap 确实修改了原变量。'),
          ],
        ),
      ],
    ),
    // ========== 递归（无参数、无教程） ==========

    CodeTemplate(
      'activitySelection',
      '活动安排问题',
      '基础',
      '#include <stdio.h>\n'
      '\n'
      'void selectActivities(int start[], int finish[], int n) {\n'
      '    int i = 0;\n'
      '    printf("%d ", i);\n'
      '    for (int j = 1; j < n; j++) {\n'
      '        if (start[j] >= finish[i]) {\n'
      '            printf("%d ", j);\n'
      '            i = j;\n'
      '        }\n'
      '    }\n'
      '    printf("\\n");\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int start[] = {1, 3, 0, 5, 8, 5};\n'
      '    int finish[] = {2, 4, 6, 7, 9, 9};\n'
      '    int n = 6;\n'
      '    selectActivities(start, finish, n);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '贪心策略',
          description: '活动安排问题的贪心策略：每次都选结束时间最早且与已选活动不冲突的活动。输入已按结束时间升序排列。',
          focusLines: [4, 5, 6, 7, 8, 9, 10, 11],
          explanations: [
            LineExplanation(line: 5, short: '选第一个', detail: '结束最早的活动一定在某个最优解中，先选它。'),
            LineExplanation(line: 7, short: '兼容性检查', detail: 'start[j] >= finish[i] 表示活动 j 与最近选中的活动 i 不冲突。'),
            LineExplanation(line: 9, short: '更新', detail: 'i = j，把 j 作为新的最近选中活动，继续向后检查。'),
          ],
        ),
        TutorialStep(
          title: '最优子结构',
          description: '贪心选择性质保证了局部最优能导致全局最优。每次选择结束最早的活动，为后续留下尽可能多的时间。',
          focusLines: [4, 5, 6, 7, 8, 9, 10, 11],
          explanations: [
            LineExplanation(line: 5, short: '第一个必最优', detail: '设第一个结束的活动为 a1，则必存在一个包含 a1 的最优解。'),
            LineExplanation(line: 6, short: '顺序扫描', detail: 'j 从 1 到 n-1 逐个检查后续活动是否与当前选中活动兼容。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'sieveOfEratosthenes',
      '埃氏筛',
      '基础',
      '#include <stdio.h>\n'
      '#define MAXN 50\n'
      '\n'
      'void sieve(int n) {\n'
      '    int isPrime[MAXN];\n'
      '    for (int i = 0; i <= n; i++) isPrime[i] = 1;\n'
      '    isPrime[0] = isPrime[1] = 0;\n'
      '    for (int i = 2; i * i <= n; i++) {\n'
      '        if (isPrime[i]) {\n'
      '            for (int j = i * i; j <= n; j += i)\n'
      '                isPrime[j] = 0;\n'
      '        }\n'
      '    }\n'
      '    for (int i = 2; i <= n; i++)\n'
      '        if (isPrime[i]) printf("%d ", i);\n'
      '    printf("\\n");\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int n = {{n:30}};\n'
      '    sieve(n);\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'n', label: '筛法上限', defaultValue: '30', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '初始化',
          description: '埃氏筛的核心思想：从小到大，把每个素数的所有倍数标记为合数。初始时假设所有数都是素数。',
          focusLines: [5, 6, 7, 8],
          explanations: [
            LineExplanation(line: 6, short: '全设为素数', detail: 'isPrime[i] = 1 表示假设 i 是素数。'),
            LineExplanation(line: 7, short: '排除 0 和 1', detail: '0 和 1 不是素数，直接标记为 0。'),
          ],
        ),
        TutorialStep(
          title: '筛去合数',
          description: '从 2 开始，如果 i 是素数，就把 i*i, i*(i+1), ... 全部标记为合数。只需要筛到 sqrt(n)。',
          focusLines: [9, 10, 11, 12, 13, 14],
          explanations: [
            LineExplanation(line: 9, short: '上界', detail: 'i * i <= n，如果 i > sqrt(n)，其倍数已经被更小的素数筛过了。'),
            LineExplanation(line: 12, short: '从 i*i 开始', detail: 'i*2, i*3, ..., i*(i-1) 已经被更小的素数筛过，所以从 i*i 开始。'),
            LineExplanation(line: 13, short: '步长为 i', detail: 'j += i，标记 i 的所有倍数。'),
          ],
        ),
      ],
    ),
];
