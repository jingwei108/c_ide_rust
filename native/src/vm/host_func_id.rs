//! Host Function ID 统一常量定义
//!
//! 编译期（bytecode_gen.rs）和运行期（host_funcs.rs）共用此映射，
//! 避免新增 host 函数时两处 ID 不一致。

pub const OUTPUT: u32 = 0;
pub const STEP: u32 = 1;
pub const MALLOC: u32 = 2;
pub const FREE: u32 = 3;
pub const PRINTF_0: u32 = 10;
pub const PRINTF_1: u32 = 11;
pub const PRINTF_2: u32 = 12;
pub const PRINTF_N: u32 = 15;
pub const SCANF_N: u32 = 21;
pub const STRLEN: u32 = 30;
pub const STRCPY: u32 = 31;
pub const STRCMP: u32 = 32;
pub const GETCHAR: u32 = 33;
pub const PUTCHAR: u32 = 34;
pub const RAND: u32 = 35;
pub const SRAND: u32 = 36;
pub const MEMSET: u32 = 37;
pub const EXIT: u32 = 38;
pub const STRCAT: u32 = 39;
pub const ATOI: u32 = 40;
pub const FPRINTF: u32 = 50;
pub const REALLOC: u32 = 51;
pub const QSORT: u32 = 52;
// 以下为预留扩展的文件 I/O 常量，当前沙盒中未实现。
// 若编译器错误引用将导致运行时 crash，请勿使用。
#[allow(dead_code)]
pub const FOPEN: u32 = 60;
#[allow(dead_code)]
pub const FREAD: u32 = 61;
#[allow(dead_code)]
pub const FWRITE: u32 = 62;
#[allow(dead_code)]
pub const FCLOSE: u32 = 63;
#[allow(dead_code)]
pub const FEOF: u32 = 64;
