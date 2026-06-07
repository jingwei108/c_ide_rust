# Cide 标准库支持矩阵

> **状态**：基于 STDLIB_AND_TEST_DESIGN.md 实施路线图，截至 2026-06-07。
> **设计原则**：All in. Record don't hide. Fix real bugs, not test cases.

---

## 一、支持的头文件

| 头文件 | 状态 | 说明 |
|--------|------|------|
| `<stdio.h>` | ✅ | 存根声明已加载，含 `printf`/`scanf`/`getchar`/`putchar`/`fopen`/`fclose`/`fread`/`fwrite`/`fgets`/`fputs`/`feof`/`fprintf` |
| `<stdlib.h>` | ✅ | 存根声明已加载，含 `malloc`/`free`/`realloc`/`atoi`/`abs`/`rand`/`srand`/`exit` |
| `<ctype.h>` | ✅ | 存根声明已加载，含 `isdigit`/`isalpha`/`islower`/`isupper`/`tolower`/`toupper`/`isspace`/`isalnum`/`isprint`/`iscntrl`/`isxdigit` |
| `<math.h>` | ✅ | 存根声明已加载，含 `sin`/`cos`/`sqrt`/`pow`/`atan`/`log`/`exp`（`libm` `double` 精度） |
| `<string.h>` | ✅ | 存根声明已加载，含 `strlen`/`strcpy`/`strncpy`/`strcmp`/`strcat`/`memcpy`/`memmove`/`memset` |

---

## 二、函数级支持矩阵

### 2.1 stdio.h

| 函数 | Layer | 类型检查 | Host Contract | Bytecode Consistency | Differential | 备注 |
|------|-------|----------|---------------|----------------------|--------------|------|
| `printf` | B | 硬编码（变参） | ✅ | N/A | N/A | 格式字符串诊断 E3032/E3062 |
| `scanf` | B | 硬编码（变参） | ✅ | N/A | N/A | — |
| `fprintf` | B | 硬编码（变参） | ✅ | N/A | N/A | — |
| `getchar` | B | 硬编码 | ✅ | N/A | N/A | — |
| `putchar` | B | 硬编码 | ✅ | N/A | N/A | — |
| `fopen` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |
| `fclose` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |
| `fread` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |
| `fwrite` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |
| `fgets` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |
| `fputs` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |
| `feof` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |

### 2.2 stdlib.h

| 函数 | Layer | 类型检查 | Host Contract | Bytecode Consistency | Differential | 备注 |
|------|-------|----------|---------------|----------------------|--------------|------|
| `malloc` | B | 硬编码 | ✅ | N/A | N/A | UAF/泄漏检测 |
| `free` | B | 硬编码 | ✅ | N/A | N/A | UAF/Double-Free 检测 |
| `realloc` | B | 硬编码 | ✅ | N/A | N/A | — |
| `atoi` | B | 硬编码 | ✅ | ✅ | ✅ | — |
| `abs` | B | 存根声明 | ✅ | ✅ | ✅ | 已替代硬编码 |
| `rand` | B | 硬编码 | ✅ | ✅ | ✅ | — |
| `srand` | B | 硬编码 | ✅ | ✅ | ✅ | — |
| `exit` | B | 硬编码 | N/A | N/A | N/A | — |
| `qsort` | B | 硬编码 | N/A | N/A | N/A | VM 回调敏感，未在存根中声明 |

### 2.3 ctype.h

| 函数 | Layer | 类型检查 | Host Contract | Bytecode Consistency | Differential | 备注 |
|------|-------|----------|---------------|----------------------|--------------|------|
| `isdigit` | B | 存根声明 | N/A | ✅ | ✅ | — |
| `isalpha` | B | 存根声明 | N/A | ✅ | ✅ | — |
| `islower` | B | 存根声明 | N/A | ✅ | ✅ | — |
| `isupper` | B | 存根声明 | N/A | ✅ | ✅ | — |
| `tolower` | B | 存根声明 | N/A | ✅ | ✅ | — |
| `toupper` | B | 存根声明 | N/A | ✅ | ✅ | — |
| `isspace` | B | 存 stub 声明 | N/A | ✅ | ✅ | — |
| `isalnum` | B | 存根声明 | N/A | ✅ | ✅ | — |
| `isprint` | B | 存根声明 | N/A | ✅ | ✅ | — |
| `iscntrl` | B | 存根声明 | N/A | ✅ | ✅ | — |
| `isxdigit` | B | 存根声明 | N/A | ✅ | ✅ | — |

### 2.4 math.h

| 函数 | Layer | 类型检查 | Host Contract | Bytecode Consistency | Differential | 备注 |
|------|-------|----------|---------------|----------------------|--------------|------|
| `sin` | B | 存根声明 | ✅ | N/A | N/A | `libm::sin`，`double` |
| `cos` | B | 存根声明 | ✅ | N/A | N/A | `libm::cos`，`double` |
| `sqrt` | B | 存根声明 | ✅ | N/A | N/A | `libm::sqrt`，`double` |
| `pow` | B | 存根声明 | ✅ | N/A | N/A | `libm::pow`，`double` |
| `atan` | B | 存根声明 | ✅ | N/A | N/A | `libm::atan`，`double` |
| `log` | B | 存根声明 | ✅ | N/A | N/A | `libm::log`（自然对数），`double` |
| `exp` | B | 存根声明 | ✅ | N/A | N/A | `libm::exp`，`double` |

### 2.5 string.h

| 函数 | Layer | 类型检查 | Host Contract | Bytecode Consistency | Differential | 备注 |
|------|-------|----------|---------------|----------------------|--------------|------|
| `strlen` | C | 存根声明 | ✅ | ✅ | ✅ | 已切换为 Bytecode Libc 路径 |
| `strcpy` | B | 硬编码 | ✅ | ✅ | ✅ | E3070 Buffer Overflow 诊断 |
| `strncpy` | B | 硬编码 | ✅ | ✅ | ✅ | — |
| `strcmp` | C | 存根声明 | N/A | ✅ | ✅ | 已切换为 Bytecode Libc 路径 |
| `strcat` | B | 硬编码 | ✅ | ✅ | ✅ | E3070 Buffer Overflow 诊断 |
| `memcpy` | B | 硬编码 | N/A | ✅ | ✅ | — |
| `memmove` | B | 硬编码 | N/A | ✅ | ✅ | — |
| `memset` | B | 硬编码 | ✅ | ✅ | ✅ | — |

---

## 三、宏与类型定义

| 名称 | 值 | 来源 | 备注 |
|------|-----|------|------|
| `NULL` | `0` | Lexer 预定义宏 | 全局可用 |
| `EOF` | `-1` | Lexer 预定义宏 | 全局可用 |
| `stdin` | `0` | Lexer 预定义宏 | 全局可用 |
| `stdout` | `1` | Lexer 预定义宏 | 全局可用 |
| `stderr` | `2` | Lexer 预定义宏 | 全局可用 |
| `size_t` | `unsigned int` | 存根 `typedef` | 多文件声明，覆盖不报错 |
| `FILE` | `void*` | 存根 `typedef` | `stdio.h` 中定义 |

---

## 四、缺口与路线图

### 已解除的缺口

| 缺口 | 解除时间 | 说明 |
|------|----------|------|
| `math.h` 不支持 | 2026-06-07 | 引入 `libm`，注册 7 个数学函数 Host Func |
| `#include <stdio.h>` 被跳过 | 2026-06-07 | Lexer 加载存根，TypeChecker 识别声明 |

### 剩余缺口（按优先级）

#### P1 — 架构完整性

| 缺口 | 影响 | 建议修复 |
|------|------|----------|
| **Bytecode Libc 产品化** | 学生代码调用 `isdigit` 走 Rust Host，无法展示 libc 源码 | Round 3：构建期预编译 `.c → .cidebc`，全局函数表预留固定索引，编译器前端识别 `#include` 后走 Bytecode 路径 |
| **VM Builtin 指令（Layer A）** | `memcpy`/`memset`/`strlen` 仍走 `CallHost`，无专用指令优化 | Round 4：实验性添加 `OpCode::Memcpy`/`Memset`/`Strlen`，profiling 确认瓶颈 |

#### P2 — 文档与长期维护

| 缺口 | 影响 | 建议修复 |
|------|------|----------|
| **函数级验证状态自动化追踪** | 手动维护矩阵易过时 | CI 中解析 `host_contract_tests.rs` / `differential_stress.rs` / `bytecode_libc_consistency.rs`，自动生成矩阵段落 |

---

*文档状态：产品化进度追踪*
*最后更新：2026-06-07*
