# Cide 标准库支持矩阵

> **状态**：基于 STDLIB_AND_TEST_DESIGN.md 四层架构与分层规则，截至 2026-06-07。
> **设计原则**：All in. Record don't hide. Fix real bugs, not test cases.

---

## 一、支持的头文件

| 头文件 | 状态 | 说明 |
|--------|------|------|
| `<stdio.h>` | ✅ | 完整 I/O：含 `printf`/`scanf`/`getchar`/`putchar`/`fopen`/`fclose`/`fread`/`fwrite`/`fgets`/`fputs`/`feof`/`fprintf`/`puts`/`sprintf`/`snprintf`/`sscanf`/`fgetc`/`fputc`/`fseek`/`ftell`/`rewind`/`fflush`/`perror`/`clearerr`/`remove`/`rename` |
| `<stdlib.h>` | ✅ | 完整核心：含 `malloc`/`free`/`realloc`/`calloc`/`atoi`/`atof`/`atol`/`abs`/`rand`/`srand`/`exit`/`qsort`/`bsearch`/`abort`/`strtol`/`strtod`/`llabs` |
| `<ctype.h>` | ✅ | 完整字符分类：含 `isdigit`/`isalpha`/`islower`/`isupper`/`tolower`/`toupper`/`isspace`/`isalnum`/`isprint`/`iscntrl`/`isxdigit`/`isgraph`/`ispunct`/`isblank` |
| `<math.h>` | ✅ | 完整教学 math：`sin`/`cos`/`tan`/`sqrt`/`pow`/`atan`/`log`/`log10`/`exp`/`fabs`/`ceil`/`floor`/`round`/`fmod`/`asin`/`acos`/`atan2`/`sinh`/`cosh`/`tanh`（`libm` `double` 精度） |
| `<string.h>` | ✅ | 完整字符串：含 `strlen`/`strcpy`/`strncpy`/`strcmp`/`strncmp`/`strcat`/`strncat`/`memcpy`/`memmove`/`memset`/`memcmp`/`strchr`/`strrchr`/`strstr`/`memchr`/`strdup`/`strerror`/`strpbrk`/`strspn`/`strcspn` |
| `<limits.h>` | ✅ | 空存根，宏由 Lexer 预定义（`INT_MAX`/`INT_MIN`/`LONG_MAX`/`LONG_MIN`/`CHAR_BIT`） |
| `<stdbool.h>` | ✅ | 存根声明已加载，`bool` typedef + `true`/`false` 宏 |
| `<stddef.h>` | ✅ | 存根声明已加载，`size_t`/`ptrdiff_t` typedef；`offsetof` 为编译器内置 |
| `<stdint.h>` | ✅ | 存根声明已加载，`int8_t`/`uint8_t`/`int16_t`/`uint16_t`/`int32_t`/`uint32_t`/`int64_t`/`uint64_t` typedef |
| `<time.h>` | ✅ | 存根声明已加载，`time_t`/`clock_t` typedef + `CLOCKS_PER_SEC` 宏；`time`/`clock` |
| `<assert.h>` | ✅ | 存根声明已加载，`assert` 宏展开为 `__cide_assert_fail` Host Func |
| `<errno.h>` | ✅ | 存根声明已加载，`extern int errno` + `EINVAL`/`ERANGE`/`EDOM`/`ENOENT`/`EACCES` 宏 |
| `<float.h>` | ✅ | 存根声明已加载，`FLT_MAX`/`DBL_MAX`/`FLT_EPSILON`/`DBL_EPSILON` 等宏 |

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
| `puts` | B | 硬编码 | ✅ | N/A | N/A | — |
| `sprintf` | B | 硬编码（变参） | ✅ | N/A | N/A | — |
| `snprintf` | B | 硬编码（变参） | ✅ | N/A | N/A | — |
| `sscanf` | B | 硬编码（变参） | ✅ | N/A | N/A | — |
| `fopen` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |
| `fclose` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |
| `fread` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |
| `fwrite` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |
| `fgets` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |
| `fputs` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |
| `feof` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |
| `ungetc` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |
| `fgetc` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |
| `fputc` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |
| `fseek` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |
| `ftell` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |
| `rewind` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |
| `fflush` | B | 硬编码 | N/A | N/A | N/A | VFS 内存模式为空操作 |
| `perror` | B | 硬编码 | N/A | N/A | N/A | 简化版：忽略 errno |
| `clearerr` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |
| `remove` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |
| `rename` | B | 硬编码 | N/A | N/A | N/A | VFS-backed |

### 2.2 stdlib.h

| 函数 | Layer | 类型检查 | Host Contract | Bytecode Consistency | Differential | 备注 |
|------|-------|----------|---------------|----------------------|--------------|------|
| `malloc` | B | 硬编码 | ✅ | N/A | N/A | UAF/泄漏检测 |
| `free` | B | 硬编码 | ✅ | N/A | N/A | UAF/Double-Free 检测 |
| `realloc` | B | 硬编码 | ✅ | N/A | N/A | — |
| `calloc` | B | 硬编码 | ✅ | N/A | N/A | — |
| `atoi` | B | 硬编码 | ✅ | ✅ | ✅ | — |
| `atof` | B | 硬编码 | ✅ | N/A | N/A | — |
| `atol` | B | 硬编码 | ✅ | N/A | N/A | — |
| `strtol` | B | 硬编码 | N/A | N/A | N/A | 支持 `endptr`，解析失败时设置 `errno=EINVAL` |
| `strtod` | B | 硬编码 | N/A | N/A | N/A | 支持 `endptr`，解析失败时设置 `errno=EINVAL` |
| `abs` | B | 存根声明 | ✅ | ✅ | ✅ | 已替代硬编码 |
| `llabs` | B | 硬编码 | N/A | N/A | N/A | `long long` 绝对值 |
| `rand` | B | 硬编码 | ✅ | ✅ | ✅ | — |
| `srand` | B | 硬编码 | ✅ | ✅ | ✅ | — |
| `exit` | B | 硬编码 | N/A | N/A | N/A | — |
| `abort` | B | 硬编码 | N/A | N/A | N/A | 终止并输出诊断 |
| `qsort` | B | 硬编码 | N/A | N/A | N/A | VM 回调敏感，未在存根中声明 |
| `bsearch` | B | 硬编码 | N/A | N/A | N/A | VM 回调敏感 |

### 2.3 ctype.h

| 函数 | Layer | 类型检查 | Host Contract | Bytecode Consistency | Differential | 备注 |
|------|-------|----------|---------------|----------------------|--------------|------|
| `isdigit` | B | 存根声明 | N/A | ✅ | ✅ | — |
| `isalpha` | B | 存根声明 | N/A | ✅ | ✅ | — |
| `islower` | B | 存根声明 | N/A | ✅ | ✅ | — |
| `isupper` | B | 存根声明 | N/A | ✅ | ✅ | — |
| `tolower` | B | 存根声明 | N/A | ✅ | ✅ | — |
| `toupper` | B | 存根声明 | N/A | ✅ | ✅ | — |
| `isspace` | B | 存根声明 | N/A | ✅ | ✅ | — |
| `isalnum` | B | 存根声明 | N/A | ✅ | ✅ | — |
| `isprint` | B | 存根声明 | N/A | ✅ | ✅ | — |
| `iscntrl` | B | 存根声明 | N/A | ✅ | ✅ | — |
| `isxdigit` | B | 存根声明 | N/A | ✅ | ✅ | — |
| `isgraph` | B | 存根声明 | N/A | N/A | N/A | — |
| `ispunct` | B | 存根声明 | N/A | N/A | N/A | — |
| `isblank` | B | 存根声明 | N/A | N/A | N/A | — |

### 2.4 math.h

| 函数 | Layer | 类型检查 | Host Contract | Bytecode Consistency | Differential | 备注 |
|------|-------|----------|---------------|----------------------|--------------|------|
| `sin` | B | 存根声明 | ✅ | N/A | N/A | `libm::sin`，`double` |
| `cos` | B | 存根声明 | ✅ | N/A | N/A | `libm::cos`，`double` |
| `tan` | B | 存根声明 | ✅ | N/A | N/A | `libm::tan`，`double` |
| `sqrt` | B | 存根声明 | ✅ | N/A | N/A | `libm::sqrt`，`double` |
| `pow` | B | 存根声明 | ✅ | N/A | N/A | `libm::pow`，`double` |
| `atan` | B | 存根声明 | ✅ | N/A | N/A | `libm::atan`，`double` |
| `log` | B | 存根声明 | ✅ | N/A | N/A | `libm::log`（自然对数），`double` |
| `log10` | B | 存根声明 | ✅ | N/A | N/A | `libm::log10`，`double` |
| `exp` | B | 存根声明 | ✅ | N/A | N/A | `libm::exp`，`double` |
| `fabs` | B | 存根声明 | ✅ | N/A | N/A | `libm::fabs`，`double` |
| `ceil` | B | 存根声明 | ✅ | N/A | N/A | `libm::ceil`，`double` |
| `floor` | B | 存根声明 | ✅ | N/A | N/A | `libm::floor`，`double` |
| `round` | B | 存根声明 | ✅ | N/A | N/A | `libm::round`，`double` |
| `fmod` | B | 存根声明 | ✅ | N/A | N/A | `libm::fmod`，`double` |
| `asin` | B | 存根声明 | ✅ | N/A | N/A | `libm::asin`，`double` |
| `acos` | B | 存根声明 | ✅ | N/A | N/A | `libm::acos`，`double` |
| `atan2` | B | 存根声明 | ✅ | N/A | N/A | `libm::atan2`，`double` |
| `sinh` | B | 存根声明 | ✅ | N/A | N/A | `libm::sinh`，`double` |
| `cosh` | B | 存根声明 | ✅ | N/A | N/A | `libm::cosh`，`double` |
| `tanh` | B | 存根声明 | ✅ | N/A | N/A | `libm::tanh`，`double` |

### 2.5 string.h

| 函数 | Layer | 类型检查 | Host Contract | Bytecode Consistency | Differential | 备注 |
|------|-------|----------|---------------|----------------------|--------------|------|
| `strlen` | C | 存根声明 | ✅ | ✅ | ✅ | 已切换为 Bytecode Libc 路径 |
| `strcpy` | B | 硬编码 | ✅ | ✅ | ✅ | E3070 Buffer Overflow 诊断 |
| `strncpy` | B | 硬编码 | ✅ | ✅ | ✅ | — |
| `strcmp` | C | 存根声明 | N/A | ✅ | ✅ | 已切换为 Bytecode Libc 路径 |
| `strncmp` | B | 硬编码 | N/A | N/A | N/A | — |
| `strcat` | B | 硬编码 | ✅ | ✅ | ✅ | E3070 Buffer Overflow 诊断 |
| `strncat` | B | 硬编码 | N/A | N/A | N/A | — |
| `memcpy` | B | 硬编码 | N/A | ✅ | ✅ | — |
| `memmove` | B | 硬编码 | N/A | ✅ | ✅ | — |
| `memset` | B | 硬编码 | ✅ | ✅ | ✅ | — |
| `memcmp` | B | 硬编码 | N/A | N/A | N/A | — |
| `strchr` | B | 硬编码 | N/A | N/A | N/A | — |
| `strrchr` | B | 硬编码 | N/A | N/A | N/A | — |
| `strstr` | B | 硬编码 | N/A | N/A | N/A | — |
| `memchr` | B | 硬编码 | N/A | N/A | N/A | — |
| `strdup` | B | 硬编码 | N/A | N/A | N/A | `malloc`+`strcpy`，内存追踪 |
| `strerror` | B | 硬编码 | N/A | N/A | N/A | 映射 5 种常见错误码 |
| `strpbrk` | B | 硬编码 | N/A | N/A | N/A | — |
| `strspn` | B | 硬编码 | N/A | N/A | N/A | — |
| `strcspn` | B | 硬编码 | N/A | N/A | N/A | — |

### 2.6 time.h

| 函数 | Layer | 类型检查 | Host Contract | Bytecode Consistency | Differential | 备注 |
|------|-------|----------|---------------|----------------------|--------------|------|
| `time` | B | 硬编码 | N/A | N/A | N/A | 返回 Unix 时间戳（秒） |
| `clock` | B | 硬编码 | N/A | N/A | N/A | 返回微秒级近似时钟 |

### 2.7 assert.h

| 宏/函数 | Layer | 类型检查 | Host Contract | Bytecode Consistency | Differential | 备注 |
|---------|-------|----------|---------------|----------------------|--------------|------|
| `assert` | B | 宏展开 | N/A | N/A | N/A | 展开为 `if (!(expr)) __cide_assert_fail()` |
| `__cide_assert_fail` | B | 硬编码 | N/A | N/A | N/A | 输出诊断并终止程序 |

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
| `ptrdiff_t` | `int` | 存根 `typedef` | `stddef.h` 中定义 |
| `time_t` | `long long` | 存根 `typedef` | `time.h` 中定义 |
| `clock_t` | `long long` | 存根 `typedef` | `time.h` 中定义 |
| `FILE` | `void*` | 存根 `typedef` | `stdio.h` 中定义 |
| `EXIT_SUCCESS` | `0` | Lexer 预定义宏 | `stdlib.h` |
| `EXIT_FAILURE` | `1` | Lexer 预定义宏 | `stdlib.h` |
| `RAND_MAX` | `32767` | Lexer 预定义宏 | `stdlib.h` |
| `SEEK_SET` | `0` | Lexer 预定义宏 | `stdio.h` |
| `SEEK_CUR` | `1` | Lexer 预定义宏 | `stdio.h` |
| `SEEK_END` | `2` | Lexer 预定义宏 | `stdio.h` |
| `INT_MAX` | `2147483647` | Lexer 预定义宏 | `limits.h` |
| `INT_MIN` | `-2147483648` | Lexer 预定义宏 | `limits.h` |
| `LONG_MAX` | `2147483647` | Lexer 预定义宏 | `limits.h` |
| `LONG_MIN` | `-2147483648` | Lexer 预定义宏 | `limits.h` |
| `CHAR_BIT` | `8` | Lexer 预定义宏 | `limits.h` |
| `true` | `1` | Lexer 预定义宏 | `stdbool.h` |
| `false` | `0` | Lexer 预定义宏 | `stdbool.h` |
| `CLOCKS_PER_SEC` | `1000000` | 存根宏 | `time.h` |
| `EINVAL` | `1` | 存根宏 | `errno.h` |
| `ERANGE` | `2` | 存根宏 | `errno.h` |
| `EDOM` | `3` | 存根宏 | `errno.h` |
| `ENOENT` | `4` | 存根宏 | `errno.h` |
| `EACCES` | `5` | 存根宏 | `errno.h` |
| `FLT_MAX` | `3.40282347e+38F` | 存根宏 | `float.h` |
| `DBL_MAX` | `1.7976931348623157e+308` | 存根宏 | `float.h` |
| `FLT_EPSILON` | `1.19209290e-7F` | 存根宏 | `float.h` |
| `DBL_EPSILON` | `2.2204460492503131e-16` | 存根宏 | `float.h` |

---

## 四、已解除的缺口

| 缺口 | 解除时间 | 说明 |
|------|----------|------|
| `math.h` 不支持 | 2026-06-07 | 引入 `libm`，注册 20 个数学函数 Host Func |
| `#include <stdio.h>` 被跳过 | 2026-06-07 | Lexer 加载存根，TypeChecker 识别声明 |
| `strdup` 不支持 | 2026-06-07 | 新增 `STRDUP` Host Func，复用 `allocate_raw` + `MemoryRegion` 追踪 |
| 大量函数缺存根声明 | 2026-06-07 | 补全全部 6 个已有头文件存根；新增 7 个存骨头文件 |
| P1 核心函数缺失 | 2026-06-07 | 完成 `abort`/`strtol`/`strtod`/`strerror`/`fflush`/`perror`/`clearerr`/`time`/`clock`/`assert`/`errno`/`remove`/`rename`/`strpbrk`/`strspn`/`strcspn` + ctype/math 补全 |

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
| `<stdlib.h>` | `qsort` | **Layer B** | ✅ 存根已添加 | LeetCode 排序、数据结构教材 |
| `<limits.h>` | `INT_MAX`, `INT_MIN`, `LONG_MAX`, `LONG_MIN`, `CHAR_BIT` | **无 Layer** | ✅ Lexer 预定义宏 + stub 头文件 | DP 初始化、边界判断 |
| `<stdbool.h>` | `bool`, `true`, `false` | **无 Layer** | ✅ stub typedef + Lexer 预定义宏 | 现代 C 代码标配 |
| `<stdlib.h>` | `EXIT_SUCCESS`, `EXIT_FAILURE`, `RAND_MAX` | **无 Layer** | ✅ Lexer 预定义宏 | 程序返回值、随机数范围 |
| `<stdio.h>` | `SEEK_SET`, `SEEK_CUR`, `SEEK_END` | **无 Layer** | ✅ Lexer 预定义宏 | `fseek` 定位 |
| `<stddef.h>` | `size_t`, `ptrdiff_t` | **无 Layer** | ✅ stub 头文件 + 编译器内置 `offsetof` | 标准类型 |
| `<stdint.h>` | `int32_t`, `uint64_t` 等 | **无 Layer** | ✅ stub typedef 头文件 | 精确宽度整数类型 |
| `<time.h>` | `time_t`, `clock_t`, `CLOCKS_PER_SEC` | **无 Layer** | ✅ stub 头文件 + 宏 | 时间/计时类型 |
| `<assert.h>` | `assert` 宏 | **无 Layer** | ✅ stub 头文件 + Host Func | 调试断言 |
| `<errno.h>` | `errno`, `EINVAL`/`ERANGE`/`EDOM`/`ENOENT`/`EACCES` | **无 Layer** | ✅ stub 头文件 + Host 设置 | 错误码处理 |
| `<float.h>` | `FLT_MAX`, `DBL_MAX`, `FLT_EPSILON`, `DBL_EPSILON` | **无 Layer** | ✅ stub 宏头文件 | 浮点边界 |

#### 5.2.2 新增函数（P0 核心）

| 头文件 | 函数 | 分层 | 学生场景 |
|--------|------|------|---------|
| `<stdio.h>` | `puts` | **Layer B** | ✅ 已实现 | 简单输出字符串+换行 |
| `<stdio.h>` | `sprintf`, `snprintf` | **Layer B** | ✅ 已实现 | 字符串构造、数字转字符串 |
| `<stdio.h>` | `sscanf` | **Layer B** | ✅ 已实现 | 字符串解析 |
| `<stdio.h>` | `fgetc`, `fputc` | **Layer B** | ✅ 已实现 | 单字符文件 I/O |
| `<stdio.h>` | `fseek`, `ftell`, `rewind` | **Layer B** | ✅ 已实现 | 文件定位 |
| `<stdio.h>` | `fflush`, `perror`, `clearerr` | **Layer B** | ✅ 已实现 | 文件错误处理 |
| `<stdio.h>` | `remove`, `rename` | **Layer B** | ✅ 已实现 | 文件删除/重命名 |
| `<stdlib.h>` | `calloc` | **Layer B** | ✅ 已实现 | 安全分配并零初始化（图论邻接表） |
| `<stdlib.h>` | `bsearch` | **Layer B** | ✅ 已实现 | 二分查找 |
| `<stdlib.h>` | `atof`, `atol` | **Layer B** | ✅ 已实现 | 字符串转数值 |
| `<stdlib.h>` | `abort` | **Layer B** | ✅ 已实现 | 异常终止 |
| `<stdlib.h>` | `strtol`, `strtod` | **Layer B** | ✅ 已实现 | 字符串转数值（支持 `endptr` + `errno`） |
| `<stdlib.h>` | `llabs` | **Layer B** | ✅ 已实现 | `long long` 绝对值 |
| `<string.h>` | `strchr`, `strrchr`, `strstr` | **Layer B** | ✅ 已实现 | 字符串搜索 |
| `<string.h>` | `strncmp`, `strncat` | **Layer B** | ✅ 已实现 | 定长字符串操作 |
| `<string.h>` | `memcmp`, `memchr` | **Layer B** | ✅ 已实现 | 内存比较/搜索 |
| `<string.h>` | `strerror` | **Layer B** | ✅ 已实现 | 错误字符串（映射 5 种错误码） |
| `<string.h>` | `strpbrk`, `strspn`, `strcspn` | **Layer B** | ✅ 已实现 | 字符串扫描 |
| `<math.h>` | `tan`, `log10`, `fabs`, `ceil`, `floor`, `round`, `fmod` | **Layer B** | ✅ 已实现 | 数学计算补全 |
| `<math.h>` | `asin`, `acos`, `atan2`, `sinh`, `cosh`, `tanh` | **Layer B** | ✅ 已实现 | 数学计算补全 |
| `<ctype.h>` | `isgraph`, `ispunct`, `isblank` | **Layer B** | ✅ 已实现 | 字符分类补全 |
| `<time.h>` | `time`, `clock` | **Layer B** | ✅ 已实现 | 系统时间/计时 |

---

### 5.3 🟠 P1 — 短期实现（教学/算法必备）

#### 5.3.1 已有头文件补全

| 头文件 | 函数 | 分层 | 分层依据 |
|--------|------|------|---------|
| `<string.h>` | `strtok` | **Layer C** | 纯计算但涉及全局静态状态；建议走 Bytecode Libc |

#### 5.3.2 新增头文件

| 头文件 | 内容 | 分层 | 实现方式 |
|--------|------|------|---------|
| `<assert.h>` | `assert` | **Layer B** | ✅ 已实现 | 宏展开为条件判断 + `__cide_assert_fail` Host Func |
| `<time.h>` | `time`, `clock` | **Layer B** | ✅ 已实现 | 系统调用封装 |
| `<time.h>` | `CLOCKS_PER_SEC` | **无 Layer** | ✅ 已实现 | 宏定义 |
| `<errno.h>` | `errno`, `EDOM`, `ERANGE`, `EINVAL`, `ENOENT`, `EACCES` | **无 Layer** | ✅ 已实现 | 全局变量 + Host 设置 |
| `<stdio.h>` | `fflush`, `perror`, `clearerr`, `remove`, `rename` | **Layer B** | ✅ 已实现 | VFS-backed |
| `<stddef.h>` | `offsetof` | **无 Layer** | ✅ 已实现 | 编译器内置（已支持） |
| `<float.h>` | `FLT_MAX`, `DBL_MAX`, `FLT_EPSILON`, `DBL_EPSILON` | **无 Layer** | ✅ 已实现 | 编译期常量宏 |

---

### 5.4 🟡 P2 — 中期实现（进阶补全）

> 以下项目经评估后**明确不支持**，已在排除项中登记。

| 头文件 | 内容 | 不支持理由 |
|--------|------|-----------|
| `<math.h>` | `INFINITY`, `NAN`, `HUGE_VAL` | 可用 `1e309` 或 `0.0/0.0` 替代；标准宏在不同平台值不同，教学不依赖 |
| `<string.h>` | `strtok` | 需全局静态状态，线程不安全；教学价值有限，学生可用 `strchr`/`strcspn` 自行实现 |

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
| `strtok` | 需全局静态状态，教学价值有限，可用 `strchr`/`strcspn` 自行实现 |
| `INFINITY` / `NAN` / `HUGE_VAL` | 可用 `1e309` 或 `0.0/0.0` 替代，教学使用频率极低 |

---

### 5.6 按分层统计的工作量估算

| 分层 | 新增项数 | 估算工作量 |
|------|---------|-----------|
| **无 Layer（宏/typedef）** | 0 | 全部完成 |
| **Layer C: Bytecode Libc** | `strtok` | 约 **1 个函数**，需全局静态状态支持 |
| **Layer B: Rust Host** | `INFINITY`/`NAN` 近似宏 | 约 **1 天** |
| **Layer A: VM Builtin** | 暂不接 codegen（待 profiling） | 0 |

**总计**：标准库教学子集已基本完整，剩余工作量 **< 1 天**。

---

### 5.7 推荐实施路线图

#### Phase A（第 1 周）：基础设施 + 立即可用 ✅
- [x] `qsort` 存根声明修复（1 行）
- [x] 新增 stub 头文件：`limits.h`, `stdbool.h`, `stddef.h`, `stdint.h`, `time.h`, `assert.h`, `errno.h`, `float.h`
- [x] 补全已有头文件宏：`EXIT_SUCCESS`, `EXIT_FAILURE`, `RAND_MAX`, `SEEK_SET/CUR/END`
- [x] `puts`, `sprintf`, `snprintf`, `sscanf`
- [x] `calloc`, `bsearch`
- [x] `fgetc`, `fputc`, `fseek`, `ftell`, `rewind`
- [x] `strchr`, `strrchr`, `strstr`, `strncmp`, `strncat`, `memcmp`, `memchr`
- [x] `atof`, `atol`
- [x] `tan`, `log10`, `fabs`, `ceil`, `floor`, `round`, `fmod`
- [x] `asin`, `acos`, `atan2`, `sinh`, `cosh`, `tanh`
- [x] `isgraph`, `ispunct`, `isblank`
- [x] `abort`, `strtol`, `strtod`, `llabs`
- [x] `fflush`, `perror`, `clearerr`, `remove`, `rename`
- [x] `strerror`, `strpbrk`, `strspn`, `strcspn`
- [x] `time`, `clock`
- [x] `assert` 宏
- [x] `errno` 全局变量 + Host 设置
- [x] `float.h` 常量宏

**目标**：LeetCode 中 90% 的 C 解法能编译通过。

#### Phase B（可选）：剩余补全
- [x] `strtok` → **明确不支持**：需全局静态状态，教学价值有限，学生可用 `strchr`/`strcspn` 自行实现
- [x] `INFINITY`, `NAN`, `HUGE_VAL` → **明确不支持**：可用 `1e309` 或 `0.0/0.0` 替代，教学使用频率极低

**目标**：K&R《C程序设计语言》全书示例代码 95% 能编译通过。

---

### 5.8 关键校正说明（与早期分析的差异）

| 函数 | 早期误标 | **严格分层后** | 原因 |
|------|---------|---------------|------|
| `bsearch` | Bytecode C | **Layer B** | VM 回调敏感，需调用用户比较函数 |
| `strncat` | Bytecode C | **Layer B** | 内存安全诊断敏感，需 Buffer Overflow 检测 |
| `strtol` / `strtod` | Bytecode C / Host B | **Layer B** | 涉及 `errno` + `endptr`，非纯计算 |
| `strtok` | Bytecode C | **Layer C** | 虽有全局状态但无诊断需求，走 C 可展示源码 |
| `strchr`/`strrchr`/`strstr`/`strncmp`/`memcmp`/`memchr` | Bytecode C（规划） | **Layer B** | 实际实现为 Host Func，保留灵活性 |

---

*文档状态：产品化进度追踪 + 下一阶段全面拓展蓝图*
*最后更新：2026-06-07*
