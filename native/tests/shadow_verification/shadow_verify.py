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
            stdout_str = re.sub(r'程序运行完成，返回值：\d+\n?', '', stdout_str)
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


def classify_compile_error(error_msg: str) -> str:
    """根据 Cide 编译错误消息分类缺失特性"""
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
        cat = classify_compile_error(d.cide_result.compile_error)
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

    # ===== 已知缺失特性（预期会失败） =====
    ShadowCase("double_basic", 'int main() { double d = 3.1415926535; printf("%.10f", d); return 0; }', "double"),
    ShadowCase("double_arr", 'int main() { double arr[3] = {1.1, 2.2, 3.3}; printf("%.1f", arr[1]); return 0; }', "double"),
    ShadowCase("double_printf_lf", 'int main() { double d = 3.14; printf("%lf", d); return 0; }', "double"),
    ShadowCase("function_pointer_decl", 'int add(int a, int b) { return a+b; } int main() { int (*fp)(int,int) = add; printf("%d", fp(1,2)); return 0; }', "function_pointer"),
    ShadowCase("function_pointer_array", 'int f1() { return 1; } int main() { int (*fp[2])() = {f1,f1}; printf("%d", fp[0]()); return 0; }', "function_pointer"),
    ShadowCase("file_fopen", 'int main() { FILE* f = fopen("test.txt", "w"); fprintf(f, "hello"); fclose(f); printf("ok"); return 0; }', "file_io"),
    ShadowCase("file_fread", 'int main() { FILE* f = fopen("test.txt", "r"); char buf[10]; fread(buf, 1, 5, f); fclose(f); printf("%s", buf); return 0; }', "file_io"),
    ShadowCase("union_basic", 'union U { int i; float f; }; int main() { union U u; u.i = 1; printf("%d", u.i); return 0; }', "union"),
    ShadowCase("long_long", 'int main() { long long ll = 9223372036854775807LL; printf("%lld", ll); return 0; }', "long_long"),
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
            cat = classify_compile_error(cide_res.compile_error)
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
            cat = classify_compile_error(d.cide_result.compile_error)
            json_data["category_frequency"][cat] = json_data["category_frequency"].get(cat, 0) + 1

    json_path.write_text(json.dumps(json_data, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"\nJSON 数据已保存: {json_path}")


if __name__ == "__main__":
    main()
