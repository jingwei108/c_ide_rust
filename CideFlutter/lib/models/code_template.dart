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
