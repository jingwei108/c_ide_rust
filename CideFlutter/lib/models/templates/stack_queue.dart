import '../code_template.dart';

const List<CodeTemplate> stackQueueTemplates = [
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
      'parenthesesMatch',
      '括号匹配',
      '结构',
      '#include <stdio.h>\n'
      '#define MAX 100\n'
      '\n'
      'int match(char expr[]) {\n'
      '    char stack[MAX];\n'
      '    int top = -1;\n'
      '    for (int i = 0; expr[i] != \'\\0\'; i++) {\n'
      '        if (expr[i] == \'(\' || expr[i] == \'[\' || expr[i] == \'{\') {\n'
      '            stack[++top] = expr[i];\n'
      '        } else if (expr[i] == \')\' || expr[i] == \']\' || expr[i] == \'}\') {\n'
      '            if (top == -1) return 0;\n'
      '            char left = stack[top--];\n'
      '            if ((expr[i] == \')\' && left != \'(\') ||\n'
      '                (expr[i] == \']\' && left != \'[\') ||\n'
      '                (expr[i] == \'}\' && left != \'{\'))\n'
      '                return 0;\n'
      '        }\n'
      '    }\n'
      '    return top == -1;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    char expr[] = "{[()]}";\n'
      '    if (match(expr))\n'
      '        printf("matched\\n");\n'
      '    else\n'
      '        printf("not matched\\n");\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '栈的应用',
          description: '括号匹配是栈的经典应用。遇到左括号就入栈，遇到右括号就出栈检查是否配对。',
          focusLines: [6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18],
          explanations: [
            LineExplanation(line: 7, short: '左括号入栈', detail: '遇到 ( [ { 就压入栈中，等待后续右括号来配对。'),
            LineExplanation(line: 9, short: '右括号出栈', detail: '遇到 ) ] } 时尝试弹出栈顶左括号。'),
            LineExplanation(line: 10, short: '栈空报错', detail: 'top == -1 说明没有左括号可配，匹配失败。'),
            LineExplanation(line: 12, short: '配对检查', detail: '弹出的左括号必须与当前右括号是同类型。'),
          ],
        ),
        TutorialStep(
          title: '最终检查',
          description: '扫描完所有字符后，如果栈中还有剩余的左括号，说明缺少右括号，匹配失败。',
          focusLines: [19, 20],
          explanations: [
            LineExplanation(line: 20, short: '栈空才成功', detail: 'top == -1 表示所有左括号都找到了对应的右括号。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'infixEvaluation',
      '中缀表达式求值',
      '结构',
      '#include <stdio.h>\n'
      '#include <ctype.h>\n'
      '#define MAX 100\n'
      '\n'
      'int opStack[MAX];\n'
      'int opTop = -1;\n'
      'int valStack[MAX];\n'
      'int valTop = -1;\n'
      '\n'
      'int precedence(char op) {\n'
      '    if (op == \'+\' || op == \'-\') return 1;\n'
      '    if (op == \'*\' || op == \'/\') return 2;\n'
      '    return 0;\n'
      '}\n'
      '\n'
      'void pushOp(char c) { opStack[++opTop] = c; }\n'
      'char popOp() { return opStack[opTop--]; }\n'
      'void pushVal(int v) { valStack[++valTop] = v; }\n'
      'int popVal() { return valStack[valTop--]; }\n'
      '\n'
      'void applyOp() {\n'
      '    char op = popOp();\n'
      '    int b = popVal();\n'
      '    int a = popVal();\n'
      '    switch (op) {\n'
      '        case \'+\': pushVal(a + b); break;\n'
      '        case \'-\': pushVal(a - b); break;\n'
      '        case \'*\': pushVal(a * b); break;\n'
      '        case \'/\': pushVal(a / b); break;\n'
      '    }\n'
      '}\n'
      '\n'
      'int evaluate(char expr[]) {\n'
      '    int i = 0;\n'
      '    while (expr[i] != \'\\0\') {\n'
      '        if (expr[i] == \' \') {\n'
      '            i++;\n'
      '            continue;\n'
      '        }\n'
      '        if (isdigit(expr[i])) {\n'
      '            int val = 0;\n'
      '            while (isdigit(expr[i])) {\n'
      '                val = val * 10 + (expr[i] - \'0\');\n'
      '                i++;\n'
      '            }\n'
      '            pushVal(val);\n'
      '            continue;\n'
      '        }\n'
      '        if (expr[i] == \'(\') {\n'
      '            pushOp(expr[i]);\n'
      '        } else if (expr[i] == \')\') {\n'
      '            while (opTop != -1 && opStack[opTop] != \'(\')\n'
      '                applyOp();\n'
      '            popOp();\n'
      '        } else {\n'
      '            while (opTop != -1 && precedence(opStack[opTop]) >= precedence(expr[i]))\n'
      '                applyOp();\n'
      '            pushOp(expr[i]);\n'
      '        }\n'
      '        i++;\n'
      '    }\n'
      '    while (opTop != -1)\n'
      '        applyOp();\n'
      '    return popVal();\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    char expr[] = "3 + 5 * 2 - 8 / 4";\n'
      '    printf("%d\\n", evaluate(expr));\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '双栈结构',
          description: '中缀表达式求值使用运算符栈和操作数栈。遇到数字入值栈，遇到运算符按优先级处理。',
          focusLines: [5, 6, 7, 8, 9, 10, 11, 12],
          explanations: [
            LineExplanation(line: 6, short: '运算符栈', detail: 'opStack 存放运算符，opTop 指向栈顶。'),
            LineExplanation(line: 8, short: '操作数栈', detail: 'valStack 存放整数操作数，valTop 指向栈顶。'),
          ],
        ),
        TutorialStep(
          title: '优先级与运算',
          description: 'precedence 定义运算符优先级。applyOp 弹出运算符和两个操作数，计算结果后压回值栈。',
          focusLines: [13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32],
          explanations: [
            LineExplanation(line: 15, short: '优先级', detail: '* / 优先级为 2，+ - 优先级为 1。'),
            LineExplanation(line: 24, short: '弹出操作数', detail: '先弹出的 b 是右操作数，后弹出的 a 是左操作数。'),
            LineExplanation(line: 26, short: '计算', detail: 'switch 根据运算符选择对应的算术运算。'),
          ],
        ),
        TutorialStep(
          title: '扫描处理',
          description: '扫描表达式时，数字可能多位，需要连续读取。左括号入栈，右括号则计算到左括号为止。',
          focusLines: [33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60],
          explanations: [
            LineExplanation(line: 39, short: '多位数', detail: '连续读取数字字符并累加：val = val * 10 + digit。'),
            LineExplanation(line: 48, short: '右括号', detail: '计算到左括号为止，然后弹出左括号。'),
            LineExplanation(line: 52, short: '优先级比较', detail: '栈顶运算符优先级 >= 当前运算符时，先计算栈顶。'),
          ],
        ),
      ],
    ),

    CodeTemplate(
      'deque',
      '双端队列',
      '结构',
      '#include <stdio.h>\n'
      '#define MAXSIZE 10\n'
      '\n'
      'struct Deque {\n'
      '    int data[MAXSIZE];\n'
      '    int front;\n'
      '    int rear;\n'
      '    int size;\n'
      '};\n'
      '\n'
      'void init(struct Deque* dq) {\n'
      '    dq->front = 0;\n'
      '    dq->rear = 0;\n'
      '    dq->size = 0;\n'
      '}\n'
      '\n'
      'int isEmpty(struct Deque* dq) { return dq->size == 0; }\n'
      'int isFull(struct Deque* dq) { return dq->size == MAXSIZE; }\n'
      '\n'
      'void pushFront(struct Deque* dq, int x) {\n'
      '    if (isFull(dq)) return;\n'
      '    dq->front = (dq->front - 1 + MAXSIZE) % MAXSIZE;\n'
      '    dq->data[dq->front] = x;\n'
      '    dq->size++;\n'
      '}\n'
      '\n'
      'void pushRear(struct Deque* dq, int x) {\n'
      '    if (isFull(dq)) return;\n'
      '    dq->data[dq->rear] = x;\n'
      '    dq->rear = (dq->rear + 1) % MAXSIZE;\n'
      '    dq->size++;\n'
      '}\n'
      '\n'
      'int popFront(struct Deque* dq) {\n'
      '    if (isEmpty(dq)) return -1;\n'
      '    int x = dq->data[dq->front];\n'
      '    dq->front = (dq->front + 1) % MAXSIZE;\n'
      '    dq->size--;\n'
      '    return x;\n'
      '}\n'
      '\n'
      'int popRear(struct Deque* dq) {\n'
      '    if (isEmpty(dq)) return -1;\n'
      '    dq->rear = (dq->rear - 1 + MAXSIZE) % MAXSIZE;\n'
      '    int x = dq->data[dq->rear];\n'
      '    dq->size--;\n'
      '    return x;\n'
      '}\n'
      '\n'
      'int main() {\n'
      '    struct Deque dq;\n'
      '    init(&dq);\n'
      '    pushRear(&dq, 10);\n'
      '    pushRear(&dq, 20);\n'
      '    pushFront(&dq, 5);\n'
      '    printf("%d ", popFront(&dq));\n'
      '    printf("%d ", popRear(&dq));\n'
      '    printf("%d\\n", popFront(&dq));\n'
      '    return 0;\n'
      '}',
      params: [],
      tutorialSteps: [
        TutorialStep(
          title: '双端队列结构',
          description: '双端队列支持在队头和队尾都进行插入和删除。用循环数组实现，size 字段记录当前元素个数，避免了牺牲一个单元区分空满。',
          focusLines: [4, 5, 6, 7, 8, 9],
          explanations: [
            LineExplanation(line: 5, short: '数据区', detail: 'data[MAXSIZE] 是底层循环数组。'),
            LineExplanation(line: 6, short: '队头', detail: 'front 指向队头元素。'),
            LineExplanation(line: 7, short: '队尾', detail: 'rear 指向队尾下一个空位。'),
            LineExplanation(line: 8, short: '计数器', detail: 'size 记录当前元素个数，直接用 size==0 判空、size==MAXSIZE 判满。'),
          ],
        ),
        TutorialStep(
          title: '头插与尾插',
          description: 'pushFront 先把 front 回退一步再写入；pushRear 先写入再把 rear 前进一步。两者都通过取模实现循环。',
          focusLines: [16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26],
          explanations: [
            LineExplanation(line: 18, short: 'front 回退', detail: '(front - 1 + MAXSIZE) % MAXSIZE 让 front 绕回数组末尾。'),
            LineExplanation(line: 24, short: 'rear 前进', detail: '(rear + 1) % MAXSIZE 让 rear 绕回数组开头。'),
          ],
        ),
        TutorialStep(
          title: '头删与尾删',
          description: 'popFront 取出 front 位置元素后让 front 前进一步；popRear 先把 rear 回退一步再取出元素。',
          focusLines: [28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39],
          explanations: [
            LineExplanation(line: 31, short: '头删', detail: 'front 前移，被删元素不再在有效范围内。'),
            LineExplanation(line: 37, short: '尾删', detail: 'rear 先回退，再取该位置元素。'),
          ],
        ),
      ],
    ),
];
