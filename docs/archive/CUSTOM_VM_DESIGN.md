# 自研轻量 VM 设计方案

> 目标：替换 wasm3，实现完全可控的执行引擎，支撑单步调试、运行时中文诊断、零侵入可视化三大核心亮点。  
> 日期：2026-04-27

---

## 一、为什么替换 wasm3？

| 能力 | wasm3 现状 | 自研 VM 后 |
|:---|:---|:---|
| **单步调试** | ❌ 无法暂停/恢复，只能阻塞宿主函数 | ✅ 每条指令后可检查暂停标志 |
| **运行时中文诊断** | ❌ 只能翻译英文 trap 字符串 | ✅ 在除零/越界现场直接读取变量值，生成 "当 i=5 时，arr[10] 越界" |
| **内存可视化** | ⚠️ `m3_GetMemory` 读原始字节，不知道变量名 | ✅ VM 自带符号表，知道 `0x1020` 是 `arr[2]` |
| **零侵入可视化** | ⚠️ 需注入 `__cide_step` 等 host call | ✅ VM 层直接发射事件（compare/swap/update） |
| **执行步数限制** | ✅ 已 patch `m3_Yield` | ✅ 原生支持，更精确 |
| **安全隔离** | ✅ 自动内存隔离 | ✅ 自己检查边界，同等安全 |

**核心洞察**：wasm3 是"通用 WASM 解释器"，而我们需要的是"教学专用 C 子集执行引擎"。通用层反而成了枷锁。

---

## 二、安全隔离如何实现？

自研 VM 的安全模型与 WASM 完全一致：**线性内存 + 指令级边界检查**。

```cpp
class CideVM {
    uint8_t* memory;      // 线性内存
    uint32_t memSize;     // 内存大小（如 256KB）
    uint32_t stackTop;    // 栈顶指针
    uint32_t heapOffset;  // 堆分配指针
    
    // 每次内存访问都检查
    int32_t LoadI32(uint32_t addr) {
        if (addr + 4 > memSize) {
            Trap("内存访问越界：地址 0x{:04X}，内存大小 {}");
        }
        return memory[addr] | ...;
    }
    
    void StoreI32(uint32_t addr, int32_t val) {
        if (addr + 4 > memSize) Trap(...);
        // write
    }
    
    // 除零检查
    int32_t DivI32(int32_t a, int32_t b) {
        if (b == 0) Trap("除零错误：{} / {}，除数 {} 来自变量 '{}'");
        return a / b;
    }
};
```

**关键安全机制**：
1. **内存隔离**：所有 load/store 统一走 `LoadI32/StoreI32`，不可能绕过检查
2. **除零/溢出捕获**：算术指令统一包装
3. **栈溢出保护**：函数调用前检查栈深度
4. **步数熔断**：每条指令执行后 `stepCount++`，超过 1000 万步自动 trap
5. **NULL 区陷阱**：`0x0000~0x0FFF` 保留，load/store 到该区域直接报错（比 wasm3 更严格）

**安全性对比**：与 wasm3 同等，且诊断信息更精确。

---

## 三、竞品差异化分析

### 现有竞品

| 产品 | 类型 | 核心能力 | 短板 |
|:---|:---|:---|:---|
| **C语言编译器IDE** (Android) | 移动端IDE | 能编译运行C | 英文错误、无调试、无可视化 |
| **Cxxdroid** | 移动端IDE | GCC编译、终端输出 | 无教学引导、无内存视图 |
| **OnlineGDB** | Web IDE | GDB调试 | 网页端、不适合手机、学习曲线陡 |
| **Scratch/Blockly** | 图形编程 | 可视化动画 | 不是真实代码，无法过渡到工业编程 |
| **Educoder/头歌** | OJ平台 | 在线评测 | 无实时调试、无可视化 |

### 我们的差异化壁垒（替换 VM 后才能完全实现）

```
壁垒 1: 运行时中文诊断（唯一）
    "当 i=5 时，arr[10] 越界了。数组大小是 5。"
    "除零错误：除数 b=0，b 在第 3 行被赋值为 0 后未被修改。"
    
壁垒 2: 内存动画（唯一）
    学生写 `int* p = &a;`，屏幕实时画出指针箭头
    写 `arr[i] = temp;`，屏幕显示两个格子的交换动画
    
壁垒 3: 零侵入算法可视化（唯一）
    学生写纯冒泡排序 → 系统自动识别 → 播放排序动画
    不需要写任何 `vis_array()` 之类的额外代码
    
壁垒 4: 单步变量追踪（差异化）
    每走一步，侧边栏显示所有变量的当前值
    指针变量的值显示为箭头指向目标地址
```

**结论**：替换 VM 不是成本，而是**投资**。没有自研 VM，上述 4 个壁垒中 3 个只能做到 60 分；有了自研 VM，可以做到 95 分。

---

## 四、架构设计

### 4.1 编译链路变化

```
替换前:
源代码 → Lexer → Parser → AST → TypeChecker → WasmCodeGen → WASM binary → wasm3

替换后:
源代码 → Lexer → Parser → AST → TypeChecker → BytecodeGen → Bytecode[] → CideVM
```

**改动范围**：
- `Lexer/Parser/AST/TypeChecker`：**完全不变**
- `WasmCodeGen`：改名为 `BytecodeGen`，输出从 `vector<uint8_t>` 改为 `vector<Instruction>`
- 新增 `CideVM`：约 800~1000 行
- C API：`cide_run` / `cide_step_next` 直接驱动 VM，不再依赖 wasm3

### 4.2 字节码设计（Instruction Set）

只保留实际用到的指令，扁平化设计：

```cpp
enum class OpCode : uint8_t {
    // 常量与变量
    PushConst,      // operand = int32 value
    LoadLocal,      // operand = local index
    StoreLocal,     // operand = local index
    LoadGlobal,     // operand = global index
    StoreGlobal,    // operand = global index
    
    // 内存
    LoadMem,        // 从线性内存读 i32
    StoreMem,       // 向线性内存写 i32
    
    // 算术
    Add, Sub, Mul, Div, Mod, Neg,
    
    // 比较
    Eq, Ne, Lt, Le, Gt, Ge,
    
    // 逻辑
    And, Or, Not,
    
    // 控制流
    Jump,           // unconditional jump
    JumpIfZero,     // conditional jump
    Call,           // operand = function index
    CallHost,       // operand = host function id
    Ret,            // return from function
    
    // 栈操作
    Pop,
    Dup,
    
    // 调试/可视化事件（零侵入核心）
    StepEvent,      // operand = source line
    VisEvent,       // operand = event type (compare/swap/access)
};

struct Instruction {
    OpCode op;
    int32_t operand;  // 通用操作数
    SourceLoc loc;    // 源码位置（用于错误映射）
};
```

### 4.3 VM 核心结构

```cpp
class CideVM {
public:
    struct CallFrame {
        size_t returnIP;      // 返回地址（Instruction 索引）
        size_t localsBase;    // 局部变量在栈中的起始位置
    };
    
    // 内存
    std::vector<uint8_t> memory;
    uint32_t heapOffset = 0x5000;
    
    // 符号表（用于运行时诊断和可视化）
    struct Symbol {
        std::string name;
        uint32_t addr;        // 全局地址或栈偏移
        bool isLocal;
        Type type;
    };
    std::vector<Symbol> symbols;
    
    // 执行状态
    std::vector<int32_t> stack;
    std::vector<CallFrame> callStack;
    size_t ip = 0;            // 指令指针
    int stepCount = 0;
    int maxSteps = 10000000;
    bool cancelled = false;
    bool paused = false;      // 单步暂停标志
    
    // 执行
    void LoadProgram(const std::vector<Instruction>& code,
                     const std::vector<int32_t>& globalsInit);
    int32_t Run();            // 运行到结束
    int32_t Step();           // 执行一条指令
    
    // 诊断
    std::string GetRuntimeError() const;
    std::vector<std::pair<std::string, int32_t>> GetLocals() const;
};
```

### 4.4 单步调试实现

```cpp
int32_t CideVM::Step() {
    if (ip >= code_.size()) return 0;
    
    const auto& inst = code_[ip];
    ip++;
    
    switch (inst.op) {
        case OpCode::PushConst: stack.push_back(inst.operand); break;
        case OpCode::Add: { int b = Pop(); int a = Pop(); stack.push_back(a + b); } break;
        // ... 其他指令
        case OpCode::StepEvent: {
            currentLine = inst.operand;
            if (paused) return STEP_PAUSED;  // 单步暂停
            break;
        }
    }
    
    stepCount++;
    if (stepCount >= maxSteps) Trap("执行步数超过限制");
    if (cancelled) Trap("执行已取消");
    
    return STEP_OK;
}
```

`cide_step_next` 直接调用 `vm.Step()` 直到遇到 `StepEvent` 且 `paused == true`。

**不再需要线程！** 单步在主线程同步执行，完全没有线程泄漏问题。

### 4.5 运行时诊断增强

```cpp
case OpCode::Div: {
    int b = Pop();
    int a = Pop();
    if (b == 0) {
        // 自研 VM 可以在这里读取变量符号表
        auto diag = FormatDivZeroError(a, b, currentLine, symbols_);
        // diag = "😵 除零错误：当 a=15, b=0 时发生。
        //         b 在第 3 行被赋值为 0，之后没有被修改。"
        Trap(diag);
    }
    stack.push_back(a / b);
}
```

---

## 五、实施计划

### Week 1: VM 核心 + 替换链路
- [ ] 定义 `Instruction` 和 `OpCode`
- [ ] 将 `WasmCodeGen` 改写为 `BytecodeGen`
- [ ] 实现 `CideVM` 核心（~30 条指令）
- [ ] 修改 C API：`cide_run` 驱动 VM
- [ ] 回归测试：确保所有现有测试通过

### Week 2: 单步 + 诊断增强
- [ ] 实现 `CideVM::Step()` 精细单步
- [ ] 重写 `cide_step_next`（同步，无线程）
- [ ] 符号表注入：编译时记录变量名→地址映射
- [ ] 运行时中文诊断：除零、越界、空指针精确报告

### Week 3: 内存可视化基础
- [ ] VM 内存区域追踪（全局/局部/堆/栈）
- [ ] C API 暴露 `cide_memory_get_regions`
- [ ] 指针追踪：建立 `地址 → 目标变量名` 映射

### Week 4: 零侵入可视化事件
- [ ] `VisEvent` 指令注入（在 BytecodeGen 中自动识别算法模式）
- [ ] VM 层发射事件到前端
- [ ] 前端 Canvas 接收事件并播放动画

---

## 六、与前端/C API 的兼容性

C API 头文件 `cide_capi.h` **完全不变**。前端不需要改任何代码。

内部实现变化：
- `cide_run`：从 `m3_CallV` 改为 `vm.Run()`
- `cide_step_next`：从"线程阻塞hack"改为 `vm.Step()`
- `cide_memory_get_value`：从 `m3_GetMemory` 改为直接读 `vm.memory`

---

## 七、风险与缓解

| 风险 | 概率 | 缓解 |
|:---|:---|:---|
| VM bug 导致测试失败 | 中 | 保留 wasm3 作为 fallback，通过 `#ifdef` 切换 |
| 性能不如 wasm3 | 低 | 教学代码很短（<100行），解释器性能完全够用 |
| 工作量超预期 | 低 | 只支持实际用到的 ~30 条指令，不实现完整 WASM |

---

## 八、结论

替换 wasm3 为自研 VM 是**正确且必要**的决策：
- 短期成本：2~3 周重构
- 长期收益：单步调试、运行时诊断、内存可视化、零侵入可视化四大壁垒全部解锁
- 安全性：与 wasm3 同等，且诊断更精确

**建议立即启动 Week 1。**
