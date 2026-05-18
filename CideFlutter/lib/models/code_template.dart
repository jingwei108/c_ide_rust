class CodeTemplate {
  final String key;
  final String displayName;
  final String category;
  final String code;

  const CodeTemplate(this.key, this.displayName, this.category, this.code);

  static const List<CodeTemplate> defaults = [
    // ========== 排序 ==========
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
    CodeTemplate('selection', '选择排序', '排序',
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
      '}'),
    CodeTemplate('insertion', '插入排序', '排序',
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
    CodeTemplate('merge', '归并排序', '排序',
      'void mergeSort(int arr[], int left, int right) {\n'
      '    if (left < right) {\n'
      '        int mid = left + (right - left) / 2;\n'
      '        mergeSort(arr, left, mid);\n'
      '        mergeSort(arr, mid + 1, right);\n'
      '        merge(arr, left, mid, right);\n'
      '    }\n'
      '}\n\n'
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
      '}'),
    // ========== 查找 ==========
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
    CodeTemplate('linear', '线性查找', '查找',
      'int linearSearch(int arr[], int n, int target) {\n'
      '    for (int i = 0; i < n; i++) {\n'
      '        if (arr[i] == target)\n'
      '            return i;\n'
      '    }\n'
      '    return -1;\n'
      '}'),
    // ========== 数据结构 ==========
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
    CodeTemplate('linkedInsert', '链表头插法', '结构',
      'struct Node* insertFront(struct Node* head, int data) {\n'
      '    struct Node* newNode = createNode(data);\n'
      '    newNode->next = head;\n'
      '    return newNode;\n'
      '}'),
    CodeTemplate('linkedTraverse', '链表遍历', '结构',
      'void printList(struct Node* head) {\n'
      '    struct Node* p = head;\n'
      '    while (p != NULL) {\n'
      '        printf("%d ", p->data);\n'
      '        p = p->next;\n'
      '    }\n'
      '    printf("\\n");\n'
      '}'),
    CodeTemplate('treeNode', '二叉树节点', '结构',
      'struct TreeNode {\n'
      '    int val;\n'
      '    struct TreeNode* left;\n'
      '    struct TreeNode* right;\n'
      '};\n\n'
      'struct TreeNode* createTreeNode(int val) {\n'
      '    struct TreeNode* node = (struct TreeNode*)malloc(sizeof(struct TreeNode));\n'
      '    node->val = val;\n'
      '    node->left = NULL;\n'
      '    node->right = NULL;\n'
      '    return node;\n'
      '}'),
    CodeTemplate('treePreorder', '先序遍历', '结构',
      'void preorder(struct TreeNode* root) {\n'
      '    if (root == NULL) return;\n'
      '    printf("%d ", root->val);\n'
      '    preorder(root->left);\n'
      '    preorder(root->right);\n'
      '}'),
    CodeTemplate('stackArray', '栈（数组）', '结构',
      'int stack[100];\n'
      'int top = -1;\n\n'
      'void push(int x) {\n'
      '    stack[++top] = x;\n'
      '}\n\n'
      'int pop() {\n'
      '    if (top < 0) return -1;\n'
      '    return stack[top--];\n'
      '}'),
    // ========== 基础 ==========
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
    // ========== 递归 ==========
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
  ];
}
