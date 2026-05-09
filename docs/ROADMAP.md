# C IDE 项目路线图（2026-04-27 修订版，2026-04-27 更新）

> **核心原则**：不急着发布，不把时间浪费在"能用"上。每一行代码都指向一个竞品没有的功能亮点。  
> **当前决策**：替换 wasm3 为自研轻量 VM，解锁运行时诊断、精确单步、内存可视化三大壁垒。

---

## 一、安全隔离方案（回答关键问题）

**Q：去掉 wasm3 后，安全隔离如何保证？**

自研 VM 采用与 WASM **完全一致的安全模型**：线性内存 + 指令级边界检查。差异仅在于——检查代码由我们自己写，因此可以在越界/除零的现场读取变量值，生成精确诊断。

```cpp
class CideVM {
    uint8_t* memory;      // 线性内存（如 256KB）
    uint32_t memSize;

    int32_t LoadI32(uint32_t addr) {
        if (addr + 4 > memSize) {
            // wasm3 只能报 "out of bounds"
            // 自研 VM 可以报："arr[10] 越界，数组大小为 5，当前 i=5"
            Trap(FormatBoundsError(addr, symbols_));
        }
        return ReadLittleEndian32(memory + addr);
    }

    int32_t DivI32(int32_t a, int32_t b) {
        if (b == 0) {
            // wasm3 只能报 "divide by zero"
            // 自研 VM 可以报："除零错误：除数 b=0，b 在第 3 行赋值后未被修改"
            Trap(FormatDivZeroError(a, b, currentLine_, symbols_));
        }
        return a / b;
    }
};
```

**完整安全机制**：

| 机制 | wasm3 实现 | 自研 VM 实现 | 额外收益 |
|:---|:---|:---|:---|
| 内存隔离 | 自动 | `LoadI32/StoreI32` 统一检查 `addr+size <= memSize` | 无 |
| 除零捕获 | 自动 | `Div/Mod` 指令中显式检查 | **可读取变量值生成精确诊断** |
| 栈溢出 | 自动 | `callStack.size() < maxDepth` | 无 |
| 步数熔断 | `m3_Yield` patch | 每条指令后 `stepCount++` | **更精确（按指令而非按 Call）** |
| NULL 陷阱区 | 无 | `0x0000~0x0FFF` 禁止访问 | **比 wasm3 更严格** |

**结论**：安全等级不低于 wasm3，且诊断能力大幅提升。

---

## 二、为什么要替换 wasm3？

| 能力 | 用 wasm3 的上限 | 自研 VM 后的上限 | 对"亮眼功能"的影响 |
|:---|:---|:---|:---|
| 运行时中文诊断 | 60 分（翻译英文 trap） | 95 分（精确到变量值的现场报告） | ⭐⭐⭐⭐⭐ |
| 单步调试 | 40 分（线程阻塞 hack） | 95 分（每条指令精确暂停） | ⭐⭐⭐⭐⭐ |
| 内存可视化 | 60 分（读原始字节，不知道变量名） | 90 分（符号表映射地址→变量名） | ⭐⭐⭐⭐ |
| 零侵入可视化 | 70 分（注入 host call 有开销） | 90 分（VM 层直接发射事件） | ⭐⭐⭐⭐ |
| 指针追踪 | 30 分（只能读内存值） | 85 分（知道 `0x1020` 是 `p->next`） | ⭐⭐⭐⭐ |

**核心洞察**：wasm3 是"通用 WASM 解释器"，而我们需要的是"教学专用 C 子集执行引擎"。通用层成了枷锁。

---

## 三、竞品分析 & 我们的差异化壁垒

### 现有竞品

| 产品 | 类型 | 核心能力 | 致命短板 |
|:---|:---|:---|:---|
| **C语言编译器IDE** (Android) | 移动端IDE | 能编译运行C | 英文错误、无调试、无可视化 |
| **Cxxdroid** | 移动端IDE | GCC编译、终端输出 | 无教学引导、无内存视图 |
| **OnlineGDB** | Web IDE | GDB调试 | 网页端、不适合手机、学习曲线陡 |
| **Scratch/Blockly** | 图形编程 | 可视化动画 | **不是真实代码**，无法过渡到工业编程 |
| **Educoder/头歌** | OJ平台 | 在线评测 | 无实时调试、无可视化、无诊断 |

### 我们的四大壁垒（替换 VM 后才能完全实现）

#### 壁垒 1：运行时中文诊断（唯一）

学生写：
```c
for (int i = 0; i <= 5; i++) { arr[i] = i; }
```

其他工具只能说：
- GDB: `Program received signal SIGSEGV`
- OnlineGDB: `Runtime Error`
- 当前 wasm3: `🚫 内存访问越界`

**我们的目标**：
```
🚫 数组越界：你访问了 arr[5]，但数组只有 5 个元素，有效索引是 0~4。

📍 发生在第 3 行：arr[i] = i;
💡 原因：循环条件写成了 i <= 5，应该改成 i < 5。
🔍 当前 i = 5，arr 声明于第 1 行，大小为 5。

✅ 一键修复：将 <= 改为 <
```

#### 壁垒 2：零侵入算法可视化（唯一）

学生写纯 C 冒泡排序，系统自动识别并播放排序动画。不需要写任何 `vis_array()` 额外代码。

```c
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

VM 在 `arr[j] > arr[j+1]` 时发射 `VisEvent::ArrayCompare`，在交换时发射 `VisEvent::ArraySwap`。前端 Canvas 实时播放。

#### 壁垒 3：内存动画（唯一）

- 写 `int* p = &a;`，屏幕实时画出指针箭头
- 写 `p = malloc(4);`，屏幕显示堆区分配动画
- 写 `free(p);`，指针箭头变灰，标记为已释放

#### 壁垒 4：单步变量追踪（差异化）

每走一步，侧边栏显示所有变量的当前值。指针变量的值显示为箭头指向目标地址，数组显示为带索引的格子。

---

## 四、新的开发阶段（功能驱动，不绑时间）

### Stage 0: 基础编译器（✅ 已完成，保留）
- Lexer → Parser → AST → TypeChecker
- 支持变量、数组、指针、struct、if/for/while/do-while、函数、malloc/free
- 支持 break/continue/switch/typedef/enum/unsigned

### Stage 1: 自研 VM（✅ 已完成）
**目标**：完全替换 wasm3，建立可控执行引擎。

- [x] **Bytecode 定义**：`OpCode.hpp` + `Instruction.hpp`
- [x] **BytecodeGen**：将 AST 编译为自定义字节码（替代 WasmCodeGen）
- [x] **CideVM 核心**：~30 条指令的解释器，线性内存管理
- [x] **C API 迁移**：`cide_run` / `cide_step_next` 驱动 VM
- [x] **回归测试**：7 个测试套件全部通过（0 回归）
- [x] **安全加固**：边界检查、除零捕获、步数熔断、NULL 区陷阱
- [x] **语法扩展**：char、do-while、break/continue、switch/case、typedef、enum、unsigned、数组初始化器

> 构建工具链已迁移至 Clang。旧 `WasmCodeGen` 死代码已清理。

### Stage 2: 运行时中文诊断（✅ 已完成）
**目标**：让错误信息精确到变量值。

**已有基础**：
- ✅ VMSymbol 结构体（`name`, `addr`, `isLocal`, `type`, `scopeDepth`）
- ✅ CideVM `SetSymbols()` / `GetSymbols()` 接口
- ✅ BytecodeGen 内部符号表（`globalIndices_`, `globalTypes_`, `localIndices_`, `localTypes_`）
- ✅ 基础中文诊断（越界/除零/NULL 指针/栈溢出/步数熔断）

**已完成**：
- ✅ **符号表导出**：BytecodeGen 生成 `vector<VMSymbol>` 并通过 C API 注入 VM
- ✅ **数组越界精确诊断**：BytecodeGen 注入 `TrapBounds` 运行时检查 → "你访问了 arr[5]，但数组只有 5 个元素，有效索引是 0~4"
- ✅ **除零精确诊断**：通过符号表定位除数变量 → "除数 b=0，b 在第 3 行赋值后未被修改"
- ✅ **空指针精确诊断**：NULL 陷阱区检查 → "指针 p 的值为 0x00000000，声明于第 3 行"
- ✅ **死循环变量分析**：步数超限时检测循环变量是否变化 → "循环已执行 10,000,000 步，i 始终是 1。你注释掉了 i++ 吗？"
- ✅ **TypeChecker 数组索引类型修复**：`VisitIndex` 成功路径正确设置元素类型，解决数组赋值"类型不匹配"问题
- ✅ **字符串内存区安全修复**：`stringMemOffset_` 正确接在全局变量区（含数组）之后，避免覆盖堆区

### Stage 3: 单步调试 + 内存可视化（✅ 已完成）
**目标**：完全可控的执行观察。

- [x] 指令级单步：`vm.Step()` 精确到每条字节码指令
- [x] 调用栈视图：`callStack_` 已记录 `returnIP` / `localsBase` / `localCount`
- [x] **变量面板**：C API `cide_variable_count/get` + `CompilerService` 封装 + 前端右侧调试面板实时显示
- [x] **内存视图**：内存区域列表 + 堆区分配状态监控
- [x] **指针追踪**：`cide_variable_find_by_addr` 反向地址查变量 + `PointerVariables` 集合显示指针指向关系

**UI 布局更新**：主界面改为左右分割（代码编辑器 + 右侧调试面板），调试面板包含变量/内存/指针三个 Tab。

### Stage 4: 零侵入可视化（✅ 已完成）
**目标**：写纯 C 代码，自动播放算法动画。

- [x] **AST 模式识别骨架**：`AlgorithmMatcher` 类 + 编译时检测接口
- [x] **冒泡排序检测**：基于 AST 遍历的外层循环 + 内层循环 + `arr[j] > arr[j+1]` + swap 模式识别
- [x] **数组实时可视化**：右侧调试面板新增"数组"Tab，`CompilerService.ReadArray()` 读取完整数组内容
- [x] **前端动画 Canvas**：柱状图渲染（高度归一化、逆序对红色高亮、正常元素青色），单步执行时自动刷新
- [x] **交换闪烁动画**：Swap 时柱子白色闪烁（`#FFFFFF`）+ 金色边框提示，前端通过对比前后数组状态自动检测交换，500ms 后自动清除
- [x] **选择排序检测**：外层 for + 内层 for + `arr[j] < arr[min]` + min 更新 + swap 模式
- [x] **插入排序检测**：外层 for + 内层 while + `arr[j] > key` + 元素后移 `arr[j+1] = arr[j]`
- [x] **VM 运行时精确事件**：`CideVM::SetVisEventLines()` + `TakeVisEvents()`，StepEvent 执行时按行号精确发射 Compare/Swap/Update 事件，替代前端推断
- [x] **二分查找检测**：while (left <= right) + mid 计算 + arr[mid] 比较 + left/right 更新模式识别
- [x] **快速排序检测**：递归调用 + 分区循环 + 数组比较（支持 `&&` 嵌套条件）+ swap 模式 + 索引移动
- [x] **归并排序检测**：两次递归调用 + 临时存储 + 合并赋值 + 回写数组模式
- [x] **链表遍历检测**：`struct Node*` 变量 + `while (p != NULL)` + `p = p->next`
- [x] **链表反转检测**：`while (curr)` + `curr->next = prev` + 指针推进
- [x] **链表插入/删除检测**
- [x] **算法运行时验证（Property-based Testing）**：`AlgorithmValidator` 自动生成测试用例，通过 VM 调用学生代码验证排序属性（长度守恒/非递减/排列守恒）和二分查找返回值

### Stage 5: 诊断与修复系统
**目标**：三级信息架构 + QuickFix。

- [x] **L1 感知**：代码行号区错误/警告高亮（红色背景=错误，黄色背景=警告），表情 + 一句话 + 修复按钮
- [x] **L2 理解**：底部诊断面板显示代码片段 + 通俗解释 + 正确/错误对比
- [x] **L3 原理**：右侧调试面板新增"知识"Tab，根据错误码自动加载知识卡片（内存动画描述 + 概念详解 + 练习题）
- [x] **QuickFix**：补分号、改 `=` 为 `==`、改 `<=` 为 `<`（编译时自动检测 + 一键应用）
- [x] **新增编译时检测**：`if (a = b)` 赋值误用为比较警告、`for (i <= n)` off-by-one 警告

### Stage 5: 诊断与修复系统（✅ 已完成）
**目标**：三级信息架构 + QuickFix。

- [x] **L1 感知**：代码行号区错误/警告高亮（红色背景=错误，黄色背景=警告），表情 + 一句话 + 修复按钮
- [x] **L2 理解**：底部诊断面板显示代码片段 + 通俗解释 + 正确/错误对比
- [x] **L3 原理**：右侧调试面板新增"知识"Tab，根据错误码自动加载知识卡片（内存动画描述 + 概念详解 + 练习题）
- [x] **QuickFix**：补分号、改 `=` 为 `==`、改 `<=` 为 `<`（编译时自动检测 + 一键应用）
- [x] **新增编译时检测**：`if (a = b)` 赋值误用为比较警告、`for (i <= n)` off-by-one 警告
- [x] **结构化 Auto-Fix**：`PopulateStructuredFix` 生成精确的 `InsertText`/`ReplaceText` 修复（E2005-E2008、E1004、W3050/W3051），前端 `CodeFixService` 消费

### Stage 6: 移动端适配（🔄 当前重点）
**目标**：在 Android 上流畅运行。

- [x] 响应式布局：手机/平板/桌面三态（骨架已实现，断点系统 + 多端 XAML 绑定）
- [x] 触控手势：Tab 滑动切换、FAB 拖拽吸附
- [x] 虚拟键盘适配：`WindowCompat.SetDecorFitsSystemWindows` + CodeMirror 6 resize 修复
- [x] Release APK 80.89MB（AOT+Trim）
- [ ] 性能优化：降帧率、简化渲染、CancelAll+Snap

---

## 五、技术架构（新）

```
+-----------------------------------------------------------------------------+
|                     C# Avalonia 前端 (Android / Desktop)                     |
|  +-------------+  +-------------+  +-------------------------------------+  |
|  | CodeEditor  |  | MemoryCanvas|  | KnowledgeCard / QuickFixPanel       |  |
|  |  代码编辑器  |  |  内存动画    |  | 知识卡片 / 一键修复面板               |  |
|  +-------------+  +-------------+  +-------------------------------------+  |
|  +-------------+  +-------------+  +-------------------------------------+  |
|  | PointerView |  | ErrorPanel  |  | ConsoleOutput / AlgoCanvas          |  |
|  |  指针视图    |  |  诊断面板    |  | 输出控制台 / 算法动画画布             |  |
|  +-------------+  +-------------+  +-------------------------------------+  |
+-----------------------------------------------------------------------------+
                                    |
                                    v P/Invoke
+-----------------------------------------------------------------------------+
|                        C++ 后端 (Native DLL / .so)                          |
|                                                                             |
|  +---------------------------------------------------------------------+    |
|  | ① C 子集编译器                                                       |    |
|  |   用户 C 代码 → Lexer → Parser → AST → TypeChecker → BytecodeGen    |    |
|  |   输出：Bytecode[] + SymbolTable                                      |    |
|  +---------------------------------------------------------------------+    |
|                                    |                                        |
|  +---------------------------------------------------------------------+    |
|  | ② CideVM 自研执行引擎（替代 wasm3）                                   |    |
|  |   加载 Bytecode → 解释执行 → 精确 trap → 符号表诊断 → VisEvent 发射   |    |
|  |   提供：单步执行 / 内存视图 / 指针追踪 / 执行轨迹                       |    |
|  +---------------------------------------------------------------------+    |
|                                    |                                        |
|  +---------------------------------------------------------------------+    |
|  | ③ 诊断与可视化引擎                                                   |    |
|  |   SourceMap / 内存布局元数据 / 指针追踪表 / 中文错误消息                  |    |
|  |   算法模式识别 / 运行时验证 / 执行轨迹分析                               |    |
|  +---------------------------------------------------------------------+    |
+-----------------------------------------------------------------------------+
```

### 目录结构（调整）

```
native/
├── CMakeLists.txt
├── include/
│   └── cide_capi.h              # 保持不变，前端无感知
├── src/
│   ├── compiler/
│   │   ├── Lexer.cpp/hpp        # 不变
│   │   ├── Parser.cpp/hpp       # 不变
│   │   ├── Ast.hpp              # 不变
│   │   ├── TypeChecker.cpp/hpp  # 不变
│   │   └── BytecodeGen.cpp/hpp  # AST → CideVM 字节码
│   ├── vm/
│   │   ├── OpCode.hpp           # 字节码操作码（新）
│   │   ├── Instruction.hpp      # 指令结构（新）
│   │   ├── CideVM.hpp           # VM 头文件（新）
│   │   └── CideVM.cpp           # VM 实现（新）
│   └── capi/
│       └── cide_capi.cpp        # 适配 VM（修改）
└── tests/
    └── ...                      # 回归测试保留
```

---

## 六、当前状态 & 下一步

### 已完成（保留资产）
- ✅ Lexer / Parser / AST / TypeChecker 全链路
- ✅ C 子集语法扩展（break/continue/char/switch/typedef/enum/unsigned）
- ✅ P/Invoke C API 接口（前端已接入）
- ✅ 基础运行验证（递归、循环、指针、struct、printf/scanf）
- ✅ P0 安全修复（线程泄漏、内存重叠、全局变量取地址）

### 正在做
- 🔄 知识图谱系统 + 学习进度追踪

### 下一步
- Stage 6 补完：降帧率、简化渲染
- OCR 照片导入（长远）

---

## 七、历史文档备份

以下文档已归档备份，保留原始内容：

| 原始文档 | 备份文件名 |
|:---|:---|
| `PHASE3_CODE_REVIEW_AND_PLAN.md` | `ARCHIVE_PHASE3_CODE_REVIEW_AND_PLAN_20260427.md` |
| `PHASE3_P0_FIX_LOG.md` | `ARCHIVE_PHASE3_P0_FIX_LOG_20260427.md` |
| `CUSTOM_VM_DESIGN.md` | `ARCHIVE_CUSTOM_VM_DESIGN_20260427.md` |

本文件 `ROADMAP.md` 为最新主文档，后续所有计划更新以此为准。
