import '../code_template.dart';

const List<CodeTemplate> stringTemplates = [
    CodeTemplate(
      'stringReverse',
      '字符串反转',
      '字符串',
      '#include <stdio.h>\n'
      '\n'
      'int main() {\n'
      '    char str[] = "hello";\n'
      '    int len = 0;\n'
      '    while (str[len] != \'\\0\') {\n'
      '        len++;\n'
      '    }\n'
      '    for (int i = 0; i < len / 2; i++) {\n'
      '        char temp = str[i];\n'
      '        str[i] = str[len - i - 1];\n'
      '        str[len - i - 1] = temp;\n'
      '    }\n'
      '    printf("%s\\n", str);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '求字符串长度',
          description: 'C 语言字符串以 \\0 结尾。通过 while 循环逐个扫描字符，直到遇到 \\0，统计出长度。',
          focusLines: [4, 5, 6, 7],
          explanations: [
            LineExplanation(line: 4, short: '字符数组', detail: 'char str[] = "hello"，编译器自动分配 6 字节（含 \\0）。'),
            LineExplanation(line: 5, short: '初始化', detail: 'len = 0，从第一个字符开始计数。'),
            LineExplanation(line: 6, short: '循环条件', detail: 'str[len] != \\0 表示还没到达字符串结尾。'),
            LineExplanation(line: 7, short: '计数', detail: 'len++，每遇到一个字符长度加 1。'),
          ],
        ),
        TutorialStep(
          title: '双指针交换',
          description: '只交换前半部分和对应的后半部分。i 从头向中间走，len-i-1 从尾向中间走，两两交换。',
          focusLines: [8, 9, 10, 11],
          explanations: [
            LineExplanation(line: 8, short: '循环范围', detail: 'i < len / 2，只需要交换前半部分，后半部分自然配对。'),
            LineExplanation(line: 9, short: '暂存', detail: 'temp = str[i]，保存左侧字符。'),
            LineExplanation(line: 10, short: '左←右', detail: 'str[i] = str[len-i-1]，把右侧字符放到左侧。'),
            LineExplanation(line: 11, short: '右←暂存', detail: 'str[len-i-1] = temp，把左侧字符放到右侧，完成交换。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'stringMatchBF',
      '朴素模式匹配',
      '字符串',
      '#include <stdio.h>\n'
      '#include <string.h>\n'
      '\n'
      'int indexBF(char S[], char T[], int pos) {\n'
      '    int i = pos;\n'
      '    int j = 0;\n'
      '    int lenS = strlen(S);\n'
      '    int lenT = strlen(T);\n'
      '    while (i < lenS && j < lenT) {\n'
      '        if (S[i] == T[j]) {\n'
      '            i++;\n'
      '            j++;\n'
      '        } else {\n'
      '            i = i - j + 1;\n'
      '            j = 0;\n'
      '        }\n'
      '    }\n'
      '    if (j >= lenT) return i - lenT;\n'
      '    else return -1;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    char S[] = "ababcabcacbab";\n'
      '    char T[] = "abcac";\n'
      '    int pos = indexBF(S, T, 0);\n'
      '    printf("%d\\n", pos);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: 'BF 算法思想',
          description: '朴素模式匹配用双指针 i 和 j 分别扫描主串 S 和模式串 T。如果字符相等就同时后移；不等时 i 回溯到本次起始位置的下一个，j 归零。',
          focusLines: [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20],
          explanations: [
            LineExplanation(line: 5, short: '主串指针', detail: 'i 扫描主串 S，初始从 pos 开始。'),
            LineExplanation(line: 6, short: '模式串指针', detail: 'j 扫描模式串 T，初始为 0。'),
            LineExplanation(line: 10, short: '匹配成功', detail: 'S[i] == T[j] 时两指针同时后移，继续比较下一对字符。'),
            LineExplanation(line: 14, short: '主串回溯', detail: 'i = i - j + 1，i 回到本次匹配起始位置的下一个位置。'),
            LineExplanation(line: 15, short: '模式串归零', detail: 'j = 0，模式串从头开始重新比较。'),
            LineExplanation(line: 19, short: '匹配成功', detail: 'j >= lenT 说明模式串全部匹配完，返回起始位置。'),
          ],
        ),
        TutorialStep(
          title: '时间复杂度',
          description: '最坏情况下，BF 算法的时间复杂度为 O(m*n)，例如主串为 "aaaaab"、模式串为 "aab" 时，每次都在最后失配。',
          focusLines: [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20],
          explanations: [
            LineExplanation(line: 14, short: '回溯代价', detail: 'i = i - j + 1 导致主串指针反复回退，效率低下。'),
            LineExplanation(line: 5, short: '最优情况', detail: '如果第一次就匹配成功，时间复杂度为 O(m)。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'stringMatchKMP',
      'KMP 模式匹配',
      '字符串',
      '#include <stdio.h>\n'
      '#include <string.h>\n'
      '\n'
      'void getNext(char T[], int next[]) {\n'
      '    int j = 0, k = -1;\n'
      '    next[0] = -1;\n'
      '    int lenT = strlen(T);\n'
      '    while (j < lenT - 1) {\n'
      '        if (k == -1 || T[j] == T[k]) {\n'
      '            j++;\n'
      '            k++;\n'
      '            next[j] = k;\n'
      '        } else {\n'
      '            k = next[k];\n'
      '        }\n'
      '    }\n'
      '}\n'
      '\n'
      'int indexKMP(char S[], char T[], int pos) {\n'
      '    int i = pos, j = 0;\n'
      '    int next[20];\n'
      '    getNext(T, next);\n'
      '    int lenS = strlen(S);\n'
      '    int lenT = strlen(T);\n'
      '    while (i < lenS && j < lenT) {\n'
      '        if (j == -1 || S[i] == T[j]) {\n'
      '            i++;\n'
      '            j++;\n'
      '        } else {\n'
      '            j = next[j];\n'
      '        }\n'
      '    }\n'
      '    if (j >= lenT) return i - lenT;\n'
      '    else return -1;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    char S[] = "ababcabcacbab";\n'
      '    char T[] = "abcac";\n'
      '    int pos = indexKMP(S, T, 0);\n'
      '    printf("%d\\n", pos);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: 'next 数组',
          description: 'KMP 的核心是 next 数组，next[j] 表示当 T[j] 失配时，j 应该回溯到的位置。利用模式串自身的重复结构避免主串指针回溯。',
          focusLines: [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
          explanations: [
            LineExplanation(line: 5, short: '双指针', detail: 'j 扫描模式串，k 记录最长相等前后缀长度。'),
            LineExplanation(line: 6, short: '初值', detail: 'next[0] = -1，首字符失配时特殊处理。'),
            LineExplanation(line: 10, short: '匹配', detail: 'T[j] == T[k] 说明最长相等前后缀可以延长。'),
            LineExplanation(line: 13, short: '填充 next', detail: 'next[j] = k，表示 j 位置失配时应回溯到 k。'),
            LineExplanation(line: 16, short: '失配回溯', detail: 'k = next[k]，利用已算出的 next 继续寻找更短的前后缀。'),
          ],
        ),
        TutorialStep(
          title: '无回溯匹配',
          description: '匹配过程中，主串指针 i 永不后退，只有模式串指针 j 根据 next 数组调整。',
          focusLines: [19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35],
          explanations: [
            LineExplanation(line: 21, short: '预处理', detail: '先调用 getNext 计算模式串的 next 数组。'),
            LineExplanation(line: 27, short: '推进条件', detail: 'j == -1 表示模式串首字符也失配，i 和 j 都前进。'),
            LineExplanation(line: 28, short: '匹配', detail: 'S[i] == T[j] 时两指针同步后移。'),
            LineExplanation(line: 31, short: 'j 回溯', detail: 'j = next[j]，i 不动，只调整模式串位置。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'stringBasicOps',
      '串基本操作',
      '字符串',
      '#include <stdio.h>\n'
      '#include <string.h>\n'
      '\n'
      'void strAssign(char T[], char chars[]) {\n'
      '    int i = 0;\n'
      '    while (chars[i] != \'\\0\') {\n'
      '        T[i] = chars[i];\n'
      '        i++;\n'
      '    }\n'
      '    T[i] = \'\\0\';\n'
      '}\n'
      '\n'
      'int strCompare(char S[], char T[]) {\n'
      '    int i = 0;\n'
      '    while (S[i] != \'\\0\' && T[i] != \'\\0\') {\n'
      '        if (S[i] != T[i])\n'
      '            return S[i] - T[i];\n'
      '        i++;\n'
      '    }\n'
      '    return S[i] - T[i];\n'
      '}\n'
      '\n'
      'void subString(char Sub[], char S[], int pos, int len) {\n'
      '    int j = 0;\n'
      '    for (int i = pos; i < pos + len; i++) {\n'
      '        Sub[j++] = S[i];\n'
      '    }\n'
      '    Sub[j] = \'\\0\';\n'
      '}\n'
      '\n'
      'void concat(char T[], char S1[], char S2[]) {\n'
      '    int i = 0, j = 0;\n'
      '    while (S1[i] != \'\\0\') {\n'
      '        T[j++] = S1[i++];\n'
      '    }\n'
      '    i = 0;\n'
      '    while (S2[i] != \'\\0\') {\n'
      '        T[j++] = S2[i++];\n'
      '    }\n'
      '    T[j] = \'\\0\';\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    char s1[20], s2[20], sub[10], con[40];\n'
      '    strAssign(s1, "hello");\n'
      '    strAssign(s2, "world");\n'
      '    printf("%d\\n", strCompare(s1, s2));\n'
      '    subString(sub, s1, 1, 3);\n'
      '    printf("%s\\n", sub);\n'
      '    concat(con, s1, s2);\n'
      '    printf("%s\\n", con);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '串赋值与比较',
          description: '串赋值逐个字符拷贝，直到遇到结束符。串比较按字典序逐个字符比较ASCII码值。',
          focusLines: [4, 5, 6, 7, 8, 9, 10, 11, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22],
          explanations: [
            LineExplanation(line: 7, short: '逐字符拷贝', detail: 'while 循环逐个把 chars 的字符复制到 T，直到遇到 \\0。'),
            LineExplanation(line: 10, short: '结束符', detail: 'T[i] = \\0，确保新串正确结尾。'),
            LineExplanation(line: 17, short: '逐字符比较', detail: 'S[i] != T[i] 时返回两者ASCII差值，差值为正则 S 更大。'),
          ],
        ),
        TutorialStep(
          title: '求子串与连接',
          description: 'subString 从主串的 pos 位置开始取 len 个字符。concat 把 S1 和 S2 首尾相接。',
          focusLines: [24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40],
          explanations: [
            LineExplanation(line: 27, short: '子串范围', detail: 'i 从 pos 到 pos+len-1，共取 len 个字符。'),
            LineExplanation(line: 29, short: '子串结尾', detail: 'Sub[j] = \\0，给子串加结束符。'),
            LineExplanation(line: 34, short: '拷贝 S1', detail: '先把 S1 的所有字符拷贝到 T。'),
            LineExplanation(line: 37, short: '拷贝 S2', detail: '再接着拷贝 S2，实现两串连接。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'computeNextVal',
      'KMP nextval 优化',
      '字符串',
      '#include <stdio.h>\n'
      '#include <string.h>\n'
      '\n'
      'void getNext(char T[], int next[]) {\n'
      '    int j = 0, k = -1;\n'
      '    next[0] = -1;\n'
      '    int lenT = strlen(T);\n'
      '    while (j < lenT - 1) {\n'
      '        if (k == -1 || T[j] == T[k]) {\n'
      '            j++;\n'
      '            k++;\n'
      '            next[j] = k;\n'
      '        } else {\n'
      '            k = next[k];\n'
      '        }\n'
      '    }\n'
      '}\n'
      '\n'
      'void getNextVal(char T[], int nextval[]) {\n'
      '    int next[20];\n'
      '    getNext(T, next);\n'
      '    int lenT = strlen(T);\n'
      '    nextval[0] = -1;\n'
      '    for (int j = 1; j < lenT; j++) {\n'
      '        if (T[j] == T[next[j]])\n'
      '            nextval[j] = nextval[next[j]];\n'
      '        else\n'
      '            nextval[j] = next[j];\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    char T[] = "ababaaaba";\n'
      '    int nextval[20];\n'
      '    getNextVal(T, nextval);\n'
      '    int lenT = strlen(T);\n'
      '    for (int i = 0; i < lenT; i++)\n'
      '        printf("%d ", nextval[i]);\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: 'next 数组回顾',
          description: '先调用 getNext 计算普通 next 数组。next[j] 表示 j 位置失配时 j 应回溯到的位置。',
          focusLines: [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
          explanations: [
            LineExplanation(line: 5, short: '初始化', detail: 'j=0, k=-1, next[0]=-1，首字符失配时特殊处理。'),
            LineExplanation(line: 10, short: '匹配延长', detail: 'T[j] == T[k] 时最长相等前后缀延长。'),
            LineExplanation(line: 14, short: '失配回溯', detail: 'k = next[k]，利用已算出的 next 寻找更短前后缀。'),
          ],
        ),
        TutorialStep(
          title: 'nextval 优化',
          description: 'nextval 在 next 基础上进一步优化：如果 T[j] == T[next[j]]，则回溯后必然再次失配，可以直接跳到 nextval[next[j]]。',
          focusLines: [19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30],
          explanations: [
            LineExplanation(line: 24, short: '条件判断', detail: 'T[j] == T[next[j]] 说明回溯后字符相同，必然再次失配。'),
            LineExplanation(line: 25, short: '继续跳跃', detail: 'nextval[j] = nextval[next[j]]，跳过无效回溯。'),
            LineExplanation(line: 27, short: '无需优化', detail: '字符不同时，nextval[j] = next[j]，与普通 next 相同。'),
          ],
        ),
      ],
    ),
];
