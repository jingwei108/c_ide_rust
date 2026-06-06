import '../code_template.dart';

const List<CodeTemplate> otherStructTemplates = [
    CodeTemplate(
      'hashTable',
      '哈希表',
      '结构',
      '#include <stdio.h>\n'
      '#define TABLE_SIZE 10\n'
      '\n'
      'struct HashEntry {\n'
      '    int key;\n'
      '    int occupied;\n'
      '};\n'
      '\n'
      'int hash(int key) {\n'
      '    return key % TABLE_SIZE;\n'
      '}\n'
      '\n'
      'void insert(struct HashEntry table[], int key) {\n'
      '    int idx = hash(key);\n'
      '    while (table[idx].occupied) {\n'
      '        idx = (idx + 1) % TABLE_SIZE;\n'
      '    }\n'
      '    table[idx].key = key;\n'
      '    table[idx].occupied = 1;\n'
      '}\n'
      '\n'
      'int search(struct HashEntry table[], int key) {\n'
      '    int idx = hash(key);\n'
      '    while (table[idx].occupied) {\n'
      '        if (table[idx].key == key) return idx;\n'
      '        idx = (idx + 1) % TABLE_SIZE;\n'
      '    }\n'
      '    return -1;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct HashEntry table[TABLE_SIZE];\n'
      '    for (int i = 0; i < TABLE_SIZE; i++) {\n'
      '        table[i].occupied = 0;\n'
      '    }\n'
      '    insert(table, 5);\n'
      '    insert(table, 15);\n'
      '    insert(table, 25);\n'
      '    int idx = search(table, 15);\n'
      '    printf("%d\\n", idx);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '哈希与冲突',
          description: '哈希表通过哈希函数把关键字映射到数组下标。当两个关键字映射到同一位置（冲突）时，用线性探测法向后找空位。',
          focusLines: [4, 5, 6, 7, 9, 10, 11],
          explanations: [
            LineExplanation(line: 5, short: '键', detail: 'key 存储实际关键字。'),
            LineExplanation(line: 6, short: '标记', detail: 'occupied 标记该位置是否被占用。'),
            LineExplanation(line: 10, short: '哈希函数', detail: 'key % TABLE_SIZE，取模运算把 key 映射到数组下标范围。'),
          ],
        ),
        TutorialStep(
          title: '线性探测',
          description: '发生冲突时，从冲突位置开始顺序向后扫描，直到找到空位（插入）或找到目标（查找）。',
          focusLines: [13, 14, 15, 16, 17, 18, 19, 20, 22, 23, 24, 25, 26, 27, 28],
          explanations: [
            LineExplanation(line: 15, short: '探测', detail: 'while (table[idx].occupied) 表示该位置已被占用，继续向后。'),
            LineExplanation(line: 16, short: '步进', detail: 'idx = (idx + 1) % TABLE_SIZE，循环回到数组开头。'),
            LineExplanation(line: 25, short: '查找命中', detail: 'table[idx].key == key 表示找到目标，返回下标。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'unionFind',
      '并查集',
      '结构',
      '#include <stdio.h>\n'
      '\n'
      'void init(int parent[], int n) {\n'
      '    for (int i = 0; i < n; i++) parent[i] = -1;\n'
      '}\n'
      '\n'
      'int Find(int parent[], int x) {\n'
      '    if (parent[x] < 0) return x;\n'
      '    return parent[x] = Find(parent, parent[x]);\n'
      '}\n'
      '\n'
      'void Union(int parent[], int x, int y) {\n'
      '    int root1 = Find(parent, x);\n'
      '    int root2 = Find(parent, y);\n'
      '    if (root1 != root2) {\n'
      '        if (parent[root1] < parent[root2]) {\n'
      '            parent[root1] += parent[root2];\n'
      '            parent[root2] = root1;\n'
      '        } else {\n'
      '            parent[root2] += parent[root1];\n'
      '            parent[root1] = root2;\n'
      '        }\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int parent[10];\n'
      '    init(parent, 5);\n'
      '    Union(parent, 0, 1);\n'
      '    Union(parent, 2, 3);\n'
      '    Union(parent, 1, 2);\n'
      '    printf("%d\\n", Find(parent, 3));\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '初始化',
          description: '并查集用数组 parent 表示森林，负值表示根节点，绝对值表示集合大小。初始时每个元素自成一集合。',
          focusLines: [3, 4, 5],
          explanations: [
            LineExplanation(line: 4, short: 'parent 数组', detail: 'parent[i] = -1 表示 i 是根节点，集合大小为 1。'),
          ],
        ),
        TutorialStep(
          title: '查找与路径压缩',
          description: 'Find 递归查找根节点，并在返回过程中把途经节点的 parent 直接指向根，实现路径压缩。',
          focusLines: [7, 8, 9, 10],
          explanations: [
            LineExplanation(line: 8, short: '找到根', detail: 'parent[x] < 0 表示 x 就是根节点。'),
            LineExplanation(line: 9, short: '路径压缩', detail: 'parent[x] = Find(...)，把 x 直接挂到根节点下，扁平化树高。'),
          ],
        ),
        TutorialStep(
          title: '按秩合并',
          description: 'Union 先找到两个元素的根，把较小的树挂到较大的树下。用负数累加记录新树的大小。',
          focusLines: [12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23],
          explanations: [
            LineExplanation(line: 13, short: '找根', detail: '分别找到 x 和 y 所在集合的根节点。'),
            LineExplanation(line: 16, short: '比较规模', detail: 'parent[root1] < parent[root2] 表示 root1 的集合更大（负数更小）。'),
            LineExplanation(line: 17, short: '累加大小', detail: 'parent[root1] += parent[root2]，更新合并后集合的大小。'),
            LineExplanation(line: 18, short: '挂树', detail: 'parent[root2] = root1，把 root2 挂到 root1 下。'),
          ],
        ),
      ],
    ),
];
