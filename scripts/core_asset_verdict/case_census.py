# -*- coding: utf-8 -*-
"""662 影子用例的静态普查（Q1/Q3 行为债量化的原始数据）。

本脚本只做**可机检的静态分类**，不做任何"脑测"：
  * 规模：行数 / 字节数分布（回应"用例窄而密"）；
  * 语言面：每个用例实际用到的教学子集特性（正则命中，best-effort，脚本内明示局限）；
  * 防线面：同一驱动对每个用例实际断言了什么（由驱动源码决定：只比 stdout）。

输出：JSON（机器可读）+ 控制台摘要（人读）。
用途：核心资产重构裁定.md 的 Q1/Q3 表，标注 [动态实测]。

用法：python scripts/core_asset_verdict/case_census.py
"""
import io
import json
import re
import sys
from collections import Counter
from pathlib import Path

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")

NATIVE = Path(__file__).resolve().parent.parent.parent / "native"
DIRS = [
    ("baseline", NATIVE / "tests/cases/baseline"),
    ("gap", NATIVE / "tests/cases/gap"),
    ("template", NATIVE / "tests/cases_template_generated"),
    ("knr", NATIVE / "tests/cases/knr"),
    ("leetcode", NATIVE / "tests/cases/leetcode"),
]

# 语言面探针（正则，命中即标记；写成保守形式，避免把注释里的词当代码）
FACETS = {
    "loop": r"\b(for|while|do)\s*\(",
    "recursion_like": r"\b([A-Za-z_]\w*)\s*\([^;]*\)\s*\{[^}]*\b\1\s*\(",
    "pointer": r"\*",
    "address_of": r"&[A-Za-z_]",
    "array": r"\w+\s*\[[^\]]*\]",
    "malloc": r"\b(malloc|calloc|realloc)\s*\(",
    "free": r"\bfree\s*\(",
    "struct": r"\bstruct\s+\w+",
    "union": r"\bunion\s+\w+",
    "enum": r"\benum\s+\w+",
    "typedef": r"\btypedef\b",
    "double_float": r"\b(double|float)\b",
    "unsigned": r"\bunsigned\b",
    "switch": r"\bswitch\s*\(",
    "goto": r"\bgoto\b",
    "scanf_stdin": r"\b(scanf|getchar|fgets|gets)\s*\(",
    "file_io": r"\b(fopen|fclose|fread|fwrite|fseek|ftell|fprintf)\s*\(",
    "stdio_printf": r"\b(printf|puts|putchar|fputs|sprintf|snprintf)\s*\(",
    "string_lib": r"\b(strlen|strcpy|strncpy|strcmp|strcat|memcpy|memset|strstr)\s*\(",
    "math_lib": r"\b(sqrt|pow|sin|cos|fabs|floor|ceil|fmod)\s*\(",
    "qsort_bsearch": r"\b(qsort|bsearch)\s*\(",
    "func_pointer": r"\(\s*\*\s*\w+\s*\)\s*\(",
    "vla": r"\w+\s*\[\s*[A-Za-z_]\w*\s*\]",
    "macro_define": r"#\s*define\b",
    "macro_call": r"#\s*define\s+\w+\s*\(",
    "include_custom": r'#\s*include\s*"',
    "multidim_array": r"\w+\s*\[[^\]]*\]\s*\[",
    "struct_by_value": r"\bstruct\s+\w+\s+\w+\s*\(",  # 结构体按值返回/参数的近似
    "ternary": r"\?[^;\n]*:",
    "bitops": r"(<<|>>|\^|~)",
    "string_literal": r'"',
    "exit_call": r"\bexit\s*\(",
    "variadic": r"\bva_(start|arg|end|list)\b",
}


def sha_lines(text: str) -> int:
    return len(text.splitlines())


def main() -> int:
    cases = []
    for src_dir, root in DIRS:
        if not root.exists():
            continue
        for path in sorted(root.glob("*.c")):
            raw = path.read_text(encoding="utf-8", errors="replace")
            # 去掉 // @ 类注释行与 /* */ 注释，避免注释里的词命中语言面
            body = re.sub(r"/\*.*?\*/", "", raw, flags=re.S)
            body = "\n".join(l for l in body.splitlines() if not l.strip().startswith("//"))
            cat_match = re.search(r"@category:\s*([A-Za-z0-9_\-]+)", raw)
            in_path = path.with_suffix(".in")
            facets = sorted(k for k, pat in FACETS.items() if re.search(pat, body, re.S))
            cases.append({
                "name": path.stem,
                "src_dir": src_dir,
                "category": cat_match.group(1) if cat_match else "baseline",
                "lines": sha_lines(body),
                "bytes": len(body.encode("utf-8")),
                "has_stdin_file": in_path.exists(),
                "stdin_bytes": len(in_path.read_bytes()) if in_path.exists() else 0,
                "facets": facets,
                # 防线实际断言面：影子驱动 analyze_diff 只比较 stdout（源码为证）
                "defense_asserts": ["program_stdout"],
            })
    out = Path(__file__).parent / "case_census.json"
    out.write_text(json.dumps(cases, ensure_ascii=False, indent=1), encoding="utf-8")

    n = len(cases)
    lines = sorted(c["lines"] for c in cases)
    def pct(p):
        return lines[min(n - 1, int(n * p / 100))]
    facet_counts = Counter(f for c in cases for f in c["facets"])
    dir_counts = Counter(c["src_dir"] for c in cases)
    cat_counts = Counter(c["category"] for c in cases)

    print(f"用例总数: {n}")
    print(f"行数: 中位 {pct(50)}  均值 {sum(lines)/n:.1f}  p90 {pct(90)}  p99 {pct(99)}  最大 {lines[-1]}")
    print(f"<=5 行: {sum(1 for x in lines if x <= 5)}  <=20 行: {sum(1 for x in lines if x <= 20)}  "
          f">100 行: {sum(1 for x in lines if x > 100)}")
    print(f"带 .in stdin 文件: {sum(1 for c in cases if c['has_stdin_file'])}")
    print("按目录:", dict(dir_counts))
    print("按 category(前 12):", dict(cat_counts.most_common(12)))
    print("\n语言面命中（用例数 / 662）：")
    for k, v in facet_counts.most_common():
        print(f"  {k:18s} {v:4d}  ({v*100//n}%)")
    print(f"\nJSON 已写出: {out}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
