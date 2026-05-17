#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Cide 影子验证框架
对比 Clang 和 Cide 对同一份 C 代码的编译/运行结果
收集 Clang 通过但 Cide 失败的用例，按缺失特性分类统计
"""

import os
import sys
import io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

import json
import subprocess
import tempfile
import time
from pathlib import Path
from dataclasses import dataclass, asdict
from typing import Optional, List, Dict

# 路径配置
SCRIPT_DIR = Path(__file__).parent.resolve()
PROJECT_ROOT = SCRIPT_DIR.parent.parent.parent
NATIVE_DIR = PROJECT_ROOT / "native"
DLL_PATH = NATIVE_DIR / "target/release/cide_native.dll"
CLANG_PATH = "clang"


@dataclass
class RunResult:
    compiler: str
    compile_success: bool
    compile_error: str
    run_success: bool
    run_error: str
    stdout: str
    stderr: str
    exit_code: int
    duration_ms: float


@dataclass
class ShadowCase:
    name: str
    source: str
    category: str  # 预期分类，如 "double", "function_pointer", "file_io"


@dataclass
class ShadowDiff:
    case_name: str
    expected_category: str
    clang_result: RunResult
    cide_result: RunResult
    diff_type: str  # "compile_gap", "runtime_gap", "output_gap", "match"


def run_with_clang(source: str) -> RunResult:
    """用 Clang 编译并运行 C 代码"""
    start = time.time()
    # 为 Clang 添加标准头文件（Cide 有内置函数不需要）
    clang_source = CLANG_HEADER + source
    with tempfile.TemporaryDirectory() as tmpdir:
        c_file = Path(tmpdir) / "test.c"
        exe_file = Path(tmpdir) / "test.exe" if sys.platform == "win32" else Path(tmpdir) / "test"
        c_file.write_text(clang_source, encoding="utf-8")

        # 编译（Windows MSVC 环境下不需要 -lm，Linux/Android 需要）
        compile_cmd = [CLANG_PATH, str(c_file), "-o", str(exe_file), "-Wno-implicit-function-declaration"]
        if sys.platform != "win32":
            compile_cmd.append("-lm")
        try:
            compile_proc = subprocess.run(
                compile_cmd, capture_output=True, text=True, timeout=30
            )
        except Exception as e:
            return RunResult(
                compiler="clang",
                compile_success=False,
                compile_error=str(e),
                run_success=False,
                run_error="",
                stdout="",
                stderr="",
                exit_code=-1,
                duration_ms=(time.time() - start) * 1000,
            )

        if compile_proc.returncode != 0:
            return RunResult(
                compiler="clang",
                compile_success=False,
                compile_error=compile_proc.stderr,
                run_success=False,
                run_error="",
                stdout="",
                stderr=compile_proc.stderr,
                exit_code=compile_proc.returncode,
                duration_ms=(time.time() - start) * 1000,
            )

        # 运行
        try:
            run_proc = subprocess.run(
                [str(exe_file)], capture_output=True, text=True, timeout=5
            )
            return RunResult(
                compiler="clang",
                compile_success=True,
                compile_error="",
                run_success=run_proc.returncode == 0,
                run_error=run_proc.stderr if run_proc.returncode != 0 else "",
                stdout=run_proc.stdout,
                stderr=run_proc.stderr,
                exit_code=run_proc.returncode,
                duration_ms=(time.time() - start) * 1000,
            )
        except Exception as e:
            return RunResult(
                compiler="clang",
                compile_success=True,
                compile_error="",
                run_success=False,
                run_error=str(e),
                stdout="",
                stderr="",
                exit_code=-1,
                duration_ms=(time.time() - start) * 1000,
            )


def run_with_cide(source: str) -> RunResult:
    """通过 C API 调用 Cide 编译并运行"""
    import ctypes

    start = time.time()
    dll = ctypes.CDLL(str(DLL_PATH))

    # C API 函数签名
    dll.cide_session_create.restype = ctypes.c_void_p
    dll.cide_session_destroy.argtypes = [ctypes.c_void_p]
    dll.cide_compile.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    dll.cide_compile.restype = ctypes.c_int
    dll.cide_run.argtypes = [ctypes.c_void_p]
    dll.cide_run.restype = ctypes.c_int
    dll.cide_get_compile_errors.restype = ctypes.c_char_p
    dll.cide_get_compile_errors.argtypes = [ctypes.c_void_p]
    dll.cide_get_runtime_error.restype = ctypes.c_char_p
    dll.cide_get_runtime_error.argtypes = [ctypes.c_void_p]
    dll.cide_get_output_length.restype = ctypes.c_int
    dll.cide_get_output_length.argtypes = [ctypes.c_void_p]
    dll.cide_get_output.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_int]

    session = dll.cide_session_create()
    if not session:
        return RunResult(
            compiler="cide", compile_success=False, compile_error="session create failed",
            run_success=False, run_error="", stdout="", stderr="", exit_code=-1,
            duration_ms=(time.time() - start) * 1000,
        )

    try:
        compile_ret = dll.cide_compile(session, source.encode("utf-8"))
        if compile_ret != 0:
            err_ptr = dll.cide_get_compile_errors(session)
            err_msg = err_ptr.decode("utf-8", errors="replace") if err_ptr else "Unknown compile error"
            return RunResult(
                compiler="cide", compile_success=False, compile_error=err_msg,
                run_success=False, run_error="", stdout="", stderr=err_msg,
                exit_code=compile_ret, duration_ms=(time.time() - start) * 1000,
            )

        run_ret = dll.cide_run(session)

        out_len = dll.cide_get_output_length(session)
        stdout_str = ""
        if out_len > 0:
            buf = ctypes.create_string_buffer(out_len + 1)
            dll.cide_get_output(session, buf, out_len + 1)
            stdout_str = buf.value.decode("utf-8", errors="replace")
            # 清理 Cide 的额外输出后缀（如 "程序运行完成，返回值：0"）
            import re
            stdout_str = re.sub(r'程序运行完成，返回值：-?\d+\n?', '', stdout_str)
            stdout_str = stdout_str.strip()

        err_ptr = dll.cide_get_runtime_error(session)
        runtime_err = err_ptr.decode("utf-8", errors="replace") if err_ptr else ""

        return RunResult(
            compiler="cide", compile_success=True, compile_error="",
            run_success=run_ret == 0 and not runtime_err,
            run_error=runtime_err, stdout=stdout_str, stderr=runtime_err,
            exit_code=run_ret, duration_ms=(time.time() - start) * 1000,
        )
    finally:
        dll.cide_session_destroy(session)


def classify_compile_error(error_msg: str, expected_category: str = "unknown") -> str:
    """根据 Cide 编译错误消息分类缺失特性
    
    优先使用用例本身的 expected_category（如果已知且不是 baseline），
    再用错误信息关键词作为 fallback 分类。
    """
    # 如果用例已经标注了明确的缺失特性分类，优先使用
    if expected_category and expected_category not in ("baseline", "unknown"):
        return expected_category
    
    err_lower = error_msg.lower()
    patterns = {
        "double": ["double"],
        "function_pointer": ["function pointer", "expected identifier"],
        "file_io": ["fopen", "fclose", "fread", "fwrite", "fprintf", "stdin", "stdout"],
        "preprocessor": ["#include", "#ifdef", "#ifndef", "#pragma"],
        "union": ["union"],
        "bitfield": ["bitfield"],
        "goto": ["goto"],
        "switch_fallthrough": ["fallthrough"],
        "inline_asm": ["asm", "__asm__"],
        "complex_number": ["complex", "_Complex"],
        "long_long": ["long long"],
        "variadic_macro": ["...", "__VA_ARGS__"],
        "typeof": ["typeof"],
        "static_assert": ["static_assert"],
        "designated_initializer": ["designated"],
        "variable_length_array": ["vla", "variable length"],
        "missing_header": ["stdio.h", "stdlib.h", "string.h", "math.h"],
        "const_string": ["char*", "const"],
    }
    for category, keywords in patterns.items():
        for kw in keywords:
            if kw in err_lower:
                return category
    return "unknown"


def analyze_diff(case: ShadowCase, clang_res: RunResult, cide_res: RunResult) -> ShadowDiff:
    """分析 Clang 和 Cide 的差异"""
    if clang_res.compile_success and not cide_res.compile_success:
        diff_type = "compile_gap"
    elif clang_res.compile_success and cide_res.compile_success:
        if not clang_res.run_success and not cide_res.run_success:
            diff_type = "match"  # 都失败
        elif clang_res.run_success and not cide_res.run_success:
            diff_type = "runtime_gap"
        elif clang_res.stdout.strip() != cide_res.stdout.strip():
            # 已知问题（预期行为差异）不统计为 output_gap
            if "bug" in case.category:
                diff_type = "known_issue"
            else:
                diff_type = "output_gap"
        else:
            diff_type = "match"
    elif not clang_res.compile_success and not cide_res.compile_success:
        diff_type = "match"  # 都编译失败（可能是用例本身有问题）
    else:
        diff_type = "cide_better"  # Cide 通过但 Clang 失败（罕见）

    return ShadowDiff(
        case_name=case.name,
        expected_category=case.category,
        clang_result=clang_res,
        cide_result=cide_res,
        diff_type=diff_type,
    )


def generate_report(diffs: List[ShadowDiff], output_path: Path):
    """生成分类统计报告"""
    # 统计各类差异
    compile_gaps = [d for d in diffs if d.diff_type == "compile_gap"]
    runtime_gaps = [d for d in diffs if d.diff_type == "runtime_gap"]
    output_gaps = [d for d in diffs if d.diff_type == "output_gap"]
    matches = [d for d in diffs if d.diff_type == "match"]

    # 编译缺口按分类统计
    category_counts: Dict[str, int] = {}
    category_cases: Dict[str, List[str]] = {}
    for d in compile_gaps:
        cat = classify_compile_error(d.cide_result.compile_error, d.expected_category)
        category_counts[cat] = category_counts.get(cat, 0) + 1
        if cat not in category_cases:
            category_cases[cat] = []
        category_cases[cat].append(d.case_name)

    sorted_categories = sorted(category_counts.items(), key=lambda x: x[1], reverse=True)

    report_lines = [
        "# Cide 影子验证报告",
        f"\n生成时间: {time.strftime('%Y-%m-%d %H:%M:%S')}",
        f"总用例数: {len(diffs)}",
        f"完全匹配: {len(matches)} ({len(matches)*100//len(diffs)}%)",
        f"编译缺口: {len(compile_gaps)} ({len(compile_gaps)*100//len(diffs)}%)",
        f"运行时缺口: {len(runtime_gaps)}",
        f"输出差异: {len(output_gaps)}",
        "\n## 缺失特性频率排序（编译缺口）\n",
    ]

    for cat, count in sorted_categories:
        pct = count * 100 // len(diffs)
        report_lines.append(f"- **{cat}**: {count} 个用例 ({pct}%) — 示例: {', '.join(category_cases[cat][:3])}")

    report_lines.append("\n## 详细差异\n")
    for d in diffs:
        if d.diff_type != "match":
            report_lines.append(f"\n### {d.case_name} [{d.diff_type}]")
            report_lines.append(f"- 预期分类: {d.expected_category}")
            report_lines.append(f"- Clang: compile={'OK' if d.clang_result.compile_success else 'FAIL'}, run={'OK' if d.clang_result.run_success else 'FAIL'}")
            report_lines.append(f"- Cide: compile={'OK' if d.cide_result.compile_success else 'FAIL'}, run={'OK' if d.cide_result.run_success else 'FAIL'}")
            if not d.cide_result.compile_success:
                report_lines.append(f"- Cide 编译错误: {d.cide_result.compile_error[:200]}")
            elif d.clang_result.stdout.strip() != d.cide_result.stdout.strip():
                report_lines.append(f"- Clang stdout: {d.clang_result.stdout.strip()[:200]}")
                report_lines.append(f"- Cide stdout: {d.cide_result.stdout.strip()[:200]}")

    output_path.write_text("\n".join(report_lines), encoding="utf-8")
    print(f"报告已生成: {output_path}")


# 为 Clang 添加标准头文件前缀
CLANG_HEADER = '#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n\n'

# ===== 测试用例库 =====
SHADOW_CASES: List[ShadowCase] = [
    # ===== 当前应支持的特性（验证基准） =====
    ShadowCase("hello_world", 'int main() { printf("Hello\\n"); return 0; }', "baseline"),
    ShadowCase("int_arith", 'int main() { printf("%d", 1+2); return 0; }', "baseline"),
    ShadowCase("float_basic", 'int main() { float f = 3.14; printf("%.2f", f); return 0; }', "baseline"),
    ShadowCase("array_index", 'int main() { int a[3] = {1,2,3}; printf("%d", a[1]); return 0; }', "baseline"),
    ShadowCase("pointer_deref", 'int main() { int x = 5; int* p = &x; printf("%d", *p); return 0; }', "baseline"),
    ShadowCase("struct_basic", 'struct S { int x; }; int main() { struct S s; s.x = 42; printf("%d", s.x); return 0; }', "baseline"),
    ShadowCase("if_else", 'int main() { int x = 5; if (x > 3) printf("yes"); else printf("no"); return 0; }', "baseline"),
    ShadowCase("for_loop", 'int main() { for (int i = 0; i < 3; i++) printf("%d", i); return 0; }', "baseline"),
    ShadowCase("while_loop", 'int main() { int i = 0; while (i < 3) { printf("%d", i); i++; } return 0; }', "baseline"),
    ShadowCase("function_call", 'int add(int a, int b) { return a+b; } int main() { printf("%d", add(1,2)); return 0; }', "baseline"),
    ShadowCase("malloc_free", 'int main() { int* p = malloc(4); *p = 42; printf("%d", *p); free(p); return 0; }', "baseline"),
    ShadowCase("scanf_printf", 'int main() { int x = 42; printf("%d", x); return 0; }', "baseline"),
    ShadowCase("typedef", 'typedef int Integer; int main() { Integer a = 5; printf("%d", a); return 0; }', "baseline"),
    ShadowCase("enum", 'enum Color { RED, GREEN }; int main() { enum Color c = GREEN; printf("%d", c); return 0; }', "baseline"),
    ShadowCase("sizeof", 'int main() { printf("%d", sizeof(int)); return 0; }', "baseline"),
    ShadowCase("ternary", 'int main() { int x = 5 > 3 ? 1 : 0; printf("%d", x); return 0; }', "baseline"),
    ShadowCase("bitwise", 'int main() { printf("%d", 5 & 3); return 0; }', "baseline"),
    ShadowCase("compound_assign", 'int main() { int a = 5; a += 3; printf("%d", a); return 0; }', "baseline"),
    ShadowCase("string_literal", 'int main() { char* s = "hello"; printf("%s", s); return 0; }', "baseline"),
    ShadowCase("char_array_init", 'int main() { char s[6] = "hello"; printf("%s", s); return 0; }', "baseline"),
    ShadowCase("multi_dim_array", 'int main() { int a[2][2] = {{1,2},{3,4}}; printf("%d", a[1][1]); return 0; }', "baseline"),
    ShadowCase("pointer_arith", 'int main() { int arr[3] = {10,20,30}; int* p = arr; p++; printf("%d", *p); return 0; }', "baseline"),
    ShadowCase("const", 'int main() { const int MAX = 100; printf("%d", MAX); return 0; }', "baseline"),
    ShadowCase("null_keyword", 'int main() { int* p = NULL; if (p == NULL) printf("null"); return 0; }', "baseline"),
    ShadowCase("qsort", 'int cmp(const void* a, const void* b) { return *(int*)a - *(int*)b; } int main() { int a[3] = {3,1,2}; qsort(a, 3, 4, cmp); printf("%d", a[0]); return 0; }', "baseline"),
    ShadowCase("realloc", 'int main() { int* p = malloc(4); *p = 1; p = realloc(p, 8); printf("%d", *p); free(p); return 0; }', "baseline"),
    ShadowCase("forward_decl", 'int foo(int); int main() { printf("%d", foo(5)); return 0; } int foo(int x) { return x*2; }', "baseline"),
    ShadowCase("switch_case", 'int main() { int x = 2; switch(x) { case 1: printf("one"); break; case 2: printf("two"); break; default: printf("other"); } return 0; }', "baseline"),
    ShadowCase("do_while", 'int main() { int i = 0; do { printf("%d", i); i++; } while (i < 3); return 0; }', "baseline"),
    ShadowCase("struct_pointer_access", 'struct S { int x; }; int main() { struct S s; struct S* p = &s; p->x = 42; printf("%d", p->x); return 0; }', "baseline"),
    ShadowCase("typedef_struct_anon", 'typedef struct { int x; } Point; int main() { Point p; p.x = 5; printf("%d", p.x); return 0; }', "baseline"),
    ShadowCase("recursive_function", 'int fact(int n) { if (n <= 1) return 1; return n * fact(n-1); } int main() { printf("%d", fact(5)); return 0; }', "baseline"),
    ShadowCase("nested_loop", 'int main() { for (int i = 0; i < 2; i++) for (int j = 0; j < 2; j++) printf("%d", i+j); return 0; }', "baseline"),
    ShadowCase("break_continue", 'int main() { for (int i = 0; i < 5; i++) { if (i == 1) continue; if (i == 3) break; printf("%d", i); } return 0; }', "baseline"),
    ShadowCase("sizeof_basic_types", 'int main() { printf("%d %d %d", sizeof(int), sizeof(char), sizeof(double)); return 0; }', "baseline"),
    ShadowCase("string_strlen", 'int main() { char* s = "hello"; printf("%d", strlen(s)); return 0; }', "baseline"),
    ShadowCase("string_strcpy", 'int main() { char d[10]; strcpy(d, "hi"); printf("%s", d); return 0; }', "baseline"),
    ShadowCase("string_strcmp", 'int main() { char* a = "abc"; char* b = "abd"; printf("%d", strcmp(a,b)); return 0; }', "baseline"),
    ShadowCase("fprintf_basic", 'int main() { fprintf(stdout, "hello"); return 0; }', "baseline"),
    ShadowCase("long_type", 'int main() { long x = 123456; printf("%ld", x); return 0; }', "baseline"),
    ShadowCase("memset_basic", 'int main() { int a[5]; memset(a, 0, 20); printf("%d", a[0]); return 0; }', "baseline"),
    ShadowCase("cast_void_pointer", 'int main() { int x = 5; void* p = (void*)&x; int* q = (int*)p; printf("%d", *q); return 0; }', "baseline"),
    ShadowCase("cast_char_pointer", 'int main() { int arr[3] = {10,20,30}; char* p = (char*)arr; printf("%d", p[4]); return 0; }', "baseline"),
    ShadowCase("cast_float_to_int", 'int main() { float f = 3.7; int a = (int)f; printf("%d", a); return 0; }', "baseline"),
    ShadowCase("multi_var_decl", 'int main() { int a = 1, b = 2; printf("%d %d", a, b); return 0; }', "baseline"),
    ShadowCase("array_index_increment", 'int main() { int a[3] = {1,2,3}; a[1]++; printf("%d", a[1]); return 0; }', "baseline"),
    ShadowCase("array_address", 'int main() { int a[3] = {1,2,3}; int* p = &a[1]; printf("%d", *p); return 0; }', "baseline"),
    ShadowCase("compound_assign_array", 'int main() { int a[3] = {1,2,3}; a[1] += 5; printf("%d", a[1]); return 0; }', "baseline"),
    ShadowCase("compound_assign_deref", 'int main() { int x = 5; int* p = &x; *p += 3; printf("%d", x); return 0; }', "baseline"),
    ShadowCase("compound_assign_struct", 'struct S { int x; }; int main() { struct S s; s.x = 10; s.x += 5; printf("%d", s.x); return 0; }', "baseline"),
    ShadowCase("global_struct_member", 'struct S { int x; }; struct S gs; int main() { gs.x = 7; printf("%d", gs.x); return 0; }', "baseline"),
    ShadowCase("exit_function", 'int main() { printf("before"); exit(0); printf("after"); return 0; }', "baseline"),
    ShadowCase("atoi_function", 'int main() { printf("%d", atoi("42")); return 0; }', "baseline"),
    ShadowCase("strcat_function", 'int main() { char a[20] = "hello"; strcat(a, " world"); printf("%s", a); return 0; }', "baseline"),
    ShadowCase("putchar_function", 'int main() { putchar(\'A\'); return 0; }', "baseline"),
    ShadowCase("srand_rand", 'int main() { srand(1); int a = rand(); srand(1); int b = rand(); printf("%d", a == b); return 0; }', "baseline"),
    ShadowCase("hex_literal", 'int main() { int x = 0xFF; printf("%d", x); return 0; }', "baseline"),
    ShadowCase("char_literal", 'int main() { char c = \'A\'; printf("%d", c); return 0; }', "baseline"),
    ShadowCase("block_comment", 'int main() { /* comment */ printf("ok"); return 0; }', "baseline"),
    ShadowCase("short_type", 'int main() { short s = 100; printf("%d", s); return 0; }', "baseline"),
    ShadowCase("unsigned_type", 'int main() { unsigned int u = 5; printf("%d", u); return 0; }', "baseline"),
    ShadowCase("signed_keyword", 'int main() { signed int x = -5; printf("%d", x); return 0; }', "baseline"),
    ShadowCase("pre_increment", 'int main() { int a = 5; printf("%d", ++a); return 0; }', "baseline"),
    ShadowCase("pre_decrement", 'int main() { int a = 5; printf("%d", --a); return 0; }', "baseline"),
    ShadowCase("logical_and", 'int main() { int a = 1, b = 0; printf("%d", a && b); return 0; }', "baseline"),
    ShadowCase("logical_or", 'int main() { int a = 1, b = 0; printf("%d", a || b); return 0; }', "baseline"),
    ShadowCase("logical_not", 'int main() { int a = 0; printf("%d", !a); return 0; }', "baseline"),
    ShadowCase("modulo", 'int main() { printf("%d", 17 % 5); return 0; }', "baseline"),
    ShadowCase("negative_number", 'int main() { int x = -42; printf("%d", x); return 0; }', "baseline"),
    ShadowCase("sizeof_expr", 'int main() { printf("%d", sizeof(1+2)); return 0; }', "baseline"),
    ShadowCase("func_return_as_arg", 'int add(int a, int b) { return a+b; } int main() { printf("%d", add(1, add(2,3))); return 0; }', "baseline"),
    ShadowCase("void_func", 'void f() { printf("ok"); } int main() { f(); return 0; }', "baseline"),
    ShadowCase("ptr_comparison", 'int main() { int a[3]; int* p = &a[1]; int* q = &a[2]; printf("%d", p < q); return 0; }', "baseline"),
    ShadowCase("while_true_break", 'int main() { int i = 0; while (1) { if (i >= 3) break; printf("%d", i); i++; } return 0; }', "baseline"),
    ShadowCase("float_compare", 'int main() { float f = 3.14; printf("%d", f > 3.0); return 0; }', "baseline"),
    ShadowCase("struct_assign", 'struct S { int x; }; int main() { struct S a; a.x = 1; struct S b = a; printf("%d", b.x); return 0; }', "baseline"),
    ShadowCase("array_param_decay", 'int sum(int a[], int n) { int s = 0; for (int i = 0; i < n; i++) s += a[i]; return s; } int main() { int arr[3] = {1,2,3}; printf("%d", sum(arr, 3)); return 0; }', "baseline"),
    ShadowCase("enum_arith", 'enum Color { RED, GREEN }; int main() { printf("%d", GREEN + 1); return 0; }', "baseline"),
    ShadowCase("sizeof_struct_union", 'struct S { int a; }; union U { int i; float f; }; int main() { printf("%d %d", sizeof(struct S), sizeof(union U)); return 0; }', "baseline"),
    ShadowCase("not_equal", 'int main() { printf("%d", 5 != 3); return 0; }', "baseline"),
    ShadowCase("nested_call", 'int add(int a, int b) { return a+b; } int main() { printf("%d", add(add(1,2), add(3,4))); return 0; }', "baseline"),
    ShadowCase("ptr_diff", 'int main() { int a[5]; int* p = &a[1]; int* q = &a[3]; printf("%d", q - p); return 0; }', "baseline"),
    ShadowCase("scanf_float", 'int main() { float f = 3.14; printf("%.2f", f); return 0; }', "baseline"),
    ShadowCase("typedef_pointer", 'typedef int* IntPtr; int main() { int x = 5; IntPtr p = &x; printf("%d", *p); return 0; }', "baseline"),
    ShadowCase("bitwise_not", 'int main() { printf("%d", ~5); return 0; }', "baseline"),
    ShadowCase("bitwise_shift", 'int main() { printf("%d %d", 1 << 3, 16 >> 2); return 0; }', "baseline"),
    ShadowCase("union_array", 'union U { int i; float f; }; int main() { union U arr[2]; arr[0].i = 42; printf("%d", arr[0].i); return 0; }', "baseline"),
    ShadowCase("for_empty_cond", 'int main() { int i = 0; for (;;) { if (i >= 3) break; printf("%d", i); i++; } return 0; }', "baseline"),
    ShadowCase("switch_break", 'int main() { int x = 1; switch(x) { case 1: printf("one"); break; case 2: printf("two"); break; } return 0; }', "baseline"),
    ShadowCase("continue_while", 'int main() { int i = 0; while (i < 5) { i++; if (i == 2) continue; if (i == 4) break; printf("%d", i); } return 0; }', "baseline"),
    ShadowCase("ternary_nested", 'int main() { int a = 5, b = 3, c = 4; int m = a > b ? (a > c ? a : c) : b; printf("%d", m); return 0; }', "baseline"),
    ShadowCase("neg_hex", 'int main() { int x = -0xFF; printf("%d", x); return 0; }', "baseline"),
    ShadowCase("octal_literal", 'int main() { int x = 077; printf("%d", x); return 0; }', "baseline"),
    ShadowCase("partial_array_init", 'int main() { int a[5] = {1, 2}; printf("%d", a[4]); return 0; }', "baseline"),
    ShadowCase("char_arr_no_size", 'int main() { char s[] = "hello"; printf("%s", s); return 0; }', "baseline"),
    ShadowCase("array_param_with_size", 'int sum(int a[3]) { return a[0]+a[1]+a[2]; } int main() { int arr[3] = {1,2,3}; printf("%d", sum(arr)); return 0; }', "baseline"),
    ShadowCase("sizeof_var", 'int main() { int x; printf("%d", sizeof(x)); return 0; }', "baseline"),
    ShadowCase("do_while_nested", 'int main() { int i = 0; do { int j = 0; do { printf("%d", i+j); j++; } while (j < 2); i++; } while (i < 2); return 0; }', "baseline"),
    ShadowCase("escape_tab", 'int main() { printf("a\\tb"); return 0; }', "baseline"),
    ShadowCase("void_ptr_malloc", 'int main() { int* p = (int*)malloc(4); *p = 42; printf("%d", *p); free(p); return 0; }', "baseline"),
    ShadowCase("union_basic", 'union U { int i; float f; }; int main() { union U u; u.i = 1; printf("%d", u.i); return 0; }', "baseline"),
    ShadowCase("long_long", 'int main() { long long ll = 9223372036854775807LL; printf("%lld", ll); return 0; }', "baseline"),

    ShadowCase("multi_if_else", 'int main() { int x = 2; if (x == 1) printf("one"); else if (x == 2) printf("two"); else printf("other"); return 0; }', "baseline"),
    ShadowCase("if_nested", 'int main() { int a = 1, b = 2; if (a == 1) { if (b == 2) printf("yes"); } return 0; }', "baseline"),
    ShadowCase("switch_default_first", 'int main() { int x = 5; switch(x) { default: printf("def"); break; case 1: printf("one"); break; } return 0; }', "baseline"),
    ShadowCase("for_no_init", 'int main() { int i = 0; for (; i < 3; i++) printf("%d", i); return 0; }', "baseline"),
    ShadowCase("for_no_incr", 'int main() { int i = 0; for (; i < 3; ) { printf("%d", i); i++; } return 0; }', "baseline"),
    ShadowCase("while_nested", 'int main() { int i = 0; while (i < 2) { int j = 0; while (j < 2) { printf("%d", i+j); j++; } i++; } return 0; }', "baseline"),
    ShadowCase("do_while_continue", 'int main() { int i = 0; do { i++; if (i == 2) continue; printf("%d", i); } while (i < 4); return 0; }', "baseline"),
    ShadowCase("break_in_nested_loop", 'int main() { for (int i = 0; i < 3; i++) { for (int j = 0; j < 3; j++) { if (j == 1) break; printf("%d", i+j); } } return 0; }', "baseline"),
    ShadowCase("short_circuit_and", 'int f() { printf("call"); return 1; } int main() { int x = 0; if (x && f()) printf("yes"); printf("done"); return 0; }', "baseline"),
    ShadowCase("short_circuit_or", 'int f() { printf("call"); return 1; } int main() { int x = 1; if (x || f()) printf("yes"); printf("done"); return 0; }', "baseline"),
    ShadowCase("ternary_as_expr", 'int main() { int a = 5, b = 3; int m = (a > b ? a : b) + 1; printf("%d", m); return 0; }', "baseline"),
    ShadowCase("assign_chain", 'int main() { int a, b, c; a = b = c = 5; printf("%d %d %d", a, b, c); return 0; }', "baseline"),
    ShadowCase("precedence", 'int main() { printf("%d", 2 + 3 * 4); return 0; }', "baseline"),
    ShadowCase("parentheses_priority", 'int main() { printf("%d", (2 + 3) * 4); return 0; }', "baseline"),
    ShadowCase("post_inc_vs_pre_inc", 'int main() { int i = 5; printf("%d", i++); printf("%d", ++i); return 0; }', "baseline"),
    ShadowCase("neg_modulo", 'int main() { printf("%d", -17 % 5); return 0; }', "baseline"),
    ShadowCase("bitwise_combo", 'int main() { printf("%d", ((5 & 3) | 6) ^ 1); return 0; }', "baseline"),
    ShadowCase("shift_zero", 'int main() { printf("%d", 1 << 0); return 0; }', "baseline"),
    ShadowCase("shift_31", 'int main() { printf("%d", 1 << 31); return 0; }', "baseline"),
    ShadowCase("not_zero", 'int main() { printf("%d", ~0); return 0; }', "baseline"),
    ShadowCase("double_compare", 'int main() { double d = 3.14; printf("%d", d > 3.0); return 0; }', "baseline"),
    ShadowCase("double_array_op", 'int main() { double arr[2] = {1.5, 2.5}; printf("%.1f", arr[0] + arr[1]); return 0; }', "baseline"),
    ShadowCase("enum_non_continuous", 'enum E { A = 10, B = 20 }; int main() { printf("%d", B); return 0; }', "baseline"),
    ShadowCase("sizeof_typedef", 'typedef int Integer; int main() { printf("%d", sizeof(Integer)); return 0; }', "baseline"),
    ShadowCase("sizeof_enum", 'enum Color { R, G }; int main() { printf("%d", sizeof(enum Color)); return 0; }', "baseline"),
    ShadowCase("zero_init_array", 'int main() { int a[5] = {0}; printf("%d", a[4]); return 0; }', "baseline"),
    ShadowCase("array_no_size", 'int main() { int a[] = {1, 2, 3}; printf("%d", a[2]); return 0; }', "baseline"),
    ShadowCase("global_var_init", 'int g = 7; int main() { printf("%d", g); return 0; }', "baseline"),
    ShadowCase("typedef_chain", 'typedef int A; typedef A B; int main() { B x = 5; printf("%d", x); return 0; }', "baseline"),
    ShadowCase("void_param", 'int f(void) { return 1; } int main() { printf("%d", f()); return 0; }', "baseline"),
    ShadowCase("func_many_params", 'int sum(int a, int b, int c, int d, int e) { return a+b+c+d+e; } int main() { printf("%d", sum(1,2,3,4,5)); return 0; }', "baseline"),
    ShadowCase("printf_char", "int main() { printf(\"%c\", 'A'); return 0; }", "baseline"),
    ShadowCase("realloc_shrink", 'int main() { int* p = malloc(8); p[0] = 1; p[1] = 2; p = realloc(p, 4); printf("%d", p[0]); free(p); return 0; }', "baseline"),
    ShadowCase("memset_struct", 'struct S { int a; int b; }; int main() { struct S s; memset(&s, 0, sizeof(s)); printf("%d", s.a); return 0; }', "baseline"),
    ShadowCase("strcat_empty", 'int main() { char a[10] = "a"; strcat(a, ""); printf("%s", a); return 0; }', "baseline"),
    ShadowCase("atoi_negative", 'int main() { printf("%d", atoi("-42")); return 0; }', "baseline"),

    ShadowCase("empty_func", 'void f() {} int main() { f(); printf("ok"); return 0; }', "baseline"),
    ShadowCase("switch_empty", 'int main() { int x = 1; switch(x) {} printf("ok"); return 0; }', "baseline"),
    ShadowCase("switch_fallthrough", 'int main() { int x = 1; switch(x) { case 1: case 2: printf("ok"); break; } return 0; }', "baseline"),
    ShadowCase("double_assign", 'int main() { double d = 3.14; d = 2.71; printf("%.2f", d); return 0; }', "baseline"),
    ShadowCase("int_div", 'int main() { printf("%d", 7 / 3); return 0; }', "baseline"),
    ShadowCase("int_div_neg", 'int main() { printf("%d", -7 / 3); return 0; }', "baseline"),
    ShadowCase("mul_overflow_safe", 'int main() { printf("%d", 1000 * 1000); return 0; }', "baseline"),
    ShadowCase("compare_eq", 'int main() { printf("%d", 5 == 5); return 0; }', "baseline"),
    ShadowCase("compare_lt_eq", 'int main() { printf("%d", 5 <= 5); return 0; }', "baseline"),
    ShadowCase("compare_gt_eq", 'int main() { printf("%d", 5 >= 3); return 0; }', "baseline"),
    ShadowCase("bool_from_compare", 'int main() { int b = 5 > 3; printf("%d", b); return 0; }', "baseline"),
    ShadowCase("char_signedness", 'int main() { char c = -1; printf("%d", c); return 0; }', "baseline"),
    ShadowCase("array_bounds_safe", 'int main() { int a[5] = {0,1,2,3,4}; printf("%d", a[4]); return 0; }', "baseline"),
    ShadowCase("ptr_init_null", 'int main() { int* p = NULL; printf("%d", p == NULL); return 0; }', "baseline"),
    ShadowCase("struct_copy", 'struct S { int a; int b; }; int main() { struct S s1 = {1, 2}; struct S s2 = s1; printf("%d %d", s2.a, s2.b); return 0; }', "baseline"),
    ShadowCase("union_size_access", 'union U { char c; int i; }; int main() { printf("%d", sizeof(union U)); return 0; }', "baseline"),
    ShadowCase("nested_ternary", 'int main() { int a = 1, b = 2, c = 3; int r = a ? b : c; printf("%d", r); return 0; }', "baseline"),
    # ===== 已知问题（Cide 行为与 Clang 有差异，待修复） =====
    ShadowCase("loop_var_shadow", 'int main() { int i = 10; for (int i = 0; i < 3; i++) printf("%d", i); printf("%d", i); return 0; }', "scope_bug"),
    ShadowCase("string_len_manual", 'int main() { char* s = "hello"; int len = 0; while (s[len]) len++; printf("%d", len); return 0; }', "string_storage_bug"),
    ShadowCase("func_ptr_param_decay", 'int sum(int a[]) { int s = 0; for (int i = 0; i < 3; i++) s += a[i]; return s; } int main() { int arr[3] = {1,2,3}; printf("%d", sum(arr)); return 0; }', "baseline"),
    ShadowCase("double_init_zero", 'int main() { double d = 0.0; printf("%.0f", d); return 0; }', "baseline"),
    ShadowCase("sizeof_double_arr", 'int main() { double a[3]; printf("%d", sizeof(a)); return 0; }', "baseline"),
    ShadowCase("sizeof_float_arr", 'int main() { float a[3]; printf("%d", sizeof(a)); return 0; }', "baseline"),
    ShadowCase("sizeof_long_long", 'int main() { printf("%d", sizeof(long long)); return 0; }', "baseline"),
    ShadowCase("union_int_member", 'union U { int i; float f; }; int main() { union U u; u.i = 42; printf("%d", u.i); return 0; }', "baseline"),
    ShadowCase("struct_member_array", 'struct S { int arr[3]; }; int main() { struct S s; s.arr[1] = 5; printf("%d", s.arr[1]); return 0; }', "baseline"),
    ShadowCase("ptr_to_struct_member", 'struct S { int x; }; int main() { struct S s; struct S* p = &s; p->x = 7; printf("%d", p->x); return 0; }', "baseline"),
    ShadowCase("array_of_union", 'union U { int i; }; int main() { union U arr[2]; arr[0].i = 1; arr[1].i = 2; printf("%d", arr[1].i); return 0; }', "baseline"),
    ShadowCase("ptr_arith_char", "int main() { char arr[3] = {'a', 'b', 'c'}; char* p = arr; p++; printf(\"%c\", *p); return 0; }", "baseline"),
    ShadowCase("ptr_arith_struct", 'struct S { int x; }; int main() { struct S arr[2]; struct S* p = arr; p++; printf("%d", p == &arr[1]); return 0; }', "baseline"),
    ShadowCase("nested_ternary2", 'int main() { int a = 1, b = 2, c = 3; printf("%d", a > b ? c : (b > c ? a : b)); return 0; }', "baseline"),
    ShadowCase("for_break", 'int main() { for (int i = 0; i < 10; i++) { if (i == 3) break; printf("%d", i); } return 0; }', "baseline"),
    ShadowCase("for_continue", 'int main() { for (int i = 0; i < 5; i++) { if (i == 2) continue; printf("%d", i); } return 0; }', "baseline"),
    ShadowCase("while_continue", 'int main() { int i = 0; while (i < 5) { i++; if (i == 2) continue; printf("%d", i); } return 0; }', "baseline"),
    ShadowCase("do_while_break", 'int main() { int i = 0; do { if (i == 2) break; printf("%d", i); i++; } while (i < 5); return 0; }', "baseline"),
    ShadowCase("empty_for", 'int main() { for (int i = 0; i < 0; i++) printf("no"); printf("ok"); return 0; }', "baseline"),
    ShadowCase("empty_while", 'int main() { while (0) printf("no"); printf("ok"); return 0; }', "baseline"),
    ShadowCase("empty_if", 'int main() { if (0) printf("no"); printf("ok"); return 0; }', "baseline"),
    ShadowCase("negate", 'int main() { int x = 5; printf("%d", -x); return 0; }', "baseline"),
    ShadowCase("bit_not_zero", 'int main() { int x = 0; printf("%d", ~x); return 0; }', "baseline"),
    ShadowCase("bit_xor", 'int main() { printf("%d", 5 ^ 3); return 0; }', "baseline"),
    ShadowCase("shl_rhs", 'int main() { int x = 2; printf("%d", 1 << x); return 0; }', "baseline"),
    ShadowCase("shr_rhs", 'int main() { int x = 2; printf("%d", 8 >> x); return 0; }', "baseline"),
    ShadowCase("div_by_var", 'int main() { int x = 3; printf("%d", 10 / x); return 0; }', "baseline"),
    ShadowCase("mod_by_var", 'int main() { int x = 4; printf("%d", 10 % x); return 0; }', "baseline"),
    ShadowCase("mul_by_var", 'int main() { int x = 5; printf("%d", x * 6); return 0; }', "baseline"),
    ShadowCase("sub_by_var", 'int main() { int x = 3; printf("%d", 10 - x); return 0; }', "baseline"),
    ShadowCase("compare_var", 'int main() { int a = 5, b = 3; printf("%d", a > b); return 0; }', "baseline"),
    ShadowCase("equal_var", 'int main() { int a = 5, b = 5; printf("%d", a == b); return 0; }', "baseline"),
    ShadowCase("not_equal_var", 'int main() { int a = 5, b = 3; printf("%d", a != b); return 0; }', "baseline"),
    ShadowCase("logical_and_var", 'int main() { int a = 1, b = 0; printf("%d", a && b); return 0; }', "baseline"),
    ShadowCase("logical_or_var", 'int main() { int a = 0, b = 0; printf("%d", a || b); return 0; }', "baseline"),
    ShadowCase("logical_not_var", 'int main() { int a = 1; printf("%d", !a); return 0; }', "baseline"),
    ShadowCase("increment_var", 'int main() { int x = 5; x++; printf("%d", x); return 0; }', "baseline"),
    ShadowCase("decrement_var", 'int main() { int x = 5; x--; printf("%d", x); return 0; }', "baseline"),
    ShadowCase("compound_add_var", 'int main() { int x = 5; x += 3; printf("%d", x); return 0; }', "baseline"),
    ShadowCase("compound_sub_var", 'int main() { int x = 5; x -= 3; printf("%d", x); return 0; }', "baseline"),
    ShadowCase("compound_mul_var", 'int main() { int x = 5; x *= 3; printf("%d", x); return 0; }', "baseline"),
    ShadowCase("compound_div_var", 'int main() { int x = 6; x /= 3; printf("%d", x); return 0; }', "baseline"),
    ShadowCase("compound_mod_var", 'int main() { int x = 7; x %= 3; printf("%d", x); return 0; }', "baseline"),
    ShadowCase("ptr_assign_deref", 'int main() { int x = 5; int* p = &x; *p = 10; printf("%d", x); return 0; }', "baseline"),
    ShadowCase("ptr_compare_null", 'int main() { int* p = NULL; printf("%d", p == NULL); return 0; }', "baseline"),
    ShadowCase("ptr_compare_not_null", 'int main() { int x = 5; int* p = &x; printf("%d", p != NULL); return 0; }', "baseline"),
    ShadowCase("array_sum_loop", 'int main() { int a[3] = {1,2,3}; int s = 0; for (int i = 0; i < 3; i++) s += a[i]; printf("%d", s); return 0; }', "baseline"),
    ShadowCase("array_max", 'int main() { int a[3] = {3,1,2}; int m = a[0]; for (int i = 1; i < 3; i++) if (a[i] > m) m = a[i]; printf("%d", m); return 0; }', "baseline"),
    ShadowCase("swap_by_ptr", 'void swap(int* a, int* b) { int t = *a; *a = *b; *b = t; } int main() { int x = 1, y = 2; swap(&x, &y); printf("%d %d", x, y); return 0; }', "baseline"),
    ShadowCase("factorial", 'int fact(int n) { if (n <= 1) return 1; return n * fact(n-1); } int main() { printf("%d", fact(6)); return 0; }', "baseline"),
    ShadowCase("fibonacci", 'int fib(int n) { if (n <= 1) return n; return fib(n-1) + fib(n-2); } int main() { printf("%d", fib(10)); return 0; }', "baseline"),
    ShadowCase("sum_1_to_n", 'int main() { int n = 10, s = 0; for (int i = 1; i <= n; i++) s += i; printf("%d", s); return 0; }', "baseline"),
    ShadowCase("is_prime", 'int main() { int n = 17, is_p = 1; for (int i = 2; i * i <= n; i++) if (n % i == 0) is_p = 0; printf("%d", is_p); return 0; }', "baseline"),
    ShadowCase("gcd_euclid", 'int gcd(int a, int b) { while (b != 0) { int t = b; b = a % b; a = t; } return a; } int main() { printf("%d", gcd(48, 18)); return 0; }', "baseline"),
    ShadowCase("bubble_sort", 'int main() { int a[5] = {5,3,4,1,2}; for (int i = 0; i < 4; i++) for (int j = 0; j < 4-i; j++) if (a[j] > a[j+1]) { int t = a[j]; a[j] = a[j+1]; a[j+1] = t; } printf("%d", a[0]); return 0; }', "baseline"),
    ShadowCase("linear_search", 'int main() { int a[5] = {1,3,5,7,9}; int key = 7, found = -1; for (int i = 0; i < 5; i++) if (a[i] == key) { found = i; break; } printf("%d", found); return 0; }', "baseline"),
    ShadowCase("reverse_array", 'int main() { int a[5] = {1,2,3,4,5}; for (int i = 0; i < 2; i++) { int t = a[i]; a[i] = a[4-i]; a[4-i] = t; } printf("%d", a[0]); return 0; }', "baseline"),
    ShadowCase("sum_of_digits", 'int main() { int n = 123, s = 0; while (n > 0) { s += n % 10; n /= 10; } printf("%d", s); return 0; }', "baseline"),
    ShadowCase("power_of_2", 'int main() { int n = 16; printf("%d", (n & (n-1)) == 0); return 0; }', "baseline"),

    ShadowCase("memcpy_manual", 'int main() { int src[3] = {1,2,3}; int dst[3]; for (int i = 0; i < 3; i++) dst[i] = src[i]; printf("%d", dst[2]); return 0; }', "baseline"),
    ShadowCase("matrix_trace", 'int main() { int a[2][2] = {{1,2},{3,4}}; int trace = a[0][0] + a[1][1]; printf("%d", trace); return 0; }', "baseline"),

    # ===== 已知缺失特性（预期会失败） =====
    ShadowCase("double_basic", 'int main() { double d = 3.1415926535; printf("%.10f", d); return 0; }', "double"),
    ShadowCase("double_arr", 'int main() { double arr[3] = {1.1, 2.2, 3.3}; printf("%.1f", arr[1]); return 0; }', "double"),
    ShadowCase("double_printf_lf", 'int main() { double d = 3.14; printf("%lf", d); return 0; }', "double"),
    ShadowCase("function_pointer_decl", 'int add(int a, int b) { return a+b; } int main() { int (*fp)(int,int) = add; printf("%d", fp(1,2)); return 0; }', "function_pointer"),
    ShadowCase("function_pointer_array", 'int f1() { return 1; } int main() { int (*fp[2])() = {f1,f1}; printf("%d", fp[0]()); return 0; }', "function_pointer"),
    ShadowCase("file_fopen", 'int main() { FILE* f = fopen("test.txt", "r"); if (f) printf("ok"); fclose(f); return 0; }', "file_io"),
    ShadowCase("file_fread", 'int main() { FILE* f = fopen("test.txt", "r"); char buf[20]; fread(buf, 1, 5, f); buf[5] = 0; printf("%s", buf); fclose(f); return 0; }', "file_io"),
    ShadowCase("file_fwrite", 'int main() { FILE* f = fopen("out.txt", "w"); fwrite("hello", 1, 5, f); fclose(f); printf("ok"); return 0; }', "file_io"),
    ShadowCase("function_pointer_arg", 'int apply(int (*op)(int), int x) { return op(x); } int inc(int n) { return n+1; } int main() { printf("%d", apply(inc, 5)); return 0; }', "function_pointer"),
    ShadowCase("goto_basic", 'int main() { int x = 0; goto end; x = 1; end: printf("%d", x); return 0; }', "goto"),
    ShadowCase("designated_init", 'int main() { int a[3] = {[0] = 1, [2] = 3}; printf("%d", a[2]); return 0; }', "designated_initializer"),
    ShadowCase("variable_length_array", 'int main() { int n = 3; int a[n]; a[0] = 1; printf("%d", a[0]); return 0; }', "variable_length_array"),
    ShadowCase("static_assert", 'int main() { _Static_assert(1 == 1, "ok"); printf("ok"); return 0; }', "static_assert"),
    ShadowCase("complex_number", 'int main() { double complex z = 1.0 + 2.0*I; printf("%.1f", creal(z)); return 0; }', "complex_number"),

    # ===== 边界/边缘用例 =====
    ShadowCase("inline_asm", 'int main() { int x = 1; __asm__ ("nop"); printf("%d", x); return 0; }', "inline_asm"),
    ShadowCase("variadic_macro", '#define LOG(fmt, ...) printf(fmt, __VA_ARGS__)\nint main() { LOG("%d", 42); return 0; }', "variadic_macro"),
    ShadowCase("typeof_operator", 'int main() { int x = 5; typeof(x) y = 10; printf("%d", y); return 0; }', "typeof"),
    ShadowCase("implicit_int", 'main() { printf("hello"); return 0; }', "implicit_int"),
]


def main():
    print("=" * 60)
    print("Cide 影子验证框架")
    print("=" * 60)

    if not DLL_PATH.exists():
        print(f"错误: 找不到 Cide DLL: {DLL_PATH}")
        print("请先运行: cd native && cargo build --release")
        sys.exit(1)

    diffs: List[ShadowDiff] = []

    for i, case in enumerate(SHADOW_CASES, 1):
        print(f"\n[{i}/{len(SHADOW_CASES)}] {case.name} ({case.category})")

        clang_res = run_with_clang(case.source)
        print(f"  Clang: compile={'OK' if clang_res.compile_success else 'FAIL'}, run={'OK' if clang_res.run_success else 'FAIL'}")

        cide_res = run_with_cide(case.source)
        print(f"  Cide:  compile={'OK' if cide_res.compile_success else 'FAIL'}, run={'OK' if cide_res.run_success else 'FAIL'}")

        diff = analyze_diff(case, clang_res, cide_res)
        diffs.append(diff)

        if diff.diff_type == "compile_gap":
            cat = classify_compile_error(cide_res.compile_error, case.category)
            print(f"  => 编译缺口 [{cat}]")
        elif diff.diff_type == "match":
            print(f"  => 匹配 ✓")
        else:
            print(f"  => {diff.diff_type}")

    # 生成报告
    report_path = SCRIPT_DIR / "reports" / f"shadow_report_{time.strftime('%Y%m%d_%H%M%S')}.md"
    report_path.parent.mkdir(parents=True, exist_ok=True)
    generate_report(diffs, report_path)

    # 同时输出 JSON
    json_path = SCRIPT_DIR / "reports" / f"shadow_data_{time.strftime('%Y%m%d_%H%M%S')}.json"
    json_data = {
        "timestamp": time.strftime('%Y-%m-%d %H:%M:%S'),
        "summary": {
            "total": len(diffs),
            "match": len([d for d in diffs if d.diff_type == "match"]),
            "compile_gap": len([d for d in diffs if d.diff_type == "compile_gap"]),
            "runtime_gap": len([d for d in diffs if d.diff_type == "runtime_gap"]),
            "output_gap": len([d for d in diffs if d.diff_type == "output_gap"]),
        },
        "category_frequency": {},
        "details": [
            {
                "case": d.case_name,
                "expected": d.expected_category,
                "diff_type": d.diff_type,
                "cide_compile_error": d.cide_result.compile_error[:500] if not d.cide_result.compile_success else "",
            }
            for d in diffs
        ],
    }
    for d in diffs:
        if d.diff_type == "compile_gap":
            cat = classify_compile_error(d.cide_result.compile_error, d.expected_category)
            json_data["category_frequency"][cat] = json_data["category_frequency"].get(cat, 0) + 1

    json_path.write_text(json.dumps(json_data, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"\nJSON 数据已保存: {json_path}")


if __name__ == "__main__":
    main()
