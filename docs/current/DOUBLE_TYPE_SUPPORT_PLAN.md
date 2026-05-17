# `double` 类型全管线支持 — 实现方案与设计文档

> 状态：**阶段 1 已完成**（Lexer 映射 `double` → `float`，教学宽松模式）| 阶段 2（全管线 f64）待实施
> 阶段 1 完成日期：2026-05-17 | 阶段 2 预估工作量：3–5 天
>
> ⚠️ **当前实现**：`double` 被解析为 `TokenType::Float`，语义上等价于 `float`（32 位）。这解决了"代码写 `double` 编译失败"的问题，但精度仍是 float 级别。影子验证显示 3 个 double 用例中 2 个完全匹配、1 个有 float/double 精度差异（预期内）。

---

## 一、背景与动机

Cide 当前已完整支持 `float`（32 位单精度浮点），但 **不支持 `double`**（64 位双精度）。这是 ROADMAP 中明确列出的缺失特性，也是学生在学习 C 语言时必然会遇到的类型（`3.14` 默认就是 `double`，`%lf` 格式符等）。

竞品（OnlineGDB、Cxxdroid）均支持 `double`，缺失此特性会在教学场景中造成"代码在本机能跑，在 Cide 里编译失败"的困惑。

---

## 二、当前 `float` 实现路径速览

| 阶段 | 关键代码 | 说明 |
|------|---------|------|
| **Lexer** | `TokenType::Float`, `TokenType::FloatLiteral` | `float` 关键字 + `3.14` 字面量（无 `f` 后缀） |
| **Parser** | `TypeKind::Float`, `Expr::FloatLiteral { value: f64 }` | AST 中用 `f64` 存字面量，但语义类型是 `TypeKind::Float` |
| **TypeChecker** | `is_scalar` 含 `Float`；`insert_implicit_cast` 处理 int↔float | 隐式转换链：char → int → **float** |
| **BytecodeGen** | `PushConstF`, `CastI2F`, `CastF2I`, `AddF`/`SubF`/`MulF`/`DivF` + 比较指令 | operand 直接存 `f32` bit pattern（`i32`） |
| **VM** | `stack: Vec<i32>`；`f32::from_bits(self.pop() as u32)` | float 复用 32 位栈 slot，内存操作复用 `LoadMem`/`StoreMem` |

---

## 三、实现方案选型

### 方案 A：VM 栈扩展为 `Vec<u64>`（推荐 ✅）

将 `stack: Vec<i32>` 改为 `stack: Vec<u64>`，每个 slot 64 位：
- **i32/f32/pointer**：零扩展为 `u64`，低 32 位有效
- **f64（double）**：完整占用 64 位
- **指令统一**：所有 `push()`/`pop()` 操作 `u64`
- **新增指令**：`PushConstD`, `AddD`/`SubD`/`MulD`/`DivD`, `CastI2D`, `CastF2D`, `CastD2I`, `CastD2F`, `EqD`/`NeD`/`LtD`/`LeD`/`GtD`/`GeD`
- **常量池**：`f64_constants: Vec<f64>`，`PushConstD` 的 operand 为常量池索引（`operand: i32` 无法直接存 64 位值）

**优点**：
- 架构干净，未来支持 `long long`（64 位整型）也无需再改栈结构
- double 运算指令与普通指令模式一致（每个值一个 slot）

**缺点**：
- 改动面大：所有现有指令（`Add`, `PushConst`, `LoadMem`, `Call` 等）的栈操作需适配 `u64`
- `frb_generated.rs` 可能需要重新生成（如果 `Session` 结构变化）
- 内存占用翻倍（栈从 4 字节/slot → 8 字节/slot）

**工作量**：3–5 天

---

### 方案 B：double 拆分为两个 32 位 slot

保持 `stack: Vec<i32>`，每个 double 占两个 slot（低 32 位 + 高 32 位）：
- `PushConstD`：压入两个 `i32`
- `AddD`：弹出 4 个 `i32`，组合为两个 `f64`，相加后拆回两个 `i32` 压栈
- 内存操作：新增 `LoadMemD`/`StoreMemD` 读写 8 字节

**优点**：
- 栈结构不变，现有指令无需修改

**缺点**：
- 栈管理复杂（每个 double 占 2 slot，SP 偏移不一致）
- `Call`/`Return`/`Local` 等指令需要感知 double 的 slot 占用
- 极易引入栈不平衡 bug

**工作量**：2–3 天（但调试成本极高）

---

### 方案 C：教学简化（`double` 映射为 `float`）

Parser 识别 `double` 关键字，但内部全部映射为 `TypeKind::Float`（32 位）。

**优点**：
- 半天即可完成

**缺点**：
- 不诚实：`sizeof(double)` 会返回 4 而非 8
- 精度丢失：学生写 `double x = 1.0000000001;` 实际只有 7 位有效数字
- 与标准 C 语义不一致，教学中会造成误解

**结论**：不推荐 ❌

---

## 四、推荐方案 A 的详细改动清单

### 4.1 Lexer（~30 分钟）

- `TokenType` 新增：`Double`
- 关键字映射新增：`"double" => Some(TokenType::Double)`
- **字面量语义调整**：
  - `3.14` 默认应为 `double`（C 标准），但当前实现为 `float`
  - 为向后兼容，可保持 `3.14` 为 `float`，新增 `3.14` 赋值给 double 变量时由 TypeChecker 自动提升
  - 或更标准：`3.14` → `DoubleLiteral`，`3.14f`/`3.14F` → `FloatLiteral`
  - Lexer 需识别 `f`/`F` 后缀（当前不支持）

### 4.2 AST（~30 分钟）

- `TypeKind` 新增：`Double`
- `Type` 新增构造函数：`Type::double() -> Self`
- `Expr::FloatLiteral` 保持 `value: f64`，但语义类型根据上下文决定（或拆分为 `FloatLiteral`/`DoubleLiteral`）

### 4.3 Parser（~1 小时）

- `parse_base_type()` 新增 `TokenType::Double` 分支 → `Type::double()`
- `parse_type_only()` 处理 `double*`

### 4.4 TypeChecker（~2 小时）

- `is_scalar()` 加入 `Double`
- 隐式转换链扩展：**char → int → float → double**
- `insert_implicit_cast` 处理：
  - `int`/`char`/`float` → `double`：允许，hint 提示
  - `double` → `float`/`int`/`char`：允许，warning 提示精度丢失
- 二元运算类型提升：
  - 任一 operand 为 `double` → 结果为 `double`
  - 无 `double` 但有 `float` → 结果为 `float`
  - 否则 `int`
- `check_builtin_printf`/`scanf`：支持 `%lf` 格式符

### 4.5 BytecodeGen（~2 小时）

- 新增 opcode 使用：
  - `PushConstD`（从 `f64_constants` 池加载）
  - `CastI2D`, `CastF2D`, `CastD2I`, `CastD2F`
  - `AddD`, `SubD`, `MulD`, `DivD`
  - `EqD`, `NeD`, `LtD`, `LeD`, `GtD`, `GeD`
- `gen_expr` 中 `FloatLiteral` 处理：根据目标类型决定 emit `PushConstF` 或 `PushConstD`
- 内存操作：`double` 占 8 字节，复用 `LoadMem`/`StoreMem`（当前 32 位），需确认 8 字节读写
  - 当前 VM 内存读写有 `write_i32`/`read_i32`，需新增 `write_f64`/`read_f64` 或通用 `write_bytes`/`read_bytes`

### 4.6 VM（~1 天）

- **栈结构**：`stack: Vec<i32>` → `Vec<u64>`
- **push/pop**：
  ```rust
  pub fn push(&mut self, val: u64) { ... }
  pub fn pop(&mut self) -> u64 { ... }
  ```
- **现有指令适配**（全部需要改）：
  - `PushConst`：`self.push(inst.operand as u64)`
  - `Add`/`Sub`/`Mul`/`Div`：`let b = self.pop() as i32; let a = self.pop() as i32; self.push((a + b) as u64)`
  - `AddF`：`let b = f32::from_bits(self.pop() as u32); ... self.push(r.to_bits() as u64)`
  - `LoadMem`：`let v = self.read_i32(addr); self.push(v as u64)`
  - `StoreMem`：`let v = self.pop() as i32; self.write_i32(addr, v)`
  - `Call`/`Return` 等涉及栈帧的指令：栈帧偏移计算需适配 8 字节 slot
- **新增 double 指令**：
  ```rust
  OpCode::PushConstD => {
      let idx = inst.operand as usize;
      let val = self.f64_constants.get(idx).copied().unwrap_or(0.0);
      self.push(val.to_bits());
  }
  OpCode::AddD => {
      let b = f64::from_bits(self.pop());
      let a = f64::from_bits(self.pop());
      self.push((a + b).to_bits());
  }
  // ... SubD, MulD, DivD, CastI2D, CastF2D, CastD2I, CastD2F, 比较指令
  ```
- **常量池**：新增 `f64_constants: Vec<f64>`，`setup_vm` 时传入
- **内存读写**：新增 `read_f64`/`write_f64`（连续 8 字节）
- **诊断**：`format_bounds_error` 等若涉及 double 数组，需适配 8 字节元素大小

### 4.7 C API / FRB 桥接（~2 小时）

- `cide_capi.h`：确认无需改动（类型信息不透明给 C 端）
- `session.rs`：若 `CompileResult`/`RunResult` 等结构不变，则无需改动
- `frb_generated.rs`：**可能需要重新生成**，若 `Instruction` 或 `CideVM` 结构变化影响 FRB 序列化
- `flutter_bridge.rs`：传入 `f64_constants` 到 `setup_vm`

### 4.8 测试（~1 天）

- Lexer 测试：`double` 关键字识别
- Parser 测试：`double x;`, `double* p;`, `double arr[10];`
- TypeChecker 测试：隐式转换、精度丢失警告、类型提升
- BytecodeGen 测试：`PushConstD`、运算指令序列
- VM E2E 测试：
  - `double x = 3.141592653589793; printf("%lf", x);`
  - `double a = 1.0; double b = 2.0; double c = a + b;`
  - `int i = 5; double d = i;`（隐式提升）
  - `double d = 3.7; int i = d;`（截断）
  - `float f = 1.5f; double d = f;`（float→double）
  - `double arr[5] = {1.1, 2.2, 3.3};`

---

## 五、风险与缓解

| 风险 | 影响 | 缓解 |
|------|------|------|
| VM 栈改为 `u64` 后所有现有指令需适配 | 高（改动面大，易引入回归 bug） | **分步验证**：每改一组指令后立即 `cargo test`，确保无回归 |
| `frb_generated.rs` 与 `Instruction` 结构不兼容 | 中 | 改动 `Instruction` 后重新运行 `flutter_rust_bridge_codegen generate` |
| double 字面量默认语义（`3.14` 是 double 还是 float？） | 中（向后兼容） | 第一阶段保持 `3.14` 为 `float`（兼容现有代码），仅新增 `double` 声明支持；第二阶段再加 `f`/`F` 后缀区分 |
| 内存对齐（8 字节） | 低 | 当前 VM 内存是 `Vec<u8>`，读写连续 8 字节即可；需确认 `u64` read/write 不跨页边界（MEM_SIZE 是 256KB，足够大） |
| `sizeof(double)` | 低 | 需确认 `sizeof` 实现返回 8 而非 4 |

---

## 六、建议的实施顺序

```
Day 1: 上午  Lexer + Parser + AST（double 关键字/类型识别）
Day 1: 下午  TypeChecker（类型检查、隐式转换、printf/scanf %lf）
Day 2: 上午  VM 栈结构改造（Vec<i32> → Vec<u64>）+ 现有指令适配
Day 2: 下午  VM 新增 double 指令 + 常量池
Day 3: 上午  BytecodeGen（生成 double 指令）
Day 3: 下午  BytecodeGen（数组、局部变量、复合赋值）
Day 4: 上午  C API / FRB 桥接 + flutter_bridge 传入常量池
Day 4: 下午  端到端测试 + 修复回归
Day 5:      边界测试 + clippy + 代码审查
```

---

## 七、替代方案：暂不实现 `double`，先完成其他高价值任务

如果 `double` 的 3–5 天工作量在当前 sprint 中过重，以下任务可在更短时间内交付更高价值：

1. **增量编译（4.9）** — 1–2 天
   - 缓存 AST + 字节码，源码未变时跳过完整编译管线
   - 教学场景中学生频繁"小改→运行"，编译速度直接影响体验

2. **Desktop Release 构建优化** — 1–2 天
   - 减小 DLL 体积（strip + LTO）、启动速度优化
   - ROADMAP 中标记为"正在做"

3. **函数指针完整支持** — 2–3 天
   - 当前仅支持函数名作为参数传递（用于 `qsort`）
   - 完整支持：函数指针变量声明、赋值、数组、比较
   - ROADMAP 明确列出

4. **`format_bounds_error` 预建索引（3.8）** — 2–3 小时
   - CODE_REVIEW 遗留项，实现简单但收益有限

---

## 八、决策记录

| 日期 | 决策 |
|------|------|
| 2026-05-16 | 完成 `double` 类型支持方案设计，文档化后暂缓实施，优先评估增量编译和函数指针的 ROI |
