#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
C++ Shadow Verification 框架
对比 Clang++ 和 Cide 对同一份 C++ 代码的编译/运行结果。

用法:
    python scripts/shadow_verify_cpp.py
"""

import os
import sys
import io
import subprocess
import tempfile
import time
import json
from pathlib import Path
from dataclasses import dataclass, asdict
from typing import Optional, List, Dict

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8')
sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8')

SCRIPT_DIR = Path(__file__).parent.resolve()
PROJECT_ROOT = SCRIPT_DIR.parent
NATIVE_DIR = PROJECT_ROOT / "native"
CIDE_CLI = NATIVE_DIR / "target/release/cide_cli.exe"
CLANG_PATH = "clang++"


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
    category: str


@dataclass
class ShadowDiff:
    case_name: str
    expected_category: str
    clang_result: RunResult
    cide_result: RunResult
    diff_type: str


CPP_CASES: List[ShadowCase] = [
    ShadowCase("cpp_class_field", r'''
#include <stdio.h>
class Point {
public:
    int x;
    Point() { x = 0; }
};
int main() {
    Point p;
    p.x = 42;
    printf("%d\n", p.x);
    return 0;
}
''', "baseline"),
    ShadowCase("cpp_new_delete", r'''
#include <stdio.h>
class Box {
public:
    int v;
    Box() { v = 0; }
};
int main() {
    Box* b = new Box();
    b->v = 7;
    printf("%d\n", b->v);
    delete b;
    return 0;
}
''', "baseline"),
    ShadowCase("cpp_virtual_call", r'''
#include <stdio.h>
class Base {
public:
    virtual int foo() { return 1; }
};
class Derived : public Base {
public:
    int foo() { return 2; }
};
int main() {
    Base* b = new Derived();
    printf("%d\n", b->foo());
    delete b;
    return 0;
}
''', "baseline"),
    ShadowCase("cpp_template_class", r'''
#include <stdio.h>
template<class T>
class Box {
public:
    T v;
    Box() { v = 0; }
};
int main() {
    Box<int> b;
    b.v = 99;
    printf("%d\n", b.v);
    return 0;
}
''', "baseline"),
    ShadowCase("cpp_lambda_capture", r'''
#include <stdio.h>
int main() {
    int x = 5;
    auto f = [x](int y) { return x + y; };
    printf("%d\n", f(3));
    return 0;
}
''', "baseline"),
    ShadowCase("cpp_reference_param", r'''
#include <stdio.h>
void inc(int& x) { x = x + 1; }
int main() {
    int a = 5;
    inc(a);
    printf("%d\n", a);
    return 0;
}
''', "baseline"),
    ShadowCase("cpp_range_for", r'''
#include <stdio.h>
int main() {
    int arr[] = {1, 2, 3};
    int sum = 0;
    for (int x : arr) sum = sum + x;
    printf("%d\n", sum);
    return 0;
}
''', "baseline"),
    ShadowCase("cpp_raii_dtor", r'''
#include <stdio.h>
int g = 0;
class A {
public:
    int id;
    A() { id = 0; }
    void init(int i) { id = i; }
    ~A() { g = g * 10 + id; }
};
void foo() {
    A a;
    a.init(1);
}
int main() {
    foo();
    printf("%d\n", g);
    return 0;
}
''', "baseline"),
    ShadowCase("cpp_new_array", r'''
#include <stdio.h>
int g = 0;
class A {
public:
    A() { g++; }
    ~A() { g--; }
};
int main() {
    A* arr = new A[3];
    printf("%d\n", g);
    delete[] arr;
    printf("%d\n", g);
    return 0;
}
''', "baseline"),
    ShadowCase("cpp_nested_class_new", r'''
#include <stdio.h>
template<class T>
class list {
    struct Node {
        T data;
        Node* next;
    };
    Node* head;
public:
    list() : head((Node*)0) {}
    void push(T x) {
        Node* n = new Node;
        n->data = x;
        n->next = head;
        head = n;
    }
    T get(int i) {
        Node* p = head;
        while (i-- > 0 && p != (Node*)0) p = p->next;
        if (p == (Node*)0) return 0;
        return p->data;
    }
    ~list() {
        Node* p = head;
        while (p != (Node*)0) {
            Node* n = p->next;
            delete p;
            p = n;
        }
    }
};
int main() {
    list<int> l;
    l.push(10);
    l.push(20);
    printf("%d\n", l.get(0));
    printf("%d\n", l.get(1));
    return 0;
}
''', "baseline"),
    ShadowCase("cpp_ctor_overload", r'''
#include <stdio.h>
class Box {
public:
    int x;
    Box() { x = 0; }
    Box(int v) { x = v; }
};
int main() {
    Box* a = new Box();
    Box* b = new Box(42);
    printf("%d %d\n", a->x, b->x);
    delete a;
    delete b;
    return 0;
}
''', "gap"),
    ShadowCase("cpp_rvalue_ref", r'''
#include <stdio.h>
int foo() { return 42; }
int main() {
    int&& r = foo();
    printf("%d\n", r);
    return 0;
}
''', "gap"),
    ShadowCase("cpp_const_ref_rvalue", r'''
#include <stdio.h>
int main() {
    const int& r = 5;
    printf("%d\n", r);
    return 0;
}
''', "gap"),
    ShadowCase("cpp_auto_ref", r'''
#include <stdio.h>
int main() {
    int x = 10;
    auto& r = x;
    r = r + 1;
    printf("%d\n", x);
    return 0;
}
''', "baseline"),
    ShadowCase("cpp_lambda_multi_capture", r'''
#include <stdio.h>
int main() {
    int a = 1, b = 2;
    auto f = [a, &b](int x) { return x + a + b; };
    b = 5;
    printf("%d\n", f(10));
    return 0;
}
''', "baseline"),
    ShadowCase("cpp_lambda_ref_capture", r'''
#include <stdio.h>
int main() {
    int x = 5;
    auto f = [&x]() { x = x + 1; };
    f();
    printf("%d\n", x);
    return 0;
}
''', "baseline"),
    ShadowCase("cpp_member_out_of_line", r'''
#include <stdio.h>
class Bar {
public:
    int x;
    void set(int v);
};
void Bar::set(int v) { x = v; }
int main() {
    Bar b;
    b.set(7);
    printf("%d\n", b.x);
    return 0;
}
''', "gap"),
    ShadowCase("cpp_auto_new_int", r'''
#include <stdio.h>
int main() {
    auto p = new int(99);
    printf("%d\n", *p);
    delete p;
    return 0;
}
''', "baseline"),
    ShadowCase("cpp_range_for_ref_modify", r'''
#include <stdio.h>
int main() {
    int arr[] = {1, 2, 3};
    for (auto& x : arr) x = x * 2;
    printf("%d %d %d\n", arr[0], arr[1], arr[2]);
    return 0;
}
''', "gap"),
    ShadowCase("cpp_template_struct", r'''
#include <stdio.h>
template<class T> struct Pair { T a, b; };
int main() {
    Pair<int> p;
    p.a = 1;
    p.b = 2;
    printf("%d %d\n", p.a, p.b);
    return 0;
}
''', "gap"),
    ShadowCase("cpp_struct_tag_alias", r'''
#include <stdio.h>
struct Node { int x; };
int main() {
    Node n;
    n.x = 42;
    printf("%d\n", n.x);
    return 0;
}
''', "baseline"),
    ShadowCase("cpp_new_int_array", r'''
#include <stdio.h>
int main() {
    int* p = new int[3];
    p[0] = 1;
    p[1] = 2;
    p[2] = 3;
    printf("%d %d %d\n", p[0], p[1], p[2]);
    delete[] p;
    return 0;
}
''', "baseline"),
]


def run_with_clang(source: str) -> RunResult:
    start = time.time()
    with tempfile.TemporaryDirectory() as tmpdir:
        cpp_file = Path(tmpdir) / "test.cpp"
        exe_file = Path(tmpdir) / "test.exe" if sys.platform == "win32" else Path(tmpdir) / "test"
        cpp_file.write_text(source, encoding="utf-8")

        compile_cmd = [CLANG_PATH, str(cpp_file), "-o", str(exe_file), "-std=c++11"]
        if sys.platform != "win32":
            compile_cmd.append("-lm")
        try:
            compile_proc = subprocess.run(
                compile_cmd, capture_output=True, text=True, encoding="utf-8", timeout=30
            )
        except Exception as e:
            return RunResult(
                compiler="clang++", compile_success=False, compile_error=str(e),
                run_success=False, run_error="", stdout="", stderr="",
                exit_code=-1, duration_ms=(time.time() - start) * 1000,
            )

        if compile_proc.returncode != 0:
            return RunResult(
                compiler="clang++", compile_success=False, compile_error=compile_proc.stderr,
                run_success=False, run_error="", stdout="", stderr=compile_proc.stderr,
                exit_code=compile_proc.returncode, duration_ms=(time.time() - start) * 1000,
            )

        try:
            run_proc = subprocess.run(
                [str(exe_file)], capture_output=True, text=True, encoding="utf-8", timeout=5
            )
            return RunResult(
                compiler="clang++", compile_success=True, compile_error="",
                run_success=run_proc.returncode == 0,
                run_error=run_proc.stderr if run_proc.returncode != 0 else "",
                stdout=run_proc.stdout, stderr=run_proc.stderr,
                exit_code=run_proc.returncode, duration_ms=(time.time() - start) * 1000,
            )
        except Exception as e:
            return RunResult(
                compiler="clang++", compile_success=True, compile_error="",
                run_success=False, run_error=str(e), stdout="", stderr="",
                exit_code=-1, duration_ms=(time.time() - start) * 1000,
            )


def run_with_cide(source: str) -> RunResult:
    start = time.time()
    if not CIDE_CLI.exists():
        return RunResult(
            compiler="cide", compile_success=False, compile_error=f"cide_cli not found: {CIDE_CLI}",
            run_success=False, run_error="", stdout="", stderr="",
            exit_code=-1, duration_ms=(time.time() - start) * 1000,
        )

    try:
        with tempfile.TemporaryDirectory() as tmpdir:
            cpp_file = Path(tmpdir) / "test.cpp"
            cpp_file.write_text(source, encoding="utf-8")
            proc = subprocess.run(
                [str(CIDE_CLI), "run", str(cpp_file)],
                capture_output=True, text=True, encoding="utf-8", timeout=15
            )
            full_out = (proc.stdout or "") + (proc.stderr or "")
            compile_ok = "编译失败。" not in full_out
            runtime_ok = "运行错误" not in full_out and "Runtime error" not in full_out
            # Extract program output
            out_lines = []
            if "=== 运行输出 ===" in full_out:
                _, rest = full_out.split("=== 运行输出 ===", 1)
                for line in rest.strip().splitlines():
                    line = line.strip()
                    if not line or line.startswith("程序运行完成"):
                        continue
                    if line.startswith("====="):
                        break
                    out_lines.append(line)
            else:
                # No output section means compile failed or no output
                pass
            return RunResult(
                compiler="cide", compile_success=compile_ok, compile_error=full_out if not compile_ok else "",
                run_success=runtime_ok and proc.returncode == 0,
                run_error=full_out if not runtime_ok else "",
                stdout="\n".join(out_lines), stderr=proc.stderr,
                exit_code=proc.returncode, duration_ms=(time.time() - start) * 1000,
            )
    except Exception as e:
        return RunResult(
            compiler="cide", compile_success=False, compile_error=str(e),
            run_success=False, run_error="", stdout="", stderr="",
            exit_code=-1, duration_ms=(time.time() - start) * 1000,
        )


def compare_results(clang: RunResult, cide: RunResult) -> str:
    if not clang.compile_success:
        return "clang_compile_fail"
    if not cide.compile_success:
        return "compile_gap"
    if not clang.run_success or not cide.run_success:
        if clang.run_success != cide.run_success:
            return "runtime_gap"
    clang_out = clang.stdout.strip().replace("\r\n", "\n")
    cide_out = cide.stdout.strip().replace("\r\n", "\n")
    if clang_out != cide_out:
        return "output_gap"
    return "match"


def main():
    diffs: List[ShadowDiff] = []
    for case in CPP_CASES:
        print(f"Running {case.name} ...", flush=True)
        clang_res = run_with_clang(case.source)
        cide_res = run_with_cide(case.source)
        diff_type = compare_results(clang_res, cide_res)
        diffs.append(ShadowDiff(
            case_name=case.name,
            expected_category=case.category,
            clang_result=clang_res,
            cide_result=cide_res,
            diff_type=diff_type,
        ))
        status = "✅ MATCH" if diff_type == "match" else f"❌ {diff_type.upper()}"
        print(f"  {status}")

    total = len(diffs)
    match = sum(1 for d in diffs if d.diff_type == "match")
    compile_gap = sum(1 for d in diffs if d.diff_type == "compile_gap")
    runtime_gap = sum(1 for d in diffs if d.diff_type == "runtime_gap")
    output_gap = sum(1 for d in diffs if d.diff_type == "output_gap")
    clang_fail = sum(1 for d in diffs if d.diff_type == "clang_compile_fail")

    print("\n" + "=" * 60)
    print("C++ Shadow Verification 报告")
    print("=" * 60)
    print(f"总用例: {total}")
    print(f"  ✅ 一致: {match}")
    print(f"  ❌ 编译差异: {compile_gap}")
    print(f"  ❌ 运行时差异: {runtime_gap}")
    print(f"  ❌ 输出差异: {output_gap}")
    print(f"  ⚠️  Clang++ 编译失败: {clang_fail}")

    report_path = PROJECT_ROOT / "native/tests/shadow_verification/reports/cpp_shadow_report.json"
    report_path.parent.mkdir(parents=True, exist_ok=True)
    with open(report_path, "w", encoding="utf-8") as f:
        json.dump([asdict(d) for d in diffs], f, ensure_ascii=False, indent=2)
    print(f"\n报告已保存: {report_path}")

    return 0 if match == total else 1


if __name__ == "__main__":
    sys.exit(main())
