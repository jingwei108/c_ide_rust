/// 参数类型
enum ParamType { int, string, identifier }

/// 模板参数定义
class TemplateParam {
  final String key;
  final String label;
  final String defaultValue;
  final ParamType type;

  const TemplateParam({
    required this.key,
    required this.label,
    required this.defaultValue,
    this.type = ParamType.int,
  });
}

/// 单行代码解释
class LineExplanation {
  final int line;
  final String short;
  final String detail;

  const LineExplanation({
    required this.line,
    required this.short,
    required this.detail,
  });
}

/// 教程步骤
class TutorialStep {
  final String title;
  final String description;
  final List<int> focusLines;
  final List<LineExplanation> explanations;

  const TutorialStep({
    required this.title,
    required this.description,
    required this.focusLines,
    this.explanations = const [],
  });
}

/// 代码模板
class CodeTemplate {
  final String key;
  final String displayName;
  final String category;
  final String code;
  final List<TemplateParam> params;
  final List<TutorialStep> tutorialSteps;

  const CodeTemplate(
    this.key,
    this.displayName,
    this.category,
    this.code, {
    this.params = const [],
    this.tutorialSteps = const [],
  });

  /// 用学生填入的参数替换代码中的占位符。
  /// 占位符语法: {{key:defaultValue}}
  String buildCode(Map<String, String> values) {
    var result = code;
    final placeholderPattern = RegExp(r'\{\{(\w+):([^}]*)\}\}');
    result = result.replaceAllMapped(placeholderPattern, (match) {
      final paramKey = match.group(1)!;
      final paramDefault = match.group(2)!;
      return values[paramKey] ?? paramDefault;
    });
    return result;
  }

  static const List<CodeTemplate> defaults = [
    // ========== 排序（含参数 + 教程） ==========
    CodeTemplate(
      'bubble',
      '冒泡排序',
      '排序',
      '#include <stdio.h>\n'
      '\n'
      'void bubbleSort(int arr[], int n) {\n'
      '    for (int i = 0; i < n - 1; i++) {\n'
      '        for (int j = 0; j < n - i - 1; j++) {\n'
      '            if (arr[j] > arr[j + 1]) {\n'
      '                int temp = arr[j];\n'
      '                arr[j] = arr[j + 1];\n'
      '                arr[j + 1] = temp;\n'
      '            }\n'
      '        }\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int arr[{{n:5}}] = {5, 3, 8, 1, 2};\n'
      '    int n = {{n:5}};\n'
      '    bubbleSort(arr, n);\n'
      '    for (int i = 0; i < n; i++) {\n'
      '        printf("%d ", arr[i]);\n'
      '    }\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'n', label: '数组长度', defaultValue: '5', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '函数签名',
          description: '冒泡排序接收一个整型数组和它的长度。数组以引用方式传入，所以函数内的修改会影响原始数组。',
          focusLines: [4],
          explanations: [
            LineExplanation(
              line: 4,
              short: '参数说明',
              detail: 'arr[] 是要排序的数组，n 是元素个数。注意数组在 C 中传参时会退化为指针，函数内修改会反映到原数组。',
            ),
          ],
        ),
        TutorialStep(
          title: '外层循环',
          description: '外层循环控制"趟数"。每一趟会把当前未排序区间中最大的元素"冒泡"到正确位置。',
          focusLines: [5],
          explanations: [
            LineExplanation(
              line: 5,
              short: '为什么到 n-1？',
              detail: 'n 个元素只需 n-1 趟。每完成一趟，就有一个最大元素沉到末尾，所以循环上界是 n-1。',
            ),
          ],
        ),
        TutorialStep(
          title: '内层循环与比较',
          description: '内层循环负责相邻元素的比较。每完成一趟，末尾就多了一个有序元素，所以内层循环的上界会逐渐减小。',
          focusLines: [6, 7],
          explanations: [
            LineExplanation(
              line: 6,
              short: '上界 n-i-1',
              detail: 'i 趟已完成，最后 i 个元素已经排好序，无需再比较。所以内层只比较前 n-i-1 个元素。',
            ),
            LineExplanation(
              line: 7,
              short: '逆序条件',
              detail: 'arr[j] > arr[j+1] 表示左边比右边大，即出现了逆序，需要交换。',
            ),
          ],
        ),
        TutorialStep(
          title: '交换操作',
          description: '通过临时变量 temp 完成两个元素的交换。这是经典的"三杯子倒水"思路，需要借助第三个容器。',
          focusLines: [8, 9, 10],
          explanations: [
            LineExplanation(
              line: 8,
              short: '暂存左边',
              detail: '先把 arr[j] 的值存到 temp，防止在下一步被覆盖而丢失。',
            ),
            LineExplanation(
              line: 9,
              short: '右边移到左边',
              detail: '将 arr[j+1] 的值赋给 arr[j]，此时 arr[j] 原来的值已经安全保存在 temp 中。',
            ),
            LineExplanation(
              line: 10,
              short: '暂存值放回右边',
              detail: '把 temp 中保存的原 arr[j] 值赋给 arr[j+1]，交换完成。',
            ),
          ],
        ),
        TutorialStep(
          title: '主函数与运行',
          description: 'main 函数创建数组、调用排序函数，并打印结果。你可以修改数组长度参数来观察不同规模的数据。',
          focusLines: [14, 15, 16, 17, 18, 19, 20],
          explanations: [
            LineExplanation(
              line: 14,
              short: '数组声明',
              detail: '这里用你填写的参数 {{n:5}} 作为数组大小。数组初始化列表中的值会被自动填充。',
            ),
            LineExplanation(
              line: 16,
              short: '调用排序',
              detail: '传入数组名（退化为指针）和长度 n，bubbleSort 会原地修改数组使其有序。',
            ),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'selection',
      '选择排序',
      '排序',
      '#include <stdio.h>\n'
      '\n'
      'void selectionSort(int arr[], int n) {\n'
      '    for (int i = 0; i < n - 1; i++) {\n'
      '        int minIdx = i;\n'
      '        for (int j = i + 1; j < n; j++) {\n'
      '            if (arr[j] < arr[minIdx])\n'
      '                minIdx = j;\n'
      '        }\n'
      '        int temp = arr[i];\n'
      '        arr[i] = arr[minIdx];\n'
      '        arr[minIdx] = temp;\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int arr[{{n:5}}] = {64, 25, 12, 22, 11};\n'
      '    int n = {{n:5}};\n'
      '    selectionSort(arr, n);\n'
      '    for (int i = 0; i < n; i++) {\n'
      '        printf("%d ", arr[i]);\n'
      '    }\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'n', label: '数组长度', defaultValue: '5', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '外层循环',
          description: '选择排序的思路是：第 i 趟从剩余未排序的元素中选出最小值，放到第 i 个位置。',
          focusLines: [4, 5],
          explanations: [
            LineExplanation(line: 4, short: '趟数控制', detail: 'n 个元素需要 n-1 趟，最后一个元素自然有序。'),
            LineExplanation(line: 5, short: '假设最小值', detail: '先假设当前位置 i 就是最小值所在位置。'),
          ],
        ),
        TutorialStep(
          title: '寻找最小值',
          description: '内层循环从 i 的后面开始扫描，找到真正最小值的索引。',
          focusLines: [6, 7, 8],
          explanations: [
            LineExplanation(line: 6, short: '扫描剩余区间', detail: 'j 从 i+1 开始到末尾，逐个比较。'),
            LineExplanation(line: 7, short: '更新最小索引', detail: '一旦发现更小的元素，就更新 minIdx。'),
          ],
        ),
        TutorialStep(
          title: '交换到正确位置',
          description: '一趟结束后，minIdx 指向最小元素。把它交换到第 i 个位置，左侧就有序了。',
          focusLines: [9, 10, 11],
          explanations: [
            LineExplanation(line: 9, short: '暂存当前位置', detail: '用 temp 保存 arr[i] 的值。'),
            LineExplanation(line: 10, short: '最小值上位', detail: '把找到的最小值 arr[minIdx] 放到位置 i。'),
            LineExplanation(line: 11, short: '原值归位', detail: '把 temp 放到 minIdx 位置，完成交换。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'insertion',
      '插入排序',
      '排序',
      '#include <stdio.h>\n'
      '\n'
      'void insertionSort(int arr[], int n) {\n'
      '    for (int i = 1; i < n; i++) {\n'
      '        int key = arr[i];\n'
      '        int j = i - 1;\n'
      '        while (j >= 0 && arr[j] > key) {\n'
      '            arr[j + 1] = arr[j];\n'
      '            j--;\n'
      '        }\n'
      '        arr[j + 1] = key;\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int arr[{{n:5}}] = {12, 11, 13, 5, 6};\n'
      '    int n = {{n:5}};\n'
      '    insertionSort(arr, n);\n'
      '    for (int i = 0; i < n; i++) {\n'
      '        printf("%d ", arr[i]);\n'
      '    }\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'n', label: '数组长度', defaultValue: '5', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '外层循环',
          description: '插入排序把数组分成"已排序"和"未排序"两部分。i 从 1 开始，因为第 0 个元素默认已排序。',
          focusLines: [4],
          explanations: [
            LineExplanation(line: 4, short: '从第二个开始', detail: '认为 arr[0] 单独一个元素已经有序，从 i=1 开始逐个插入。'),
          ],
        ),
        TutorialStep(
          title: '取出待插入元素',
          description: '把当前元素 arr[i] 暂存到 key 中，然后腾出位置。',
          focusLines: [5, 6],
          explanations: [
            LineExplanation(line: 5, short: '保存当前值', detail: 'key = arr[i]，防止在移动过程中被覆盖。'),
            LineExplanation(line: 6, short: '从左侧开始比较', detail: 'j 从 i-1 开始向左扫描，寻找 key 应该插入的位置。'),
          ],
        ),
        TutorialStep(
          title: '移动与插入',
          description: '只要左边的元素比 key 大，就把它向右移一格，直到找到合适位置再把 key 放进去。',
          focusLines: [7, 8, 9, 10],
          explanations: [
            LineExplanation(line: 7, short: '比较并移动', detail: 'arr[j] > key 说明 key 应该插在 arr[j] 前面，所以 arr[j] 右移。'),
            LineExplanation(line: 8, short: '元素右移', detail: 'arr[j+1] = arr[j]，给 key 腾出一个空位。'),
            LineExplanation(line: 9, short: '继续向左', detail: 'j-- 继续与更左边的元素比较。'),
            LineExplanation(line: 10, short: '插入到位', detail: '循环结束时 j 指向比 key 小的元素，key 应放在 j+1 处。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'quick',
      '快速排序',
      '排序',
      '#include <stdio.h>\n'
      '\n'
      'void quickSort(int arr[], int low, int high) {\n'
      '    if (low < high) {\n'
      '        int pivot = partition(arr, low, high);\n'
      '        quickSort(arr, low, pivot - 1);\n'
      '        quickSort(arr, pivot + 1, high);\n'
      '    }\n'
      '}\n'
      '\n'
      'int partition(int arr[], int low, int high) {\n'
      '    int pivot = arr[high];\n'
      '    int i = low - 1;\n'
      '    for (int j = low; j < high; j++) {\n'
      '        if (arr[j] <= pivot) {\n'
      '            i++;\n'
      '            int temp = arr[i];\n'
      '            arr[i] = arr[j];\n'
      '            arr[j] = temp;\n'
      '        }\n'
      '    }\n'
      '    int temp = arr[i + 1];\n'
      '    arr[i + 1] = arr[high];\n'
      '    arr[high] = temp;\n'
      '    return i + 1;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int arr[{{n:5}}] = {10, 7, 8, 9, 1};\n'
      '    int n = {{n:5}};\n'
      '    quickSort(arr, 0, n - 1);\n'
      '    for (int i = 0; i < n; i++) {\n'
      '        printf("%d ", arr[i]);\n'
      '    }\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'n', label: '数组长度', defaultValue: '5', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '递归框架',
          description: '快速排序采用分治策略：选一个基准值，把数组分成"小于基准"和"大于基准"两部分，再递归排序。',
          focusLines: [4, 5, 6, 7, 8],
          explanations: [
            LineExplanation(line: 4, short: '递归终止', detail: 'low < high 时才需要排序，否则区间只有一个元素，天然有序。'),
            LineExplanation(line: 5, short: '分区', detail: 'partition 函数把数组按基准值分成左右两部分，返回基准值的最终位置。'),
            LineExplanation(line: 6, short: '递归左半', detail: '对基准值左侧的子数组递归排序。'),
            LineExplanation(line: 7, short: '递归右半', detail: '对基准值右侧的子数组递归排序。'),
          ],
        ),
        TutorialStep(
          title: '分区函数',
          description: 'partition 选择末尾元素作为基准，把小于等于基准的元素都移到左边。',
          focusLines: [11, 12, 13, 14],
          explanations: [
            LineExplanation(line: 11, short: '选基准', detail: '这里选择 arr[high] 作为基准值 pivot。'),
            LineExplanation(line: 12, short: 'i 指针', detail: 'i 指向"已处理的小于基准"区间的末尾，初始在 low-1。'),
            LineExplanation(line: 13, short: 'j 扫描', detail: 'j 从 low 扫到 high-1，逐个与基准比较。'),
            LineExplanation(line: 14, short: '小于基准', detail: 'arr[j] <= pivot 说明该元素应放在左侧区间。'),
          ],
        ),
        TutorialStep(
          title: '基准归位',
          description: '扫描结束后，i+1 就是基准值应该在的位置。把它与末尾元素交换，返回这个位置。',
          focusLines: [19, 20, 21, 22],
          explanations: [
            LineExplanation(line: 19, short: '暂存基准位置', detail: 'temp 保存 arr[i+1]，也就是基准值该去的地方的当前值。'),
            LineExplanation(line: 20, short: '基准到位', detail: '把基准值 arr[high] 放到 i+1 位置。'),
            LineExplanation(line: 21, short: '原值归位', detail: '把 temp 放到 high 位置，交换完成。'),
            LineExplanation(line: 22, short: '返回位置', detail: '返回 i+1，供递归调用确定左右子区间边界。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'merge',
      '归并排序',
      '排序',
      '#include <stdio.h>\n'
      '\n'
      'void mergeSort(int arr[], int left, int right) {\n'
      '    if (left < right) {\n'
      '        int mid = left + (right - left) / 2;\n'
      '        mergeSort(arr, left, mid);\n'
      '        mergeSort(arr, mid + 1, right);\n'
      '        merge(arr, left, mid, right);\n'
      '    }\n'
      '}\n'
      '\n'
      'void merge(int arr[], int left, int mid, int right) {\n'
      '    int n1 = mid - left + 1;\n'
      '    int n2 = right - mid;\n'
      '    int L[10], R[10];\n'
      '    for (int i = 0; i < n1; i++) L[i] = arr[left + i];\n'
      '    for (int j = 0; j < n2; j++) R[j] = arr[mid + 1 + j];\n'
      '    int i = 0, j = 0, k = left;\n'
      '    while (i < n1 && j < n2) {\n'
      '        if (L[i] <= R[j]) arr[k++] = L[i++];\n'
      '        else arr[k++] = R[j++];\n'
      '    }\n'
      '    while (i < n1) arr[k++] = L[i++];\n'
      '    while (j < n2) arr[k++] = R[j++];\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int arr[{{n:5}}] = {12, 11, 13, 5, 6};\n'
      '    int n = {{n:5}};\n'
      '    mergeSort(arr, 0, n - 1);\n'
      '    for (int i = 0; i < n; i++) {\n'
      '        printf("%d ", arr[i]);\n'
      '    }\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'n', label: '数组长度', defaultValue: '5', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '分治思想',
          description: '归并排序先递归地把数组分成两半，分别排序，再把两个有序数组合并成一个。',
          focusLines: [4, 5, 6, 7, 8],
          explanations: [
            LineExplanation(line: 4, short: '递归终止', detail: 'left < right 时才需要继续拆分。'),
            LineExplanation(line: 5, short: '取中点', detail: 'mid = left + (right-left)/2，防止 (left+right) 溢出。'),
            LineExplanation(line: 6, short: '排序左半', detail: '递归调用，对左半区间 [left, mid] 排序。'),
            LineExplanation(line: 7, short: '排序右半', detail: '递归调用，对右半区间 [mid+1, right] 排序。'),
            LineExplanation(line: 8, short: '合并', detail: '左右都排好序后，调用 merge 合并成整体有序。'),
          ],
        ),
        TutorialStep(
          title: '合并过程',
          description: 'merge 函数把两个有序子数组合并。先拷贝到临时数组，再用双指针逐位比较回填。',
          focusLines: [11, 12, 13, 14, 15, 16],
          explanations: [
            LineExplanation(line: 11, short: '左半长度', detail: 'n1 = mid - left + 1，左半区间元素个数。'),
            LineExplanation(line: 12, short: '右半长度', detail: 'n2 = right - mid，右半区间元素个数。'),
            LineExplanation(line: 13, short: '临时数组', detail: 'L 和 R 分别存放左右两半的数据。这里大小固定为 10，仅作示例。'),
            LineExplanation(line: 14, short: '拷贝左半', detail: '把原数组左半部分复制到临时数组 L。'),
            LineExplanation(line: 15, short: '拷贝右半', detail: '把原数组右半部分复制到临时数组 R。'),
            LineExplanation(line: 16, short: '三指针初始化', detail: 'i 扫描 L，j 扫描 R，k 指向原数组回填位置。'),
          ],
        ),
        TutorialStep(
          title: '双指针归并',
          description: '比较 L[i] 和 R[j]，把较小者放回 arr[k]，直到某一侧耗尽。',
          focusLines: [17, 18, 19],
          explanations: [
            LineExplanation(line: 17, short: '同时有剩余', detail: '只要两边都没耗尽，就继续比较。'),
            LineExplanation(line: 18, short: '左边更小', detail: 'L[i] <= R[j] 时取左边，i++ 和 k++。'),
            LineExplanation(line: 19, short: '右边更小', detail: '否则取右边，j++ 和 k++。'),
          ],
        ),
        TutorialStep(
          title: '处理剩余',
          description: '一侧耗尽后，另一侧可能还有剩余元素，直接依次拷贝回原数组。',
          focusLines: [20, 21],
          explanations: [
            LineExplanation(line: 20, short: '左剩余', detail: '如果 L 还有剩，直接按顺序放回原数组。'),
            LineExplanation(line: 21, short: '右剩余', detail: '如果 R 还有剩，同理直接放回。'),
          ],
        ),
      ],
    ),
    // ========== 查找（含参数 + 教程） ==========
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
      'linked',
      '链表节点',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '\n'
      'struct Node {\n'
      '    int data;\n'
      '    struct Node* next;\n'
      '};\n'
      '\n'
      'struct Node* createNode(int data) {\n'
      '    struct Node* newNode = (struct Node*)malloc(sizeof(struct Node));\n'
      '    newNode->data = data;\n'
      '    newNode->next = NULL;\n'
      '    return newNode;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct Node* head = createNode(1);\n'
      '    head->next = createNode(2);\n'
      '    head->next->next = createNode(3);\n'
      '    printf("Head: %d\\n", head->data);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '节点结构',
          description: '链表节点由数据域和指针域组成。data 存储值，next 指向下一个节点。',
          focusLines: [4, 5, 6],
          explanations: [
            LineExplanation(line: 4, short: '结构体定义', detail: 'struct Node 定义了链表节点的结构。'),
            LineExplanation(line: 5, short: '数据域', detail: 'int data 存储节点的值。'),
            LineExplanation(line: 6, short: '指针域', detail: 'struct Node* next 存放下一个节点的地址，NULL 表示结尾。'),
          ],
        ),
        TutorialStep(
          title: '创建节点',
          description: '用 malloc 在堆上分配内存，初始化数据和 next 指针。',
          focusLines: [9, 10, 11, 12, 13],
          explanations: [
            LineExplanation(line: 9, short: '分配内存', detail: 'malloc(sizeof(struct Node)) 申请一个节点大小的堆内存。'),
            LineExplanation(line: 10, short: '赋值', detail: 'newNode->data = data，给新节点的数据域赋值。'),
            LineExplanation(line: 11, short: '收尾', detail: 'newNode->next = NULL，新节点默认是链表末尾。'),
            LineExplanation(line: 12, short: '返回', detail: '返回新节点的地址，供调用者使用。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'linkedInsert',
      '链表头插法',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '\n'
      'struct Node {\n'
      '    int data;\n'
      '    struct Node* next;\n'
      '};\n'
      '\n'
      'struct Node* createNode(int data) {\n'
      '    struct Node* node = (struct Node*)malloc(sizeof(struct Node));\n'
      '    node->data = data;\n'
      '    node->next = NULL;\n'
      '    return node;\n'
      '}\n'
      '\n'
      'struct Node* insertFront(struct Node* head, int data) {\n'
      '    struct Node* newNode = createNode(data);\n'
      '    newNode->next = head;\n'
      '    return newNode;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct Node* head = NULL;\n'
      '    head = insertFront(head, 3);\n'
      '    head = insertFront(head, 2);\n'
      '    head = insertFront(head, 1);\n'
      '    printf("Head: %d\\n", head->data);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '头插法',
          description: '头插法把新节点插入到链表头部，这样最后插入的节点反而在最前面。',
          focusLines: [17, 18, 19, 20],
          explanations: [
            LineExplanation(line: 17, short: '创建新节点', detail: '用 createNode 在堆上分配新节点。'),
            LineExplanation(line: 18, short: '链接原头', detail: 'newNode->next = head，让新节点指向原来的头节点。'),
            LineExplanation(line: 19, short: '更新头指针', detail: '返回 newNode 作为新的头指针。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'linkedTraverse',
      '链表遍历',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '\n'
      'struct Node {\n'
      '    int data;\n'
      '    struct Node* next;\n'
      '};\n'
      '\n'
      'struct Node* createNode(int data) {\n'
      '    struct Node* node = (struct Node*)malloc(sizeof(struct Node));\n'
      '    node->data = data;\n'
      '    node->next = NULL;\n'
      '    return node;\n'
      '}\n'
      '\n'
      'void printList(struct Node* head) {\n'
      '    struct Node* p = head;\n'
      '    while (p != NULL) {\n'
      '        printf("%d ", p->data);\n'
      '        p = p->next;\n'
      '    }\n'
      '    printf("\\n");\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct Node* head = createNode(1);\n'
      '    head->next = createNode(2);\n'
      '    head->next->next = createNode(3);\n'
      '    printList(head);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '遍历逻辑',
          description: '用临时指针 p 从头节点开始，沿着 next 指针依次访问每个节点，直到 NULL。',
          focusLines: [18, 19, 20, 21, 22],
          explanations: [
            LineExplanation(line: 18, short: '临时指针', detail: 'p = head，用 p 代替 head 去遍历，避免丢失头指针。'),
            LineExplanation(line: 19, short: '循环条件', detail: 'p != NULL 表示还有节点未访问。'),
            LineExplanation(line: 20, short: '访问数据', detail: 'p->data 读取当前节点的值。'),
            LineExplanation(line: 21, short: '移动到下一个', detail: 'p = p->next，让 p 指向下一个节点。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'treeNode',
      '二叉树节点',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '\n'
      'struct TreeNode {\n'
      '    int val;\n'
      '    struct TreeNode* left;\n'
      '    struct TreeNode* right;\n'
      '};\n'
      '\n'
      'struct TreeNode* createTreeNode(int val) {\n'
      '    struct TreeNode* node = (struct TreeNode*)malloc(sizeof(struct TreeNode));\n'
      '    node->val = val;\n'
      '    node->left = NULL;\n'
      '    node->right = NULL;\n'
      '    return node;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct TreeNode* root = createTreeNode(1);\n'
      '    root->left = createTreeNode(2);\n'
      '    root->right = createTreeNode(3);\n'
      '    printf("Root: %d\\n", root->val);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '二叉树节点',
          description: '二叉树每个节点包含一个值和左右两个子节点指针。NULL 表示该方向没有子树。',
          focusLines: [4, 5, 6, 7],
          explanations: [
            LineExplanation(line: 4, short: '节点结构', detail: 'struct TreeNode 定义二叉树节点。'),
            LineExplanation(line: 5, short: '节点值', detail: 'int val 存储当前节点的数据。'),
            LineExplanation(line: 6, short: '左子树', detail: 'left 指向左子树根节点，NULL 表示无左子树。'),
            LineExplanation(line: 7, short: '右子树', detail: 'right 指向右子树根节点，NULL 表示无右子树。'),
          ],
        ),
        TutorialStep(
          title: '创建节点',
          description: '在堆上分配节点内存，初始化 val 并把左右子树设为 NULL。',
          focusLines: [10, 11, 12, 13, 14],
          explanations: [
            LineExplanation(line: 10, short: '分配内存', detail: 'malloc 申请节点大小的堆空间。'),
            LineExplanation(line: 11, short: '赋值', detail: 'node->val = val。'),
            LineExplanation(line: 12, short: '初始化左子树', detail: 'node->left = NULL。'),
            LineExplanation(line: 13, short: '初始化右子树', detail: 'node->right = NULL。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'treePreorder',
      '先序遍历',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '\n'
      'struct TreeNode {\n'
      '    int val;\n'
      '    struct TreeNode* left;\n'
      '    struct TreeNode* right;\n'
      '};\n'
      '\n'
      'struct TreeNode* createTreeNode(int val) {\n'
      '    struct TreeNode* node = (struct TreeNode*)malloc(sizeof(struct TreeNode));\n'
      '    node->val = val;\n'
      '    node->left = NULL;\n'
      '    node->right = NULL;\n'
      '    return node;\n'
      '}\n'
      '\n'
      'void preorder(struct TreeNode* root) {\n'
      '    if (root == NULL) return;\n'
      '    printf("%d ", root->val);\n'
      '    preorder(root->left);\n'
      '    preorder(root->right);\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct TreeNode* root = createTreeNode(1);\n'
      '    root->left = createTreeNode(2);\n'
      '    root->right = createTreeNode(3);\n'
      '    root->left->left = createTreeNode(4);\n'
      '    root->left->right = createTreeNode(5);\n'
      '    preorder(root);\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '递归遍历',
          description: '先序遍历的顺序是：根节点 → 左子树 → 右子树。用递归自然实现。',
          focusLines: [19, 20, 21, 22, 23],
          explanations: [
            LineExplanation(line: 19, short: '终止条件', detail: 'root == NULL 表示空树，直接返回。'),
            LineExplanation(line: 20, short: '访问根', detail: '先打印当前节点的值。'),
            LineExplanation(line: 21, short: '递归左子树', detail: 'preorder(root->left) 遍历左子树。'),
            LineExplanation(line: 22, short: '递归右子树', detail: 'preorder(root->right) 遍历右子树。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'inorder',
      '中序遍历',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '\n'
      'struct TreeNode {\n'
      '    int val;\n'
      '    struct TreeNode* left;\n'
      '    struct TreeNode* right;\n'
      '};\n'
      '\n'
      'struct TreeNode* createTreeNode(int val) {\n'
      '    struct TreeNode* node = (struct TreeNode*)malloc(sizeof(struct TreeNode));\n'
      '    node->val = val;\n'
      '    node->left = NULL;\n'
      '    node->right = NULL;\n'
      '    return node;\n'
      '}\n'
      '\n'
      'void inorder(struct TreeNode* root) {\n'
      '    if (root == NULL) return;\n'
      '    inorder(root->left);\n'
      '    printf("%d ", root->val);\n'
      '    inorder(root->right);\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct TreeNode* root = createTreeNode(1);\n'
      '    root->left = createTreeNode(2);\n'
      '    root->right = createTreeNode(3);\n'
      '    root->left->left = createTreeNode(4);\n'
      '    root->left->right = createTreeNode(5);\n'
      '    inorder(root);\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '递归遍历',
          description: '中序遍历的顺序是：左子树 → 根节点 → 右子树。对二叉搜索树进行中序遍历可得到有序序列。',
          focusLines: [18, 19, 20, 21, 22],
          explanations: [
            LineExplanation(line: 18, short: '终止条件', detail: 'root == NULL 表示空树，直接返回。'),
            LineExplanation(line: 19, short: '递归左子树', detail: 'inorder(root->left) 先遍历左子树。'),
            LineExplanation(line: 20, short: '访问根', detail: '左子树遍历完成后，打印当前节点的值。'),
            LineExplanation(line: 21, short: '递归右子树', detail: '最后遍历右子树。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'postorder',
      '后序遍历',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '\n'
      'struct TreeNode {\n'
      '    int val;\n'
      '    struct TreeNode* left;\n'
      '    struct TreeNode* right;\n'
      '};\n'
      '\n'
      'struct TreeNode* createTreeNode(int val) {\n'
      '    struct TreeNode* node = (struct TreeNode*)malloc(sizeof(struct TreeNode));\n'
      '    node->val = val;\n'
      '    node->left = NULL;\n'
      '    node->right = NULL;\n'
      '    return node;\n'
      '}\n'
      '\n'
      'void postorder(struct TreeNode* root) {\n'
      '    if (root == NULL) return;\n'
      '    postorder(root->left);\n'
      '    postorder(root->right);\n'
      '    printf("%d ", root->val);\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct TreeNode* root = createTreeNode(1);\n'
      '    root->left = createTreeNode(2);\n'
      '    root->right = createTreeNode(3);\n'
      '    root->left->left = createTreeNode(4);\n'
      '    root->left->right = createTreeNode(5);\n'
      '    postorder(root);\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '递归遍历',
          description: '后序遍历的顺序是：左子树 → 右子树 → 根节点。常用于释放树的内存（先释放子节点，再释放根节点）。',
          focusLines: [18, 19, 20, 21, 22],
          explanations: [
            LineExplanation(line: 18, short: '终止条件', detail: 'root == NULL 表示空树，直接返回。'),
            LineExplanation(line: 19, short: '递归左子树', detail: 'postorder(root->left) 先遍历左子树。'),
            LineExplanation(line: 20, short: '递归右子树', detail: '再遍历右子树。'),
            LineExplanation(line: 21, short: '访问根', detail: '左右子树都遍历完成后，打印当前节点的值。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'stackArray',
      '栈（数组）',
      '结构',
      '#include <stdio.h>\n'
      '\n'
      'int stack[100];\n'
      'int top = -1;\n'
      '\n'
      'void push(int x) {\n'
      '    stack[++top] = x;\n'
      '}\n'
      '\n'
      'int pop() {\n'
      '    if (top < 0) return -1;\n'
      '    return stack[top--];\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    push(10);\n'
      '    push(20);\n'
      '    push(30);\n'
      '    printf("%d ", pop());\n'
      '    printf("%d ", pop());\n'
      '    printf("%d\\n", pop());\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '栈的定义',
          description: '用数组实现栈，top 指针指向栈顶元素。-1 表示栈为空。',
          focusLines: [4, 5],
          explanations: [
            LineExplanation(line: 4, short: '数组', detail: 'stack[100] 是栈的底层存储，容量 100。'),
            LineExplanation(line: 5, short: '栈顶指针', detail: 'top = -1 表示栈空。top = 0 表示有一个元素。'),
          ],
        ),
        TutorialStep(
          title: '入栈',
          description: '++top 先让 top 加 1，再把 x 放到新的栈顶位置。',
          focusLines: [7],
          explanations: [
            LineExplanation(line: 7, short: '前置++', detail: '++top 先自增再作为索引，所以第一个元素放在 stack[0]。'),
          ],
        ),
        TutorialStep(
          title: '出栈',
          description: '先检查栈是否为空，再返回栈顶元素并让 top 减 1。',
          focusLines: [10, 11],
          explanations: [
            LineExplanation(line: 10, short: '判空', detail: 'top < 0 时栈为空，返回 -1 表示出错。'),
            LineExplanation(line: 11, short: '后置--', detail: 'stack[top--] 先取 top 位置的值，再自减，栈顶下移。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'queueArray',
      '队列（数组）',
      '结构',
      '#include <stdio.h>\n'
      '\n'
      'int queue[100];\n'
      'int front = 0, rear = 0;\n'
      '\n'
      'void enqueue(int x) {\n'
      '    queue[rear++] = x;\n'
      '}\n'
      '\n'
      'int dequeue() {\n'
      '    if (front == rear) return -1;\n'
      '    return queue[front++];\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    enqueue(10);\n'
      '    enqueue(20);\n'
      '    enqueue(30);\n'
      '    printf("%d ", dequeue());\n'
      '    printf("%d ", dequeue());\n'
      '    printf("%d\\n", dequeue());\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '队列的定义',
          description: '用数组实现队列，front 指向队头，rear 指向队尾下一个位置。front == rear 时队列为空。',
          focusLines: [4, 5],
          explanations: [
            LineExplanation(line: 4, short: '数组', detail: 'queue[100] 是队列的底层存储，容量 100。'),
            LineExplanation(line: 5, short: '头尾指针', detail: 'front 和 rear 都从 0 开始。front 是待出队位置，rear 是待入队位置。'),
          ],
        ),
        TutorialStep(
          title: '入队',
          description: 'rear++ 先使用当前 rear 作为索引存放 x，再让 rear 后移一位。',
          focusLines: [7],
          explanations: [
            LineExplanation(line: 7, short: '后置++', detail: 'queue[rear++] = x 先把 x 放到 queue[rear]，然后 rear 自增。'),
          ],
        ),
        TutorialStep(
          title: '出队',
          description: '先检查队列是否为空，再返回队头元素并让 front 后移。',
          focusLines: [10, 11],
          explanations: [
            LineExplanation(line: 10, short: '判空', detail: 'front == rear 时队列为空，返回 -1 表示出错。'),
            LineExplanation(line: 11, short: '后置++', detail: 'queue[front++] 先取 front 位置的值，再自增，队头上移。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'heapSort',
      '堆排序',
      '排序',
      '#include <stdio.h>\n'
      '\n'
      'void heapify(int arr[], int n, int i) {\n'
      '    int largest = i;\n'
      '    int left = 2 * i + 1;\n'
      '    int right = 2 * i + 2;\n'
      '    if (left < n && arr[left] > arr[largest])\n'
      '        largest = left;\n'
      '    if (right < n && arr[right] > arr[largest])\n'
      '        largest = right;\n'
      '    if (largest != i) {\n'
      '        int temp = arr[i];\n'
      '        arr[i] = arr[largest];\n'
      '        arr[largest] = temp;\n'
      '        heapify(arr, n, largest);\n'
      '    }\n'
      '}\n'
      '\n'
      'void heapSort(int arr[], int n) {\n'
      '    for (int i = n / 2 - 1; i >= 0; i--)\n'
      '        heapify(arr, n, i);\n'
      '    for (int i = n - 1; i > 0; i--) {\n'
      '        int temp = arr[0];\n'
      '        arr[0] = arr[i];\n'
      '        arr[i] = temp;\n'
      '        heapify(arr, i, 0);\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int arr[{{n:5}}] = {12, 11, 13, 5, 6};\n'
      '    int n = {{n:5}};\n'
      '    heapSort(arr, n);\n'
      '    for (int i = 0; i < n; i++) {\n'
      '        printf("%d ", arr[i]);\n'
      '    }\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'n', label: '数组长度', defaultValue: '5', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '堆化函数',
          description: 'heapify 假设左右子树已经是堆，只需要把根节点下沉到正确位置，使整个子树满足堆性质。',
          focusLines: [3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
          explanations: [
            LineExplanation(line: 4, short: '当前节点', detail: 'largest 初始设为 i，表示当前假设 i 就是最大值。'),
            LineExplanation(line: 5, short: '左孩子', detail: 'left = 2*i + 1，完全二叉树左孩子下标公式。'),
            LineExplanation(line: 6, short: '右孩子', detail: 'right = 2*i + 2，右孩子下标公式。'),
            LineExplanation(line: 7, short: '比较左孩子', detail: '如果左孩子存在且比当前最大值大，更新 largest。'),
            LineExplanation(line: 9, short: '比较右孩子', detail: '同理比较右孩子。'),
            LineExplanation(line: 11, short: '需要交换', detail: 'largest != i 说明孩子比父亲大，需要交换。'),
            LineExplanation(line: 12, short: '交换', detail: '经典三变量交换，把最大值放到根位置。'),
            LineExplanation(line: 15, short: '递归下沉', detail: '交换后原来 largest 位置的值变小了，需要递归对它继续 heapify。'),
          ],
        ),
        TutorialStep(
          title: '建堆与排序',
          description: 'heapSort 先自底向上建堆，再反复把堆顶（最大值）换到末尾，缩小堆范围继续 heapify。',
          focusLines: [20, 21, 22, 23, 24, 25, 26, 27, 28, 29],
          explanations: [
            LineExplanation(line: 20, short: '建堆起点', detail: '从最后一个非叶子节点 n/2-1 开始往前逐个 heapify。'),
            LineExplanation(line: 22, short: '交换堆顶', detail: 'arr[0] 是当前最大值，把它换到位置 i（末尾）。'),
            LineExplanation(line: 26, short: '缩小堆', detail: 'heapify(arr, i, 0) 只对前 i 个元素重新堆化，末尾已有序部分不再参与。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'bfs',
      'BFS 广度优先',
      '图算法',
      '#include <stdio.h>\n'
      '\n'
      'int graph[5][5] = {\n'
      '    {0, 1, 1, 0, 0},\n'
      '    {1, 0, 0, 1, 1},\n'
      '    {1, 0, 0, 0, 0},\n'
      '    {0, 1, 0, 0, 0},\n'
      '    {0, 1, 0, 0, 0}\n'
      '};\n'
      'int visited[5] = {0, 0, 0, 0, 0};\n'
      'int queue[5];\n'
      'int front = 0, rear = 0;\n'
      '\n'
      'void bfs(int start, int n) {\n'
      '    visited[start] = 1;\n'
      '    queue[rear++] = start;\n'
      '    while (front < rear) {\n'
      '        int u = queue[front++];\n'
      '        printf("%d ", u);\n'
      '        for (int v = 0; v < n; v++) {\n'
      '            if (graph[u][v] == 1 && visited[v] == 0) {\n'
      '                visited[v] = 1;\n'
      '                queue[rear++] = v;\n'
      '            }\n'
      '        }\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int n = {{n:5}};\n'
      '    bfs(0, n);\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'n', label: '节点数', defaultValue: '5', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '图与队列',
          description: 'BFS 使用队列实现。graph 是邻接矩阵，visited 标记已访问节点，queue 存储待访问节点。',
          focusLines: [3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
          explanations: [
            LineExplanation(line: 3, short: '邻接矩阵', detail: 'graph[i][j] = 1 表示节点 i 和 j 之间有边。'),
            LineExplanation(line: 10, short: '访问标记', detail: 'visited 数组防止节点被重复访问。'),
            LineExplanation(line: 11, short: '队列', detail: 'queue 存放按层序待访问的节点。'),
          ],
        ),
        TutorialStep(
          title: 'BFS 过程',
          description: '从起点出发，标记为已访问并入队。每次出队一个节点，访问其所有未访问的邻居并入队。',
          focusLines: [15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25],
          explanations: [
            LineExplanation(line: 15, short: '标记起点', detail: 'visited[start] = 1，标记起点已访问。'),
            LineExplanation(line: 16, short: '起点入队', detail: 'queue[rear++] = start，起点入队。'),
            LineExplanation(line: 17, short: '循环条件', detail: 'front < rear 表示队列非空。'),
            LineExplanation(line: 18, short: '出队', detail: 'u = queue[front++]，取出队头节点。'),
            LineExplanation(line: 20, short: '扫描邻居', detail: 'v 从 0 到 n-1 扫描所有可能邻居。'),
            LineExplanation(line: 21, short: '未访问邻居', detail: 'graph[u][v] == 1 且 visited[v] == 0 说明是未访问的邻居。'),
            LineExplanation(line: 22, short: '标记并入队', detail: '先标记 visited[v]，再入队，保证同一节点不会重复入队。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'dfs',
      'DFS 深度优先',
      '图算法',
      '#include <stdio.h>\n'
      '\n'
      'int graph[5][5] = {\n'
      '    {0, 1, 1, 0, 0},\n'
      '    {1, 0, 0, 1, 1},\n'
      '    {1, 0, 0, 0, 0},\n'
      '    {0, 1, 0, 0, 0},\n'
      '    {0, 1, 0, 0, 0}\n'
      '};\n'
      'int visited[5] = {0, 0, 0, 0, 0};\n'
      '\n'
      'void dfs(int u, int n) {\n'
      '    visited[u] = 1;\n'
      '    printf("%d ", u);\n'
      '    for (int v = 0; v < n; v++) {\n'
      '        if (graph[u][v] == 1 && visited[v] == 0) {\n'
      '            dfs(v, n);\n'
      '        }\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int n = {{n:5}};\n'
      '    dfs(0, n);\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'n', label: '节点数', defaultValue: '5', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '图与访问标记',
          description: 'DFS 使用递归实现。graph 是邻接矩阵，visited 标记已访问节点，防止循环访问。',
          focusLines: [3, 4, 5, 6, 7, 8, 9, 10],
          explanations: [
            LineExplanation(line: 3, short: '邻接矩阵', detail: 'graph[i][j] = 1 表示节点 i 和 j 之间有边。'),
            LineExplanation(line: 10, short: '访问标记', detail: 'visited 数组防止节点被重复访问和无限递归。'),
          ],
        ),
        TutorialStep(
          title: 'DFS 过程',
          description: '从当前节点出发，标记为已访问并输出。然后对每个未访问的邻居递归调用 dfs，一条路走到黑再回溯。',
          focusLines: [12, 13, 14, 15, 16, 17, 18, 19, 20],
          explanations: [
            LineExplanation(line: 12, short: '标记访问', detail: 'visited[u] = 1，标记当前节点已访问。'),
            LineExplanation(line: 13, short: '输出节点', detail: '打印当前节点编号。'),
            LineExplanation(line: 14, short: '扫描邻居', detail: 'v 从 0 到 n-1 扫描所有可能邻居。'),
            LineExplanation(line: 15, short: '未访问邻居', detail: 'graph[u][v] == 1 且 visited[v] == 0 说明是未访问的邻居。'),
            LineExplanation(line: 16, short: '递归深入', detail: 'dfs(v, n) 递归访问邻居，直到没有未访问邻居才返回。'),
          ],
        ),
      ],
    ),
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
      'shellSort',
      '希尔排序',
      '排序',
      '#include <stdio.h>\n'
      '\n'
      'void shellSort(int arr[], int n) {\n'
      '    for (int gap = n / 2; gap > 0; gap /= 2) {\n'
      '        for (int i = gap; i < n; i++) {\n'
      '            int temp = arr[i];\n'
      '            int j;\n'
      '            for (j = i; j >= gap && arr[j - gap] > temp; j -= gap)\n'
      '                arr[j] = arr[j - gap];\n'
      '            arr[j] = temp;\n'
      '        }\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int arr[{{n:5}}] = {64, 34, 25, 12, 22};\n'
      '    int n = {{n:5}};\n'
      '    shellSort(arr, n);\n'
      '    for (int i = 0; i < n; i++) {\n'
      '        printf("%d ", arr[i]);\n'
      '    }\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'n', label: '数组长度', defaultValue: '5', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '增量序列',
          description: '希尔排序先取一个增量 gap，把数组分成 gap 组，每组内部进行插入排序。然后逐步缩小 gap，最后 gap=1 时就是普通的插入排序。',
          focusLines: [4],
          explanations: [
            LineExplanation(line: 4, short: 'gap 初始值', detail: 'gap 从 n/2 开始，每次减半。gap 较大时元素移动跨度大，能快速减少逆序。'),
          ],
        ),
        TutorialStep(
          title: '分组插入排序',
          description: '对每个元素，和它同组前面 gap 距离的元素比较，如果前面更大就后移，直到找到正确位置插入 temp。',
          focusLines: [5, 6, 7, 8, 9, 10],
          explanations: [
            LineExplanation(line: 5, short: '遍历无序区', detail: 'i 从 gap 开始，因为前 gap 个元素分别是各组的第一个元素，默认有序。'),
            LineExplanation(line: 6, short: '保存当前值', detail: 'temp = arr[i]，防止在移动过程中被覆盖。'),
            LineExplanation(line: 8, short: '同组比较', detail: 'j >= gap 保证 j-gap 不会越界；arr[j-gap] > temp 说明前面更大，需要后移。'),
            LineExplanation(line: 9, short: '元素后移', detail: 'arr[j] = arr[j-gap]，把前面较大的元素后移 gap 个位置。'),
            LineExplanation(line: 10, short: '插入到位', detail: '循环结束时 j 指向应插入的位置，把 temp 放进去。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'countingSort',
      '计数排序',
      '排序',
      '#include <stdio.h>\n'
      '\n'
      'void countingSort(int arr[], int n) {\n'
      '    int count[10] = {0};\n'
      '    for (int i = 0; i < n; i++) {\n'
      '        count[arr[i]]++;\n'
      '    }\n'
      '    int index = 0;\n'
      '    for (int i = 0; i < 10; i++) {\n'
      '        while (count[i] > 0) {\n'
      '            arr[index++] = i;\n'
      '            count[i]--;\n'
      '        }\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int arr[{{n:5}}] = {4, 2, 2, 8, 3};\n'
      '    int n = {{n:5}};\n'
      '    countingSort(arr, n);\n'
      '    for (int i = 0; i < n; i++) {\n'
      '        printf("%d ", arr[i]);\n'
      '    }\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [
        TemplateParam(key: 'n', label: '数组长度', defaultValue: '5', type: ParamType.int),
      ],
      tutorialSteps: [
        TutorialStep(
          title: '统计出现次数',
          description: '计数排序假设数据范围很小（这里固定为 0~9）。用 count 数组统计每个数字出现了几次。',
          focusLines: [4, 5, 6],
          explanations: [
            LineExplanation(line: 4, short: '计数数组', detail: 'count[10] 初始全为 0，下标代表数值，值代表出现次数。'),
            LineExplanation(line: 5, short: '遍历原数组', detail: '逐个读取 arr[i]。'),
            LineExplanation(line: 6, short: '统计', detail: 'count[arr[i]]++，对应数值的计数加 1。'),
          ],
        ),
        TutorialStep(
          title: '回填有序数组',
          description: '按从小到大的顺序扫描 count 数组，如果 count[i] 有 k 个，就把 k 个 i 依次写回原数组。',
          focusLines: [8, 9, 10, 11, 12],
          explanations: [
            LineExplanation(line: 8, short: '扫描计数数组', detail: 'i 从 0 到 9 扫描所有可能的数值。'),
            LineExplanation(line: 9, short: '还有剩余', detail: 'count[i] > 0 说明数值 i 还没放完。'),
            LineExplanation(line: 10, short: '回填', detail: 'arr[index++] = i，把数值 i 放到结果数组，index 后移。'),
            LineExplanation(line: 11, short: '计数减一', detail: 'count[i]--，该数值的剩余个数减 1。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'linkedDelete',
      '链表删除',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '\n'
      'struct Node {\n'
      '    int data;\n'
      '    struct Node* next;\n'
      '};\n'
      '\n'
      'struct Node* createNode(int data) {\n'
      '    struct Node* node = (struct Node*)malloc(sizeof(struct Node));\n'
      '    node->data = data;\n'
      '    node->next = NULL;\n'
      '    return node;\n'
      '}\n'
      '\n'
      'struct Node* deleteNode(struct Node* head, int key) {\n'
      '    struct Node* temp = head;\n'
      '    struct Node* prev = NULL;\n'
      '    if (temp != NULL && temp->data == key) {\n'
      '        head = temp->next;\n'
      '        free(temp);\n'
      '        return head;\n'
      '    }\n'
      '    while (temp != NULL && temp->data != key) {\n'
      '        prev = temp;\n'
      '        temp = temp->next;\n'
      '    }\n'
      '    if (temp == NULL) return head;\n'
      '    prev->next = temp->next;\n'
      '    free(temp);\n'
      '    return head;\n'
      '}\n'
      '\n'
      'void printList(struct Node* head) {\n'
      '    struct Node* p = head;\n'
      '    while (p != NULL) {\n'
      '        printf("%d ", p->data);\n'
      '        p = p->next;\n'
      '    }\n'
      '    printf("\\n");\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct Node* head = createNode(1);\n'
      '    head->next = createNode(2);\n'
      '    head->next->next = createNode(3);\n'
      '    head = deleteNode(head, 2);\n'
      '    printList(head);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '删除头节点',
          description: '如果要删除的节点正好是头节点，直接让 head 指向下一个节点，然后释放原头节点。',
          focusLines: [16, 17, 18, 19, 20, 21, 22],
          explanations: [
            LineExplanation(line: 16, short: '初始化', detail: 'temp 从头节点开始扫描，prev 记录前一个节点。'),
            LineExplanation(line: 17, short: '头节点判断', detail: 'temp != NULL && temp->data == key，检查头节点是否就是要删除的目标。'),
            LineExplanation(line: 18, short: '移动头指针', detail: 'head = temp->next，让头指针跳过待删除节点。'),
            LineExplanation(line: 19, short: '释放内存', detail: 'free(temp) 释放被删除节点的堆内存，防止内存泄漏。'),
          ],
        ),
        TutorialStep(
          title: '删除中间/尾节点',
          description: '从头开始遍历，找到目标节点后让前一个节点的 next 跳过目标节点，然后释放目标节点。',
          focusLines: [23, 24, 25, 26, 27, 28, 29, 30, 31],
          explanations: [
            LineExplanation(line: 23, short: '遍历查找', detail: 'while 循环沿着链表搜索，直到找到 key 或到达末尾。'),
            LineExplanation(line: 24, short: '保存前驱', detail: 'prev = temp，在 temp 前移前先保存当前位置。'),
            LineExplanation(line: 25, short: '前移', detail: 'temp = temp->next，继续向后扫描。'),
            LineExplanation(line: 27, short: '未找到', detail: 'temp == NULL 说明遍历完整个链表也没找到 key，直接返回原 head。'),
            LineExplanation(line: 28, short: '跳过目标', detail: 'prev->next = temp->next，让前驱节点直接指向目标的后继，完成逻辑删除。'),
            LineExplanation(line: 29, short: '释放内存', detail: 'free(temp) 释放被删除节点的堆内存。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'bstInsert',
      'BST 插入',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '\n'
      'struct TreeNode {\n'
      '    int val;\n'
      '    struct TreeNode* left;\n'
      '    struct TreeNode* right;\n'
      '};\n'
      '\n'
      'struct TreeNode* createNode(int val) {\n'
      '    struct TreeNode* node = (struct TreeNode*)malloc(sizeof(struct TreeNode));\n'
      '    node->val = val;\n'
      '    node->left = NULL;\n'
      '    node->right = NULL;\n'
      '    return node;\n'
      '}\n'
      '\n'
      'struct TreeNode* insert(struct TreeNode* root, int val) {\n'
      '    if (root == NULL) return createNode(val);\n'
      '    if (val < root->val)\n'
      '        root->left = insert(root->left, val);\n'
      '    else\n'
      '        root->right = insert(root->right, val);\n'
      '    return root;\n'
      '}\n'
      '\n'
      'void inorder(struct TreeNode* root) {\n'
      '    if (root == NULL) return;\n'
      '    inorder(root->left);\n'
      '    printf("%d ", root->val);\n'
      '    inorder(root->right);\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct TreeNode* root = NULL;\n'
      '    root = insert(root, 5);\n'
      '    insert(root, 3);\n'
      '    insert(root, 7);\n'
      '    insert(root, 1);\n'
      '    insert(root, 9);\n'
      '    inorder(root);\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: 'BST 性质',
          description: '二叉搜索树（BST）的左子树所有节点值小于根，右子树所有节点值大于等于根。中序遍历可得到升序序列。',
          focusLines: [4, 5, 6, 7],
          explanations: [
            LineExplanation(line: 5, short: '节点值', detail: 'val 存储当前节点的数据。'),
            LineExplanation(line: 6, short: '左子树', detail: 'left 指向比当前节点值小的子树。'),
            LineExplanation(line: 7, short: '右子树', detail: 'right 指向比当前节点值大或相等的子树。'),
          ],
        ),
        TutorialStep(
          title: '递归插入',
          description: '从根出发，如果为空则创建新节点；如果待插入值小于当前节点值就递归插入左子树，否则插入右子树。',
          focusLines: [18, 19, 20, 21, 22, 23, 24],
          explanations: [
            LineExplanation(line: 18, short: '空树', detail: 'root == NULL 表示找到了正确的空位，创建新节点并返回。'),
            LineExplanation(line: 19, short: '去左边', detail: 'val < root->val 说明新节点应该放在左子树。'),
            LineExplanation(line: 20, short: '递归左插', detail: 'root->left = insert(root->left, val)，递归插入并更新左指针。'),
            LineExplanation(line: 22, short: '去右边', detail: 'val >= root->val 时放入右子树。'),
          ],
        ),
      ],
    ),
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
      ],
    ),
    // ========== 数据结构（经典教材案例） ==========
    CodeTemplate(
      'seqList',
      '顺序表',
      '结构',
      '#include <stdio.h>\n'
      '#define MAXSIZE 10\n'
      '\n'
      'struct SeqList {\n'
      '    int data[MAXSIZE];\n'
      '    int length;\n'
      '};\n'
      '\n'
      'void init(struct SeqList* L) {\n'
      '    L->length = 0;\n'
      '}\n'
      '\n'
      'int listInsert(struct SeqList* L, int pos, int x) {\n'
      '    if (pos < 0 || pos > L->length || L->length >= MAXSIZE) return 0;\n'
      '    for (int i = L->length; i > pos; i--)\n'
      '        L->data[i] = L->data[i - 1];\n'
      '    L->data[pos] = x;\n'
      '    L->length++;\n'
      '    return 1;\n'
      '}\n'
      '\n'
      'int listDelete(struct SeqList* L, int pos) {\n'
      '    if (pos < 0 || pos >= L->length) return 0;\n'
      '    for (int i = pos; i < L->length - 1; i++)\n'
      '        L->data[i] = L->data[i + 1];\n'
      '    L->length--;\n'
      '    return 1;\n'
      '}\n'
      '\n'
      'int listFind(struct SeqList* L, int x) {\n'
      '    for (int i = 0; i < L->length; i++) {\n'
      '        if (L->data[i] == x) return i;\n'
      '    }\n'
      '    return -1;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct SeqList L;\n'
      '    init(&L);\n'
      '    listInsert(&L, 0, 5);\n'
      '    listInsert(&L, 1, 3);\n'
      '    listInsert(&L, 2, 8);\n'
      '    listDelete(&L, 1);\n'
      '    for (int i = 0; i < L.length; i++) {\n'
      '        printf("%d ", L.data[i]);\n'
      '    }\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '顺序表结构',
          description: '顺序表是用一段地址连续的存储单元依次存储数据元素。这里用数组 data 存数据，length 记录当前长度。',
          focusLines: [4, 5, 6, 7],
          explanations: [
            LineExplanation(line: 5, short: '数据区', detail: 'data[MAXSIZE] 是底层数组，MAXSIZE 定义了最大容量。'),
            LineExplanation(line: 6, short: '当前长度', detail: 'length 表示表中实际元素个数，不是数组总大小。'),
          ],
        ),
        TutorialStep(
          title: '插入操作',
          description: '在位置 pos 插入元素时，需要把 pos 及之后的元素全部后移一位，然后放入新元素，长度加 1。',
          focusLines: [13, 14, 15, 16, 17, 18, 19, 20],
          explanations: [
            LineExplanation(line: 13, short: '合法性检查', detail: 'pos 必须在 [0, length] 范围内，且表不能已满。'),
            LineExplanation(line: 14, short: '后移元素', detail: '从末尾开始逐个后移，避免覆盖数据。'),
            LineExplanation(line: 16, short: '放入新元素', detail: '在腾出的位置 pos 放入 x。'),
            LineExplanation(line: 17, short: '长度加 1', detail: 'length++ 反映表长变化。'),
          ],
        ),
        TutorialStep(
          title: '删除与查找',
          description: '删除时把 pos 之后的元素前移；查找时逐个比较。',
          focusLines: [22, 23, 24, 25, 26, 27, 30, 31, 32, 33, 34, 35],
          explanations: [
            LineExplanation(line: 23, short: '前移元素', detail: '从 pos 开始，用后一个元素覆盖前一个。'),
            LineExplanation(line: 32, short: '按值查找', detail: '顺序扫描，时间复杂度 O(n)。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'linkedListTail',
      '链表尾插法',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '\n'
      'struct Node {\n'
      '    int data;\n'
      '    struct Node* next;\n'
      '};\n'
      '\n'
      'struct Node* createNode(int data) {\n'
      '    struct Node* node = (struct Node*)malloc(sizeof(struct Node));\n'
      '    node->data = data;\n'
      '    node->next = NULL;\n'
      '    return node;\n'
      '}\n'
      '\n'
      'struct Node* append(struct Node* head, int data) {\n'
      '    struct Node* newNode = createNode(data);\n'
      '    if (head == NULL) return newNode;\n'
      '    struct Node* p = head;\n'
      '    while (p->next != NULL) p = p->next;\n'
      '    p->next = newNode;\n'
      '    return head;\n'
      '}\n'
      '\n'
      'void printList(struct Node* head) {\n'
      '    struct Node* p = head;\n'
      '    while (p != NULL) {\n'
      '        printf("%d ", p->data);\n'
      '        p = p->next;\n'
      '    }\n'
      '    printf("\\n");\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct Node* head = NULL;\n'
      '    head = append(head, 1);\n'
      '    append(head, 2);\n'
      '    append(head, 3);\n'
      '    printList(head);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '尾插法',
          description: '尾插法将新节点追加到链表末尾，这样遍历输出的顺序与插入顺序一致。',
          focusLines: [16, 17, 18, 19, 20, 21, 22, 23],
          explanations: [
            LineExplanation(line: 16, short: '创建新节点', detail: 'createNode 在堆上分配节点内存。'),
            LineExplanation(line: 17, short: '空表处理', detail: '如果链表为空，新节点就是头节点。'),
            LineExplanation(line: 19, short: '找尾节点', detail: 'while 循环一直走到最后一个节点（next 为 NULL）。'),
            LineExplanation(line: 21, short: '链接', detail: 'p->next = newNode，把新节点挂到末尾。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'doublyLinkedList',
      '双向链表',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '\n'
      'struct DNode {\n'
      '    int data;\n'
      '    struct DNode* prev;\n'
      '    struct DNode* next;\n'
      '};\n'
      '\n'
      'struct DNode* createNode(int data) {\n'
      '    struct DNode* node = (struct DNode*)malloc(sizeof(struct DNode));\n'
      '    node->data = data;\n'
      '    node->prev = NULL;\n'
      '    node->next = NULL;\n'
      '    return node;\n'
      '}\n'
      '\n'
      'struct DNode* append(struct DNode* head, int data) {\n'
      '    struct DNode* newNode = createNode(data);\n'
      '    if (head == NULL) return newNode;\n'
      '    struct DNode* p = head;\n'
      '    while (p->next != NULL) p = p->next;\n'
      '    p->next = newNode;\n'
      '    newNode->prev = p;\n'
      '    return head;\n'
      '}\n'
      '\n'
      'void printForward(struct DNode* head) {\n'
      '    struct DNode* p = head;\n'
      '    while (p != NULL) {\n'
      '        printf("%d ", p->data);\n'
      '        p = p->next;\n'
      '    }\n'
      '    printf("\\n");\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct DNode* head = NULL;\n'
      '    head = append(head, 1);\n'
      '    head = append(head, 2);\n'
      '    head = append(head, 3);\n'
      '    printForward(head);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '双向节点',
          description: '双向链表每个节点有两个指针：prev 指向前驱，next 指向后继。可以双向遍历。',
          focusLines: [4, 5, 6, 7, 8],
          explanations: [
            LineExplanation(line: 5, short: '数据', detail: 'data 存储节点值。'),
            LineExplanation(line: 6, short: '前驱', detail: 'prev 指向前一个节点，头节点的 prev 为 NULL。'),
            LineExplanation(line: 7, short: '后继', detail: 'next 指向下一个节点，尾节点的 next 为 NULL。'),
          ],
        ),
        TutorialStep(
          title: '尾插与双向链接',
          description: '尾插时不仅要让原尾节点的 next 指向新节点，还要让新节点的 prev 指向原尾节点，维护双向关系。',
          focusLines: [18, 19, 20, 21, 22, 23, 24, 25],
          explanations: [
            LineExplanation(line: 23, short: '正向链接', detail: 'p->next = newNode，原尾节点指向新节点。'),
            LineExplanation(line: 24, short: '反向链接', detail: 'newNode->prev = p，新节点指回原尾节点。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'circularQueue',
      '循环队列',
      '结构',
      '#include <stdio.h>\n'
      '#define MAXSIZE 5\n'
      '\n'
      'struct CircularQueue {\n'
      '    int data[MAXSIZE];\n'
      '    int front;\n'
      '    int rear;\n'
      '};\n'
      '\n'
      'void init(struct CircularQueue* q) {\n'
      '    q->front = 0;\n'
      '    q->rear = 0;\n'
      '}\n'
      '\n'
      'int isEmpty(struct CircularQueue* q) {\n'
      '    return q->front == q->rear;\n'
      '}\n'
      '\n'
      'int isFull(struct CircularQueue* q) {\n'
      '    return (q->rear + 1) % MAXSIZE == q->front;\n'
      '}\n'
      '\n'
      'void enqueue(struct CircularQueue* q, int x) {\n'
      '    if (isFull(q)) return;\n'
      '    q->data[q->rear] = x;\n'
      '    q->rear = (q->rear + 1) % MAXSIZE;\n'
      '}\n'
      '\n'
      'int dequeue(struct CircularQueue* q) {\n'
      '    if (isEmpty(q)) return -1;\n'
      '    int x = q->data[q->front];\n'
      '    q->front = (q->front + 1) % MAXSIZE;\n'
      '    return x;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct CircularQueue q;\n'
      '    init(&q);\n'
      '    enqueue(&q, 10);\n'
      '    enqueue(&q, 20);\n'
      '    enqueue(&q, 30);\n'
      '    printf("%d ", dequeue(&q));\n'
      '    printf("%d ", dequeue(&q));\n'
      '    printf("%d\\n", dequeue(&q));\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '循环队列结构',
          description: '用数组实现队列，front 指向队头，rear 指向队尾下一个位置。故意牺牲一个单元来区分空和满。',
          focusLines: [4, 5, 6, 7, 8],
          explanations: [
            LineExplanation(line: 5, short: '数组', detail: 'data[MAXSIZE] 是底层存储。'),
            LineExplanation(line: 6, short: '队头', detail: 'front 指向队头元素。'),
            LineExplanation(line: 7, short: '队尾', detail: 'rear 指向队尾下一个空位。'),
          ],
        ),
        TutorialStep(
          title: '判空与判满',
          description: '空队列时 front == rear；满队列时 (rear+1)%MAXSIZE == front。牺牲一个单元避免了用计数器或标记位。',
          focusLines: [15, 16, 17, 19, 20, 21],
          explanations: [
            LineExplanation(line: 16, short: '空', detail: 'front == rear 表示队列空。'),
            LineExplanation(line: 20, short: '满', detail: '(rear+1)%MAXSIZE == front 表示队列满，此时还剩一个空位未用。'),
          ],
        ),
        TutorialStep(
          title: '入队与出队',
          description: '入队和出队都用取模运算让指针在数组末尾绕回到开头，形成"循环"。',
          focusLines: [23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33],
          explanations: [
            LineExplanation(line: 26, short: '入队', detail: 'q->data[q->rear] = x，元素放到 rear 位置。'),
            LineExplanation(line: 27, short: 'rear 前移', detail: '(q->rear + 1) % MAXSIZE，rear 绕回数组开头。'),
            LineExplanation(line: 31, short: '出队', detail: '取出 front 位置元素。'),
            LineExplanation(line: 32, short: 'front 前移', detail: '(q->front + 1) % MAXSIZE，front 绕回数组开头。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'linkedStack',
      '链栈',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '\n'
      'struct Node {\n'
      '    int data;\n'
      '    struct Node* next;\n'
      '};\n'
      '\n'
      'struct Node* push(struct Node* top, int x) {\n'
      '    struct Node* node = (struct Node*)malloc(sizeof(struct Node));\n'
      '    node->data = x;\n'
      '    node->next = top;\n'
      '    return node;\n'
      '}\n'
      '\n'
      'struct Node* pop(struct Node* top) {\n'
      '    if (top == NULL) return NULL;\n'
      '    struct Node* temp = top;\n'
      '    top = top->next;\n'
      '    free(temp);\n'
      '    return top;\n'
      '}\n'
      '\n'
      'void printStack(struct Node* top) {\n'
      '    struct Node* p = top;\n'
      '    while (p != NULL) {\n'
      '        printf("%d ", p->data);\n'
      '        p = p->next;\n'
      '    }\n'
      '    printf("\\n");\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct Node* top = NULL;\n'
      '    top = push(top, 30);\n'
      '    top = push(top, 20);\n'
      '    top = push(top, 10);\n'
      '    printStack(top);\n'
      '    top = pop(top);\n'
      '    printStack(top);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '链栈结构',
          description: '链栈用单链表实现，top 指针指向栈顶。没有容量限制（除非内存耗尽）。',
          focusLines: [9, 10, 11, 12, 13],
          explanations: [
            LineExplanation(line: 10, short: '分配节点', detail: 'malloc 申请新节点内存。'),
            LineExplanation(line: 12, short: '链接原栈顶', detail: 'node->next = top，新节点指向原来的栈顶。'),
            LineExplanation(line: 13, short: '更新栈顶', detail: '返回新节点作为新 top。'),
          ],
        ),
        TutorialStep(
          title: '出栈与内存释放',
          description: '出栈时保存当前栈顶，让 top 指向下一个节点，然后释放原栈顶内存。',
          focusLines: [16, 17, 18, 19, 20, 21, 22],
          explanations: [
            LineExplanation(line: 17, short: '判空', detail: 'top == NULL 时栈已空。'),
            LineExplanation(line: 19, short: '下移栈顶', detail: 'top = top->next，栈顶指针下移。'),
            LineExplanation(line: 20, short: '释放内存', detail: 'free(temp) 释放弹出的节点，防止泄漏。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'linkedQueue',
      '链队列',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '\n'
      'struct QNode {\n'
      '    int data;\n'
      '    struct QNode* next;\n'
      '};\n'
      '\n'
      'struct LinkedQueue {\n'
      '    struct QNode* front;\n'
      '    struct QNode* rear;\n'
      '};\n'
      '\n'
      'void init(struct LinkedQueue* q) {\n'
      '    q->front = NULL;\n'
      '    q->rear = NULL;\n'
      '}\n'
      '\n'
      'void enqueue(struct LinkedQueue* q, int x) {\n'
      '    struct QNode* node = (struct QNode*)malloc(sizeof(struct QNode));\n'
      '    node->data = x;\n'
      '    node->next = NULL;\n'
      '    if (q->rear == NULL) {\n'
      '        q->front = node;\n'
      '        q->rear = node;\n'
      '    } else {\n'
      '        q->rear->next = node;\n'
      '        q->rear = node;\n'
      '    }\n'
      '}\n'
      '\n'
      'int dequeue(struct LinkedQueue* q) {\n'
      '    if (q->front == NULL) return -1;\n'
      '    struct QNode* temp = q->front;\n'
      '    int x = temp->data;\n'
      '    q->front = q->front->next;\n'
      '    if (q->front == NULL) q->rear = NULL;\n'
      '    free(temp);\n'
      '    return x;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct LinkedQueue q;\n'
      '    init(&q);\n'
      '    enqueue(&q, 10);\n'
      '    enqueue(&q, 20);\n'
      '    enqueue(&q, 30);\n'
      '    printf("%d ", dequeue(&q));\n'
      '    printf("%d ", dequeue(&q));\n'
      '    printf("%d\\n", dequeue(&q));\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '链队列结构',
          description: '链队列用 front 指向队头节点，rear 指向队尾节点。队空时两者都为 NULL。',
          focusLines: [9, 10, 11, 12],
          explanations: [
            LineExplanation(line: 10, short: '队头', detail: 'front 指向队头节点。'),
            LineExplanation(line: 11, short: '队尾', detail: 'rear 指向队尾节点。'),
          ],
        ),
        TutorialStep(
          title: '入队',
          description: '新节点放入队尾。如果队列为空，新节点既是队头也是队尾。',
          focusLines: [19, 20, 21, 22, 23, 24, 25, 26, 27, 28],
          explanations: [
            LineExplanation(line: 22, short: '空队', detail: 'rear == NULL 时队列空，front 和 rear 都指向新节点。'),
            LineExplanation(line: 27, short: '链接', detail: 'q->rear->next = node，原尾节点指向新节点。'),
            LineExplanation(line: 28, short: '更新尾指针', detail: 'q->rear = node，rear 指向新尾节点。'),
          ],
        ),
        TutorialStep(
          title: '出队',
          description: '从队头移除节点。如果移除后队列为空，需要把 rear 也置为 NULL。',
          focusLines: [31, 32, 33, 34, 35, 36, 37, 38, 39],
          explanations: [
            LineExplanation(line: 35, short: '前移队头', detail: 'q->front = q->front->next，队头指向下一个。'),
            LineExplanation(line: 36, short: '处理空队', detail: '如果 front 变为 NULL，说明队列已空，rear 也要置空。'),
            LineExplanation(line: 37, short: '释放内存', detail: 'free(temp) 释放被删除节点。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'levelOrder',
      '层序遍历',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '\n'
      'struct TreeNode {\n'
      '    int val;\n'
      '    struct TreeNode* left;\n'
      '    struct TreeNode* right;\n'
      '};\n'
      '\n'
      'struct TreeNode* createNode(int val) {\n'
      '    struct TreeNode* node = (struct TreeNode*)malloc(sizeof(struct TreeNode));\n'
      '    node->val = val;\n'
      '    node->left = NULL;\n'
      '    node->right = NULL;\n'
      '    return node;\n'
      '}\n'
      '\n'
      '#define MAX 20\n'
      '\n'
      'void levelOrder(struct TreeNode* root) {\n'
      '    if (root == NULL) return;\n'
      '    struct TreeNode* queue[MAX];\n'
      '    int front = 0, rear = 0;\n'
      '    queue[rear++] = root;\n'
      '    while (front < rear) {\n'
      '        struct TreeNode* node = queue[front++];\n'
      '        printf("%d ", node->val);\n'
      '        if (node->left != NULL) queue[rear++] = node->left;\n'
      '        if (node->right != NULL) queue[rear++] = node->right;\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct TreeNode* root = createNode(1);\n'
      '    root->left = createNode(2);\n'
      '    root->right = createNode(3);\n'
      '    root->left->left = createNode(4);\n'
      '    root->left->right = createNode(5);\n'
      '    levelOrder(root);\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '队列辅助',
          description: '层序遍历（广度优先）需要用队列记录每一层的节点。先访问根节点，然后把左右子节点依次入队。',
          focusLines: [20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30],
          explanations: [
            LineExplanation(line: 20, short: '空树', detail: 'root == NULL 直接返回。'),
            LineExplanation(line: 21, short: '队列', detail: 'queue[MAX] 是用数组模拟的队列，存放节点指针。'),
            LineExplanation(line: 23, short: '根入队', detail: 'queue[rear++] = root，根节点入队。'),
            LineExplanation(line: 24, short: '循环', detail: 'front < rear 表示队列非空。'),
            LineExplanation(line: 25, short: '出队', detail: 'node = queue[front++]，取出队头节点。'),
            LineExplanation(line: 28, short: '左子入队', detail: '如果左子树非空，左子节点入队。'),
            LineExplanation(line: 29, short: '右子入队', detail: '如果右子树非空，右子节点入队。'),
          ],
        ),
      ],
    ),
    CodeTemplate(
      'bstSearch',
      'BST 查找',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '\n'
      'struct TreeNode {\n'
      '    int val;\n'
      '    struct TreeNode* left;\n'
      '    struct TreeNode* right;\n'
      '};\n'
      '\n'
      'struct TreeNode* createNode(int val) {\n'
      '    struct TreeNode* node = (struct TreeNode*)malloc(sizeof(struct TreeNode));\n'
      '    node->val = val;\n'
      '    node->left = NULL;\n'
      '    node->right = NULL;\n'
      '    return node;\n'
      '}\n'
      '\n'
      'struct TreeNode* insert(struct TreeNode* root, int val) {\n'
      '    if (root == NULL) return createNode(val);\n'
      '    if (val < root->val)\n'
      '        root->left = insert(root->left, val);\n'
      '    else\n'
      '        root->right = insert(root->right, val);\n'
      '    return root;\n'
      '}\n'
      '\n'
      'struct TreeNode* search(struct TreeNode* root, int key) {\n'
      '    if (root == NULL || root->val == key) return root;\n'
      '    if (key < root->val)\n'
      '        return search(root->left, key);\n'
      '    else\n'
      '        return search(root->right, key);\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct TreeNode* root = NULL;\n'
      '    root = insert(root, 5);\n'
      '    insert(root, 3);\n'
      '    insert(root, 7);\n'
      '    insert(root, 1);\n'
      '    insert(root, 9);\n'
      '    struct TreeNode* res = search(root, 7);\n'
      '    if (res != NULL)\n'
      '        printf("Found %d\\n", res->val);\n'
      '    else\n'
      '        printf("Not found\\n");\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: 'BST 查找',
          description: '利用二叉搜索树性质：左子树所有节点小于根，右子树大于等于根。每次比较可以排除一半子树。',
          focusLines: [27, 28, 29, 30, 31, 32, 33],
          explanations: [
            LineExplanation(line: 27, short: '基准情况', detail: 'root == NULL 表示没找到；root->val == key 表示找到。'),
            LineExplanation(line: 29, short: '去左边', detail: 'key < root->val 时目标只能在左子树。'),
            LineExplanation(line: 31, short: '去右边', detail: 'key >= root->val 时目标在右子树。'),
          ],
        ),
      ],
    ),
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
    CodeTemplate('array', '数组遍历', '基础',
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
      '}'),
    CodeTemplate('pointer', '指针交换', '指针',
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
      '}'),
    // ========== 递归（无参数、无教程） ==========
    CodeTemplate('factorial', '递归阶乘', '递归',
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
      '}'),
    CodeTemplate('fib', '斐波那契', '递归',
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
      '}'),
  ];
}
