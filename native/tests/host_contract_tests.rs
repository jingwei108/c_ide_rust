//! Host Function 契约测试（Phase A）
//!
//! 目标：验证 Layer B（Rust Host Func）的每个函数在边界条件、安全注入、标准一致性上是否达标。
//!
//! 测试哲学：
//! - NO_CODE_DISTORTION：不扭曲 C 语义去迎合 Cide。
//! - RECORD_DONT_HIDE：任何异常行为必须记录。
//! - FIX_REAL_BUGS：测试失败时，修 Host Func 的实现，而不是改测试预期值让它通过。

use cide_native::session::Session;
use cide_native::vm::vm::{CideVM, MEM_SIZE, NULL_TRAP_SIZE};
use cide_native::vm::host_funcs::{
    host_malloc, host_free, host_realloc, host_strlen, host_strcpy, host_strcmp,
    host_strcat, host_memset, host_atoi, host_printf_n, host_scanf_n,
    host_getchar, host_putchar, host_rand, host_srand,
    host_sin, host_cos, host_sqrt, host_pow, host_atan, host_log, host_exp,
};

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn fresh_session() -> (CideVM, Session) {
    (CideVM::new(), Session::default())
}

/// 在 VM 内存的合法区域写入一个 C 风格字符串，返回起始地址。
fn write_test_string(vm: &mut CideVM, addr: u32, s: &str) {
    vm.write_cstring(addr, s);
}

/// 从 VM 内存读取一个 C 风格字符串（遇到 \0 停止）。
fn read_test_string(vm: &CideVM, addr: u32) -> String {
    let mem = vm.memory_ref();
    let start = addr as usize;
    if start >= mem.len() {
        return String::new();
    }
    let bytes: Vec<u8> = mem[start..]
        .iter()
        .take_while(|&&b| b != 0)
        .copied()
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

// ─── malloc 契约 ─────────────────────────────────────────────────────────────

#[test]
fn test_malloc_zero_returns_null_with_warning() {
    let (mut vm, mut session) = fresh_session();
    vm.push(0);
    host_malloc(&mut vm, &mut session);
    let addr = vm.pop() as u32;
    assert_eq!(addr, 0, "malloc(0) 必须返回 NULL（0）");
    let warns: Vec<_> = session.runtime.output_lines.iter()
        .filter(|l| l.contains("malloc(0)"))
        .collect();
    assert!(!warns.is_empty(), "malloc(0) 必须输出警告说明其行为是实现定义的");
}

#[test]
fn test_malloc_negative_returns_null() {
    let (mut vm, mut session) = fresh_session();
    vm.push((-1i32) as u64);
    host_malloc(&mut vm, &mut session);
    let addr = vm.pop() as u32;
    assert_eq!(addr, 0, "malloc(负数) 必须返回 NULL");
}

#[test]
fn test_malloc_normal_returns_non_null() {
    let (mut vm, mut session) = fresh_session();
    vm.push(64);
    host_malloc(&mut vm, &mut session);
    let addr = vm.pop() as u32;
    assert!(addr >= NULL_TRAP_SIZE, "malloc(64) 必须返回非 NULL 的合法地址");
    assert!(addr < MEM_SIZE, "malloc(64) 返回的地址必须在 VM 内存范围内");
}

#[test]
fn test_malloc_records_region_metadata() {
    let (mut vm, mut session) = fresh_session();
    vm.push(100);
    host_malloc(&mut vm, &mut session);
    let addr = vm.pop() as u32;

    let region = session.memory.regions.iter()
        .find(|r| r.addr == addr && !r.is_freed)
        .expect("malloc 后必须在 session.memory.regions 中记录未释放的 MemoryRegion");
    assert_eq!(region.size, 100);
    assert!(region.is_heap);
    assert_eq!(region.alloc_by, "malloc");
}

#[test]
fn test_malloc_excessive_returns_null() {
    let (mut vm, mut session) = fresh_session();
    vm.push(MEM_SIZE as u64 + 1);
    host_malloc(&mut vm, &mut session);
    let addr = vm.pop() as u32;
    assert_eq!(addr, 0, "malloc(超过 VM 内存大小) 必须返回 NULL");
}

// ─── free 契约 ───────────────────────────────────────────────────────────────

#[test]
fn test_free_null_is_safe() {
    let (mut vm, mut session) = fresh_session();
    let regions_before = session.memory.regions.len();
    vm.push(0);
    host_free(&mut vm, &mut session);
    assert!(!vm.has_error(), "free(NULL) 必须是安全的，不得触发 trap");
    assert_eq!(session.memory.regions.len(), regions_before, "free(NULL) 不得新增或删除 region");
}

#[test]
fn test_free_valid_ptr_marks_freed() {
    let (mut vm, mut session) = fresh_session();
    vm.push(64);
    host_malloc(&mut vm, &mut session);
    let addr = vm.pop() as u32;

    vm.push(addr as u64);
    host_free(&mut vm, &mut session);

    let region = session.memory.regions.iter()
        .find(|r| r.addr == addr)
        .expect("free 后 region 必须仍然存在（用于诊断）");
    assert!(region.is_freed, "free 后 region 必须标记为 is_freed");
}

#[test]
fn test_free_already_freed_traps_double_free() {
    let (mut vm, mut session) = fresh_session();
    vm.push(64);
    host_malloc(&mut vm, &mut session);
    let addr = vm.pop() as u32;

    vm.push(addr as u64);
    host_free(&mut vm, &mut session);

    // 重置错误状态以便观察第二次 free
    // CideVM 没有公开重置 error 的方法，但 trap 只在 error.is_empty() 时写入
    // 由于第一次 free 没有 trap，error 为空，第二次 free 应该触发 Double-Free
    vm.push(addr as u64);
    host_free(&mut vm, &mut session);

    assert!(vm.has_error(), "Double-Free 必须触发 trap");
    let err = vm.get_error();
    assert!(
        err.contains("Double-Free") || err.contains("E3061"),
        "Double-Free 错误信息必须包含 'Double-Free' 或 'E3061'，实际: {}",
        err
    );
}

// ─── realloc 契约 ────────────────────────────────────────────────────────────

#[test]
fn test_realloc_null_equivalent_to_malloc() {
    let (mut vm, mut session) = fresh_session();
    vm.push(64); // new_size
    vm.push(0);  // ptr
    host_realloc(&mut vm, &mut session);
    let addr = vm.pop() as u32;
    assert!(addr >= NULL_TRAP_SIZE, "realloc(NULL, size) 必须等价于 malloc(size)，返回非 NULL");
}

#[test]
fn test_realloc_zero_equivalent_to_free() {
    let (mut vm, mut session) = fresh_session();
    // 先分配
    vm.push(64);
    host_malloc(&mut vm, &mut session);
    let addr = vm.pop() as u32;

    // realloc(ptr, 0)
    vm.push(0);      // new_size
    vm.push(addr as u64); // ptr
    host_realloc(&mut vm, &mut session);
    let new_addr = vm.pop() as u32;

    assert_eq!(new_addr, 0, "realloc(ptr, 0) 应返回 NULL");
    let region = session.memory.regions.iter().find(|r| r.addr == addr).unwrap();
    assert!(region.is_freed, "realloc(ptr, 0) 必须释放原内存");
}

#[test]
fn test_realloc_larger_copies_data() {
    let (mut vm, mut session) = fresh_session();
    vm.push(8);
    host_malloc(&mut vm, &mut session);
    let old_addr = vm.pop() as u32;

    // 写入数据 "ABCDEFG\0"
    write_test_string(&mut vm, old_addr, "ABCDEFG");

    // realloc 扩大到 64
    vm.push(64);
    vm.push(old_addr as u64);
    host_realloc(&mut vm, &mut session);
    let new_addr = vm.pop() as u32;

    assert!(new_addr >= NULL_TRAP_SIZE, "realloc 扩大必须返回合法地址");
    let copied = read_test_string(&vm, new_addr);
    assert_eq!(copied, "ABCDEFG", "realloc 扩大后必须拷贝旧数据");
}

// ─── strlen 契约 ─────────────────────────────────────────────────────────────

#[test]
fn test_strlen_normal() {
    let (mut vm, mut session) = fresh_session();
    let addr = 0x2000;
    write_test_string(&mut vm, addr, "hello");
    vm.push(addr as u64);
    host_strlen(&mut vm, &mut session);
    assert_eq!(vm.pop(), 5, "strlen(\"hello\") 必须为 5");
}

#[test]
fn test_strlen_empty_string() {
    let (mut vm, mut session) = fresh_session();
    let addr = 0x2000;
    write_test_string(&mut vm, addr, "");
    vm.push(addr as u64);
    host_strlen(&mut vm, &mut session);
    assert_eq!(vm.pop(), 0, "strlen(\"\") 必须为 0");
}

#[test]
fn test_strlen_null_address_returns_zero() {
    let (mut vm, mut session) = fresh_session();
    // VM 内存起始处为全 0，因此 read_cbytes(0) 返回空
    vm.push(0);
    host_strlen(&mut vm, &mut session);
    assert_eq!(vm.pop(), 0, "strlen(0) 在 Cide 中返回 0（VM 内存首字节为 0）");
}

// ─── strcpy 契约 ─────────────────────────────────────────────────────────────

#[test]
fn test_strcpy_normal_copy() {
    let (mut vm, mut session) = fresh_session();
    let src = 0x2000;
    let dst = 0x3000;
    write_test_string(&mut vm, src, "abc");
    vm.push(src as u64);
    vm.push(dst as u64);
    host_strcpy(&mut vm, &mut session);
    let result = read_test_string(&vm, dst);
    assert_eq!(result, "abc", "strcpy 必须正确拷贝字符串");
}

#[test]
fn test_strcpy_dest_at_high_boundary() {
    let (mut vm, mut session) = fresh_session();
    let src = 0x2000;
    let dst = MEM_SIZE - 4; // 靠近内存末尾
    write_test_string(&mut vm, src, "ab");
    vm.push(src as u64);
    vm.push(dst as u64);
    host_strcpy(&mut vm, &mut session);
    let result = read_test_string(&vm, dst);
    assert_eq!(result, "ab", "strcpy 在边界内必须正确拷贝");
}

// ─── strcmp 契约 ─────────────────────────────────────────────────────────────

#[test]
fn test_strcmp_equal() {
    let (mut vm, mut session) = fresh_session();
    let a = 0x2000;
    let b = 0x3000;
    write_test_string(&mut vm, a, "hello");
    write_test_string(&mut vm, b, "hello");
    vm.push(a as u64);
    vm.push(b as u64);
    host_strcmp(&mut vm, &mut session);
    assert_eq!(vm.pop() as i32, 0, "strcmp 相同字符串必须返回 0");
}

#[test]
fn test_strcmp_less() {
    let (mut vm, mut session) = fresh_session();
    let s1 = 0x2000;
    let s2 = 0x3000;
    write_test_string(&mut vm, s1, "abc");
    write_test_string(&mut vm, s2, "def");
    // VM 调用约定：从右到左入栈；host_strcmp 先 pop addr1(s1)，再 pop addr2(s2)
    vm.push(s2 as u64);
    vm.push(s1 as u64);
    host_strcmp(&mut vm, &mut session);
    assert!((vm.pop() as i32) < 0, "strcmp(\"abc\", \"def\") 必须返回负数");
}

#[test]
fn test_strcmp_greater() {
    let (mut vm, mut session) = fresh_session();
    let s1 = 0x2000;
    let s2 = 0x3000;
    write_test_string(&mut vm, s1, "xyz");
    write_test_string(&mut vm, s2, "abc");
    vm.push(s2 as u64);
    vm.push(s1 as u64);
    host_strcmp(&mut vm, &mut session);
    assert!((vm.pop() as i32) > 0, "strcmp(\"xyz\", \"abc\") 必须返回正数");
}

// ─── strcat 契约 ─────────────────────────────────────────────────────────────

#[test]
fn test_strcat_normal() {
    let (mut vm, mut session) = fresh_session();
    let dest = 0x2000;
    let src = 0x3000;
    write_test_string(&mut vm, dest, "hello");
    write_test_string(&mut vm, src, " world");
    vm.push(src as u64);
    vm.push(dest as u64);
    host_strcat(&mut vm, &mut session);
    let result = read_test_string(&vm, dest);
    assert_eq!(result, "hello world", "strcat 必须正确拼接");
}

// ─── memset 契约 ─────────────────────────────────────────────────────────────

#[test]
fn test_memset_normal() {
    let (mut vm, mut session) = fresh_session();
    let ptr = 0x2000;
    let val = 0x42;
    let size = 8u64;
    vm.push(size);
    vm.push(val);
    vm.push(ptr as u64);
    host_memset(&mut vm, &mut session);
    let mem = vm.memory_ref();
    for i in 0..8 {
        assert_eq!(mem[ptr as usize + i], val as u8, "memset 必须正确填充内存");
    }
}

#[test]
fn test_memset_returns_original_ptr() {
    let (mut vm, mut session) = fresh_session();
    let ptr = 0x2000u64;
    vm.push(4u64);
    vm.push(0u64);
    vm.push(ptr);
    host_memset(&mut vm, &mut session);
    assert_eq!(vm.pop(), ptr, "memset 必须返回原指针");
}

// ─── atoi 契约 ───────────────────────────────────────────────────────────────

#[test]
fn test_atoi_standard_conformance() {
    let (mut vm, mut session) = fresh_session();
    let addr = 0x2000;
    write_test_string(&mut vm, addr, "  -123abc");
    vm.push(addr as u64);
    host_atoi(&mut vm, &mut session);
    assert_eq!(vm.pop() as i32, -123, "atoi(\"  -123abc\") 必须返回 -123（C 标准行为）");
}

#[test]
fn test_atoi_empty_string() {
    let (mut vm, mut session) = fresh_session();
    let addr = 0x2000;
    write_test_string(&mut vm, addr, "");
    vm.push(addr as u64);
    host_atoi(&mut vm, &mut session);
    assert_eq!(vm.pop() as i32, 0, "atoi(\"\") 必须返回 0");
}

#[test]
fn test_atoi_no_digits() {
    let (mut vm, mut session) = fresh_session();
    let addr = 0x2000;
    write_test_string(&mut vm, addr, "abc");
    vm.push(addr as u64);
    host_atoi(&mut vm, &mut session);
    assert_eq!(vm.pop() as i32, 0, "atoi(\"abc\") 必须返回 0");
}

// ─── printf 契约 ─────────────────────────────────────────────────────────────

#[test]
fn test_printf_basic_string() {
    let (mut vm, mut session) = fresh_session();
    let fmt = 0x2000;
    write_test_string(&mut vm, fmt, "hello");
    vm.push(fmt as u64);
    host_printf_n(&mut vm, &mut session);
    assert_eq!(session.runtime.output_lines.last().unwrap(), "hello");
}

#[test]
fn test_printf_integer() {
    let (mut vm, mut session) = fresh_session();
    let fmt = 0x2000;
    write_test_string(&mut vm, fmt, "%d");
    vm.push(42u64);
    vm.push(fmt as u64);
    host_printf_n(&mut vm, &mut session);
    assert_eq!(session.runtime.output_lines.last().unwrap(), "42");
}

#[test]
fn test_printf_string_arg() {
    let (mut vm, mut session) = fresh_session();
    let fmt = 0x2000;
    let arg = 0x3000;
    write_test_string(&mut vm, fmt, "%s");
    write_test_string(&mut vm, arg, "world");
    vm.push(arg as u64);
    vm.push(fmt as u64);
    host_printf_n(&mut vm, &mut session);
    assert_eq!(session.runtime.output_lines.last().unwrap(), "world");
}

// ─── scanf 契约 ──────────────────────────────────────────────────────────────

#[test]
fn test_scanf_integer() {
    let (mut vm, mut session) = fresh_session();
    session.runtime.input_lines.push("42".to_string());
    let fmt = 0x2000;
    let dst = 0x3000;
    write_test_string(&mut vm, fmt, "%d");
    vm.push(dst as u64);
    vm.push(fmt as u64);
    host_scanf_n(&mut vm, &mut session);
    assert!(!session.runtime.waiting_input, "scanf 有输入时不应进入 waiting_input");
    let val = vm.load_i32(dst, &cide_native::vm::instruction::SourceLoc::default());
    assert_eq!(val, 42, "scanf(\"%%d\", ptr) 必须将 42 写入目标地址");
}

#[test]
fn test_scanf_multiple_integers() {
    let (mut vm, mut session) = fresh_session();
    session.runtime.input_lines.push("10 20".to_string());
    let fmt = 0x2000;
    let dst1 = 0x3000;
    let dst2 = 0x3004;
    write_test_string(&mut vm, fmt, "%d %d");
    vm.push(dst2 as u64);
    vm.push(dst1 as u64);
    vm.push(fmt as u64);
    host_scanf_n(&mut vm, &mut session);
    let v1 = vm.load_i32(dst1, &cide_native::vm::instruction::SourceLoc::default());
    let v2 = vm.load_i32(dst2, &cide_native::vm::instruction::SourceLoc::default());
    assert_eq!(v1, 10);
    assert_eq!(v2, 20);
}

// ─── getchar / putchar 契约 ──────────────────────────────────────────────────

#[test]
fn test_getchar_reads_from_input_lines() {
    let (mut vm, mut session) = fresh_session();
    session.runtime.input_lines.push("ab".to_string());
    host_getchar(&mut vm, &mut session);
    assert_eq!(vm.pop() as i32, 'a' as i32);
    host_getchar(&mut vm, &mut session);
    assert_eq!(vm.pop() as i32, 'b' as i32);
}

#[test]
fn test_putchar_outputs_char() {
    let (mut vm, mut session) = fresh_session();
    vm.push('X' as u64);
    host_putchar(&mut vm, &mut session);
    assert_eq!(session.runtime.output_lines.last().unwrap(), "X");
}

// ─── rand / srand 契约 ───────────────────────────────────────────────────────

#[test]
fn test_rand_deterministic_with_seed() {
    let (mut vm1, mut session1) = fresh_session();
    let (mut vm2, mut session2) = fresh_session();

    vm1.push(12345u64);
    host_srand(&mut vm1, &mut session1);

    vm2.push(12345u64);
    host_srand(&mut vm2, &mut session2);

    host_rand(&mut vm1, &mut session1);
    host_rand(&mut vm2, &mut session2);

    assert_eq!(vm1.pop(), vm2.pop(), "相同种子必须产生相同的 rand 序列");
}

#[test]
fn test_rand_returns_non_negative() {
    let (mut vm, mut session) = fresh_session();
    vm.push(1u64);
    host_srand(&mut vm, &mut session);
    host_rand(&mut vm, &mut session);
    let v = vm.pop() as i64;
    assert!(v >= 0, "rand() 返回值必须非负");
    assert!(v <= 0x7fff, "rand() 返回值必须 <= RAND_MAX (0x7fff)");
}

// ─── 边界安全契约（部分为 KNOWN_FAILURE，待修复 Host Func 实现） ─────────────

#[test]
fn test_memset_null_address_traps() {
    let (mut vm, mut session) = fresh_session();
    vm.push(1u64);
    vm.push(0x42u64);
    vm.push(0u64); // NULL ptr
    host_memset(&mut vm, &mut session);
    assert!(vm.has_error(), "memset(NULL, ..., non-zero) 必须触发 NULL trap");
}

#[test]
fn test_strcpy_null_dest_traps() {
    let (mut vm, mut session) = fresh_session();
    let src = 0x2000;
    write_test_string(&mut vm, src, "ab");
    vm.push(src as u64);
    vm.push(0u64); // NULL dest
    host_strcpy(&mut vm, &mut session);
    assert!(vm.has_error(), "strcpy(NULL, src) 必须触发 NULL trap");
}

#[test]
fn test_strcpy_overflow_must_trap() {
    let (mut vm, mut session) = fresh_session();
    // 分配一个 3 字节的堆缓冲区（含 \0 只能放 2 个字符）
    vm.push(3);
    host_malloc(&mut vm, &mut session);
    let dst = vm.pop() as u32;

    let src = 0x3000;
    write_test_string(&mut vm, src, "hello"); // 5 个字符 + \0 = 6 字节

    vm.push(src as u64);
    vm.push(dst as u64);
    host_strcpy(&mut vm, &mut session);
    assert!(
        vm.has_error(),
        "strcpy 越界必须触发 trap，而不是静默截断"
    );
    assert!(
        vm.get_error().contains("E3070"),
        "strcpy 溢出错误信息应包含 E3070"
    );
}

#[test]
fn test_strcat_overflow_must_trap() {
    let (mut vm, mut session) = fresh_session();
    // 分配一个 4 字节的堆缓冲区
    vm.push(4);
    host_malloc(&mut vm, &mut session);
    let dest = vm.pop() as u32;
    write_test_string(&mut vm, dest, "ab"); // 已有 2 字节 + \0 = 3 字节，只剩 1 字节空间

    let src = 0x3000;
    write_test_string(&mut vm, src, " world"); // 需要 6 字节 + \0

    vm.push(src as u64);
    vm.push(dest as u64);
    host_strcat(&mut vm, &mut session);
    assert!(
        vm.has_error(),
        "strcat 越界必须触发 trap，而不是静默截断"
    );
    assert!(
        vm.get_error().contains("E3070"),
        "strcat 溢出错误信息应包含 E3070"
    );
}

// ─── math.h 契约 ─────────────────────────────────────────────────────────────

const MATH_EPS: f64 = 1e-9;

#[test]
fn test_math_sin_zero() {
    let (mut vm, mut session) = fresh_session();
    vm.push(0.0f64.to_bits());
    host_sin(&mut vm, &mut session);
    let result = f64::from_bits(vm.pop());
    assert!(result.abs() < MATH_EPS, "sin(0.0) 应接近 0，实际 {}", result);
}

#[test]
fn test_math_cos_zero() {
    let (mut vm, mut session) = fresh_session();
    vm.push(0.0f64.to_bits());
    host_cos(&mut vm, &mut session);
    let result = f64::from_bits(vm.pop());
    assert!((result - 1.0).abs() < MATH_EPS, "cos(0.0) 应接近 1，实际 {}", result);
}

#[test]
fn test_math_sqrt_four() {
    let (mut vm, mut session) = fresh_session();
    vm.push(4.0f64.to_bits());
    host_sqrt(&mut vm, &mut session);
    let result = f64::from_bits(vm.pop());
    assert!((result - 2.0).abs() < MATH_EPS, "sqrt(4.0) 应接近 2，实际 {}", result);
}

#[test]
fn test_math_pow_two_three() {
    let (mut vm, mut session) = fresh_session();
    vm.push(3.0f64.to_bits()); // y (先压栈，后 pop)
    vm.push(2.0f64.to_bits()); // x
    host_pow(&mut vm, &mut session);
    let result = f64::from_bits(vm.pop());
    assert!((result - 8.0).abs() < MATH_EPS, "pow(2.0, 3.0) 应接近 8，实际 {}", result);
}

#[test]
fn test_math_atan_zero() {
    let (mut vm, mut session) = fresh_session();
    vm.push(0.0f64.to_bits());
    host_atan(&mut vm, &mut session);
    let result = f64::from_bits(vm.pop());
    assert!(result.abs() < MATH_EPS, "atan(0.0) 应接近 0，实际 {}", result);
}

#[test]
fn test_math_log_one() {
    let (mut vm, mut session) = fresh_session();
    vm.push(1.0f64.to_bits());
    host_log(&mut vm, &mut session);
    let result = f64::from_bits(vm.pop());
    assert!(result.abs() < MATH_EPS, "log(1.0) 应接近 0，实际 {}", result);
}

#[test]
fn test_math_exp_zero() {
    let (mut vm, mut session) = fresh_session();
    vm.push(0.0f64.to_bits());
    host_exp(&mut vm, &mut session);
    let result = f64::from_bits(vm.pop());
    assert!((result - 1.0).abs() < MATH_EPS, "exp(0.0) 应接近 1，实际 {}", result);
}

#[test]
fn test_math_sin_half_pi() {
    let (mut vm, mut session) = fresh_session();
    vm.push((std::f64::consts::PI / 2.0).to_bits());
    host_sin(&mut vm, &mut session);
    let result = f64::from_bits(vm.pop());
    assert!((result - 1.0).abs() < MATH_EPS, "sin(pi/2) 应接近 1，实际 {}", result);
}

#[test]
fn test_math_sqrt_negative_returns_nan() {
    let (mut vm, mut session) = fresh_session();
    vm.push((-1.0f64).to_bits());
    host_sqrt(&mut vm, &mut session);
    let result = f64::from_bits(vm.pop());
    assert!(result.is_nan(), "sqrt(-1.0) 应返回 NaN，实际 {}", result);
}

#[test]
fn test_math_log_zero_returns_neg_inf() {
    let (mut vm, mut session) = fresh_session();
    vm.push(0.0f64.to_bits());
    host_log(&mut vm, &mut session);
    let result = f64::from_bits(vm.pop());
    assert!(result.is_infinite() && result.is_sign_negative(), "log(0.0) 应返回 -inf，实际 {}", result);
}
