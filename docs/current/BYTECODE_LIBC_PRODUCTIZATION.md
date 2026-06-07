# Bytecode Libc 产品化设计文档

> **状态**: 已实现（2026-06-07）  
> **关联文档**: `STDLIB_AND_TEST_DESIGN.md`、`SUPPORTED_LIBC.md`

---

## 一、设计目标

将 `native/runtime_libc/src/*.c` 从"仅在测试中编译"提升为**产品默认路径**，使学生代码调用 `isdigit(c)` / `abs(n)` / `tolower(c)` 等纯计算函数时，执行 Bytecode Libc 的 C 实现而非 Rust Host Func。

核心诉求：
- **教学展示价值**：学生可以看到 libc 函数的纯 C 实现（`runtime_libc/src/ctype.c` 等）
- **消除重复编译开销**：构建期一次预编译，运行时直接嵌入
- **函数索引固定**：为 JIT trace 缓存、静态分析提供稳定的函数地址空间

---

## 二、核心架构

### 2.1 三层架构

```
构建期                          编译期                          运行时
─────────────────────────────────────────────────────────────────────────────────
runtime_libc/src/*.c
      │
      ▼
cide_cli export ──→ bytecode_libc_data.json  ──→ include_str!  ──→ VM setup
      │                                              │
      └──→ bytecode_libc_index.rs  ──────────────→ BytecodeGen::new()
             (固定索引映射 + 代码长度常量)              (预注册 func_index + 全局地址偏移)
```

### 2.2 VM Code 布局

```
IP 0           IP=LIBC_CODE_LEN                 IP=LIBC_CODE_LEN+user_code_len
│              │                                │
▼              ▼                                ▼
┌──────────────┬────────────────────────────────┐
│ Bytecode     │  User Code                     │
│ Libc 代码    │  (Jump/source_map 已自然偏移)   │
└──────────────┴────────────────────────────────┘
        ▲
        │ 构建期预编译确定长度
        │ setup_vm 加载 + 拼接 + 重定位
```

### 2.3 函数索引布局

```
索引 0        保留（未使用）
索引 1..N     Bytecode Libc 原始索引（内部调用使用）
索引 1000..   Bytecode Libc 固定索引段（用户代码 Call 使用）
索引 1022..   用户函数
```

**双重注册机制**：
- **原始索引**（如 `isdigit` = 1）：Bytecode Libc 内部函数调用（如 `isalnum` → `isdigit`）使用原始索引，确保预编译字节码中的 `Call` 指令无需重定位
- **固定索引**（如 `isdigit` = 1000）：用户代码调用使用固定索引，编译器前端预注册到 `func_index`，保证跨编译会话的稳定性

---

## 三、关键文件说明

| 文件 | 类型 | 说明 |
|------|------|------|
| `scripts/precompile_bytecode_libc.py` | 构建脚本 | 构建期预编译脚本，生成产物文件 |
| `native/src/vm/bytecode_libc_data.json` | 产物 | 预编译字节码 + 函数元数据 + 全局初始化数据（提交 git） |
| `native/src/vm/bytecode_libc_index.rs` | 产物 | 固定索引映射常量（提交 git） |
| `native/src/vm/bytecode_libc_loader.rs` | 源码 | 运行时加载器，解析 JSON 产物 |
| `native/src/bin/cide_cli.rs` | 源码 | `export` 子命令：编译 C 源码并输出 JSON |

---

## 四、产物生成与更新流程

### 4.1 首次生成

```bash
python scripts/precompile_bytecode_libc.py
```

脚本逻辑：
1. `cargo build --release --bin cide_cli`
2. `cide_cli export runtime_libc/src/*.c -o native/src/vm/bytecode_libc_data.json`
3. 读取 JSON，生成 `native/src/vm/bytecode_libc_index.rs`

### 4.2 修改 runtime_libc C 源码后

```bash
# 1. 修改 native/runtime_libc/src/*.c
# 2. 重新预编译
python scripts/precompile_bytecode_libc.py
# 3. 提交产物与源码
git add native/src/vm/bytecode_libc_data.json native/src/vm/bytecode_libc_index.rs native/runtime_libc/src/
git commit -m "update runtime_libc: ..."
```

### 4.3 CI 检查

`.github/workflows/ci.yml` 已集成检查步骤：

```yaml
- name: Check Bytecode Libc precompiled artifacts are up-to-date
  run: python scripts/precompile_bytecode_libc.py --check
```

若产物与 C 源码不同步，CI 会失败并提示重新运行预编译脚本。

---

## 五、运行时加载流程（setup_vm）

```rust
pub fn setup_vm(vm: &mut CideVM, session: &Session) {
    // 1. 加载预编译产物
    let libc = load_artifact();
    let libc_code_len = libc.code.len();

    // 2. 重定位用户代码中的 Jump 指令
    let mut user_code = session.compile.bytecode.clone();
    for instr in &mut user_code {
        match instr.op {
            OpCode::Jump | OpCode::JumpIfZero | OpCode::JumpIfNotZero => {
                instr.operand += libc_code_len as i32;
            }
            _ => {}
        }
    }

    // 3. 拼接代码
    let mut full_code = libc.code.clone();
    full_code.extend_from_slice(&user_code);
    vm.load_program(full_code);

    // 4. 注册原始索引（Bytecode Libc 内部调用）
    for (name, meta) in &libc.func_table {
        if let Some(&raw_idx) = libc.func_index.get(name) {
            vm.register_function(raw_idx as u32, ...);
        }
    }

    // 5. 注册固定索引（用户代码调用）
    for (name, meta) in &libc.func_table {
        if let Some(idx) = bytecode_libc_index(name) {
            vm.register_function(idx as u32, ...);
        }
    }

    // 6. 注册用户函数（IP 偏移 libc_code_len）
    for (name, meta) in &session.compile.func_table {
        if let Some(&idx) = session.compile.func_index.get(name) {
            vm.register_function(idx as u32, FuncMeta {
                ip: meta.ip + libc_code_len,  // 关键：IP 偏移
                ...
            });
        }
    }

    // 7. 合并常量池、全局变量、字符串数据
    // ...

    // 8. 设置 VM 入口为用户代码起始位置
    vm.set_ip(libc_code_len);
}
```

---

## 六、编译器前端路径切换

### 6.1 BytecodeGen 预注册

```rust
// BytecodeGen::new()
for &name in BYTECODE_LIBC_PURE_FUNCS.iter() {
    if let Some(idx) = bytecode_libc_index(name) {
        func_index.insert(name.to_string(), idx);
    }
}
next_func_idx = BYTECODE_LIBC_BASE_INDEX + BYTECODE_LIBC_FUNC_COUNT as i32 + 1;
```

### 6.2 Host Func ID 清理

```rust
pub const BYTECODE_LIBC_PURE_FUNCS: &[&str] = &[
    "isdigit", "isalpha", "islower", "isupper",
    "tolower", "toupper", "isspace", "isalnum",
    "isprint", "iscntrl", "isxdigit", "abs",
];

pub fn by_user_name(name: &str) -> Option<u32> {
    if is_bytecode_libc_pure(name) {
        return None;  // 禁用 Host 映射，生成 Call 而非 CallHost
    }
    // ... 其他 Host-only 函数
}
```

### 6.3 用户代码调用路径对比

**修改前**（Host 路径）：
```
用户代码 isdigit(c)
    → BytecodeGen: by_user_name("isdigit") → ISDIGIT (42)
    → 生成 OpCode::CallHost(42)
    → VM: execute_host_func → Rust host_isdigit()
```

**修改后**（Bytecode 路径）：
```
用户代码 isdigit(c)
    → BytecodeGen: func_index["isdigit"] → 1000
    → 生成 OpCode::Call(1000)
    → VM: do_call(1000) → Bytecode Libc 的 C 实现
```

---

## 七、地址空间规划

| 区域 | 起始地址 | 大小 | 说明 |
|------|----------|------|------|
| Bytecode Libc 全局变量 | `0x1000` (GLOBAL_START) | `BYTECODE_LIBC_GLOBALS_RESERVED` (1024 bytes) | 预编译产物中的 `__rand_seed` 等 |
| 用户全局变量 | `0x1000 + 1024` | 动态分配 | BytecodeGen 中 `next_global_offset` 从 1024 开始 |
| Bytecode Libc 字符串数据 | `0x1000 + globals_size` | 动态分配 | 预编译产物中的字符串常量 |
| 用户字符串数据 | `0x1000 + BYTECODE_LIBC_GLOBALS_RESERVED + user_globals_size` | 动态分配 | 用户代码中的字符串常量 |

---

## 八、已知限制

### 8.1 函数覆盖范围有限

**当前走 Bytecode 路径的函数**：仅 ctype 全家桶（`isdigit`/`isalpha`/.../`isxdigit`）+ `abs`。  
**仍走 Host 路径的函数**：`strcpy`/`strcat`/`strncpy`/`memcpy`/`memmove`/`strlen`/`strcmp`/`atoi`/`rand`/`srand` 等。

**原因**：
- `strcpy`/`memcpy` 等函数在 Host 层注入了 **E3070 Buffer Overflow** 诊断，走 Bytecode 路径会丢失教学安全检测
- `rand`/`srand` 在 Host 层维护全局状态，与 CRT 行为保持一致更为安全
- `printf`/`scanf` 等 I/O 函数需要操作 `session.runtime.output_lines` 和 VFS，无法纯字节码化

**后续扩展**：可逐步将纯计算函数迁移到 Bytecode 路径，但每次迁移需评估诊断能力损失。

### 8.2 CRT 行为差异

C 标准只保证 ctype 函数返回**非零值**表示真，不保证具体数值：
- **Bytecode Libc**：返回 `0` 或 `1`
- **CRT（Windows MSVC）**：通常返回位掩码（如 `isdigit` 返回 `4`，`isalpha` 返回 `2`/`8`）

**影响**：依赖 `printf("%d\n", isdigit('5'))` 输出具体数值的代码，在 Clang（CRT）与 Cide（Bytecode Libc）之间会产生输出差异。

**缓解**：
- 测试 driver 统一转换为布尔值输出：`printf("%d\n", isdigit('5') ? 1 : 0)`
- 已修改 `test_isdigit.c` / `test_ctype_extra.c`

### 8.3 预编译产物版本控制

**循环依赖**：`cide_cli` 依赖 `cide_native`，而 `cide_native` 的编译产物（`.cidebc`）又需要 `cide_cli` 生成。  
**解决方案**：预编译产物始终提交到 git，开发者仅在修改 `runtime_libc/src/*.c` 时手动运行脚本更新。

**风险**：忘记运行预编译脚本即提交 C 源码修改，会导致 CI 失败（`--check` 模式会检测）。

### 8.4 全局变量命名冲突

`stdlib.c` 中的 `static unsigned int __rand_seed` 是全局变量，合并编译后对用户代码可见。虽然使用了内部命名 `__rand_seed`，但理论上仍可能与用户代码冲突。

**缓解**：若发生冲突，可改名为 `__cide_rand_seed` 等更独特的内部名称。

### 8.5 VLA 与 Bytecode Libc

预编译的 Bytecode Libc 代码长度是固定的（`BYTECODE_LIBC_CODE_LEN`）。如果未来 Bytecode Libc 引入了 VLA（变长数组），其栈帧大小是动态的，可能影响 `local_count` 的预编译值。当前 `runtime_libc/src/*.c` 中无 VLA，暂不受影响。

### 8.6 source_map 偏移的边界情况

`source_map` 中的 IP 已统一偏移 `BYTECODE_LIBC_CODE_LEN`，确保 capi 调试查询正确。但如果用户代码的 `return_ip` 恰好落在 Bytecode Libc 区域内（如 Bytecode Libc 函数返回时），`capi/mod.rs` 中的 source_map 查询会返回 `best_line = 0`（无对应条目）。这是可接受的行为，表示"无法定位到用户源代码"。

---

## 九、验收标准（已实现）

| 标准 | 状态 |
|------|------|
| 用户代码 `#include <ctype.h>` 后调用 `isdigit('5')`，VM 执行 Bytecode Libc C 实现 | ✅ |
| `cargo test --test bytecode_libc_consistency` 全绿 | ✅ |
| `cargo test --test differential_stress` 全绿 | ✅ |
| `cargo test --test host_contract_tests` 全绿 | ✅ |
| Shadow Verification 无新增失败 | ✅ |
| `cargo clippy -- -D warnings` 0 警告 | ✅ |
| CI 集成预编译产物检查 | ✅ |

---

*文档状态：实现完成 + 已知限制记录*  
*最后更新：2026-06-07*
