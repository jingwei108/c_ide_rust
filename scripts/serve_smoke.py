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
    {"id": 16, "method": "capabilities"},
    {"id": 17, "method": "semantic_labels"},
    {"id": 18, "method": "contracts"},
    {"id": 19, "method": "session.create"},
    {"id": 20, "method": "shutdown"},
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
    # C2：三段式内存地图（kind + region_counts + 栈/全局的 name/alloc_line）
    counts = regions.get("region_counts") or {}
    check(
        {"global", "stack", "heap"} <= set(counts.keys()),
        "memory.regions 三段式计数（C2）",
        str(counts),
    )
    check(
        all(isinstance(r.get("kind"), str) for r in regions["regions"]),
        "每个 region 都带 kind 段标识（C2）",
        str([r.get("kind") for r in regions["regions"]]),
    )
    stack_regions = [r for r in regions["regions"] if r.get("kind") == "stack"]
    check(
        counts.get("stack", 0) >= 1 and any(r.get("name") == "main" for r in stack_regions),
        "栈帧区域带函数名（C2）",
        str(stack_regions[:2]),
    )
    check(
        all(r.get("alloc_by") == "call" and r.get("alloc_line") is not None for r in stack_regions),
        "栈帧区域带 alloc_by=call / alloc_line（C2）",
        str(stack_regions[:2]),
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
    check(by_id[16]["ok"] is True, "capabilities 可用")
    caps = by_id[16]["result"]
    check(
        caps.get("languages", {}).get("c", {}).get("stdc_version_macro_nominal") == "202311L",
        "capabilities 版本宏名义锚点",
    )
    check(
        caps.get("memory_model", {}).get("global_region_limit") == 65536,
        "capabilities 内存模型常量（单源 cide_runtime）",
    )
    # B2：schema 轨道与行为契约进能力清单（消费方可直读版本协商信息）
    check(
        caps.get("schema", {}).get("version") == "v0.1"
        and caps.get("schema", {}).get("reserved_fields_v0_2")
        == ["handler_depth", "unwinding", "unwind_frames_left", "current_exception"],
        "capabilities 携带 schema 轨道与预留位（B2）",
        str(caps.get("schema")),
    )
    check(
        any(c.get("id") == "unwinding_step_granularity" for c in caps.get("behavior_contracts", [])),
        "capabilities 携带行为契约（B2：UNWINDING 不合并单步）",
    )

    # B2：词汇表导出（词汇只增不改；异常域条目以 reserved 预登记）
    labels = by_id[17]["result"]
    label_ids = [l.get("id") for l in labels.get("labels", [])]
    check(
        "swap" in label_ids and "loop" in label_ids,
        "semantic_labels 导出 C 域词汇（B2）",
        str(label_ids),
    )
    check(
        {"throw", "unwind", "catch_enter", "finally"}
        <= {l.get("id") for l in labels.get("labels", []) if l.get("status") == "reserved"},
        "semantic_labels 预登记异常域词汇（reserved，B2）",
    )

    # B2：契约导出（预留位 + v0.2 台账 + 激活清单）
    contracts = by_id[18]["result"]
    check(
        len(contracts.get("v0_2_activation_checklist", [])) >= 4
        and any(f.get("field") == "code_file" for f in contracts.get("v0_2_field_ledger", [])),
        "contracts 导出激活清单与 v0.2 台账（B2）",
    )

    # D2：单 serve 进程 = 单活跃会话，响应显式回带拓扑字段
    created = by_id[19]["result"]
    sess = created.get("session") or {}
    check(
        created.get("created") is True
        and sess.get("model") == "single-active-session"
        and sess.get("active_sessions") == 1
        and sess.get("concurrent_sessions") is False,
        "session.create 显式回带单会话语义（D2）",
        str(sess),
    )
    check(by_id[20]["result"]["shutdown"] is True, "shutdown 回应")

    print()
    if failures:
        print(f"FAILED: {len(failures)} 项 -> {failures}")
        return 1
    print("serve 冒烟全部通过")
    return 0


if __name__ == "__main__":
    sys.exit(main())
