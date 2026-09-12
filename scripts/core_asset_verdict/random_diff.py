# -*- coding: utf-8 -*-
"""随机程序差分测试：生成器 × clang × Cide 三路对照（独立于 662 用例作者的通道）。

**已被 Go 版取代（2026-09-12，D5 语言迁移第三站）**：
`go run scripts/core_asset_verdict/random_diff.go` 与本版双轨对账一致
（RNG 逐比特复刻 → 同 seed 同用例集合；1000 例 verdict + expected 完全一致）。
本文件保留为**双轨对照基准**——Go 版判定异常时用于归因复现；日常运行用 Go 版。

为什么需要它（T2）：662 影子用例与驱动大量由 AI 编写，人审又受"想不到＝测不到"限制。
本通道的**输入不是任何人写的用例**，而是随机生成器产物，且带**第三路独立 oracle**：

    随机生成器（本脚本）──→ C 源码 ──→ clang  ──→ stdout_clang
            │                  └──────→ Cide   ──→ stdout_cide (纯程序 stdout 通道)
            └──→ Python 语义模型 ──→ expected

判定：
  * model ≠ clang  → 生成器/渲染有误（不算引擎缺陷，单独统计，用于自检生成器）
  * clang ≠ cide   → **引擎与 C 标准的差异**（保留最小复现到 .findings/）

规模：默认四族 × 100 例 = 400 例，种子固定可复现。
用法：python scripts/core_asset_verdict/random_diff.py [--per-family 100] [--seed 20260912] [--jobs 6]
输出：random_diff.json + .findings/*.c。标注 [动态实测]。
"""
import io
import json
import random
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
SHADOW = ROOT / "native/tests/shadow_verification"
sys.path.insert(0, str(SHADOW))
import shadow_verify as sv  # noqa: E402

WORK = HERE / ".randomdiff"
FINDINGS = HERE / ".findings"
MASK32 = 0xFFFFFFFF


def s32(v: int) -> int:
    v &= MASK32
    return v - 0x100000000 if v >= 0x80000000 else v


def u32(v: int) -> int:
    return v & MASK32


# ───────────────────────── 族 1：int 表达式 ─────────────────────────
INT_BINOPS = ["+", "-", "*", "/", "%", "&", "|", "^", "<<", ">>"]


def gen_int_expr(rng, env, depth):
    """返回 (c_code, py_value)。保证不自溢出（值域自检 + 重采样）。"""
    if depth <= 0 or (env and rng.random() < 0.35):
        if env and rng.random() < 0.6:
            name = rng.choice(list(env))
            return name, env[name]
        return str(rng.randint(0, 40)), None  # None = 由调用方填
    op = rng.choice(INT_BINOPS)
    lc, lv = gen_int_expr(rng, env, depth - 1)
    if op in ("<<", ">>"):
        rc, rv = str(rng.randint(0, 4)), None
        rv = int(rc)
    else:
        rc, rv = gen_int_expr(rng, env, depth - 1)
    return f"({lc} {op} {rc})", (op, lv, rv, lc, rc)


def eval_int(op, lv, rv):
    if op == "+":
        return lv + rv
    if op == "-":
        return lv - rv
    if op == "*":
        return lv * rv
    if op == "/":
        return None if rv == 0 else int(lv / rv if (lv < 0) != (rv < 0) and lv % rv else lv // rv)
    if op == "%":
        if rv == 0:
            return None
        r = abs(lv) % abs(rv)
        return -r if lv < 0 else r
    if op == "&":
        return lv & rv
    if op == "|":
        return lv | rv
    if op == "^":
        return lv ^ rv
    if op == "<<":
        return lv << rv
    if op == ">>":
        return lv >> rv
    raise AssertionError(op)


def build_int_program(rng):
    """生成 + 求值（值域受限，避免实现定义/未定义行为）。"""
    for _ in range(200):
        env = {}
        lines = []
        ok = True
        nvars = rng.randint(2, 4)
        for i in range(nvars):
            e = rng.randint(0, 30)
            env[f"v{i}"] = e
            lines.append(f"    int v{i} = {e};")
        for _ in range(rng.randint(2, 5)):
            tgt = rng.choice(list(env))
            # 手写小步求值：递归展开
            code, val = render_int(rng, env, 2)
            if val is None or abs(val) > 2_000_000:
                ok = False
                break
            env[tgt] = val
            lines.append(f"    {tgt} = {code};")
        if not ok:
            continue
        prints = "".join(f'    printf("%d\\n", {v});\n' for v in env)
        expected = "".join(f"{env[v]}\n" for v in env)
        return lines, prints, expected
    return None


def render_int(rng, env, depth):
    """返回值受限的 int 表达式（c_code, py_value）。"""
    if depth <= 0:
        if env and rng.random() < 0.6:
            n = rng.choice(list(env))
            return n, env[n]
        k = rng.randint(0, 20)
        return str(k), k
    op = rng.choice(["+", "-", "*", "/", "%", "&", "|", "^", "<<", ">>"])
    lc, lv = render_int(rng, env, depth - 1)
    if op in ("<<", ">>"):
        k = rng.randint(0, 3)
        rc, rv = str(k), k
    elif op in ("*",):
        k = rng.randint(0, 5)
        rc, rv = str(k), k
    elif op in ("/", "%"):
        k = rng.randint(1, 9)
        rc, rv = str(k), k
    else:
        rc, rv = render_int(rng, env, depth - 1)
    v = eval_int(op, lv, rv)
    if v is None or abs(v) > 500_000:
        # 退回常量，保证语义确定
        k = rng.randint(0, 20)
        return str(k), k
    return f"({lc} {op} {rc})", v


def fam_int(rng):
    r = build_int_program(rng)
    if r is None:
        return None
    lines, prints, expected = r
    src = "#include <stdio.h>\nint main() {\n" + "\n".join(lines) + "\n" + prints + "    return 0;\n}\n"
    return src, expected


# ───────────────────────── 族 2：unsigned 算术 ─────────────────────────
UNSIGNED_OPS = ["+", "-", "*", "/", "%", "&", "|", "^", "<<", ">>"]


def fam_unsigned(rng):
    env = {}
    lines = []
    for i in range(rng.randint(2, 4)):
        e = rng.randint(0, 4_000_000_000)
        env[f"u{i}"] = e
        lines.append(f"    unsigned int u{i} = {e}u;")
    for _ in range(rng.randint(2, 4)):
        tgt = rng.choice(list(env))
        a_name = rng.choice(list(env))
        a = env[a_name]
        op = rng.choice(UNSIGNED_OPS)
        if op in ("<<", ">>"):
            k = rng.randint(0, 31)
            b, rhs = k, str(k)
        elif op in ("/", "%"):
            k = rng.randint(1, 65535)
            b, rhs = k, str(k)
        elif op == "*":
            k = rng.randint(0, 1_000_000)
            b, rhs = k, f"{k}u"
        else:
            b_name = rng.choice(list(env))
            b, rhs = env[b_name], b_name
        if op == "+":
            v = u32(a + b)
        elif op == "-":
            v = u32(a - b)
        elif op == "*":
            v = u32(a * b)
        elif op == "/":
            v = u32(a // b)
        elif op == "%":
            v = u32(a % b)
        elif op == "&":
            v = u32(a & b)
        elif op == "|":
            v = u32(a | b)
        elif op == "^":
            v = u32(a ^ b)
        elif op == "<<":
            v = u32(a << b)
        elif op == ">>":
            v = u32(a >> b)
        env[tgt] = v
        lines.append(f"    {tgt} = {a_name} {op} {rhs};")
    prints = "".join(f'    printf("%u\\n", {v});\n' for v in env)
    expected = "".join(f"{env[v]}\n" for v in env)
    src = "#include <stdio.h>\nint main() {\n" + "\n".join(lines) + "\n" + prints + "    return 0;\n}\n"
    return src, expected


# ───────────────────────── 族 3：数组/指针循环 ─────────────────────────
def fam_array(rng):
    k = rng.randint(2, 8)
    init = [rng.randint(-20, 20) for _ in range(k)]
    mult = [rng.randint(-3, 3) for _ in range(k)]
    src = ["#include <stdio.h>", "int main() {",
           "    int a[%d];" % k]
    src.append("    " + " ".join(f"a[{i}] = {init[i]};" for i in range(k)))
    src.append("    int s = 0;")
    src.append(f"    for (int i = 0; i < {k}; i++) {{ s += a[i] * {mult[0]}; }}")
    src.append("    int t = 0;")
    src.append(f"    int* p = a;")
    src.append(f"    for (int i = 0; i < {k}; i++) {{ t += *(p + i) * {mult[-1] if mult[-1] != 0 else 1}; }}")
    src.append("    printf(\"%d\\n\", s);")
    src.append("    printf(\"%d\\n\", t);")
    src.append("    return 0;")
    src.append("}")
    s = sum(init[i] * mult[0] for i in range(k))
    m = mult[-1] if mult[-1] != 0 else 1
    t = sum(init[i] * m for i in range(k))
    return "\n".join(src) + "\n", f"{s}\n{t}\n"


# ───────────────────────── 族 4：控制流 ─────────────────────────
def fam_control(rng):
    x0 = rng.randint(-10, 10)
    m = rng.randint(3, 12)
    step_add = rng.randint(-4, 4)
    step_sub = rng.randint(-4, 4)
    lim = rng.randint(-20, 20)
    brk = rng.randint(0, m)
    x = x0
    for i in range(m):
        if i > brk:
            break
        if i % 3 == 0:
            x += step_add
            continue
        x -= step_sub
    src = f"""#include <stdio.h>
int main() {{
    int x = {x0};
    for (int i = 0; i < {m}; i++) {{
        if (i > {brk}) break;
        if (i % 3 == 0) {{ x += {step_add}; continue; }}
        x -= {step_sub};
    }}
    printf("%d\\n", x);
    printf("%d\\n", x > {lim} ? 1 : 0);
    return 0;
}}
"""
    return src, f"{x}\n{1 if x > lim else 0}\n"


# ───────────────────────── 族 5：函数调用 + 递归 ─────────────────────────
def fam_funcs(rng):
    a1, b1 = rng.randint(1, 30), rng.randint(1, 30)
    a2, b2 = rng.randint(1, 30), rng.randint(1, 30)
    n = rng.randint(1, 7)
    src = f"""#include <stdio.h>
int f1(int a, int b) {{ return a * {a1} + b - {b1}; }}
int f2(int a, int b) {{ return a + b * {a2} % ({b2} + 1); }}
int rec(int n) {{ if (n <= 1) return 1; return n + rec(n - 1); }}
int main() {{
    printf("%d\\n", f1({a1}, {b1}));
    printf("%d\\n", f2({a2}, {b2}));
    printf("%d\\n", rec({n}));
    printf("%d\\n", f1(f2({a1}, {b1}), rec({n})));
    return 0;
}}
"""
    v1 = a1 * a1 + b1 - b1
    v2 = a2 + b2 * a2 % (b2 + 1)          # f2(a2, b2)
    v3 = sum(range(1, n + 1)) if n > 1 else 1
    w = a1 + b1 * a2 % (b2 + 1)           # f2(a1, b1)
    v4 = w * a1 + v3 - b1                 # f1(f2(a1,b1), rec(n))
    return src, f"{v1}\n{v2}\n{v3}\n{v4}\n"


# ───────────────────────── 族 6：struct 与 struct 指针 ─────────────────────────
def fam_struct(rng):
    k = 4
    vals = [(rng.randint(-15, 15), rng.randint(-15, 15)) for _ in range(k)]
    rows = " ".join(f"a[{i}].x = {vals[i][0]}; a[{i}].y = {vals[i][1]};" for i in range(k))
    src = f"""#include <stdio.h>
struct S {{ int x; int y; }};
int main() {{
    struct S a[{k}];
    {rows}
    int s = 0;
    for (int i = 0; i < {k}; i++) {{ s += a[i].x * a[i].y; }}
    struct S* p = a;
    int t = 0;
    for (int i = 0; i < {k}; i++) {{ t += p->x - p->y; p++; }}
    printf("%d\\n", s);
    printf("%d\\n", t);
    return 0;
}}
"""
    s = sum(x * y for x, y in vals)
    t = sum(x - y for x, y in vals)
    return src, f"{s}\n{t}\n"


# ───────────────────────── 族 7：char 数组 / 字符串 ─────────────────────────
WORDS = ["hello", "abcde", "xyz12", "World", "qwert"]


def fam_char(rng):
    w = rng.choice(WORDS)
    src = f"""#include <stdio.h>
#include <string.h>
int main() {{
    char s[{len(w) + 1}] = "{w}";
    int sum = 0;
    for (int i = 0; i < {len(w)}; i++) {{ sum += s[i]; }}
    printf("%d\\n", sum);
    printf("%d\\n", (int)strlen(s));
    char d[{len(w) + 1}];
    for (int i = 0; i < {len(w)}; i++) {{ d[i] = s[{len(w) - 1} - i]; }}
    d[{len(w)}] = 0;
    printf("%s\\n", d);
    return 0;
}}
"""
    return src, f"{sum(ord(c) for c in w)}\n{len(w)}\n{w[::-1]}\n"


# ───────────────────────── 族 8：malloc 链表 ─────────────────────────
def fam_malloc_list(rng):
    k = rng.randint(2, 6)
    vals = [rng.randint(-9, 9) for _ in range(k)]
    pushes = "\n".join(
        f"    struct Node* n{i} = (struct Node*)malloc(sizeof(struct Node));"
        f" n{i}->v = {vals[i]}; n{i}->next = head; head = n{i};" for i in range(k))
    src = f"""#include <stdio.h>
#include <stdlib.h>
struct Node {{ int v; struct Node* next; }};
int main() {{
    struct Node* head = 0;
{pushes}
    int s = 0;
    for (struct Node* p = head; p != 0; p = p->next) {{ s += p->v; }}
    printf("%d\\n", s);
    struct Node* p = head;
    while (p != 0) {{ struct Node* nx = p->next; free(p); p = nx; }}
    printf("freed\\n");
    return 0;
}}
"""
    return src, f"{sum(vals)}\nfreed\n"


# ───────────────────────── 族 9：二维数组 + 嵌套循环 ─────────────────────────
def fam_nested(rng):
    r, c = rng.randint(2, 4), rng.randint(2, 4)
    src = f"""#include <stdio.h>
int main() {{
    int a[{r}][{c}];
    int s = 0;
    for (int i = 0; i < {r}; i++) {{
        for (int j = 0; j < {c}; j++) {{
            a[i][j] = i * {c} + j;
            if (j == {c - 1}) continue;
            s += a[i][j];
        }}
    }}
    int t = 0;
    for (int i = 0; i < {r}; i++) {{
        for (int j = 0; j < {c}; j++) {{
            if (a[i][j] > 5) break;
            t += a[i][j];
        }}
    }}
    printf("%d\\n", s);
    printf("%d\\n", t);
    return 0;
}}
"""
    s = sum(i * c + j for i in range(r) for j in range(c) if j != c - 1)
    t = 0
    for i in range(r):
        for j in range(c):
            if i * c + j > 5:
                break
            t += i * c + j
    return src, f"{s}\n{t}\n"


# ───────────────────────── 族 10：switch / do-while / 位运算 ─────────────────────────
def fam_switch(rng):
    n = rng.randint(3, 8)
    src = f"""#include <stdio.h>
int main() {{
    int s = 0;
    for (int i = 0; i < {n}; i++) {{
        switch (i % 4) {{
            case 0: s += 1; break;
            case 1: s += i; break;
            case 2: s -= 2;
            default: s += 3; break;
        }}
    }}
    int j = 0;
    do {{ s ^= (1 << (j % 5)); j++; }} while (j < {n});
    printf("%d\\n", s);
    return 0;
}}
"""
    s = 0
    for i in range(n):
        m = i % 4
        if m == 0:
            s += 1
        elif m == 1:
            s += i
        elif m == 2:
            s -= 2
            s += 3
        else:
            s += 3
    j = 0
    while True:
        s ^= (1 << (j % 5))
        j += 1
        if not (j < n):
            break
    return src, f"{s}\n"


FAMILIES = {"int_expr": fam_int, "unsigned": fam_unsigned,
            "array_ptr": fam_array, "control": fam_control,
            "funcs": fam_funcs, "struct": fam_struct, "char_str": fam_char,
            "malloc_list": fam_malloc_list, "nested_loop": fam_nested,
            "switch_bits": fam_switch}


def run_one(item):
    fam, idx, src, expected = item
    work = WORK / f"{fam}_{idx}"
    work.mkdir(parents=True, exist_ok=True)
    clang = sv.run_with_clang(src, path=None, work_dir=work)
    cide = sv.run_with_cide(src)
    g_clang = (clang.stdout or "").strip()
    g_cide = (cide.stdout or "").strip()
    exp = expected.strip()
    verdict = "model_clang_mismatch" if g_clang != exp else (
        "clang_cide_mismatch" if g_clang != g_cide else "agree")
    rec = {"family": fam, "index": idx, "verdict": verdict,
           "expected": exp[:200], "clang": g_clang[:200], "cide": g_cide[:200],
           "clang_compile": clang.compile_success, "cide_compile": cide.compile_success,
           "cide_err": (cide.compile_error or "")[:200]}
    if verdict != "agree":
        FINDINGS.mkdir(exist_ok=True)
        p = FINDINGS / f"{fam}_{idx}_{verdict}.c"
        p.write_text(src + f"\n/* expected={exp!r}\n   clang={g_clang!r}\n   cide={g_cide!r} */\n",
                     encoding="utf-8")
        rec["saved"] = str(p.relative_to(ROOT))
    return rec


def main() -> int:
    per, seed, jobs = 100, 20260912, 6
    if "--per-family" in sys.argv:
        per = int(sys.argv[sys.argv.index("--per-family") + 1])
    if "--seed" in sys.argv:
        seed = int(sys.argv[sys.argv.index("--seed") + 1])
    if "--jobs" in sys.argv:
        jobs = int(sys.argv[sys.argv.index("--jobs") + 1])
    print(f"per_family={per} seed={seed} jobs={jobs}")
    items = []
    for fam, fn in FAMILIES.items():
        rng = random.Random(f"{seed}:{fam}")
        made = 0
        attempt = 0
        while made < per and attempt < per * 5:
            attempt += 1
            r = fn(rng)
            if r is None:
                continue
            items.append((fam, made, r[0], r[1]))
            made += 1
    with ThreadPoolExecutor(max_workers=jobs) as ex:
        results = list(ex.map(run_one, items))

    from collections import Counter
    cnt = Counter(r["verdict"] for r in results)
    print(f"\n总样本 {len(results)}")
    for k, v in cnt.most_common():
        print(f"  {k}: {v}")
    for fam in FAMILIES:
        sub = [r for r in results if r["family"] == fam]
        c = Counter(r["verdict"] for r in sub)
        print(f"  [{fam:10s}] {dict(c)}")
    bad = [r for r in results if r["verdict"] != "agree"]
    for r in bad[:25]:
        print(f"    ! {r['family']}/{r['index']}: {r['verdict']}")
        print(f"      expected={r['expected'][:80]!r}")
        print(f"      clang   ={r['clang'][:80]!r}")
        print(f"      cide    ={r['cide'][:80]!r}")
    p = HERE / "random_diff.json"
    p.write_text(json.dumps(results, ensure_ascii=False, indent=1), encoding="utf-8")
    print(f"\nJSON 已写出: {p}\n最小复现目录: {FINDINGS}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
