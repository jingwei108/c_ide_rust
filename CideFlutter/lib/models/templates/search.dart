import '../code_template.dart';

const List<CodeTemplate> searchTemplates = [
    CodeTemplate(
      'binary',
      '二分查找',
      '查找',
      '#include <stdio.h>\n'
      '\n'
      'int binarySearch(int arr[], int n, int target) {\n'
      '    int left = 0, right = n - 1;\n'
      '    while (left <= right) {\n'
      '        int mid = left + (right - left) / 2;\n'
      '        if (arr[mid] == target) return mid;\n'
      '        if (arr[mid] < target) left = mid + 1;\n'
      '        else right = mid - 1;\n'
      '    }\n'
      '    return -1;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int arr[] = {1, 3, 5, 7, 9};\n'
      '    int n = 5;\n'
      '    int target = {{target:5}};\n'
      '    int result = binarySearch(arr, n, target);\n'
      '    printf("%d\\n", result);\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'target', label: '查找目标', defaultValue: '5', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '区间定义',
          description: '二分查找要求数组已有序。left 和 right 定义当前搜索区间的左右边界。',
          focusLines: [4],
          explanations: [
            LineExplanation(line: 4, short: '闭区间', detail: 'left=0, right=n-1，表示搜索区间是闭区间 [0, n-1]。'),
          ],
        ),
        TutorialStep(
          title: '循环条件',
          description: '只要区间内还有元素，就继续查找。left <= right 意味着区间非空。',
          focusLines: [5],
          explanations: [
            LineExplanation(line: 5, short: '何时停止', detail: '当 left > right 时，区间为空，说明目标不存在。'),
          ],
        ),
        TutorialStep(
          title: '取中点',
          description: '取区间中点进行比较。用 left + (right-left)/2 避免 (left+right) 溢出。',
          focusLines: [6],
          explanations: [
            LineExplanation(line: 6, short: '防溢出', detail: '若用 (left+right)/2，极端大值时可能溢出。这种写法更安全。'),
          ],
        ),
        TutorialStep(
          title: '三种情况',
          description: '比较中点值和目标值：相等则返回；目标大则去右半；目标小则去左半。',
          focusLines: [7, 8, 9],
          explanations: [
            LineExplanation(line: 7, short: '命中', detail: 'arr[mid] == target，直接返回索引 mid。'),
            LineExplanation(line: 8, short: '目标更大', detail: 'target 在 mid 右侧，收缩左边界到 mid+1。'),
            LineExplanation(line: 9, short: '目标更小', detail: 'target 在 mid 左侧，收缩右边界到 mid-1。'),
          ],
        ),
        TutorialStep(
          title: '未找到',
          description: '循环结束说明区间内没有目标值，返回 -1 表示查找失败。',
          focusLines: [10],
          explanations: [
            LineExplanation(line: 10, short: '返回 -1', detail: 'C 语言中常用 -1 表示"未找到"或"无效索引"。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'linear',
      '线性查找',
      '查找',
      '#include <stdio.h>\n'
      '\n'
      'int linearSearch(int arr[], int n, int target) {\n'
      '    for (int i = 0; i < n; i++) {\n'
      '        if (arr[i] == target)\n'
      '            return i;\n'
      '    }\n'
      '    return -1;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int arr[] = {10, 20, 30, 40, 50};\n'
      '    int n = 5;\n'
      '    int target = {{target:30}};\n'
      '    int result = linearSearch(arr, n, target);\n'
      '    printf("%d\\n", result);\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'target', label: '查找目标', defaultValue: '30', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '逐个扫描',
          description: '线性查找从左到右逐个比较，直到找到目标或遍历完数组。',
          focusLines: [4, 5, 6],
          explanations: [
            LineExplanation(line: 4, short: '遍历数组', detail: 'i 从 0 到 n-1，逐个检查。'),
            LineExplanation(line: 5, short: '比较', detail: 'arr[i] == target 表示找到目标。'),
            LineExplanation(line: 6, short: '返回索引', detail: '找到后立即返回当前索引 i。'),
          ],
        ),
        TutorialStep(
          title: '未找到',
          description: '如果循环结束还没找到，说明目标不在数组中，返回 -1。',
          focusLines: [8],
          explanations: [
            LineExplanation(line: 8, short: '返回 -1', detail: '循环正常结束意味着目标不存在。'),
          ],
        ),
      ],
    ),
    // ========== 数据结构（含参数 + 教程） ==========

    CodeTemplate(
      'interpolationSearch',
      '插值查找',
      '查找',
      '#include <stdio.h>\n'
      '\n'
      'int interpolationSearch(int arr[], int n, int key) {\n'
      '    int low = 0, high = n - 1;\n'
      '    while (low <= high && key >= arr[low] && key <= arr[high]) {\n'
      '        if (low == high) {\n'
      '            if (arr[low] == key) return low;\n'
      '            return -1;\n'
      '        }\n'
      '        int pos = low + (int)((double)(key - arr[low]) / (arr[high] - arr[low]) * (high - low));\n'
      '        if (arr[pos] == key) return pos;\n'
      '        if (arr[pos] < key) low = pos + 1;\n'
      '        else high = pos - 1;\n'
      '    }\n'
      '    return -1;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int arr[] = {10, 20, 30, 40, 50, 60, 70, 80, 90, 100};\n'
      '    int n = 10;\n'
      '    int key = {{key:30}};\n'
      '    int result = interpolationSearch(arr, n, key);\n'
      '    printf("%d\\n", result);\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'key', label: '查找目标', defaultValue: '30', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '插值公式',
          description: '插值查找是二分查找的改进，按 key 在值域中的比例预测中点位置，适合数据均匀分布。',
          focusLines: [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
          explanations: [
            LineExplanation(line: 5, short: '边界保护', detail: 'key 必须在 [arr[low], arr[high]] 范围内，否则直接结束。'),
            LineExplanation(line: 10, short: '插值位置', detail: 'pos = low + (key-arr[low])/(arr[high]-arr[low])*(high-low)，按值比例预测。'),
            LineExplanation(line: 12, short: '调整区间', detail: '与二分查找类似，命中返回，否则缩小区间。'),
          ],
        ),
        TutorialStep(
          title: '与二分查找对比',
          description: '二分查找总是取中点，插值查找则根据 key 的大小自适应选择更靠近目标的位置。在数据均匀时，平均时间复杂度接近 O(log log n)。',
          focusLines: [4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
          explanations: [
            LineExplanation(line: 10, short: '自适应中点', detail: '如果 key 接近 arr[high]，pos 会偏向 high 一侧。'),
            LineExplanation(line: 5, short: '不均匀数据', detail: '数据分布不均匀时，插值查找可能退化为 O(n)。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'fibonacciSearch',
      '斐波那契查找',
      '查找',
      '#include <stdio.h>\n'
      '\n'
      'int fib[20];\n'
      '\n'
      'void initFib() {\n'
      '    fib[0] = 0;\n'
      '    fib[1] = 1;\n'
      '    for (int i = 2; i < 20; i++)\n'
      '        fib[i] = fib[i - 1] + fib[i - 2];\n'
      '}\n'
      '\n'
      'int fibonacciSearch(int arr[], int n, int key) {\n'
      '    int k = 0;\n'
      '    while (fib[k] < n + 1) k++;\n'
      '    int temp[20];\n'
      '    for (int i = 0; i < n; i++) temp[i] = arr[i];\n'
      '    for (int i = n; i < fib[k]; i++) temp[i] = arr[n - 1];\n'
      '    int low = 0, high = n - 1;\n'
      '    while (low <= high) {\n'
      '        int mid = low + fib[k - 1] - 1;\n'
      '        if (key < temp[mid]) {\n'
      '            high = mid - 1;\n'
      '            k = k - 1;\n'
      '        } else if (key > temp[mid]) {\n'
      '            low = mid + 1;\n'
      '            k = k - 2;\n'
      '        } else {\n'
      '            if (mid < n) return mid;\n'
      '            else return n - 1;\n'
      '        }\n'
      '    }\n'
      '    return -1;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    initFib();\n'
      '    int arr[] = {10, 20, 30, 40, 50, 60, 70, 80};\n'
      '    int n = 8;\n'
      '    int key = {{key:30}};\n'
      '    int result = fibonacciSearch(arr, n, key);\n'
      '    printf("%d\\n", result);\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'key', label: '查找目标', defaultValue: '30', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '斐波那契数列',
          description: '先预计算斐波那契数列。查找时利用 fib[k] = fib[k-1] + fib[k-2] 的性质划分区间。',
          focusLines: [3, 4, 5, 6, 7, 8, 9],
          explanations: [
            LineExplanation(line: 6, short: '初始化', detail: 'fib[0]=0, fib[1]=1，后续每项为前两项之和。'),
          ],
        ),
        TutorialStep(
          title: '区间划分',
          description: '找到满足 fib[k] >= n+1 的最小 k。把数组补齐到长度 fib[k]，然后按 fib[k-1] 和 fib[k-2] 划分左右区间。',
          focusLines: [12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33],
          explanations: [
            LineExplanation(line: 13, short: '找最小 k', detail: 'fib[k] < n+1 时 k 不够大，需要继续增大。'),
            LineExplanation(line: 17, short: '填充', detail: '用末尾元素填充到 fib[k] 长度，保证比较时不出错。'),
            LineExplanation(line: 20, short: '中点公式', detail: 'mid = low + fib[k-1] - 1，左半区间长度为 fib[k-1]。'),
            LineExplanation(line: 23, short: '去左半', detail: 'key < temp[mid] 时去左半，k = k-1。'),
            LineExplanation(line: 26, short: '去右半', detail: 'key > temp[mid] 时去右半，k = k-2。'),
          ],
        ),
      ],
    ),
];
