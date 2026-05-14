# C IDE 项目路线图（2026-05-14 修订版）

> **核心原则**：不急着发布，不把时间浪费在"能用"上。每一行代码都指向一个竞品没有的功能亮点。
> **当前状态**：Rust 后端全链路稳定，Flutter 前端已接棒 MAUI，算法可视化 + 诊断修复系统全部就绪。

---

## 一、技术架构（当前）

```
+-----------------------------------------------------------------------------+
|                     Flutter 前端 (Android / Desktop Windows)                 |
|  +-------------+  +-------------+  +-------------------------------------+  |
|  | CodeEditor  |  | MemoryView  |  | KnowledgeCard / QuickFixPanel       |  |
|  |  re_editor  |  |  内存映射    |  | 知识卡片 / 一键修复面板               |  |
|  +-------------+  +-------------+  +-------------------------------------+  |
|  +-------------+  +-------------+  +-------------------------------------+  |
|  | PointerView |  | ErrorPanel  |  | ConsoleOutput / AlgoCanvas          |  |
|  |  指针视图    |  |  诊断面板    |  | 输出控制台 / 算法动画画布             |  |
|  +-------------+  +-------------+  +-------------------------------------+  |
+-----------------------------------------------------------------------------+
                                    |
                                    v flutter_rust_bridge v2 (SSE codec)
+-----------------------------------------------------------------------------+
|                        Rust 后端 (Native DLL / .so)                          |
|                                                                             |
|  +---------------------------------------------------------------------+    |
|  | ① C 子集编译器                                                       |    |
|  |   用户 C 代码 → Lexer → Parser → AST → TypeChecker → BytecodeGen    |    |
|  |   输出：自定义字节码指令序列 + 符号表 + 字符串数据段                   |    |
|  +---------------------------------------------------------------------+    |
|                                    |                                        |
|  +---------------------------------------------------------------------+    |
|  | ② CideVM 教学虚拟机（自研）                                          |    |
|  |   加载字节码 → 解释执行 → 捕获 trap → StepEvent 单步暂停             |    |
|  |   提供内存视图、指针追踪、执行步进、中文错误映射                        |    |
|  +---------------------------------------------------------------------+    |
|                                    |                                        |
|  +---------------------------------------------------------------------+    |
|  | ③ 诊断与可视化引擎                                                   |    |
|  |   源码位置映射 / 内存布局元数据 / 指针追踪表 / 中文错误消息             |    |
|  |   算法模式识别 / 运行时验证 / 执行轨迹分析                             |    |
|  +---------------------------------------------------------------------+    |
+-----------------------------------------------------------------------------+
```

### 目录结构

```
native/
├── Cargo.toml
├── include/
│   └── cide_capi.h              # C API 头文件
├── src/
│   ├── compiler/                 # Lexer / Parser / AST / TypeChecker / BytecodeGen
│   │   ├── lexer.rs
│   │   ├── parser.rs
│   │   ├── ast.rs
│   │   ├── type_checker.rs
│   │   └── bytecode_gen.rs
│   ├── vm/                       # CideVM 字节码解释器
│   │   ├── vm.rs
│   │   ├── opcode.rs
│   │   ├── instruction.rs
│   │   └── host_funcs.rs
│   ├── diagnostics/              # 结构化诊断与自动修复
│   │   ├── error_codes.rs
│   │   └── error_catalog.rs
│   ├── capi/                     # C API 桥接层
│   ├── api/                      # flutter_rust_bridge API
│   └── session.rs                # Session 状态管理
└── tests/                        # Rust 集成测试
CideFlutter/                      # Flutter 前端（Android + Desktop）
├── lib/
│   ├── src/
│   │   ├── rust/                 # FRB 桥接代码
│   │   ├── screens/              # 页面
│   │   ├── widgets/              # 自定义组件
│   │   ├── providers/            # Riverpod 状态管理
│   │   └── services/             # 业务逻辑
│   └── assets/                   # 知识卡片等资源
```

---

## 二、竞品分析 & 差异化壁垒

### 现有竞品

| 产品 | 类型 | 核心能力 | 致命短板 |
|:---|:---|:---|:---|
| **C语言编译器IDE** (Android) | 移动端IDE | 能编译运行C | 英文错误、无调试、无可视化 |
| **Cxxdroid** | 移动端IDE | GCC编译、终端输出 | 无教学引导、无内存视图 |
| **OnlineGDB** | Web IDE | GDB调试 | 网页端、不适合手机、学习曲线陡 |
| **Scratch/Blockly** | 图形编程 | 可视化动画 | **不是真实代码**，无法过渡到工业编程 |
| **Educoder/头歌** | OJ平台 | 在线评测 | 无实时调试、无可视化、无诊断 |

### 四大壁垒

#### 壁垒 1：运行时中文诊断（唯一）

学生写：
```c
for (int i = 0; i <= 5; i++) { arr[i] = i; }
```

其他工具只能说：
- GDB: `Program received signal SIGSEGV`
- OnlineGDB: `Runtime Error`

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

#### 壁垒 3：内存动画（唯一）

- 写 `int* p = &a;`，屏幕实时画出指针箭头
- 写 `p = malloc(4);`，屏幕显示堆区分配动画
- 写 `free(p);`，指针箭头变灰，标记为已释放

#### 壁垒 4：单步变量追踪（差异化）

每走一步，侧边栏显示所有变量的当前值。指针变量的值显示为箭头指向目标地址，数组显示为带索引的格子。

---

## 三、开发阶段（已完成的里程碑）

### Stage 0: 基础编译器（✅ 已完成）
- Lexer → Parser → AST → TypeChecker
- 支持变量、数组、指针、struct、if/for/while/do-while、函数、malloc/free
- 支持 break/continue/switch/typedef/enum/unsigned

### Stage 1: 自研 VM（✅ 已完成）
- **Bytecode 定义**：`opcode.rs` + `instruction.rs`
- **BytecodeGen**：将 AST 编译为自定义字节码
- **CideVM 核心**：~30 条指令的解释器，线性内存管理
- **C API 迁移**：`cide_run` / `cide_step_next` 驱动 VM
- **安全加固**：边界检查、除零捕获、步数熔断、NULL 区陷阱

### Stage 2: 运行时中文诊断（✅ 已完成）
- 符号表导出 + 数组越界精确诊断
- 除零精确诊断 + 空指针精确诊断
- 死循环变量分析
- 56+ 错误码中文元数据 + 结构化自动修复

### Stage 3: 单步调试 + 内存可视化（✅ 已完成）
- 指令级单步：`vm.Step()` 精确到每条字节码指令
- 变量面板 + 内存视图 + 指针追踪
- 内存映射 Canvas（256KB 64×4KB 网格彩色编码）

### Stage 4: 零侵入可视化（✅ 已完成）
- AST 模式识别骨架 + 8 种核心算法检测（冒泡/选择/插入/快排/归并/二分/链表）
- VM 运行时精确事件：`VisEvent::Compare` / `Swap` / `Update`
- 数组实时可视化 Canvas + 交换闪烁动画
- 算法运行时验证（Property-based Testing）

### Stage 5: 诊断与修复系统（✅ 已完成）
- L1/L2/L3 三级信息架构
- QuickFix：补分号、改 `=` 为 `==`、改 `<=` 为 `<`
- 结构化 Auto-Fix：`InsertText` / `ReplaceText` 精确修复
- 知识卡片系统（JSON + 内存动画描述）
- 隐式转换提示系统（warning + hint 分级）

### Stage 6: 前端迁移与体验优化（✅ 已完成）
- Flutter 前端从零搭建：IDE 界面 + `re_editor` 编辑器 + 调试面板
- 内存映射 Canvas + 算法可视化事件 FRB 集成
- VS-style Enter 格式化、Touch swipe tabs、Execution speed slider
- 教程引导 overlay (`IntroOverlay`)
- 学习进度追踪系统（5 维度 + 本地持久化）

### Stage 7: C 子集拓展（✅ 已完成）
- `float` 类型全管线支持（Lexer→Parser→TypeChecker→BytecodeGen→VM）
- 位运算符 `& | ^ ~ << >>`
- 三目运算符 `? :`
- 指针算术（`p++` / `p+i` / `p-q`，自动步长缩放）
- `const` 语义、`NULL` 关键字、块注释 `/* */`
- 复合赋值扩展到数组索引/指针解引用/结构体成员
- 新增宿主函数：`getchar`/`putchar`/`rand`/`srand`/`memset`/`exit`/`strcat`/`atoi`
- `fprintf`/`realloc`/`qsort`
- 函数指针基础支持（用于 `qsort` 回调）

---

## 四、当前状态 & 下一步

### 已完成（保留资产）
- ✅ Rust 后端全链路：Lexer / Parser / AST / TypeChecker / BytecodeGen / VM
- ✅ C 子集语法扩展（float/char/位运算/三目/指针算术/const/NULL）
- ✅ flutter_rust_bridge v2 桥接（SSE codec）
- ✅ 基础运行验证（递归、循环、指针、struct、printf/scanf/float）
- ✅ P0 安全修复（VM 栈-堆碰撞、u32 溢出、移位越界、trap 边界）
- ✅ Flutter 前端端到端可用（编辑器 + 编译 + 运行 + 调试 + 可视化）
- ✅ 学习进度追踪 + 知识卡片系统

### 正在做
- 🔄 知识图谱系统
- 🔄 Desktop 端 Release 构建优化

### 下一步
- `double` 类型支持
- 函数指针完整支持
- 社区贡献算法模板
- iOS 目标支持评估

---

## 五、历史文档备份

以下文档已归档至 `docs/archive/`，保留原始内容：

| 原始文档 | 备份文件名 |
|:---|:---|
| `PHASE3_CODE_REVIEW_AND_PLAN.md` | `ARCHIVE_PHASE3_CODE_REVIEW_AND_PLAN_20260427.md` |
| `CUSTOM_VM_DESIGN.md` | `ARCHIVE_CUSTOM_VM_DESIGN_20260427.md` |
| `OCR_IMPORT_DESIGN.md` | `ARCHIVE_OCR_IMPORT_DESIGN.md` |

本文件 `ROADMAP.md` 为最新主文档，后续所有计划更新以此为准。
