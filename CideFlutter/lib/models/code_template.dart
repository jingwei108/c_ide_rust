class CodeTemplate {
  final String key;
  final String displayName;
  final String category;
  final String code;

  const CodeTemplate(this.key, this.displayName, this.category, this.code);

  static const List<CodeTemplate> defaults = [
    CodeTemplate('bubble', '冒泡排序', '排序',
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
      '}'),
    CodeTemplate('binary', '二分查找', '查找',
      'int binarySearch(int arr[], int n, int target) {\n'
      '    int left = 0, right = n - 1;\n'
      '    while (left <= right) {\n'
      '        int mid = left + (right - left) / 2;\n'
      '        if (arr[mid] == target) return mid;\n'
      '        if (arr[mid] < target) left = mid + 1;\n'
      '        else right = mid - 1;\n'
      '    }\n'
      '    return -1;\n'
      '}'),
    CodeTemplate('linked', '链表节点', '结构',
      'struct Node {\n'
      '    int data;\n'
      '    struct Node* next;\n'
      '};\n\n'
      'struct Node* createNode(int data) {\n'
      '    struct Node* newNode = (struct Node*)malloc(sizeof(struct Node));\n'
      '    newNode->data = data;\n'
      '    newNode->next = NULL;\n'
      '    return newNode;\n'
      '}'),
    CodeTemplate('quick', '快速排序', '排序',
      'void quickSort(int arr[], int low, int high) {\n'
      '    if (low < high) {\n'
      '        int pivot = partition(arr, low, high);\n'
      '        quickSort(arr, low, pivot - 1);\n'
      '        quickSort(arr, pivot + 1, high);\n'
      '    }\n'
      '}\n\n'
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
      '}'),
    CodeTemplate('factorial', '递归阶乘', '递归',
      'int factorial(int n) {\n'
      '    if (n <= 1) return 1;\n'
      '    return n * factorial(n - 1);\n'
      '}'),
    CodeTemplate('fib', '斐波那契', '递归',
      'int fibonacci(int n) {\n'
      '    if (n <= 1) return n;\n'
      '    return fibonacci(n - 1) + fibonacci(n - 2);\n'
      '}'),
    CodeTemplate('array', '数组遍历', '基础',
      'int arr[5] = {1, 2, 3, 4, 5};\n'
      'int sum = 0;\n'
      'for (int i = 0; i < 5; i++) {\n'
      '    sum = sum + arr[i];\n'
      '}\n'
      'printf("%d", sum);'),
    CodeTemplate('pointer', '指针交换', '指针',
      'void swap(int* a, int* b) {\n'
      '    int temp = *a;\n'
      '    *a = *b;\n'
      '    *b = temp;\n'
      '}'),
  ];
}
