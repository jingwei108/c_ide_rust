import '../code_template.dart';

const List<CodeTemplate> linkedListTemplates = [
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
        TutorialStep(
          title: '头插法的顺序',
          description: '头插法导致遍历输出顺序与插入顺序相反。先插入 3，再插入 2，最后插入 1，输出为 1 2 3。',
          focusLines: [23, 24, 25, 26, 27],
          explanations: [
            LineExplanation(line: 24, short: '第一次插入', detail: 'head = insertFront(head, 3)，链表变为 [3]。'),
            LineExplanation(line: 25, short: '第二次插入', detail: 'head = insertFront(head, 2)，链表变为 [2, 3]。'),
            LineExplanation(line: 26, short: '第三次插入', detail: 'head = insertFront(head, 1)，链表变为 [1, 2, 3]。'),
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
        TutorialStep(
          title: '头指针保护',
          description: '遍历链表时绝不能用 head 直接移动，否则会丢失链表起点，后续无法再访问链表。',
          focusLines: [18, 19, 20, 21, 22],
          explanations: [
            LineExplanation(line: 18, short: '复制头指针', detail: 'struct Node* p = head，创建临时指针 p 来遍历。'),
            LineExplanation(line: 19, short: '为什么不能动 head', detail: 'head 是链表的唯一入口，动了它就找不到链表起点了。'),
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
        TutorialStep(
          title: '时间复杂度',
          description: '尾插法每次都要从 head 遍历到尾部，时间复杂度为 O(n)。如果频繁尾插，可以维护一个 tail 指针优化到 O(1)。',
          focusLines: [19, 20, 21],
          explanations: [
            LineExplanation(line: 19, short: '遍历', detail: 'while (p->next != NULL) 逐个后移，直到找到尾节点。'),
            LineExplanation(line: 20, short: 'O(n)', detail: '链表长度为 n 时，需要 n 步才能到达尾部。'),
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
      'circularLinkedList',
      '循环链表',
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
      '    if (head == NULL) {\n'
      '        newNode->next = newNode;\n'
      '        return newNode;\n'
      '    }\n'
      '    struct Node* p = head;\n'
      '    while (p->next != head) p = p->next;\n'
      '    p->next = newNode;\n'
      '    newNode->next = head;\n'
      '    return head;\n'
      '}\n'
      '\n'
      'void printList(struct Node* head) {\n'
      '    if (head == NULL) return;\n'
      '    struct Node* p = head;\n'
      '    do {\n'
      '        printf("%d ", p->data);\n'
      '        p = p->next;\n'
      '    } while (p != head);\n'
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
          title: '循环链表结构',
          description: '循环链表的尾节点 next 指针回指头节点，而不是指向 NULL。遍历时用 do-while 判断终止条件。',
          focusLines: [4, 5, 6, 7, 16, 17, 18, 19, 20, 23, 24, 25],
          explanations: [
            LineExplanation(line: 5, short: '数据域', detail: 'data 存储节点值。'),
            LineExplanation(line: 6, short: '指针域', detail: 'next 指向下一个节点，尾节点指向头节点形成环。'),
            LineExplanation(line: 18, short: '空表建环', detail: 'head == NULL 时新节点自己指向自己，形成只有一个节点的环。'),
            LineExplanation(line: 24, short: '尾插链接', detail: 'p->next = newNode，原尾节点指向新节点。'),
            LineExplanation(line: 25, short: '闭环', detail: 'newNode->next = head，新尾节点回指头节点。'),
          ],
        ),
        TutorialStep(
          title: '遍历循环链表',
          description: '遍历循环链表不能简单判断 p != NULL，因为永远不会遇到 NULL。用 do-while 至少执行一次，终止条件是 p 再次等于 head。',
          focusLines: [29, 30, 31, 32, 33, 34, 35, 36],
          explanations: [
            LineExplanation(line: 30, short: '空表返回', detail: 'head == NULL 时直接返回，避免空指针访问。'),
            LineExplanation(line: 32, short: 'do-while', detail: '先执行循环体，再判断条件，保证至少访问一个节点。'),
            LineExplanation(line: 35, short: '终止条件', detail: 'p != head 表示还没绕完一圈；等于 head 时说明遍历完成。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'staticLinkedList',
      '静态链表',
      '结构',
      '#include <stdio.h>\n'
      '#define MAXSIZE 10\n'
      '\n'
      'struct Component {\n'
      '    int data;\n'
      '    int cur;\n'
      '};\n'
      '\n'
      'void initSpace(struct Component space[]) {\n'
      '    for (int i = 0; i < MAXSIZE - 1; i++)\n'
      '        space[i].cur = i + 1;\n'
      '    space[MAXSIZE - 1].cur = 0;\n'
      '}\n'
      '\n'
      'int mallocNode(struct Component space[]) {\n'
      '    int i = space[0].cur;\n'
      '    if (i) space[0].cur = space[i].cur;\n'
      '    return i;\n'
      '}\n'
      '\n'
      'void freeNode(struct Component space[], int k) {\n'
      '    space[k].cur = space[0].cur;\n'
      '    space[0].cur = k;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct Component space[MAXSIZE];\n'
      '    initSpace(space);\n'
      '    int head = mallocNode(space);\n'
      '    space[head].data = 10;\n'
      '    int p = mallocNode(space);\n'
      '    space[p].data = 20;\n'
      '    space[head].cur = p;\n'
      '    space[p].cur = 0;\n'
      '    printf("%d %d\\n", space[head].data, space[space[head].cur].data);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '静态链表结构',
          description: '静态链表用数组模拟链表，每个数组元素包含 data 和 cur（游标）。游标存放下一个元素的下标，0 表示空。',
          focusLines: [4, 5, 6, 7],
          explanations: [
            LineExplanation(line: 5, short: '数据域', detail: 'data 存储实际数据。'),
            LineExplanation(line: 6, short: '游标', detail: 'cur 存放下一个节点在数组中的下标，模拟指针。'),
          ],
        ),
        TutorialStep(
          title: '初始化与分配',
          description: '数组下标 0 的元素作为备用链表头，cur 指向第一个可用位置。初始化时把所有元素串成备用链表。',
          focusLines: [9, 10, 11, 12, 15, 16, 17, 18],
          explanations: [
            LineExplanation(line: 10, short: '备用链表', detail: 'space[i].cur = i+1，每个元素指向下一个空闲位置。'),
            LineExplanation(line: 12, short: '尾标记', detail: '最后一个空闲元素的 cur 置 0，表示备用链表结束。'),
            LineExplanation(line: 16, short: '取节点', detail: 'space[0].cur 是备用链表第一个可用位置。'),
            LineExplanation(line: 17, short: '更新头', detail: '分配后把备用链表头指向被分配节点的下一个空闲位置。'),
          ],
        ),
        TutorialStep(
          title: '回收节点',
          description: 'freeNode 把释放的节点插回备用链表头部，实现复用。',
          focusLines: [21, 22, 23],
          explanations: [
            LineExplanation(line: 22, short: '挂到备用链表', detail: 'space[k].cur = space[0].cur，被释放节点指向原来的第一个空闲节点。'),
            LineExplanation(line: 23, short: '更新头', detail: 'space[0].cur = k，备用链表头指向被释放节点。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'mergeSortedLists',
      '有序表合并',
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
      'struct Node* merge(struct Node* L1, struct Node* L2) {\n'
      '    struct Node dummy;\n'
      '    struct Node* tail = &dummy;\n'
      '    dummy.next = NULL;\n'
      '    while (L1 != NULL && L2 != NULL) {\n'
      '        if (L1->data <= L2->data) {\n'
      '            tail->next = L1;\n'
      '            L1 = L1->next;\n'
      '        } else {\n'
      '            tail->next = L2;\n'
      '            L2 = L2->next;\n'
      '        }\n'
      '        tail = tail->next;\n'
      '    }\n'
      '    if (L1 != NULL) tail->next = L1;\n'
      '    if (L2 != NULL) tail->next = L2;\n'
      '    return dummy.next;\n'
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
      '    struct Node* L1 = createNode(1);\n'
      '    L1->next = createNode(3);\n'
      '    L1->next->next = createNode(5);\n'
      '    struct Node* L2 = createNode(2);\n'
      '    L2->next = createNode(4);\n'
      '    L2->next->next = createNode(6);\n'
      '    struct Node* L3 = merge(L1, L2);\n'
      '    printList(L3);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '归并思想',
          description: '两个有序链表合并时，每次比较两个链表的当前节点，把较小者接到结果链表尾部。',
          focusLines: [18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32],
          explanations: [
            LineExplanation(line: 19, short: '哑节点', detail: 'dummy 作为结果链表的临时头节点，简化边界处理。'),
            LineExplanation(line: 22, short: '比较', detail: 'L1->data <= L2->data 时取 L1 的节点。'),
            LineExplanation(line: 27, short: '移动尾指针', detail: 'tail = tail->next，尾指针始终指向结果链表末尾。'),
            LineExplanation(line: 29, short: '链接剩余', detail: '其中一个链表遍历完后，直接把另一个链表剩余部分接上。'),
          ],
        ),
        TutorialStep(
          title: '复杂度分析',
          description: '合并过程只需要遍历两个链表各一次，时间复杂度为 O(m+n)，额外空间复杂度为 O(1)（只创建了哑节点，结果节点复用原链表节点）。',
          focusLines: [18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32],
          explanations: [
            LineExplanation(line: 22, short: '逐个比较', detail: '每轮循环处理一个节点，总共处理 m+n 个节点。'),
            LineExplanation(line: 29, short: 'O(1) 空间', detail: '结果链表节点来自原链表，没有额外 malloc。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'polynomialAdd',
      '多项式相加',
      '结构',
      '#include <stdio.h>\n'
      '#include <stdlib.h>\n'
      '\n'
      'struct Term {\n'
      '    int coef;\n'
      '    int exp;\n'
      '    struct Term* next;\n'
      '};\n'
      '\n'
      'struct Term* createTerm(int c, int e) {\n'
      '    struct Term* node = (struct Term*)malloc(sizeof(struct Term));\n'
      '    node->coef = c;\n'
      '    node->exp = e;\n'
      '    node->next = NULL;\n'
      '    return node;\n'
      '}\n'
      '\n'
      'struct Term* addPoly(struct Term* pa, struct Term* pb) {\n'
      '    struct Term dummy;\n'
      '    struct Term* tail = &dummy;\n'
      '    dummy.next = NULL;\n'
      '    while (pa && pb) {\n'
      '        if (pa->exp > pb->exp) {\n'
      '            tail->next = createTerm(pa->coef, pa->exp);\n'
      '            pa = pa->next;\n'
      '        } else if (pa->exp < pb->exp) {\n'
      '            tail->next = createTerm(pb->coef, pb->exp);\n'
      '            pb = pb->next;\n'
      '        } else {\n'
      '            int sum = pa->coef + pb->coef;\n'
      '            if (sum != 0)\n'
      '                tail->next = createTerm(sum, pa->exp);\n'
      '            pa = pa->next;\n'
      '            pb = pb->next;\n'
      '        }\n'
      '        if (tail->next) tail = tail->next;\n'
      '    }\n'
      '    while (pa) {\n'
      '        tail->next = createTerm(pa->coef, pa->exp);\n'
      '        tail = tail->next; pa = pa->next;\n'
      '    }\n'
      '    while (pb) {\n'
      '        tail->next = createTerm(pb->coef, pb->exp);\n'
      '        tail = tail->next; pb = pb->next;\n'
      '    }\n'
      '    return dummy.next;\n'
      '}\n'
      '\n'
      'void printPoly(struct Term* p) {\n'
      '    while (p) {\n'
      '        printf("%dx^%d ", p->coef, p->exp);\n'
      '        p = p->next;\n'
      '    }\n'
      '    printf("\\n");\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct Term* pa = createTerm(3, 3);\n'
      '    pa->next = createTerm(2, 1);\n'
      '    pa->next->next = createTerm(1, 0);\n'
      '    struct Term* pb = createTerm(4, 3);\n'
      '    pb->next = createTerm(-2, 1);\n'
      '    struct Term* pc = addPoly(pa, pb);\n'
      '    printPoly(pc);\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '多项式存储',
          description: '用链表存储多项式，每个节点包含系数 coef、指数 exp 和 next 指针。按指数降序排列。',
          focusLines: [4, 5, 6, 7],
          explanations: [
            LineExplanation(line: 5, short: '系数', detail: 'coef 存储该项的系数。'),
            LineExplanation(line: 6, short: '指数', detail: 'exp 存储该项的指数，决定项的排序位置。'),
          ],
        ),
        TutorialStep(
          title: '逐项合并',
          description: '类似归并：比较两个多项式当前节点的指数，大的先接入结果；相等则系数相加，和为 0 时跳过。',
          focusLines: [18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35],
          explanations: [
            LineExplanation(line: 20, short: 'pa 指数大', detail: 'pa->exp > pb->exp，pa 的项先接入结果链表。'),
            LineExplanation(line: 26, short: '指数相等', detail: 'pa->exp == pb->exp，系数相加。'),
            LineExplanation(line: 28, short: '消零', detail: 'sum != 0 时才创建节点，系数为 0 的项不保留。'),
            LineExplanation(line: 33, short: '链接剩余', detail: '其中一个多项式遍历完后，把另一个的剩余部分直接接上。'),
          ],
        ),
      ],
    ),
];
