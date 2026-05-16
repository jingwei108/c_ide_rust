# 零侵入可视化设计

> 核心问题：内置可视化函数需要用户调用，对初学者是负担。如何实现"写纯 C 代码，自动出可视化"？

---

## 1. 问题分析：为什么 `vis_*` 函数是负担？

### 1.1 VisualBinaryTree 的做法及其问题

```c
// VisualBinaryTree 的侵入式可视化（学生必须写这些代码）
void bubbleSort(int arr[], int n) {
    for (int i = 0; i < n - 1; i++) {
        for (int j = 0; j < n - i - 1; j++) {
            vis_step(3);                          // ❌ 初学者：这是什么？
            vis_array("arr", arr, n);             // ❌ 为什么要注册数组？
            vis_array_highlight("arr", j, "compare");  // ❌ 颜色怎么选？
            
            if (arr[j] > arr[j + 1]) {
                vis_array_highlight("arr", j, "swap");   // ❌ 又一条
                int temp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
        }
    }
}
```

**初学者的心智负担**：
1. 还没学会 `if/for`，就要学 `vis_step`
2. 代码里混着大量与算法无关的内容
3. 忘记写 `vis_array` → 没有可视化，不知道哪里错了
4. 写错颜色名 → 可视化异常，困惑

### 1.2 理想状态：写纯 C，自动可视化

```c
// 用户写的代码（纯净的 C 语言）
void bubbleSort(int arr[], int n) {
    for (int i = 0; i < n - 1; i++) {
        for (int j = 0; j < n - i - 1; j++) {
            if (arr[j] > arr[j + 1]) {
                int temp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
        }
    }
}
```

**系统自动**：
- ✅ 检测到数组 `arr` → 自动显示数组状态
- ✅ 检测到 `arr[j]` 和 `arr[j+1]` 比较 → 自动高亮比较元素
- ✅ 检测到交换操作 → 自动标记交换
- ✅ 每执行一步 → 自动记录状态变化

---

## 2. 核心方案：编译器自动注入

### 2.1 架构设计

```
用户源码（纯 C）
    ↓
编译器前端（Lexer/Parser/AST）
    ↓
┌─────────────────────────────────────────────┐
│ 可视化注入引擎（Visualization Injector）     │
│                                             │
│  1. AST 模式识别                            │
│     • 这是什么算法？（排序/搜索/链表/树）    │
│     • 有哪些数据结构参与？（数组/链表/树）   │
│                                             │
│  2. 自动注入规则匹配                         │
│     • 排序算法 → 注入数组比较/交换可视化     │
│     • 链表操作 → 注入节点/指针可视化         │
│     • 树遍历 → 注入递归/节点访问可视化       │
│                                             │
│  3. 生成增强 AST                            │
│     • 在关键位置插入可视化节点               │
│     • 对用户完全透明                         │
└─────────────────────────────────────────────┘
    ↓
WASM CodeGen（生成带可视化指令的 WASM）
    ↓
wasm3 执行
    ↓
可视化状态机自动更新 UI
```

### 2.2 可视化注入规则库

```cpp
// ========== 规则定义 ==========

enum class InjectionPoint {
    FunctionEntry,      // 函数入口
    FunctionExit,       // 函数退出
    LoopEntry,          // 循环开始
    LoopIterationEnd,   // 循环迭代结束
    InnerLoopCondition, // 内层循环条件
    Comparison,         // 比较操作
    SwapOperation,      // 交换操作
    MemoryAlloc,        // malloc
    MemoryFree,         // free
    PointerAssign,      // 指针赋值
    RecursionCall,      // 递归调用
    ReturnStmt,         // return 语句
};

struct VisualizationAction {
    std::string type;           // "register_array", "compare", "swap", "highlight"
    std::vector<std::string> params;  // 参数（变量名、索引等）
};

struct AutoVisualizationRule {
    std::string name;                          // 规则名称
    std::string description;                   // 描述
    ASTPattern pattern;                        // AST 匹配模式
    std::vector<std::pair<InjectionPoint, VisualizationAction>> injections;
};

// ========== 冒泡排序规则 ==========
AutoVisualizationRule BubbleSortRule = {
    .name = "bubble_sort",
    .description = "冒泡排序自动可视化",
    .pattern = {
        // 匹配条件：
        .requiredParams = {ParamType::IntArray, ParamType::Int},  // 参数包含 (int[], int)
        .hasNestedLoop = true,                                    // 有嵌套循环
        .hasArrayAccess = true,                                   // 有数组访问
        .hasAdjacentIndexAccess = true,                           // 有相邻索引访问 arr[i] 和 arr[i+1]
        .hasConditionalSwap = true,                               // 有条件交换
    },
    .injections = {
        // 函数入口：注册数组参数
        {InjectionPoint::FunctionEntry, 
         {.type = "register_array", .params = {"arr", "n"}}},
        
        // 内层循环条件处：高亮当前比较的元素
        {InjectionPoint::InnerLoopCondition,
         {.type = "array_compare", .params = {"arr", "j", "j+1"}}},
        
        // 比较操作中：记录比较事件
        {InjectionPoint::Comparison,
         {.type = "compare_event", .params = {"arr[j]", "arr[j+1]"}}},
        
        // 交换操作中：标记交换动画
        {InjectionPoint::SwapOperation,
         {.type = "array_swap", .params = {"arr", "j", "j+1"}}},
        
        // 每次循环迭代后：更新数组显示
        {InjectionPoint::LoopIterationEnd,
         {.type = "array_update", .params = {"arr"}}},
        
        // 函数退出：标记排序完成
        {InjectionPoint::FunctionExit,
         {.type = "sort_complete", .params = {"arr"}}},
    }
};

// ========== 链表头插法规则 ==========
AutoVisualizationRule ListInsertHeadRule = {
    .name = "list_insert_head",
    .description = "链表头插法自动可视化",
    .pattern = {
        .requiredParams = {ParamType::StructPointer},  // struct ListNode*
        .hasMalloc = true,
        .hasPointerAssign = true,  // node->next = head
        .returnsPointer = true,    // return node
    },
    .injections = {
        // malloc 后：显示新节点创建
        {InjectionPoint::MemoryAlloc,
         {.type = "list_node_create", .params = {"node", "val"}}},
        
        // 指针赋值时：显示指针连接动画
        {InjectionPoint::PointerAssign,
         {.type = "list_pointer_connect", .params = {"node->next", "head"}}},
        
        // return 时：显示新的头指针
        {InjectionPoint::ReturnStmt,
         {.type = "list_head_update", .params = {"node"}}},
    }
};

// ========== 二叉树前序遍历规则 ==========
AutoVisualizationRule TreePreorderRule = {
    .name = "tree_preorder",
    .description = "二叉树前序遍历自动可视化",
    .pattern = {
        .requiredParams = {ParamType::StructPointer},  // struct TreeNode*
        .isRecursive = true,
        .hasNullCheck = true,      // if (root == NULL)
        .accessesStructField = {"left", "right", "val"},
    },
    .injections = {
        // 函数入口：高亮当前节点
        {InjectionPoint::FunctionEntry,
         {.type = "tree_node_visit", .params = {"root", "val"}}},
        
        // 递归调用 left 前：显示递归深入动画
        {InjectionPoint::RecursionCall,
         {.type = "tree_recurse_left", .params = {"root->left"}}},
        
        // 递归调用 right 前：显示递归深入动画
        {InjectionPoint::RecursionCall,
         {.type = "tree_recurse_right", .params = {"root->right"}}},
        
        // 函数退出：标记节点访问完成
        {InjectionPoint::FunctionExit,
         {.type = "tree_node_done", .params = {"root"}}},
    }
};
```

### 2.3 AST 模式识别实现

```cpp
class VisualizationInjector {
    std::vector<AutoVisualizationRule> rules;
    
public:
    VisualizationInjector() {
        // 注册所有规则
        rules.push_back(BubbleSortRule);
        rules.push_back(SelectionSortRule);
        rules.push_back(QuickSortRule);
        rules.push_back(ListInsertHeadRule);
        rules.push_back(ListDeleteRule);
        rules.push_back(TreePreorderRule);
        rules.push_back(TreeInorderRule);
        rules.push_back(TreePostorderRule);
        rules.push_back(BinarySearchRule);
        // ... 更多规则
    }
    
    // 主入口：对 AST 进行可视化注入
    ASTNode Inject(ASTNode ast) {
        for (auto& func : ast.Functions) {
            // 尝试匹配规则
            for (const auto& rule : rules) {
                if (MatchPattern(func, rule.pattern)) {
                    func = ApplyInjections(func, rule.injections);
                    func.visualizationTag = rule.name;  // 标记算法类型
                    break;
                }
            }
            
            // 如果没有匹配到特定规则，进行通用注入
            if (!func.visualizationTag.has_value()) {
                func = ApplyGenericInjections(func);
            }
        }
        
        return ast;
    }
    
private:
    bool MatchPattern(const FuncDecl& func, const ASTPattern& pattern) {
        // 1. 检查参数类型
        if (!MatchParams(func.params, pattern.requiredParams)) return false;
        
        // 2. 检查是否有嵌套循环
        if (pattern.hasNestedLoop && !HasNestedLoop(func.body)) return false;
        
        // 3. 检查是否有数组访问
        if (pattern.hasArrayAccess && !HasArrayAccess(func.body)) return false;
        
        // 4. 检查是否有相邻索引访问（arr[i] 和 arr[i+1]）
        if (pattern.hasAdjacentIndexAccess && !HasAdjacentIndexAccess(func.body)) return false;
        
        // 5. 检查是否有条件交换
        if (pattern.hasConditionalSwap && !HasConditionalSwap(func.body)) return false;
        
        // 6. 检查是否是递归
        if (pattern.isRecursive && !IsRecursive(func)) return false;
        
        // 7. 检查是否有 malloc
        if (pattern.hasMalloc && !HasMalloc(func.body)) return false;
        
        return true;
    }
    
    FuncDecl ApplyInjections(FuncDecl func, 
                              const std::vector<std::pair<InjectionPoint, VisualizationAction>>& injections) {
        for (const auto& [point, action] : injections) {
            switch (point) {
                case InjectionPoint::FunctionEntry:
                    func.body = InsertAtEntry(func.body, CreateVisNode(action));
                    break;
                    
                case InjectionPoint::LoopEntry:
                    func.body = InsertAtLoopEntry(func.body, CreateVisNode(action));
                    break;
                    
                case InjectionPoint::Comparison:
                    func.body = WrapComparisons(func.body, CreateVisNode(action));
                    break;
                    
                case InjectionPoint::SwapOperation:
                    func.body = WrapSwapOperations(func.body, CreateVisNode(action));
                    break;
                    
                case InjectionPoint::MemoryAlloc:
                    func.body = WrapMallocCalls(func.body, CreateVisNode(action));
                    break;
                    
                // ... 其他注入点
            }
        }
        
        return func;
    }
    
    // 通用注入：即使不匹配任何规则，也进行基础可视化
    FuncDecl ApplyGenericInjections(FuncDecl func) {
        // 1. 注册所有数组参数
        for (const auto& param : func.params) {
            if (param.type.isArray) {
                func.body = InsertAtEntry(func.body, 
                    CreateVisNode({"register_array", {param.name, "array_size"}}));
            }
        }
        
        // 2. 注册所有 struct 指针参数
        for (const auto& param : func.params) {
            if (param.type.isStructPointer) {
                func.body = InsertAtEntry(func.body,
                    CreateVisNode({"register_struct", {param.name, param.type.structName}}));
            }
        }
        
        // 3. 在每行代码执行后记录状态
        func.body = InsertStepTracking(func.body);
        
        return func;
    }
};
```

### 2.4 注入后的 AST 示例

```c
// 用户源码
void bubbleSort(int arr[], int n) {
    for (int i = 0; i < n - 1; i++) {
        for (int j = 0; j < n - i - 1; j++) {
            if (arr[j] > arr[j + 1]) {
                int temp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
        }
    }
}
```

```
注入后的 AST（概念表示）

FunctionDecl: bubbleSort
├── Params: [arr: int[], n: int]
├── VisualizationTag: "bubble_sort"
├── Body:
│   ├── VisNode: register_array("arr", "n")     ← 自动注入：函数入口
│   │
│   ├── ForStmt: i = 0; i < n - 1; i++
│   │   ├── VisNode: loop_start("outer", i)      ← 自动注入：循环开始
│   │   │
│   │   ├── ForStmt: j = 0; j < n - i - 1; j++
│   │   │   ├── VisNode: array_compare(arr, j, j+1)  ← 自动注入：比较前
│   │   │   │
│   │   │   ├── IfStmt: arr[j] > arr[j+1]
│   │   │   │   ├── VisNode: compare_highlight(arr[j], arr[j+1])  ← 自动注入
│   │   │   │   │
│   │   │   │   ├── Block:
│   │   │   │   │   ├── VisNode: array_swap(arr, j, j+1)  ← 自动注入：交换标记
│   │   │   │   │   │
│   │   │   │   │   ├── VarDecl: temp = arr[j]
│   │   │   │   │   ├── Assign: arr[j] = arr[j+1]
│   │   │   │   │   └── Assign: arr[j+1] = temp
│   │   │   │   │
│   │   │   │   └── VisNode: array_update(arr)  ← 自动注入：交换后更新
│   │   │   │
│   │   │   └── VisNode: loop_iteration_end()  ← 自动注入：迭代结束
│   │   │
│   │   └── VisNode: loop_end("inner")  ← 自动注入：循环结束
│   │
│   └── VisNode: sort_complete(arr)  ← 自动注入：函数退出
│
└── VisNode: unregister_array("arr")  ← 自动注入：清理
```

**对用户完全透明**：用户看到的代码仍然是纯净的 C，注入只在编译器内部进行。

---

## 3. 可视化状态机

### 3.1 运行时自动追踪

```cpp
class VisualizationStateMachine {
public:
    // 当前可视化状态
    struct State {
        std::map<std::string, ArrayState> arrays;
        std::map<std::string, ListState> lists;
        std::map<std::string, TreeState> trees;
        std::map<std::string, int> variables;
        std::vector<VisEvent> events;
    };
    
    State currentState;
    std::vector<State> history;  // 状态历史（用于回放）
    
    // 处理可视化事件（由 wasm3 宿主函数调用）
    void OnVisEvent(const VisEvent& event) {
        switch (event.type) {
            case VisEventType::ArrayCompare:
                HandleArrayCompare(event);
                break;
                
            case VisEventType::ArraySwap:
                HandleArraySwap(event);
                break;
                
            case VisEventType::ListNodeCreate:
                HandleListNodeCreate(event);
                break;
                
            case VisEventType::ListPointerConnect:
                HandleListPointerConnect(event);
                break;
                
            case VisEventType::TreeNodeVisit:
                HandleTreeNodeVisit(event);
                break;
                
            case VisEventType::VariableUpdate:
                HandleVariableUpdate(event);
                break;
        }
        
        // 保存状态快照
        history.push_back(currentState);
    }
    
private:
    void HandleArrayCompare(const VisEvent& event) {
        auto& arr = currentState.arrays[event.arrayName];
        arr.highlights.clear();
        arr.highlights[event.index1] = "compare";
        arr.highlights[event.index2] = "compare";
        
        currentState.events.push_back({
            .type = "compare",
            .description = format("比较 %s[%d]=%d 和 %s[%d]=%d",
                event.arrayName, event.index1, arr.values[event.index1],
                event.arrayName, event.index2, arr.values[event.index2])
        });
    }
    
    void HandleArraySwap(const VisEvent& event) {
        auto& arr = currentState.arrays[event.arrayName];
        
        // 执行交换
        std::swap(arr.values[event.index1], arr.values[event.index2]);
        
        arr.highlights.clear();
        arr.highlights[event.index1] = "swap";
        arr.highlights[event.index2] = "swap";
        
        currentState.events.push_back({
            .type = "swap",
            .description = format("交换 %s[%d] 和 %s[%d]",
                event.arrayName, event.index1,
                event.arrayName, event.index2)
        });
    }
};
```

### 3.2 前端同步

```csharp
// C# 前端：接收状态更新并渲染
public class VisualizationViewModel : ViewModelBase {
    private readonly VisualizationStateMachine _stateMachine;
    
    [ObservableProperty]
    private ObservableCollection<ArrayViewModel> arrays = new();
    
    [ObservableProperty]
    private ObservableCollection<ListViewModel> lists = new();
    
    [ObservableProperty]
    private ObservableCollection<TreeViewModel> trees = new();
    
    [ObservableProperty]
    private ObservableCollection<VisEvent> events = new();
    
    // 从后端接收状态更新
    public void OnStateUpdate(VisStateUpdate update) {
        // 更新数组视图
        foreach (var arrUpdate in update.ArrayUpdates) {
            var arr = arrays.FirstOrDefault(a => a.Name == arrUpdate.Name);
            if (arr == null) {
                arr = new ArrayViewModel { Name = arrUpdate.Name };
                arrays.Add(arr);
            }
            arr.Values = arrUpdate.Values;
            arr.Highlights = arrUpdate.Highlights;
        }
        
        // 更新链表视图
        foreach (var listUpdate in update.ListUpdates) {
            // ...
        }
        
        // 添加事件
        if (update.NewEvent != null) {
            events.Add(update.NewEvent);
        }
    }
}
```

---

## 4. 用户可控的精细调节

### 4.1 IDE 侧边栏控制面板

即使自动模式下，用户也可以通过简单的开关控制可视化内容：

```
┌─────────────────────────────┐
│ 🎨 可视化设置                │
├─────────────────────────────┤
│                             │
│ ☑️ 显示数组状态              │
│ ☑️ 高亮比较操作              │
│ ☑️ 高亮交换操作              │
│ ☑️ 显示变量值                │
│ ☐  显示内存布局   [进阶]     │
│ ☐  显示指针关系   [进阶]     │
│                             │
│ 动画速度：                   │
│ [慢────●────快]             │
│                             │
│ [🔄 重置为默认]              │
│                             │
│ ─────────────────────────── │
│                             │
│ 📝 当前识别为：冒泡排序       │
│                             │
│ [👁️ 预览可视化效果]          │
└─────────────────────────────┘
```

### 4.2 进阶模式：特殊注释

对于进阶用户，允许通过**特殊注释**进行精细控制（不影响代码语义）：

```c
void bubbleSort(int arr[], int n) {
    for (int i = 0; i < n - 1; i++) {
        // @vis: hide loop           ← 隐藏外层循环可视化
        for (int j = 0; j < n - i - 1; j++) {
            if (arr[j] > arr[j + 1]) {
                // @vis: highlight arr[j] red
                // @vis: highlight arr[j+1] red
                // @vis: delay 500ms        ← 延迟 500ms
                int temp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
        }
    }
}
```

**特殊注释特点**：
- 以 `// @vis:` 开头
- 不影响代码编译和执行
- 仅被可视化注入引擎识别
- 初学者完全可以忽略

### 4.3 完全手动模式

```csharp
// IDE 设置
public enum VisualizationMode {
    Auto,        // 自动模式（默认，初学者）
    Guided,      // 引导模式（自动 + 允许注释控制）
    Manual       // 手动模式（完全由 vis_* 函数控制）
}
```

手动模式用于特殊场景：
- 自定义可视化需求
- 教学演示（教师需要精细控制）
- 调试可视化引擎本身

---

## 5. 数据结构自动可视化示例

### 5.1 链表操作（零侵入）

```c
// 用户写的纯 C 代码
struct ListNode* insertHead(struct ListNode* head, int val) {
    struct ListNode* node = malloc(sizeof(struct ListNode));
    node->val = val;
    node->next = head;
    return node;
}
```

**系统自动识别**：
1. 函数参数包含 `struct ListNode*` → 链表操作
2. 有 `malloc` + `struct ListNode` → 创建新节点
3. 有 `node->next = head` → 头插法
4. 返回 `node` → 更新头指针

**自动注入的可视化**：
```
执行步骤：
  Step 1: malloc 创建新节点
          [可视化] 显示新节点 node(val=10)
          
  Step 2: node->next = head
          [可视化] 动画：node.next 指针指向 head
          
  Step 3: return node
          [可视化] 动画：head 指针更新为 node
          [可视化] 显示完整链表：10 → 5 → 3 → NULL
```

### 5.2 树遍历（零侵入）

```c
// 用户写的纯 C 代码
void preorder(struct TreeNode* root) {
    if (root == NULL) return;
    printf("%d ", root->val);
    preorder(root->left);
    preorder(root->right);
}
```

**系统自动识别**：
1. 参数 `struct TreeNode*` → 二叉树操作
2. 有 `if (root == NULL)` → 递归终止条件
3. 递归调用 `root->left` 和 `root->right` → 前序遍历模式

**自动注入的可视化**：
```
执行步骤：
  Step 1: 访问根节点 10
          [可视化] 高亮节点 10，标记为 "visiting"
          
  Step 2: 递归左子树
          [可视化] 动画：箭头指向左子节点 5
          
  Step 3: 访问节点 5
          [可视化] 高亮节点 5
          
  Step 4: 递归左子树 (NULL)
          [可视化] 显示 "NULL"，箭头变灰
          
  Step 5: 递归右子树 (NULL)
          [可视化] 显示 "NULL"
          
  Step 6: 返回节点 5，标记为 "done"
          [可视化] 节点 5 变为绿色
          
  Step 7: 递归右子树
          [可视化] 动画：箭头指向右子节点 15
          
  ...
```

---

## 6. 实现优先级

### Phase 1：自动模式 MVP（核心）
- [x] 可视化注入引擎框架
- [x] 8 种核心算法规则（冒泡排序、选择排序、插入排序、二分查找、链表遍历、链表反转、快速排序、归并排序）
- [x] 通用注入（数组注册、变量追踪）
- [x] 前端自动渲染（数组状态、变量值）
- [x] IDE 侧边栏控制面板

### Phase 2：扩展自动规则
- [ ] 更多算法规则（链表插入/删除、树遍历）
- [ ] 特殊注释支持（`// @vis:`）
- [ ] 手动模式（完整的 `vis_*` API）

### Phase 3：智能增强
- [ ] 算法模式识别置信度评分
- [ ] 用户反馈学习（"这不是冒泡排序"→ 调整规则）
- [ ] 自定义规则（教师可以添加新的可视化规则）

---

## 7. 总结

### 核心设计哲学

> **初学者不应该为了看到可视化而学习任何额外的语法。写纯 C 代码，系统自动理解并展示。**

### 三种模式

| 模式 | 用户代码 | 适用人群 | 控制粒度 |
|:---|:---|:---|:---|
| **自动**（默认） | 纯 C，无任何额外代码 | 初学者 | IDE 侧边栏开关 |
| **引导** | 纯 C + `// @vis:` 注释 | 进阶学习者 | 注释精细控制 |
| **手动** | C + `vis_*()` 函数调用 | 教师/高级用户 | 完全控制 |

### 关键技术

1. **AST 模式识别**：编译器自动识别算法类型和数据结构操作
2. **规则驱动注入**：根据识别结果，在关键位置自动插入可视化指令
3. **状态机同步**：运行时通过 wasm3 宿主函数更新可视化状态
4. **前端自动渲染**：根据状态自动更新数组/链表/树的可视化视图

### 用户体验

```
初学者：
  1. 写纯 C 代码
  2. 点击运行
  3. 自动看到数组排序的动画过程
  4. 零学习成本

进阶用户：
  1. 写纯 C 代码
  2. 在关键点添加 // @vis: highlight 注释
  3. 精细控制可视化效果

教师：
  1. 切换到手动模式
  2. 使用完整的 vis_* API
  3. 制作精细的教学演示
```
