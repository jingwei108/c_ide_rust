import '../code_template.dart';

const List<CodeTemplate> dpTemplates = [
    CodeTemplate(
      'dpFib',
      'DP 斐波那契',
      '动态规划',
      '#include <stdio.h>\n'
      '\n'
      'int main() {\n'
      '    int n = {{n:10}};\n'
      '    int dp[20];\n'
      '    dp[0] = 0;\n'
      '    dp[1] = 1;\n'
      '    for (int i = 2; i <= n; i++) {\n'
      '        dp[i] = dp[i - 1] + dp[i - 2];\n'
      '    }\n'
      '    printf("%d\\n", dp[n]);\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'n', label: '求第 n 项', defaultValue: '10', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '状态定义',
          description: 'dp[i] 表示斐波那契数列的第 i 项。用数组保存中间结果，避免递归的重复计算。',
          focusLines: [4, 5, 6, 7],
          explanations: [
            LineExplanation(line: 4, short: '边界', detail: 'dp[0] = 0 是数列第 0 项。'),
            LineExplanation(line: 5, short: '边界', detail: 'dp[1] = 1 是数列第 1 项。'),
            LineExplanation(line: 6, short: '递推', detail: 'dp[i] = dp[i-1] + dp[i-2]，从底向上填充。'),
          ],
        ),
        TutorialStep(
          title: '与递归对比',
          description: '递归版本 fib(n) 会重复计算 fib(n-2) 等子问题。DP 版本每个子问题只算一次，时间复杂度从指数降为线性。',
          focusLines: [8, 9],
          explanations: [
            LineExplanation(line: 8, short: '循环填充', detail: 'for 循环从 2 到 n 依次计算，保证计算 dp[i] 时 dp[i-1] 和 dp[i-2] 都已知。'),
            LineExplanation(line: 9, short: '结果输出', detail: 'dp[n] 即为所求，时间复杂度 O(n)，空间复杂度 O(n)。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'dpKnapsack',
      '01 背包',
      '动态规划',
      '#include <stdio.h>\n'
      '\n'
      'int max(int a, int b) {\n'
      '    return a > b ? a : b;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int W = {{capacity:10}};\n'
      '    int wt[4] = {2, 3, 4, 5};\n'
      '    int val[4] = {3, 4, 5, 6};\n'
      '    int n = 4;\n'
      '    int dp[5][15];\n'
      '    for (int i = 0; i < 5; i++) {\n'
      '        for (int j = 0; j < 15; j++) {\n'
      '            dp[i][j] = 0;\n'
      '        }\n'
      '    }\n'
      '    for (int i = 1; i <= n; i++) {\n'
      '        for (int w = 1; w <= W; w++) {\n'
      '            if (wt[i - 1] <= w) {\n'
      '                dp[i][w] = max(val[i - 1] + dp[i - 1][w - wt[i - 1]], dp[i - 1][w]);\n'
      '            } else {\n'
      '                dp[i][w] = dp[i - 1][w];\n'
      '            }\n'
      '        }\n'
      '    }\n'
      '    printf("%d\\n", dp[n][W]);\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'capacity', label: '背包容量', defaultValue: '10', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '状态定义',
          description: 'dp[i][w] 表示前 i 个物品、背包容量为 w 时能获得的最大价值。',
          focusLines: [11, 12, 13, 14, 15, 16, 17],
          explanations: [
            LineExplanation(line: 11, short: '表格', detail: 'dp[5][15] 足够存放 4 个物品、容量 10 以内的所有状态。'),
            LineExplanation(line: 12, short: '初始化', detail: '用双重循环把 dp 全部清零，表示初始价值为 0。'),
          ],
        ),
        TutorialStep(
          title: '状态转移',
          description: '对每个物品，有两种选择：放或不放。如果重量 wt[i-1] 不超过当前容量 w，取两者最大值；否则只能不放。',
          focusLines: [18, 19, 20, 21, 22, 23, 24, 25],
          explanations: [
            LineExplanation(line: 19, short: '能放下', detail: 'wt[i-1] <= w 表示当前物品可以放入背包。'),
            LineExplanation(line: 20, short: '放或不放', detail: 'max(放入后的价值, 不放的价值)。放入后的价值 = 当前物品价值 + 剩余容量的最优价值。'),
            LineExplanation(line: 22, short: '放不下', detail: 'wt[i-1] > w 时只能继承前 i-1 个物品的最优值。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'dpLCS',
      '最长公共子序列',
      '动态规划',
      '#include <stdio.h>\n'
      '#include <string.h>\n'
      '\n'
      'int max(int a, int b) { return a > b ? a : b; }\n'
      '\n'
      'int lcs(char X[], char Y[]) {\n'
      '    int m = strlen(X);\n'
      '    int n = strlen(Y);\n'
      '    int dp[20][20];\n'
      '    for (int i = 0; i <= m; i++)\n'
      '        for (int j = 0; j <= n; j++)\n'
      '            dp[i][j] = 0;\n'
      '    for (int i = 1; i <= m; i++) {\n'
      '        for (int j = 1; j <= n; j++) {\n'
      '            if (X[i - 1] == Y[j - 1])\n'
      '                dp[i][j] = dp[i - 1][j - 1] + 1;\n'
      '            else\n'
      '                dp[i][j] = max(dp[i - 1][j], dp[i][j - 1]);\n'
      '        }\n'
      '    }\n'
      '    return dp[m][n];\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    char X[] = "ABCBDAB";\n'
      '    char Y[] = "BDCABA";\n'
      '    printf("%d\\n", lcs(X, Y));\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '状态定义',
          description: 'dp[i][j] 表示 X[0..i-1] 和 Y[0..j-1] 的最长公共子序列长度。',
          focusLines: [6, 7, 8, 9, 10, 11, 12, 13],
          explanations: [
            LineExplanation(line: 9, short: '初始化', detail: 'dp 数组全部清零，任意串与空串的 LCS 为 0。'),
          ],
        ),
        TutorialStep(
          title: '状态转移',
          description: '如果 X[i-1] == Y[j-1]，则该字符属于 LCS；否则取不包含 X[i-1] 或不包含 Y[j-1] 两种情况的最大值。',
          focusLines: [14, 15, 16, 17, 18, 19],
          explanations: [
            LineExplanation(line: 15, short: '字符相等', detail: 'X[i-1] == Y[j-1] 时，LCS 长度加 1。'),
            LineExplanation(line: 17, short: '字符不等', detail: '取 max(dp[i-1][j], dp[i][j-1])，即舍弃其中一个字符。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'dpLIS',
      '最长递增子序列',
      '动态规划',
      '#include <stdio.h>\n'
      '\n'
      'int max(int a, int b) { return a > b ? a : b; }\n'
      '\n'
      'int lis(int arr[], int n) {\n'
      '    int dp[20];\n'
      '    for (int i = 0; i < n; i++) dp[i] = 1;\n'
      '    int result = 1;\n'
      '    for (int i = 1; i < n; i++) {\n'
      '        for (int j = 0; j < i; j++) {\n'
      '            if (arr[j] < arr[i])\n'
      '                dp[i] = max(dp[i], dp[j] + 1);\n'
      '        }\n'
      '        result = max(result, dp[i]);\n'
      '    }\n'
      '    return result;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int arr[] = {10, 9, 2, 5, 3, 7, 101, 18};\n'
      '    int n = 8;\n'
      '    printf("%d\\n", lis(arr, n));\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '状态定义',
          description: 'dp[i] 表示以 arr[i] 结尾的最长递增子序列长度。初始时每个元素自身构成长度为 1 的 LIS。',
          focusLines: [6, 7, 8, 9],
          explanations: [
            LineExplanation(line: 7, short: '初始化', detail: 'dp[i] = 1，每个元素单独构成长度为 1 的递增子序列。'),
          ],
        ),
        TutorialStep(
          title: '状态转移',
          description: '对每个 i，扫描所有 j < i。如果 arr[j] < arr[i]，说明 arr[i] 可以接在 arr[j] 后面，更新 dp[i]。',
          focusLines: [10, 11, 12, 13, 14, 15],
          explanations: [
            LineExplanation(line: 11, short: '扫描前面', detail: 'j 从 0 到 i-1，检查 arr[j] 能否作为 arr[i] 的前驱。'),
            LineExplanation(line: 13, short: '更新 dp', detail: 'dp[i] = max(dp[i], dp[j]+1)，取所有可行前驱中的最大值。'),
            LineExplanation(line: 15, short: '全局最大', detail: 'result 记录所有 dp[i] 中的最大值，即 LIS 长度。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'dpCoinChange',
      '硬币找零',
      '动态规划',
      '#include <stdio.h>\n'
      '\n'
      'int min(int a, int b) { return a < b ? a : b; }\n'
      '\n'
      'int coinChange(int coins[], int coinCount, int amount) {\n'
      '    int dp[101];\n'
      '    for (int i = 0; i <= amount; i++) dp[i] = 100000;\n'
      '    dp[0] = 0;\n'
      '    for (int i = 0; i < coinCount; i++) {\n'
      '        for (int j = coins[i]; j <= amount; j++) {\n'
      '            dp[j] = min(dp[j], dp[j - coins[i]] + 1);\n'
      '        }\n'
      '    }\n'
      '    if (dp[amount] == 100000) return -1;\n'
      '    return dp[amount];\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int coins[] = {1, 2, 5};\n'
      '    int amount = {{amount:11}};\n'
      '    int result = coinChange(coins, 3, amount);\n'
      '    printf("%d\\n", result);\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'amount', label: '目标金额', defaultValue: '11', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '状态定义',
          description: 'dp[i] 表示凑成金额 i 所需的最少硬币数。初始化为极大值，dp[0] = 0。',
          focusLines: [6, 7, 8, 9],
          explanations: [
            LineExplanation(line: 7, short: '初始化', detail: 'dp[i] = 100000 表示暂不可达，用一个很大的数代替无穷大。'),
            LineExplanation(line: 8, short: '边界', detail: 'dp[0] = 0，凑成金额 0 不需要任何硬币。'),
          ],
        ),
        TutorialStep(
          title: '完全背包转移',
          description: '每种硬币可以无限使用。外层遍历硬币，内层从硬币面值开始正序遍历金额，更新 dp[j]。',
          focusLines: [10, 11, 12, 13, 14],
          explanations: [
            LineExplanation(line: 11, short: '正序遍历', detail: 'j 从 coins[i] 到 amount 正序，保证每种硬币可以重复选取。'),
            LineExplanation(line: 12, short: '状态转移', detail: 'dp[j] = min(dp[j], dp[j-coins[i]]+1)，取不选或选一枚当前硬币的最小值。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'matrixChain',
      '矩阵连乘',
      '动态规划',
      '#include <stdio.h>\n'
      '\n'
      'int matrixChainOrder(int p[], int n) {\n'
      '    int dp[10][10];\n'
      '    for (int i = 0; i < n; i++)\n'
      '        for (int j = 0; j < n; j++)\n'
      '            dp[i][j] = 0;\n'
      '    for (int len = 2; len < n; len++) {\n'
      '        for (int i = 1; i < n - len + 1; i++) {\n'
      '            int j = i + len - 1;\n'
      '            dp[i][j] = 100000000;\n'
      '            for (int k = i; k < j; k++) {\n'
      '                int cost = dp[i][k] + dp[k + 1][j] + p[i - 1] * p[k] * p[j];\n'
      '                if (cost < dp[i][j]) dp[i][j] = cost;\n'
      '            }\n'
      '        }\n'
      '    }\n'
      '    return dp[1][n - 1];\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int p[] = {30, 35, 15, 5, 10, 20, 25};\n'
      '    int n = 6;\n'
      '    printf("%d\\n", matrixChainOrder(p, n + 1));\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '状态定义',
          description: 'dp[i][j] 表示计算矩阵 Ai..j 的最少乘法次数。i == j 时为 0（单个矩阵无需乘法）。',
          focusLines: [4, 5, 6, 7, 8],
          explanations: [
            LineExplanation(line: 6, short: '初始化', detail: 'dp 全部清零，单个矩阵的乘法次数为 0。'),
          ],
        ),
        TutorialStep(
          title: '区间 DP',
          description: '按链长 len 从小到大枚举。对每个区间 [i,j]，枚举分割点 k，取最小值。',
          focusLines: [9, 10, 11, 12, 13, 14, 15, 16, 17],
          explanations: [
            LineExplanation(line: 10, short: '区间长度', detail: 'len 从 2 开始，表示至少两个矩阵相乘。'),
            LineExplanation(line: 13, short: '初始化极大值', detail: 'dp[i][j] 先设为极大值，准备取 min。'),
            LineExplanation(line: 15, short: '分割代价', detail: 'cost = dp[i][k] + dp[k+1][j] + p[i-1]*p[k]*p[j]，即左右子链代价加合并代价。'),
          ],
        ),
      ],
    ),
];
