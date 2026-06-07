# Cide 标准库支持矩阵

> **状态**：基于 STDLIB_AND_TEST_DESIGN.md 四层架构与分层规则，截至 2026-06-07。
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
| `strdup` | B | 硬编码 | N/A | N/A | N/A | `malloc`+`strcpy`，内存追踪 |

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

## 四、已解除的缺口

| 缺口 | 解除时间 | 说明 |
|------|----------|------|
| `math.h` 不支持 | 2026-06-07 | 引入 `libm`，注册 7 个数学函数 Host Func |
| `#include <stdio.h>` 被跳过 | 2026-06-07 | Lexer 加载存根，TypeChecker 识别声明 |
| `strdup` 不支持 | 2026-06-07 | 新增 `STRDUP` Host Func，复用 `allocate_raw` + `MemoryRegion` 追踪 |

---

## 五、下一阶段全面拓展蓝图（基于四层架构分层规则）

> 目标：一次性规划到位，覆盖 C89 教学高频函数 + C99 补充，避免反复修补。
> 分层依据：STDLIB_AND_TEST_DESIGN.md 第 2.1 节分层规则。

### 5.1 分层规则速查

| 类型 | 必须放在 | 原因 |
|------|---------|------|
| **内存安全诊断敏感** | **Layer B** | 需要注入边界检查、UAF 检测、行号追踪 |
| **I/O 沙盒敏感** | **Layer B** | 需要操作 `session.runtime.output_lines` / VFS |
| **VM 回调敏感** | **Layer B** | 需要 `call_user_function` 回调 VM 函数 |
| **纯计算、无副作用** | **Layer C**（优先）或 **Layer B** | Bytecode 可教学展示源码；Rust 可借助 `libm` |
| **超高频内存原语** | **Layer A** | 原生执行，避免 CallHost 开销（待 profiling） |

---

### 5.2 🔴 P0 — 立即填补（学生代码编译失败最高频）

#### 5.2.1 零成本修复（Host Func 已实现，仅缺存根声明 / 宏定义）

| 头文件 | 内容 | 分层 | 动作 | 学生场景 |
|--------|------|------|------|---------|
| `<stdlib.h>` | `qsort` | **Layer B** | 存根添加声明 | LeetCode 排序、数据结构教材 |
| `<limits.h>` | `INT_MAX`, `INT_MIN`, `LONG_MAX`, `LONG_MIN`, `CHAR_BIT` | **无 Layer** | 新增 stub 头文件（宏定义） | DP 初始化、边界判断 |
| `<stdbool.h>` | `bool`, `true`, `false` | **无 Layer** | 新增 stub 头文件 | 现代 C 代码标配 |
| `<stdlib.h>` | `EXIT_SUCCESS`, `EXIT_FAILURE`, `RAND_MAX` | **无 Layer** | 存根添加宏定义 | 程序返回值、随机数范围 |
| `<stdio.h>` | `SEEK_SET`, `SEEK_CUR`, `SEEK_END` | **无 Layer** | 存根添加宏定义 | `fseek` 定位 |

#### 5.2.2 新增函数（P0 核心）

| 头文件 | 函数 | 分层 | 学生场景 |
|--------|------|------|---------|
| `<stdio.h>` | `puts` | **Layer B** | 简单输出字符串+换行 |
| `<stdio.h>` | `sprintf`, `snprintf` | **Layer B** | 字符串构造、数字转字符串 |
| `<stdio.h>` | `sscanf` | **Layer B** | 字符串解析 |
| `<stdlib.h>` | `calloc` | **Layer B** | 安全分配并零初始化（图论邻接表） |

---

### 5.3 🟠 P1 — 短期实现（教学/算法必备）

#### 5.3.1 已有头文件补全

| 头文件 | 函数 | 分层 | 分层依据 |
|--------|------|------|---------|
| `<stdlib.h>` | `bsearch` | **Layer B** | VM 回调敏感（需调用用户比较函数） |
| `<stdlib.h>` | `atof`, `atol` | **Layer C** | 纯计算，无副作用 |
| `<stdlib.h>` | `strtol`, `strtod` | **Layer B** | 涉及 `errno` 设置 + `endptr` 写入 |
| `<stdlib.h>` | `abort` | **Layer B** | 需操作 session state（终止+诊断） |
| `<stdlib.h>` | `labs`, `llabs` | **Layer C** | 纯计算，和 `abs` 同理 |
| `<string.h>` | `strchr`, `strrchr` | **Layer C** | 纯计算，指针遍历算法可展示源码 |
| `<string.h>` | `strstr` | **Layer C** | 纯计算，KMP/朴素算法教学价值 |
| `<string.h>` | `strncmp` | **Layer C** | 纯计算 |
| `<string.h>` | `strncat` | **Layer B** | 内存安全诊断敏感（Buffer Overflow 检测） |
| `<string.h>` | `memcmp` | **Layer C** | 纯计算 |
| `<string.h>` | `memchr` | **Layer C** | 纯计算 |
| `<string.h>` | `strtok` | **Layer C** | 纯计算但涉及全局静态状态；走 C 可展示源码 |
| `<string.h>` | `strerror` | **Layer B** | 返回静态错误字符串表，需 Host 管理 |
| `<math.h>` | `tan`, `fabs`, `ceil`, `floor`, `round` | **Layer B** | 与现有 7 个 math 函数保持一致（`libm` 精度） |
| `<math.h>` | `fmod`, `log10` | **Layer B** | 同上，`libm` 直接可用 |
| `<math.h>` | `asin`, `acos`, `atan2` | **Layer B** | 同上 |
| `<math.h>` | `sinh`, `cosh`, `tanh` | **Layer B** | 同上 |
| `<ctype.h>` | `isgraph`, `ispunct`, `isblank` | **Layer C** | 纯计算，和现有 ctype 函数同理 |

#### 5.3.2 新增头文件

| 头文件 | 内容 | 分层 | 实现方式 |
|--------|------|------|---------|
| `<assert.h>` | `assert` | **Layer B** | 宏展开为条件判断 + `__cide_assert_fail` Host Func |
| `<time.h>` | `time`, `clock` | **Layer B** | 系统调用封装 |
| `<time.h>` | `CLOCKS_PER_SEC` | **无 Layer** | 宏定义 |
| `<errno.h>` | `errno`, `EDOM`, `ERANGE` | **Layer B** | 全局变量 + Host 在 math/IO 错误时设置 |
| `<stdio.h>` | `fgetc`, `fputc` | **Layer B** | VFS-backed |
| `<stdio.h>` | `fseek`, `ftell`, `rewind` | **Layer B** | VFS-backed |
| `<stdio.h>` | `fflush`, `perror`, `clearerr` | **Layer B** | VFS-backed |
| `<stddef.h>` | `ptrdiff_t`, `offsetof` | **无 Layer** | typedef + 编译期宏 |

---

### 5.4 🟡 P2 — 中期实现（进阶补全）

| 头文件 | 内容 | 分层 | 说明 |
|--------|------|------|------|
| `<math.h>` | `INFINITY`, `NAN`, `HUGE_VAL` | **无 Layer** | 编译期常量宏 |
| `<float.h>` | `FLT_MAX`, `DBL_MAX`, `FLT_EPSILON` 等 | **无 Layer** | 编译期常量宏 |
| `<stdint.h>` | `int32_t`, `uint64_t` 等 | **无 Layer** | typedef 映射到已有类型 |
| `<stdio.h>` | `remove`, `rename` | **Layer B** | VFS 操作 |
| `<string.h>` | `strpbrk`, `strspn`, `strcspn` | **Layer C** | 纯计算 |

---

### 5.5 ⚫ 明确排除项（实现复杂 / 教学价值极低）

| 特性/头文件 | 排除理由 |
|-------------|---------|
| `bitfield`（位域） | 文档已排除；嵌入式专用，初学者不需要 |
| `<complex.h>` / `_Complex` | 数学/工程专用，教学不用 |
| `<fenv.h>` | 浮点环境控制，教学不用 |
| `<locale.h>` | 本地化，教学不用 |
| `<wchar.h>` / `<wctype.h>` / `<uchar.h>` | 宽字符/国际化，除非专门课程 |
| `<threads.h>` / `<stdatomic.h>` | 并发编程，通常用 pthreads 或 C++ 教学 |
| `<setjmp.h>` / `longjmp` | 非局部跳转，教学不鼓励 |
| `_Generic` / `_Static_assert` / `_Alignas` / `_Alignof` | C11 进阶，学生几乎不用 |
| `_Noreturn` / `_Thread_local` / `_Atomic` | 同上 |
| `div` / `ldiv` / `lldiv` | 除法同时获取商和余数，极少使用 |
| `atexit` / `at_quick_exit` | 退出处理函数，教学不用 |
| `getenv` / `system` | 环境变量/系统命令，沙盒中意义有限 |
| `frexp` / `ldexp` / `modf` / `scalbn` 等 math 高级函数 | 数值分析专用 |

---

### 5.6 按分层统计的工作量估算

| 分层 | 新增项数 | 估算工作量 |
|------|---------|-----------|
| **无 Layer（宏/typedef）** | ~35 个宏/typedef | 写 stub 头文件，约 **1 天** |
| **Layer C: Bytecode Libc** | `strchr`, `strrchr`, `strstr`, `strncmp`, `memcmp`, `memchr`, `atof`, `atol`, `labs`, `llabs`, `isgraph`, `ispunct`, `isblank` | 约 **13 个函数**，每个 30 分钟 + 测试，约 **3 天** |
| **Layer B: Rust Host** | `puts`, `sprintf`, `snprintf`, `sscanf`, `calloc`, `bsearch`, `abort`, `strtol`, `strtod`, `strerror`, `strncat`, `fgetc`, `fputc`, `fseek`, `ftell`, `rewind`, `fflush`, `perror`, `clearerr`, `remove`, `rename`, `time`, `clock`, `errno` 管理, `assert`, + math 补全 11 个 | 约 **35 个函数**，每个 30-60 分钟，约 **7-10 天** |
| **Layer A: VM Builtin** | 暂不接 codegen（待 profiling） | 0 |

**总计**：约 **2-3 周** 可完成标准库全面拓展。

---

### 5.7 推荐实施路线图

#### Phase A（第 1 周）：基础设施 + 立即可用
- [ ] `qsort` 存根声明修复（1 行）
- [ ] 新增 stub 头文件：`limits.h`, `stdbool.h`
- [ ] 补全已有头文件宏：`EXIT_SUCCESS`, `EXIT_FAILURE`, `RAND_MAX`, `SEEK_SET/CUR/END`
- [ ] `puts`, `sprintf`, `snprintf`, `sscanf`
- [ ] `calloc`, `bsearch`

**目标**：LeetCode 中 90% 的 C 解法能编译通过。

#### Phase B（第 1-2 周）：字符串 + 文件 I/O 完整化
- [ ] `strchr`, `strrchr`, `strstr`, `strncmp`, `strncat`, `memcmp`, `memchr`
- [ ] `fgetc`, `fputc`, `fseek`, `ftell`, `rewind`, `fflush`, `perror`, `clearerr`
- [ ] `atof`, `atol`, `strtol`, `strtod`, `labs`, `llabs`, `abort`
- [ ] `time`, `clock`

**目标**：数据结构教材示例代码 95% 能编译通过。

#### Phase C（第 2-3 周）：语法补全 + 进阶
- [ ] `assert` 宏
- [ ] `errno`
- [ ] `strtok`, `strerror`
- [ ] `math.h` 补全（`tan`, `fabs`, `ceil`, `floor`, `round`, `fmod`, `log10`, `asin`, `acos`, `atan2`, `sinh`, `cosh`, `tanh`）
- [ ] `isgraph`, `ispunct`, `isblank`
- [ ] `stdint.h`, `float.h` 宏

**目标**：K&R《C程序设计语言》全书示例代码 95% 能编译通过。

#### Phase D（可选）：高级特性
- [ ] `remove`, `rename`
- [ ] `strpbrk`, `strspn`, `strcspn`
- [ ] `INFINITY`, `NAN`, `HUGE_VAL`

---

### 5.8 关键校正说明（与早期分析的差异）

| 函数 | 早期误标 | **严格分层后** | 原因 |
|------|---------|---------------|------|
| `bsearch` | Bytecode C | **Layer B** | VM 回调敏感，需调用用户比较函数 |
| `strncat` | Bytecode C | **Layer B** | 内存安全诊断敏感，需 Buffer Overflow 检测 |
| `strtol` / `strtod` | Bytecode C / Host B | **Layer B** | 涉及 `errno` + `endptr`，非纯计算 |
| `strtok` | Bytecode C | **Layer C** | 虽有全局状态但无诊断需求，走 C 可展示源码 |

---

*文档状态：产品化进度追踪 + 下一阶段全面拓展蓝图*
*最后更新：2026-06-07*
