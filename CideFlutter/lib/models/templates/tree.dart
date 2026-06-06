import '../code_template.dart';

const List<CodeTemplate> treeTemplates = [
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
        TutorialStep(
          title: '先序的应用',
          description: '先序遍历在复制树、序列化树等场景很有用，因为根节点信息最先输出，便于重建树结构。',
          focusLines: [19, 20, 21, 22, 23],
          explanations: [
            LineExplanation(line: 20, short: '根优先', detail: '先输出根，可以在遍历早期就知道树的根节点。'),
            LineExplanation(line: 19, short: '递归框架', detail: '三种遍历（先/中/后）唯一的区别是访问根的时机不同。'),
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
        TutorialStep(
          title: 'BST 的中序性质',
          description: '对二叉搜索树进行中序遍历，输出结果一定是升序序列。这是 BST 最重要的性质之一，也是验证 BST 的常用方法。',
          focusLines: [18, 19, 20, 21, 22],
          explanations: [
            LineExplanation(line: 19, short: '左子树', detail: 'BST 左子树所有节点 < 根，所以根之前输出的都更小。'),
            LineExplanation(line: 20, short: '根', detail: '根的值介于左子树最大值和右子树最小值之间。'),
            LineExplanation(line: 21, short: '右子树', detail: 'BST 右子树所有节点 >= 根，所以根之后输出的都更大。'),
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
        TutorialStep(
          title: '后序的应用',
          description: '后序遍历在删除树、计算树高、表达式树求值等场景非常有用，因为需要先处理完所有子节点才能处理根。',
          focusLines: [18, 19, 20, 21, 22],
          explanations: [
            LineExplanation(line: 21, short: '最后访问根', detail: '确保在访问根之前，左右子树已经完全处理完毕。'),
            LineExplanation(line: 19, short: '删除树', detail: 'free(root) 之前必须先 free(root->left) 和 free(root->right)。'),
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
        TutorialStep(
          title: 'BFS 思想',
          description: '层序遍历按树的深度一层一层访问，与 DFS（深度优先）的递归遍历形成对比。队列保证了先访问的节点其孩子也先被访问。',
          focusLines: [20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30],
          explanations: [
            LineExplanation(line: 24, short: 'FIFO', detail: '队列的先进先出特性保证了同一层节点按从左到右顺序访问。'),
            LineExplanation(line: 28, short: '逐层展开', detail: '父节点出队后，其子节点入队，自然形成按层遍历。'),
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
        TutorialStep(
          title: '查找效率',
          description: 'BST 查找的时间复杂度取决于树高。平衡时 O(log n)，退化为链时 O(n)。',
          focusLines: [27, 28, 29, 30, 31, 32, 33],
          explanations: [
            LineExplanation(line: 27, short: '二分思想', detail: '每次比较排除约一半节点，类似二分查找。'),
            LineExplanation(line: 29, short: '最坏情况', detail: '如果插入序列有序，BST 退化为链表，查找变为 O(n)。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'threadedBinaryTree',
      '线索二叉树',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '\n'
      'typedef enum { Link, Thread } PointerTag;\n'
      '\n'
      'struct BiThrNode {\n'
      '    int data;\n'
      '    struct BiThrNode* lchild;\n'
      '    struct BiThrNode* rchild;\n'
      '    PointerTag LTag;\n'
      '    PointerTag RTag;\n'
      '};\n'
      '\n'
      'struct BiThrNode* createNode(int data) {\n'
      '    struct BiThrNode* node = (struct BiThrNode*)malloc(sizeof(struct BiThrNode));\n'
      '    node->data = data;\n'
      '    node->lchild = NULL;\n'
      '    node->rchild = NULL;\n'
      '    node->LTag = Link;\n'
      '    node->RTag = Link;\n'
      '    return node;\n'
      '}\n'
      '\n'
      'struct BiThrNode* pre = NULL;\n'
      '\n'
      'void InThreading(struct BiThrNode* p) {\n'
      '    if (p) {\n'
      '        InThreading(p->lchild);\n'
      '        if (p->lchild == NULL) {\n'
      '            p->LTag = Thread;\n'
      '            p->lchild = pre;\n'
      '        }\n'
      '        if (pre && pre->rchild == NULL) {\n'
      '            pre->RTag = Thread;\n'
      '            pre->rchild = p;\n'
      '        }\n'
      '        pre = p;\n'
      '        InThreading(p->rchild);\n'
      '    }\n'
      '}\n'
      '\n'
      'void InOrderTraverse_Thr(struct BiThrNode* T) {\n'
      '    struct BiThrNode* p = T;\n'
      '    while (p) {\n'
      '        while (p->LTag == Link) p = p->lchild;\n'
      '        printf("%d ", p->data);\n'
      '        while (p->RTag == Thread && p->rchild != T) {\n'
      '            p = p->rchild;\n'
      '            printf("%d ", p->data);\n'
      '        }\n'
      '        p = p->rchild;\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct BiThrNode* root = createNode(1);\n'
      '    root->lchild = createNode(2);\n'
      '    root->rchild = createNode(3);\n'
      '    root->lchild->lchild = createNode(4);\n'
      '    root->lchild->rchild = createNode(5);\n'
      '    InThreading(root);\n'
      '    pre->rchild = NULL;\n'
      '    pre->RTag = Thread;\n'
      '    InOrderTraverse_Thr(root);\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '线索化标记',
          description: '为每个节点增加 LTag 和 RTag：Link 表示指向孩子，Thread 表示指向中序前驱或后继。',
          focusLines: [4, 5, 6, 7, 8, 9, 10, 11, 12],
          explanations: [
            LineExplanation(line: 5, short: '枚举', detail: 'Link 表示正常的孩子指针，Thread 表示线索。'),
            LineExplanation(line: 9, short: '左标记', detail: 'LTag 区分 lchild 是左孩子还是中序前驱。'),
            LineExplanation(line: 10, short: '右标记', detail: 'RTag 区分 rchild 是右孩子还是中序后继。'),
          ],
        ),
        TutorialStep(
          title: '中序线索化',
          description: '中序遍历过程中，遇到左孩子为空的节点就把它左指针指向前驱 pre；如果前驱的右孩子为空，就把前驱右指针指向当前节点。',
          focusLines: [26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39],
          explanations: [
            LineExplanation(line: 28, short: '递归左子树', detail: '中序线索化左子树。'),
            LineExplanation(line: 30, short: '左指针为空', detail: 'p->lchild == NULL 说明 p 是中序第一个或叶子节点。'),
            LineExplanation(line: 31, short: '加前驱线索', detail: 'LTag = Thread，lchild 指向中序前驱 pre。'),
            LineExplanation(line: 33, short: '前驱右指针为空', detail: 'pre 存在且右孩子为空，可以给 pre 加后继线索。'),
            LineExplanation(line: 35, short: '加后继线索', detail: 'pre->rchild = p，pre 的右指针指向中序后继。'),
            LineExplanation(line: 37, short: '更新 pre', detail: 'pre = p，当前节点成为下一个节点的前驱。'),
          ],
        ),
        TutorialStep(
          title: '无栈中序遍历',
          description: '线索化后，不需要递归或栈。先找到最左节点，然后如果有后继线索就直接沿线索访问，否则回到右子树继续找最左节点。',
          focusLines: [42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53],
          explanations: [
            LineExplanation(line: 45, short: '找最左', detail: 'while (LTag == Link) 一路向左，找到中序第一个节点。'),
            LineExplanation(line: 46, short: '访问', detail: '输出节点值。'),
            LineExplanation(line: 47, short: '沿线索走', detail: 'RTag == Thread 时 rchild 是中序后继，直接访问。'),
            LineExplanation(line: 52, short: '转右子树', detail: '没有后继线索时，p = p->rchild，进入右子树继续找最左。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'huffmanTree',
      '哈夫曼树',
      '结构',
      '#include <stdio.h>\n'
      '#define N 5\n'
      '#define M 9\n'
      '\n'
      'typedef struct {\n'
      '    int weight;\n'
      '    int parent;\n'
      '    int lchild;\n'
      '    int rchild;\n'
      '} HTNode;\n'
      '\n'
      'void Select(HTNode HT[], int n, int* s1, int* s2) {\n'
      '    int min1 = 100000, min2 = 100000;\n'
      '    *s1 = *s2 = -1;\n'
      '    for (int i = 0; i < n; i++) {\n'
      '        if (HT[i].parent == -1 && HT[i].weight < min1) {\n'
      '            min2 = min1;\n'
      '            *s2 = *s1;\n'
      '            min1 = HT[i].weight;\n'
      '            *s1 = i;\n'
      '        } else if (HT[i].parent == -1 && HT[i].weight < min2) {\n'
      '            min2 = HT[i].weight;\n'
      '            *s2 = i;\n'
      '        }\n'
      '    }\n'
      '}\n'
      '\n'
      'void CreateHuffmanTree(HTNode HT[], int w[], int n) {\n'
      '    int m = 2 * n - 1;\n'
      '    for (int i = 0; i < m; i++) {\n'
      '        HT[i].lchild = HT[i].rchild = HT[i].parent = -1;\n'
      '    }\n'
      '    for (int i = 0; i < n; i++) HT[i].weight = w[i];\n'
      '    for (int i = n; i < m; i++) {\n'
      '        int s1, s2;\n'
      '        Select(HT, i, &s1, &s2);\n'
      '        HT[s1].parent = i;\n'
      '        HT[s2].parent = i;\n'
      '        HT[i].lchild = s1;\n'
      '        HT[i].rchild = s2;\n'
      '        HT[i].weight = HT[s1].weight + HT[s2].weight;\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    int w[N] = {5, 29, 7, 8, 14};\n'
      '    HTNode ht[M];\n'
      '    CreateHuffmanTree(ht, w, N);\n'
      '    printf("root weight=%d\\n", ht[M-1].weight);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '哈夫曼树结构',
          description: '用结构体数组表示哈夫曼树，每个节点记录 weight、parent、lchild、rchild。parent = -1 表示尚未合并。',
          focusLines: [5, 6, 7, 8, 9, 10, 11],
          explanations: [
            LineExplanation(line: 6, short: '权值', detail: 'weight 表示该节点的权值，叶子节点为原始数据权值。'),
            LineExplanation(line: 7, short: '双亲', detail: 'parent 记录该节点在数组中的双亲下标，-1 表示无双亲。'),
            LineExplanation(line: 8, short: '左右孩子', detail: 'lchild 和 rchild 记录左右孩子在数组中的下标。'),
          ],
        ),
        TutorialStep(
          title: '选择最小权值节点',
          description: 'Select 函数在尚未合并的节点中找出权值最小的两个节点。',
          focusLines: [13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26],
          explanations: [
            LineExplanation(line: 14, short: '初始化', detail: 'min1、min2 初始为极大值，s1、s2 为最小两个节点的下标。'),
            LineExplanation(line: 17, short: '筛选条件', detail: 'HT[i].parent == -1 表示该节点尚未被合并。'),
            LineExplanation(line: 18, short: '更新最小', detail: '发现更小的权值时，原最小退为次小，再更新最小。'),
          ],
        ),
        TutorialStep(
          title: '构造哈夫曼树',
          description: '每次选出两个权值最小的节点，合并为新节点，新节点权值为两者之和。重复 n-1 次直到所有节点合并完成。',
          focusLines: [28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42],
          explanations: [
            LineExplanation(line: 30, short: '初始化', detail: '所有节点的 parent、lchild、rchild 初始化为 -1。'),
            LineExplanation(line: 33, short: '填充叶子', detail: '前 n 个节点为叶子，填入原始权值。'),
            LineExplanation(line: 36, short: '选最小两节点', detail: 'Select 找出当前森林中权值最小的两棵树。'),
            LineExplanation(line: 37, short: '建立双亲关系', detail: 'HT[s1].parent = i，s1 和 s2 的双亲设为 i。'),
            LineExplanation(line: 40, short: '新节点权值', detail: 'HT[i].weight = HT[s1].weight + HT[s2].weight。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'avlTree',
      'AVL 树插入',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '\n'
      'struct AVLNode {\n'
      '    int data;\n'
      '    struct AVLNode* lchild;\n'
      '    struct AVLNode* rchild;\n'
      '    int bf;\n'
      '};\n'
      '\n'
      'struct AVLNode* createNode(int data) {\n'
      '    struct AVLNode* node = (struct AVLNode*)malloc(sizeof(struct AVLNode));\n'
      '    node->data = data;\n'
      '    node->lchild = NULL;\n'
      '    node->rchild = NULL;\n'
      '    node->bf = 0;\n'
      '    return node;\n'
      '}\n'
      '\n'
      'void R_Rotate(struct AVLNode** p) {\n'
      '    struct AVLNode* lc = (*p)->lchild;\n'
      '    (*p)->lchild = lc->rchild;\n'
      '    lc->rchild = *p;\n'
      '    *p = lc;\n'
      '}\n'
      '\n'
      'void L_Rotate(struct AVLNode** p) {\n'
      '    struct AVLNode* rc = (*p)->rchild;\n'
      '    (*p)->rchild = rc->lchild;\n'
      '    rc->lchild = *p;\n'
      '    *p = rc;\n'
      '}\n'
      '\n'
      'void LeftBalance(struct AVLNode** T) {\n'
      '    struct AVLNode* lc = (*T)->lchild;\n'
      '    switch (lc->bf) {\n'
      '        case 1:\n'
      '            (*T)->bf = lc->bf = 0;\n'
      '            R_Rotate(T);\n'
      '            break;\n'
      '        case -1: {\n'
      '            struct AVLNode* rd = lc->rchild;\n'
      '            switch (rd->bf) {\n'
      '                case 1: (*T)->bf = -1; lc->bf = 0; break;\n'
      '                case 0: (*T)->bf = lc->bf = 0; break;\n'
      '                case -1: (*T)->bf = 0; lc->bf = 1; break;\n'
      '            }\n'
      '            rd->bf = 0;\n'
      '            L_Rotate(&((*T)->lchild));\n'
      '            R_Rotate(T);\n'
      '            break;\n'
      '        }\n'
      '    }\n'
      '}\n'
      '\n'
      'void RightBalance(struct AVLNode** T) {\n'
      '    struct AVLNode* rc = (*T)->rchild;\n'
      '    switch (rc->bf) {\n'
      '        case -1:\n'
      '            (*T)->bf = rc->bf = 0;\n'
      '            L_Rotate(T);\n'
      '            break;\n'
      '        case 1: {\n'
      '            struct AVLNode* ld = rc->lchild;\n'
      '            switch (ld->bf) {\n'
      '                case 1: (*T)->bf = 0; rc->bf = -1; break;\n'
      '                case 0: (*T)->bf = rc->bf = 0; break;\n'
      '                case -1: (*T)->bf = 1; rc->bf = 0; break;\n'
      '            }\n'
      '            ld->bf = 0;\n'
      '            R_Rotate(&((*T)->rchild));\n'
      '            L_Rotate(T);\n'
      '            break;\n'
      '        }\n'
      '    }\n'
      '}\n'
      '\n'
      'int InsertAVL(struct AVLNode** T, int e) {\n'
      '    if (*T == NULL) {\n'
      '        *T = createNode(e);\n'
      '        return 1;\n'
      '    }\n'
      '    if (e == (*T)->data) return 0;\n'
      '    else if (e < (*T)->data) {\n'
      '        if (!InsertAVL(&((*T)->lchild), e)) return 0;\n'
      '        switch ((*T)->bf) {\n'
      '            case 1: LeftBalance(T); break;\n'
      '            case 0: (*T)->bf = 1; break;\n'
      '            case -1: (*T)->bf = 0; break;\n'
      '        }\n'
      '    } else {\n'
      '        if (!InsertAVL(&((*T)->rchild), e)) return 0;\n'
      '        switch ((*T)->bf) {\n'
      '            case -1: RightBalance(T); break;\n'
      '            case 0: (*T)->bf = -1; break;\n'
      '            case 1: (*T)->bf = 0; break;\n'
      '        }\n'
      '    }\n'
      '    return 1;\n'
      '}\n'
      '\n'
      'void inorder(struct AVLNode* T) {\n'
      '    if (T) {\n'
      '        inorder(T->lchild);\n'
      '        printf("%d ", T->data);\n'
      '        inorder(T->rchild);\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct AVLNode* T = NULL;\n'
      '    InsertAVL(&T, 3);\n'
      '    InsertAVL(&T, 2);\n'
      '    InsertAVL(&T, 1);\n'
      '    InsertAVL(&T, 4);\n'
      '    InsertAVL(&T, 5);\n'
      '    inorder(T);\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: 'AVL 节点结构',
          description: 'AVL 树在 BST 基础上增加平衡因子 bf（左子树深度减右子树深度）。bf 的绝对值不超过 1 时树平衡。',
          focusLines: [4, 5, 6, 7, 8, 9],
          explanations: [
            LineExplanation(line: 5, short: '数据域', detail: 'data 存储节点值。'),
            LineExplanation(line: 6, short: '左右子树', detail: 'lchild 和 rchild 指向左右子节点。'),
            LineExplanation(line: 7, short: '平衡因子', detail: 'bf = 左子树高度 - 右子树高度，合法值为 -1、0、1。'),
          ],
        ),
        TutorialStep(
          title: '旋转操作',
          description: '当插入导致不平衡时，通过单旋转（LL/RR）或双旋转（LR/RL）恢复平衡。R_Rotate 是右旋，L_Rotate 是左旋。',
          focusLines: [16, 17, 18, 19, 20, 21, 23, 24, 25, 26, 27, 28],
          explanations: [
            LineExplanation(line: 17, short: '保存左孩子', detail: 'lc = (*p)->lchild，右旋围绕 lc 进行。'),
            LineExplanation(line: 18, short: '接管右子树', detail: '(*p)->lchild = lc->rchild，p 的左子树变成 lc 的右子树。'),
            LineExplanation(line: 19, short: '右旋', detail: 'lc->rchild = *p，lc 成为新的子树根。'),
            LineExplanation(line: 24, short: '保存右孩子', detail: 'rc = (*p)->rchild，左旋围绕 rc 进行。'),
            LineExplanation(line: 26, short: '左旋', detail: 'rc->lchild = *p，rc 成为新的子树根。'),
          ],
        ),
        TutorialStep(
          title: '插入与平衡调整',
          description: '递归插入后，根据当前节点的 bf 判断是否失衡，并调用对应的平衡函数。插入左子树后 bf 增加，插入右子树后 bf 减少。',
          focusLines: [54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80],
          explanations: [
            LineExplanation(line: 56, short: '空树', detail: '*T == NULL 时创建新节点。'),
            LineExplanation(line: 60, short: '递归左插', detail: 'e < (*T)->data 时递归插入左子树。'),
            LineExplanation(line: 63, short: '失衡处理', detail: 'bf 原来是 1，左插后变成 2，需要左平衡。'),
            LineExplanation(line: 66, short: '更新 bf', detail: 'bf 原来是 0，左插后变成 1，树仍平衡。'),
            LineExplanation(line: 72, short: '递归右插', detail: 'e >= (*T)->data 时递归插入右子树。'),
            LineExplanation(line: 75, short: '失衡处理', detail: 'bf 原来是 -1，右插后变成 -2，需要右平衡。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'bstDelete',
      'BST 节点删除',
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
      'struct TreeNode* findMin(struct TreeNode* root) {\n'
      '    while (root->left != NULL) root = root->left;\n'
      '    return root;\n'
      '}\n'
      '\n'
      'struct TreeNode* deleteNode(struct TreeNode* root, int key) {\n'
      '    if (root == NULL) return NULL;\n'
      '    if (key < root->val)\n'
      '        root->left = deleteNode(root->left, key);\n'
      '    else if (key > root->val)\n'
      '        root->right = deleteNode(root->right, key);\n'
      '    else {\n'
      '        if (root->left == NULL) {\n'
      '            struct TreeNode* temp = root->right;\n'
      '            free(root);\n'
      '            return temp;\n'
      '        } else if (root->right == NULL) {\n'
      '            struct TreeNode* temp = root->left;\n'
      '            free(root);\n'
      '            return temp;\n'
      '        }\n'
      '        struct TreeNode* temp = findMin(root->right);\n'
      '        root->val = temp->val;\n'
      '        root->right = deleteNode(root->right, temp->val);\n'
      '    }\n'
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
      '    insert(root, 2);\n'
      '    insert(root, 4);\n'
      '    insert(root, 6);\n'
      '    insert(root, 8);\n'
      '    root = deleteNode(root, 3);\n'
      '    inorder(root);\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '删除叶子或单孩子',
          description: '如果目标节点没有左子树，用右子树替代；没有右子树，用左子树替代。然后释放原节点。',
          focusLines: [29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39],
          explanations: [
            LineExplanation(line: 30, short: '去左子树找', detail: 'key < root->val 时递归到左子树删除。'),
            LineExplanation(line: 33, short: '找到目标', detail: 'key == root->val 时开始处理删除。'),
            LineExplanation(line: 35, short: '无左子树', detail: '直接用右子树替代，free 释放当前节点。'),
            LineExplanation(line: 38, short: '无右子树', detail: '直接用左子树替代，free 释放当前节点。'),
          ],
        ),
        TutorialStep(
          title: '删除双孩子节点',
          description: '如果目标节点有两个孩子，找到右子树的最小节点（中序后继），用它的值替换目标节点，然后递归删除那个最小节点。',
          focusLines: [40, 41, 42, 43],
          explanations: [
            LineExplanation(line: 41, short: '找中序后继', detail: 'findMin(root->right) 找到右子树中最小节点。'),
            LineExplanation(line: 42, short: '值替换', detail: '把中序后继的值复制到当前节点，保持 BST 性质。'),
            LineExplanation(line: 43, short: '删除后继', detail: '递归删除右子树中的中序后继节点。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'binarySearchTreeValidation',
      '验证 BST',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '#include <limits.h>\n'
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
      'int isValidBST(struct TreeNode* root, long long min, long long max) {\n'
      '    if (root == NULL) return 1;\n'
      '    if (root->val <= min || root->val >= max) return 0;\n'
      '    return isValidBST(root->left, min, root->val) &&\n'
      '           isValidBST(root->right, root->val, max);\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct TreeNode* root = createNode(5);\n'
      '    root->left = createNode(3);\n'
      '    root->right = createNode(7);\n'
      '    root->left->left = createNode(1);\n'
      '    root->left->right = createNode(4);\n'
      '    if (isValidBST(root, (long long)INT_MIN - 1, (long long)INT_MAX + 1))\n'
      '        printf("valid\\n");\n'
      '    else\n'
      '        printf("invalid\\n");\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: 'BST 验证思想',
          description: 'BST 的每个节点值必须在 (min, max) 开区间内。递归检查左右子树时更新区间边界。',
          focusLines: [18, 19, 20, 21, 22],
          explanations: [
            LineExplanation(line: 19, short: '空树是BST', detail: '空节点满足 BST 定义，返回 1。'),
            LineExplanation(line: 20, short: '区间检查', detail: '当前节点值必须在 (min, max) 之间。'),
            LineExplanation(line: 21, short: '递归左子树', detail: '左子树所有节点必须小于 root->val，所以 max 更新为 root->val。'),
            LineExplanation(line: 22, short: '递归右子树', detail: '右子树所有节点必须大于 root->val，所以 min 更新为 root->val。'),
          ],
        ),
        TutorialStep(
          title: '为什么不能用中序遍历',
          description: '中序遍历递增只能验证无重复值的 BST。如果有重复值，中序遍历无法区分 BST 和普通二叉树。区间法更通用。',
          focusLines: [18, 19, 20, 21, 22],
          explanations: [
            LineExplanation(line: 20, short: '重复值', detail: '<= min 或 >= max 时返回 0，严格保证了 BST 性质。'),
            LineExplanation(line: 18, short: '初始区间', detail: '用 long long 扩展边界，避免 INT_MIN/MAX 的边界问题。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'treeToForest',
      '树与森林转换',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '\n'
      'struct CSNode {\n'
      '    int data;\n'
      '    struct CSNode* firstChild;\n'
      '    struct CSNode* nextSibling;\n'
      '};\n'
      '\n'
      'struct CSNode* createNode(int data) {\n'
      '    struct CSNode* node = (struct CSNode*)malloc(sizeof(struct CSNode));\n'
      '    node->data = data;\n'
      '    node->firstChild = NULL;\n'
      '    node->nextSibling = NULL;\n'
      '    return node;\n'
      '}\n'
      '\n'
      'void traverse(struct CSNode* root) {\n'
      '    if (root == NULL) return;\n'
      '    printf("%d ", root->data);\n'
      '    traverse(root->firstChild);\n'
      '    traverse(root->nextSibling);\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct CSNode* root = createNode(1);\n'
      '    root->firstChild = createNode(2);\n'
      '    root->firstChild->nextSibling = createNode(3);\n'
      '    root->firstChild->nextSibling->nextSibling = createNode(4);\n'
      '    root->firstChild->firstChild = createNode(5);\n'
      '    traverse(root);\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '孩子-兄弟表示法',
          description: '把一般的树用二叉链表存储：firstChild 指向第一个孩子，nextSibling 指向右邻兄弟。这样可以把任意树/森林转换成二叉树形式。',
          focusLines: [4, 5, 6, 7],
          explanations: [
            LineExplanation(line: 5, short: '数据域', detail: 'data 存储节点值。'),
            LineExplanation(line: 6, short: '第一个孩子', detail: 'firstChild 指向该节点的长子。'),
            LineExplanation(line: 7, short: '右兄弟', detail: 'nextSibling 指向该节点的右邻兄弟。'),
          ],
        ),
        TutorialStep(
          title: '先根遍历',
          description: '孩子-兄弟表示法的先根遍历：先访问当前节点，再递归遍历长子，然后递归遍历右兄弟。',
          focusLines: [16, 17, 18, 19, 20, 21],
          explanations: [
            LineExplanation(line: 18, short: '访问根', detail: '先输出当前节点的值。'),
            LineExplanation(line: 19, short: '遍历孩子', detail: 'traverse(root->firstChild) 递归遍历所有子树。'),
            LineExplanation(line: 20, short: '遍历兄弟', detail: 'traverse(root->nextSibling) 递归遍历右邻兄弟。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'redBlackTree',
      '红黑树',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '\n'
      'enum Color { RED, BLACK };\n'
      '\n'
      'struct RBNode {\n'
      '    int data;\n'
      '    enum Color color;\n'
      '    struct RBNode* left;\n'
      '    struct RBNode* right;\n'
      '    struct RBNode* parent;\n'
      '};\n'
      '\n'
      'struct RBNode* createNode(int data) {\n'
      '    struct RBNode* node = (struct RBNode*)malloc(sizeof(struct RBNode));\n'
      '    node->data = data;\n'
      '    node->color = RED;\n'
      '    node->left = node->right = node->parent = NULL;\n'
      '    return node;\n'
      '}\n'
      '\n'
      'struct RBNode* leftRotate(struct RBNode* root, struct RBNode* x) {\n'
      '    struct RBNode* y = x->right;\n'
      '    x->right = y->left;\n'
      '    if (y->left) y->left->parent = x;\n'
      '    y->parent = x->parent;\n'
      '    if (!x->parent) root = y;\n'
      '    else if (x == x->parent->left) x->parent->left = y;\n'
      '    else x->parent->right = y;\n'
      '    y->left = x;\n'
      '    x->parent = y;\n'
      '    return root;\n'
      '}\n'
      '\n'
      'struct RBNode* rightRotate(struct RBNode* root, struct RBNode* y) {\n'
      '    struct RBNode* x = y->left;\n'
      '    y->left = x->right;\n'
      '    if (x->right) x->right->parent = y;\n'
      '    x->parent = y->parent;\n'
      '    if (!y->parent) root = x;\n'
      '    else if (y == y->parent->left) y->parent->left = x;\n'
      '    else y->parent->right = x;\n'
      '    x->right = y;\n'
      '    y->parent = x;\n'
      '    return root;\n'
      '}\n'
      '\n'
      'struct RBNode* fixInsert(struct RBNode* root, struct RBNode* z) {\n'
      '    while (z->parent && z->parent->color == RED) {\n'
      '        if (z->parent == z->parent->parent->left) {\n'
      '            struct RBNode* y = z->parent->parent->right;\n'
      '            if (y && y->color == RED) {\n'
      '                z->parent->color = BLACK;\n'
      '                y->color = BLACK;\n'
      '                z->parent->parent->color = RED;\n'
      '                z = z->parent->parent;\n'
      '            } else {\n'
      '                if (z == z->parent->right) {\n'
      '                    z = z->parent;\n'
      '                    root = leftRotate(root, z);\n'
      '                }\n'
      '                z->parent->color = BLACK;\n'
      '                z->parent->parent->color = RED;\n'
      '                root = rightRotate(root, z->parent->parent);\n'
      '            }\n'
      '        } else {\n'
      '            struct RBNode* y = z->parent->parent->left;\n'
      '            if (y && y->color == RED) {\n'
      '                z->parent->color = BLACK;\n'
      '                y->color = BLACK;\n'
      '                z->parent->parent->color = RED;\n'
      '                z = z->parent->parent;\n'
      '            } else {\n'
      '                if (z == z->parent->left) {\n'
      '                    z = z->parent;\n'
      '                    root = rightRotate(root, z);\n'
      '                }\n'
      '                z->parent->color = BLACK;\n'
      '                z->parent->parent->color = RED;\n'
      '                root = leftRotate(root, z->parent->parent);\n'
      '            }\n'
      '        }\n'
      '    }\n'
      '    root->color = BLACK;\n'
      '    return root;\n'
      '}\n'
      '\n'
      'struct RBNode* insert(struct RBNode* root, int data) {\n'
      '    struct RBNode* z = createNode(data);\n'
      '    struct RBNode* y = NULL;\n'
      '    struct RBNode* x = root;\n'
      '    while (x) {\n'
      '        y = x;\n'
      '        if (z->data < x->data) x = x->left;\n'
      '        else x = x->right;\n'
      '    }\n'
      '    z->parent = y;\n'
      '    if (!y) root = z;\n'
      '    else if (z->data < y->data) y->left = z;\n'
      '    else y->right = z;\n'
      '    return fixInsert(root, z);\n'
      '}\n'
      '\n'
      'void inorder(struct RBNode* root) {\n'
      '    if (root) {\n'
      '        inorder(root->left);\n'
      '        printf("%d ", root->data);\n'
      '        inorder(root->right);\n'
      '    }\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct RBNode* root = NULL;\n'
      '    root = insert(root, 7);\n'
      '    root = insert(root, 3);\n'
      '    root = insert(root, 18);\n'
      '    root = insert(root, 10);\n'
      '    root = insert(root, 22);\n'
      '    inorder(root);\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '红黑树性质',
          description: '红黑树是平衡二叉搜索树，每个节点有颜色（红/黑）。新插入节点为红色，通过旋转和重新着色保持平衡。',
          focusLines: [4, 5, 6, 7, 8, 9, 10, 11, 12],
          explanations: [
            LineExplanation(line: 5, short: '颜色枚举', detail: 'RED 或 BLACK，用于维护红黑树的平衡性质。'),
            LineExplanation(line: 9, short: '父指针', detail: 'parent 指向父节点，插入修复时需要向上回溯。'),
            LineExplanation(line: 15, short: '新节点为红', detail: '新插入的节点初始为红色，减少违反黑高的概率。'),
          ],
        ),
        TutorialStep(
          title: '旋转操作',
          description: '左旋和右旋是红黑树调整树形的基础操作，旋转后保持 BST 性质不变，同时更新 parent 指针。',
          focusLines: [19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38],
          explanations: [
            LineExplanation(line: 21, short: '接管子树', detail: 'x->right = y->left，y 的左子树变成 x 的右子树。'),
            LineExplanation(line: 24, short: '更新根', detail: '如果 x 是根，旋转后 y 成为新根。'),
          ],
        ),
        TutorialStep(
          title: '插入修复',
          description: '插入新红节点后，如果父节点也是红色就违反了性质。通过检查叔叔节点颜色，决定是变色还是旋转。',
          focusLines: [41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76],
          explanations: [
            LineExplanation(line: 45, short: '叔叔为红', detail: 'y->color == RED 时，把父节点和叔叔变黑，祖父变红，然后继续向上检查。'),
            LineExplanation(line: 51, short: 'LR 双旋', detail: 'z 是右孩子时，先对父节点左旋，变成 LL 情况，再对祖父右旋。'),
            LineExplanation(line: 75, short: '根为黑', detail: '无论怎么调整，最后把根节点设为黑色，保证性质。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'bTree',
      'B 树插入与查找',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '#define M 3\n'
      '\n'
      'struct BTreeNode {\n'
      '    int keys[M];\n'
      '    struct BTreeNode* children[M + 1];\n'
      '    int keyCount;\n'
      '    int isLeaf;\n'
      '};\n'
      '\n'
      'struct BTreeNode* createNode(int isLeaf) {\n'
      '    struct BTreeNode* node = (struct BTreeNode*)malloc(sizeof(struct BTreeNode));\n'
      '    node->keyCount = 0;\n'
      '    node->isLeaf = isLeaf;\n'
      '    for (int i = 0; i <= M; i++) node->children[i] = NULL;\n'
      '    return node;\n'
      '}\n'
      '\n'
      'void traverse(struct BTreeNode* root) {\n'
      '    if (root) {\n'
      '        int i;\n'
      '        for (i = 0; i < root->keyCount; i++) {\n'
      '            if (!root->isLeaf) traverse(root->children[i]);\n'
      '            printf("%d ", root->keys[i]);\n'
      '        }\n'
      '        if (!root->isLeaf) traverse(root->children[i]);\n'
      '    }\n'
      '}\n'
      '\n'
      'int search(struct BTreeNode* root, int key) {\n'
      '    int i = 0;\n'
      '    while (i < root->keyCount && key > root->keys[i]) i++;\n'
      '    if (i < root->keyCount && key == root->keys[i]) return 1;\n'
      '    if (root->isLeaf) return 0;\n'
      '    return search(root->children[i], key);\n'
      '}\n'
      '\n'
      'void splitChild(struct BTreeNode* parent, int i, struct BTreeNode* child) {\n'
      '    struct BTreeNode* newNode = createNode(child->isLeaf);\n'
      '    newNode->keyCount = M / 2;\n'
      '    for (int j = 0; j < M / 2; j++)\n'
      '        newNode->keys[j] = child->keys[j + M / 2 + 1];\n'
      '    if (!child->isLeaf) {\n'
      '        for (int j = 0; j <= M / 2; j++)\n'
      '            newNode->children[j] = child->children[j + M / 2 + 1];\n'
      '    }\n'
      '    child->keyCount = M / 2;\n'
      '    for (int j = parent->keyCount; j >= i + 1; j--)\n'
      '        parent->children[j + 1] = parent->children[j];\n'
      '    parent->children[i + 1] = newNode;\n'
      '    for (int j = parent->keyCount - 1; j >= i; j--)\n'
      '        parent->keys[j + 1] = parent->keys[j];\n'
      '    parent->keys[i] = child->keys[M / 2];\n'
      '    parent->keyCount++;\n'
      '}\n'
      '\n'
      'void insertNonFull(struct BTreeNode* node, int key) {\n'
      '    int i = node->keyCount - 1;\n'
      '    if (node->isLeaf) {\n'
      '        while (i >= 0 && key < node->keys[i]) {\n'
      '            node->keys[i + 1] = node->keys[i];\n'
      '            i--;\n'
      '        }\n'
      '        node->keys[i + 1] = key;\n'
      '        node->keyCount++;\n'
      '    } else {\n'
      '        while (i >= 0 && key < node->keys[i]) i--;\n'
      '        i++;\n'
      '        if (node->children[i]->keyCount == M - 1) {\n'
      '            splitChild(node, i, node->children[i]);\n'
      '            if (key > node->keys[i]) i++;\n'
      '        }\n'
      '        insertNonFull(node->children[i], key);\n'
      '    }\n'
      '}\n'
      '\n'
      'struct BTreeNode* insert(struct BTreeNode* root, int key) {\n'
      '    if (!root) root = createNode(1);\n'
      '    if (root->keyCount == M - 1) {\n'
      '        struct BTreeNode* newRoot = createNode(0);\n'
      '        newRoot->children[0] = root;\n'
      '        splitChild(newRoot, 0, root);\n'
      '        insertNonFull(newRoot, key);\n'
      '        return newRoot;\n'
      '    }\n'
      '    insertNonFull(root, key);\n'
      '    return root;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct BTreeNode* root = NULL;\n'
      '    root = insert(root, 10);\n'
      '    root = insert(root, 20);\n'
      '    root = insert(root, 5);\n'
      '    root = insert(root, 6);\n'
      '    root = insert(root, 12);\n'
      '    root = insert(root, 30);\n'
      '    root = insert(root, 7);\n'
      '    root = insert(root, 17);\n'
      '    printf("%d\\n", search(root, 6));\n'
      '    traverse(root);\n'
      '    printf("\\n");\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: 'B 树节点结构',
          description: 'B 树是多路平衡查找树。每个节点最多 M-1 个关键字、M 个子树。这里 M=3（2-3 树）。',
          focusLines: [5, 6, 7, 8, 9, 10, 11],
          explanations: [
            LineExplanation(line: 6, short: '关键字数组', detail: 'keys[M] 存储节点内的关键字，按升序排列。'),
            LineExplanation(line: 7, short: '子树指针', detail: 'children[M+1] 指向子节点，数量比关键字多 1。'),
            LineExplanation(line: 9, short: '叶子标记', detail: 'isLeaf 标记是否为叶子节点，影响查找和插入路径。'),
          ],
        ),
        TutorialStep(
          title: '查找过程',
          description: '从根开始，在当前节点内顺序查找关键字。如果命中返回 1；如果没命中且是叶子返回 0；否则进入对应的子树继续查找。',
          focusLines: [25, 26, 27, 28, 29, 30, 31],
          explanations: [
            LineExplanation(line: 27, short: '节点内查找', detail: 'while 循环找到第一个 >= key 的位置。'),
            LineExplanation(line: 28, short: '命中', detail: 'key == root->keys[i] 时返回 1。'),
            LineExplanation(line: 30, short: '进入子树', detail: '递归进入第 i 棵子树继续查找。'),
          ],
        ),
        TutorialStep(
          title: '插入与分裂',
          description: '插入从根开始向下找叶子。如果途经的节点已满（M-1 个关键字），先分裂该节点，再继续插入。根满时树高加 1。',
          focusLines: [34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80],
          explanations: [
            LineExplanation(line: 38, short: '分裂', detail: '把满节点的中间关键字提升到父节点，节点分裂为左右两半。'),
            LineExplanation(line: 57, short: '叶子插入', detail: '在叶子节点中从后向前移动关键字，腾出位置插入新 key。'),
            LineExplanation(line: 70, short: '根满', detail: '根节点满时创建新根，原来的根分裂为两个子节点，树高加 1。'),
          ],
        ),
      ],
    ),
];
