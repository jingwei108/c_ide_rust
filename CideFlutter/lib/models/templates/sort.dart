import '../code_template.dart';

const List<CodeTemplate> sortTemplates = [
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
      'radixSort',
      '基数排序',
      '排序',
      '#include <stdio.h>\n'
      '\n'
      'void RadixSort(int arr[], int n) {\n'
      '    int max = arr[0];\n'
      '    for (int i = 1; i < n; i++) {\n'
      '        if (arr[i] > max) max = arr[i];\n'
      '    }\n'
      '    int exp;\n'
      '    int output[10];\n'
      '    int count[10];\n'
      '    for (exp = 1; max / exp > 0; exp *= 10) {\n'
      '        for (int i = 0; i < 10; i++) count[i] = 0;\n'
      '        for (int i = 0; i < n; i++) count[(arr[i] / exp) % 10]++;\n'
      '        for (int i = 1; i < 10; i++) count[i] += count[i - 1];\n'
      '        for (int i = n - 1; i >= 0; i--) {\n'
      '            output[count[(arr[i] / exp) % 10] - 1] = arr[i];\n'
      '            count[(arr[i] / exp) % 10]--;\n'
      '        }\n'
      '        for (int i = 0; i < n; i++) arr[i] = output[i];\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int arr[7] = {170, 45, 75, 90, 2, 802, 24};\n'
      '    int n = 7;\n'
      '    RadixSort(arr, n);\n'
      '    for (int i = 0; i < n; i++) printf("%d ", arr[i]);\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '确定位数',
          description: '基数排序先找出数组中的最大值，确定需要进行多少轮分配-收集（由最大值的位数决定）。',
          focusLines: [4, 5, 6, 7, 8, 9, 10, 11],
          explanations: [
            LineExplanation(line: 4, short: '找最大值', detail: 'max 记录数组中的最大值。'),
            LineExplanation(line: 10, short: '按位处理', detail: 'exp 表示当前处理的是第几位（1、10、100...）。'),
            LineExplanation(line: 11, short: '终止条件', detail: 'max / exp > 0 表示最大值还有更高位需要处理。'),
          ],
        ),
        TutorialStep(
          title: '计数排序（按位）',
          description: '对每一位使用计数排序：先统计每个数字（0~9）出现次数，再计算前缀和确定位置，最后从后往前稳定地放入 output 数组。',
          focusLines: [12, 13, 14, 15, 16, 17, 18, 19, 20, 21],
          explanations: [
            LineExplanation(line: 13, short: '统计', detail: 'count[d] 记录当前位等于 d 的元素个数。'),
            LineExplanation(line: 15, short: '前缀和', detail: 'count[i] += count[i-1]，计算每个数字在结果中的结束位置。'),
            LineExplanation(line: 17, short: '稳定放置', detail: '从后往前扫描，保证相同数字的相对顺序不变（稳定性）。'),
            LineExplanation(line: 20, short: '回填', detail: '把 output 复制回 arr，进入下一位的排序。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'bucketSort',
      '桶排序',
      '排序',
      '#include <stdio.h>\n'
      '\n'
      'void bucketSort(int arr[], int n) {\n'
      '    int bucket[10][10];\n'
      '    int count[10] = {0};\n'
      '    int max = arr[0];\n'
      '    for (int i = 1; i < n; i++)\n'
      '        if (arr[i] > max) max = arr[i];\n'
      '    for (int i = 0; i < n; i++) {\n'
      '        int idx = (arr[i] * 10) / (max + 1);\n'
      '        bucket[idx][count[idx]++] = arr[i];\n'
      '    }\n'
      '    for (int i = 0; i < 10; i++) {\n'
      '        for (int j = 0; j < count[i] - 1; j++) {\n'
      '            for (int k = 0; k < count[i] - j - 1; k++) {\n'
      '                if (bucket[i][k] > bucket[i][k + 1]) {\n'
      '                    int temp = bucket[i][k];\n'
      '                    bucket[i][k] = bucket[i][k + 1];\n'
      '                    bucket[i][k + 1] = temp;\n'
      '                }\n'
      '            }\n'
      '        }\n'
      '    }\n'
      '    int idx = 0;\n'
      '    for (int i = 0; i < 10; i++) {\n'
      '        for (int j = 0; j < count[i]; j++)\n'
      '            arr[idx++] = bucket[i][j];\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int arr[] = {29, 25, 3, 49, 9, 37, 21, 43};\n'
      '    int n = 8;\n'
      '    bucketSort(arr, n);\n'
      '    for (int i = 0; i < n; i++)\n'
      '        printf("%d ", arr[i]);\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '分桶',
          description: '桶排序把数据划分到若干个桶中。这里按值域均匀分桶，每个桶内用插入排序（示例用冒泡）排序。',
          focusLines: [4, 5, 6, 7, 8, 9, 10, 11, 12],
          explanations: [
            LineExplanation(line: 6, short: '找最大值', detail: 'max 用于确定值域范围，方便均匀分桶。'),
            LineExplanation(line: 10, short: '计算桶号', detail: 'idx = (arr[i] * 10) / (max + 1)，把数值映射到 0~9 号桶。'),
            LineExplanation(line: 11, short: '入桶', detail: 'bucket[idx][count[idx]++] = arr[i]，把元素放入对应桶。'),
          ],
        ),
        TutorialStep(
          title: '桶内排序与收集',
          description: '对每个桶内部进行排序（此处用冒泡排序示例），然后按桶号顺序依次收集所有元素。',
          focusLines: [13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27],
          explanations: [
            LineExplanation(line: 14, short: '桶内排序', detail: '对每个非空桶内部进行冒泡排序。'),
            LineExplanation(line: 25, short: '顺序收集', detail: '按桶号 0 到 9 的顺序依次取出元素，保证整体有序。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'externalSort',
      '外部排序（置换-选择）',
      '排序',
      '#include <stdio.h>\n'
      '#define MAX 4\n'
      '\n'
      'void replacementSelection(int arr[], int n) {\n'
      '    int heap[MAX];\n'
      '    int heapSize = n < MAX ? n : MAX;\n'
      '    for (int i = 0; i < heapSize; i++) heap[i] = arr[i];\n'
      '    int idx = heapSize;\n'
      '    int output[30];\n'
      '    int outCount = 0;\n'
      '    while (heapSize > 0) {\n'
      '        int minIdx = 0;\n'
      '        for (int i = 1; i < heapSize; i++)\n'
      '            if (heap[i] < heap[minIdx]) minIdx = i;\n'
      '        output[outCount++] = heap[minIdx];\n'
      '        if (idx < n) {\n'
      '            int next = arr[idx++];\n'
      '            if (next >= heap[minIdx])\n'
      '                heap[minIdx] = next;\n'
      '            else {\n'
      '                heap[minIdx] = heap[heapSize - 1];\n'
      '                heap[heapSize - 1] = next;\n'
      '                heapSize--;\n'
      '            }\n'
      '        } else {\n'
      '            heap[minIdx] = heap[heapSize - 1];\n'
      '            heapSize--;\n'
      '        }\n'
      '    }\n'
      '    for (int i = 0; i < outCount; i++)\n'
      '        printf("%d ", output[i]);\n'
      '    printf("\\n");\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int arr[] = {51, 49, 39, 46, 38, 29, 14, 61, 15, 30, 1, 48, 52, 3, 63, 27, 4, 13, 89, 21, 53, 5, 34};\n'
      '    int n = 23;\n'
      '    replacementSelection(arr, n);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '置换-选择思想',
          description: '外部排序中，置换-选择排序用容量有限的堆生成较长的初始归并段。每次输出当前最小值，然后用下一个输入替换它。',
          focusLines: [4, 5, 6, 7, 8, 9, 10, 11],
          explanations: [
            LineExplanation(line: 5, short: '堆容量', detail: 'heapSize 取 min(n, MAX)，表示内存工作区大小。'),
            LineExplanation(line: 8, short: '输出缓冲区', detail: 'output 数组收集当前归并段的有序输出。'),
          ],
        ),
        TutorialStep(
          title: '选择最小值',
          description: '在工作区中找出最小值输出。然后用下一个输入元素替换：如果新元素 >= 刚输出的最小值，可以留在当前归并段；否则必须放到工作区末尾，等下一个归并段再处理。',
          focusLines: [12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28],
          explanations: [
            LineExplanation(line: 14, short: '找最小', detail: '简单选择最小值（教材常用方式，非堆实现）。'),
            LineExplanation(line: 19, short: '置换条件', detail: 'next >= heap[minIdx] 时新元素可接在当前归并段后面。'),
            LineExplanation(line: 22, short: '缩减工作区', detail: '新元素太小，放到工作区末尾并缩小有效范围，留待下一段处理。'),
          ],
        ),
      ],
    ),
];
