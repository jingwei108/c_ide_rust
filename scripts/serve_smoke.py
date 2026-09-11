# -*- coding: utf-8 -*-
"""`cide_cli serve`（JSON-lines 会话模式）端到端冒烟 + 协议断言。

防线定位：出口 3 的协议契约验证 —— id 关联 / 错误帧同构 / session.reset /
与 capi 共用入口语义（StepPayload 字段、隔离预算默认值等）。

运行：`python scripts/serve_smoke.py`（需先构建 cide_cli：`cargo build --bin cide_cli`）
或经环境变量指定可执行文件：`CIDE_CLI=/path/to/cide_cli python scripts/serve_smoke.py`
"""
import json
import os
import subprocess
import io
import sys
# Windows CI 控制台默认 cp1252，编码不了中文/✓ 等字符（UnicodeEncodeError）。
# 与 shadow_verify.py 的入口处理一致：显式把 stdout/stderr 重包为 UTF-8。
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding="utf-8", errors="replace")

from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent


def resolve_exe() -> Path:
    override = os.environ.get("CIDE_CLI")
    if override:
        return Path(override)
    name = "cide_cli.exe" if sys.platform == "win32" else "cide_cli"
    debug = PROJECT_ROOT / "native" / "target" / "debug" / name
    release = PROJECT_ROOT / "native" / "target" / "release" / name
    return debug if debug.exists() else release


PROGRAM = (
    '#include <stdio.h>\n'
    'int main(){ int a = 1; int b = 2; printf("%d", a + b); return 0; }\n'
)

REQUESTS = [
    {"id": 1, "method": "ping"},
    {"id": 2, "method": "compile", "params": {"source": PROGRAM}},
    {"id": 3, "method": "run"},
    {"id": 4, "method": "output.delta", "params": {"cursor": 0}},
    {"id": 5, "method": "step.begin"},
    {"id": 6, "method": "step.next"},
    {"id": 7, "method": "step.next"},
    {"id": 8, "method": "payload.get", "params": {"start": 0, "end": 50}},
    {"id": 9, "method": "breakpoints.set", "params": {"lines": [3]}},
    {"id": 10, "method": "seek", "params": {"step": 1}},
    {"id": 11, "method": "memory.regions"},
    {"id": 12, "method": "config.set", "params": {"quarantine_budget": 0}},
    {"id": 13, "method": "compile", "params": {"source": "int main(){ int x = ; }"}},
    {"id": 14, "method": "no.such.method"},
    {"id": 15, "method": "session.reset"},
    {"id": 16, "method": "shutdown"},
]

DEFAULT_QUARANTINE_BUDGET = 256 * 1024  # 1MB 堆上限的 1/4（堆决议 §1）

failures = []


def check(cond, label, detail=""):
    if cond:
        print(f"  PASS  {label}")
    else:
        print(f"  FAIL  {label}  {detail}")
        failures.append(label)


def main():
    exe = resolve_exe()
    if not exe.exists():
        print(f"错误: 找不到 {exe}，请先 `cd native && cargo build --bin cide_cli`")
        return 2
    print(f"cide_cli: {exe}")

    payload = "\n".join(json.dumps(r, ensure_ascii=False) for r in REQUESTS) + "\n"
    proc = subprocess.run(
        [str(exe), "serve"],
        input=payload,
        capture_output=True,
        text=True,
        encoding="utf-8",
        timeout=120,
    )
    print(f"exit={proc.returncode}")
    lines = [l for l in proc.stdout.splitlines() if l.strip()]
    print(f"responses={len(lines)} (requests={len(REQUESTS)})")
    check(proc.returncode == 0, "进程正常退出", proc.stderr[:300])
    check(len(lines) == len(REQUESTS), "每个请求一行响应")

    responses = []
    for i, line in enumerate(lines):
        try:
            responses.append(json.loads(line))
        except json.JSONDecodeError as e:
            check(False, f"响应 {i} 是合法 JSON", f"{e}: {line[:120]}")
            return 1

    # id 关联 + 帧同构（每帧都有 id/ok，二选一携带 result/error）
    check(
        [r.get("id") for r in responses] == [r["id"] for r in REQUESTS],
        "响应 id 与请求一一对应",
        str([r.get("id") for r in responses]),
    )
    check(
        all(("result" in r) ^ ("error" in r) for r in responses),
        "帧同构：result / error 二选一",
    )
    check(
        all(r.get("ok") is (("result" in r)) for r in responses),
        "ok 与 result/error 一致",
    )

    by_id = {r["id"]: r for r in responses}

    check(by_id[1]["result"].get("pong") is True, "ping 回应 pong")
    check(bool(by_id[1]["result"].get("abi")), "ping 携带 ABI 版本")

    check(by_id[2]["ok"] and by_id[2]["result"]["ok"] is True, "compile 成功")
    check(isinstance(by_id[2]["result"]["diagnostics"], list), "compile 返回 diagnostics 数组")

    check(by_id[3]["result"]["status"] == "finished", "run 正常结束", str(by_id[3]))
    delta = by_id[4]["result"]["delta"]
    check("3" in delta, "output.delta 含程序输出", repr(delta[:80]))
    check(by_id[4]["result"]["cursor"] == by_id[4]["result"]["total"], "游标推进到末尾")

    step = by_id[6]["result"]
    check(isinstance(step.get("payloads"), list) and bool(step["payloads"]), "step.next 返回 payload")
    payload_keys = set(step["payloads"][0].keys())
    check(
        {"step_index", "code_line", "func_name", "local_vars", "pointer_snapshots"} <= payload_keys,
        "StepPayload 含 schema 字段",
        str(sorted(payload_keys)),
    )

    pg = by_id[8]["result"]
    check("cache_start_step" in pg and "max_collected_step" in pg, "payload.get 携带窗口字段")

    check(by_id[9]["result"]["lines"] == [3], "breakpoints.set 回显行号")
    check(by_id[10]["result"].get("success") is True, "seek 成功", str(by_id[10])[:200])

    regions = by_id[11]["result"]
    check("regions" in regions and "quarantine" in regions, "memory.regions 结构完整")
    check(
        regions["quarantine"]["budget"] == DEFAULT_QUARANTINE_BUDGET,
        "默认隔离预算 256KB（与 capi 一致）",
        str(regions["quarantine"]),
    )

    check(by_id[12]["result"]["quarantine_budget"] == 0, "config.set 生效（预算可调）")

    diag_bad = by_id[13]["result"]
    check(diag_bad["ok"] is False and bool(diag_bad["diagnostics"]), "非法程序编译失败并给诊断")

    err = by_id[14]["error"]
    check(err["kind"] == "protocol" and "未知方法" in err["message"], "未知方法 → protocol 错误帧")

    check(by_id[15]["result"]["reset"] is True, "session.reset 成功")
    check(by_id[15]["result"]["config"]["compiled"] is False, "reset 后 compiled=false")
    check(
        by_id[15]["result"]["config"]["quarantine_budget"] == 0,
        "reset 保留会话级配置（隔离预算）",
    )
    check(by_id[16]["result"]["shutdown"] is True, "shutdown 回应")

    print()
    if failures:
        print(f"FAILED: {len(failures)} 项 -> {failures}")
        return 1
    print("serve 冒烟全部通过")
    return 0


if __name__ == "__main__":
    sys.exit(main())
