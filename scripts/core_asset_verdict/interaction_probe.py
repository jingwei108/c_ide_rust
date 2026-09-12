# -*- coding: utf-8 -*-
"""交互切面探针：随机交互序列 + 恶意输入 fuzz（影子防线从未测过的切面）。

影子防线只做「compile → run → 比 stdout」的批处理；本探针走 `cide_cli serve` 的
JSON-lines 会话，做两件影子防线做不到的事：

  A. **随机交互序列**：step / seek / payload.get / breakpoints / memory.regions /
     config.set / session.reset / input.feed 按固定种子随机交错，检查
     （1）进程存活、（2）每请求一行合法 JSON、（3）id 关联、
     （4）步级 payload 的结构不变量（step_index 单调、code_line 在源码行域内、
         pointer_snapshots[].status ∈ 四态、local_vars 名字是合法标识符）。
     这些不变量是**内部一致性**，不是"标注对不对"——交互切面没有独立 oracle
     （这正是 Q2/Q3 要量化的事实）。

  B. **恶意输入 fuzz**：畸形 JSON / 缺字段 / 类型错 / 负参 / 巨数 / 深嵌套，
     断言"一行畸形输入不得到达 panic"（路线图 U0 验收标准）。

宿主内存由驱动侧 psapi 采样（winmem.py），不采信被测代码自报。

用法：python scripts/core_asset_verdict/interaction_probe.py [--seed 20260912] [--ops 200] [--rss]
输出：interaction_probe.json + 控制台摘要。标注 [动态实测]。
"""
import io
import json
import random
import subprocess
import sys
import threading
import time
from pathlib import Path

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from winmem import commit_mb, peak_commit_mb, working_set_mb  # noqa: E402

ROOT = HERE.parent.parent
NATIVE = ROOT / "native"
CLI = NATIVE / "target/release/cide_cli.exe"
if not CLI.exists():
    CLI = NATIVE / "target/debug/cide_cli.exe"

CASES_DIR = NATIVE / "tests/cases/baseline"
CURATED = [
    "doubly_linked_list", "bst_insert_search", "tree_level_order",
    "linked_queue", "linked_stack", "switch_case", "float_basic", "for_empty_cond",
]

LONG_PROG = """#include <stdio.h>
int main() {
    int s = 0;
    for (int i = 0; i < 5000; i++) { s += i % 7; }
    printf("%d\\n", s);
    return 0;
}
"""

VALID_STATUS = {"Valid", "Freed", "Null", "Dangling"}
IDENT = __import__("re").compile(r"^[A-Za-z_]\w*$")
HARD_DEADLINE = 90.0


class Serve:
    def __init__(self, tag: str):
        self.tag = tag
        self.err_path = HERE / f".serve_{tag}.err.log"
        self.err_f = open(self.err_path, "w", encoding="utf-8")
        self.proc = subprocess.Popen(
            [str(CLI), "serve"], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=self.err_f, text=True, encoding="utf-8", errors="replace",
        )
        self.rid = 0
        self.dead = False
        self.start = time.time()
        threading.Thread(target=self._watchdog, daemon=True).start()

    def _watchdog(self):
        while not self.dead and self.proc.poll() is None:
            if time.time() - self.start > HARD_DEADLINE:
                self.proc.kill()
                return
            time.sleep(0.2)

    def req(self, method, params=None):
        if self.proc.poll() is not None:
            self.dead = True
            return None
        self.rid += 1
        r = {"id": self.rid, "method": method}
        if params is not None:
            r["params"] = params
        try:
            self.proc.stdin.write(json.dumps(r, ensure_ascii=False) + "\n")
            self.proc.stdin.flush()
            line = self.proc.stdout.readline()
        except Exception:
            self.dead = True
            return None
        if not line:
            self.dead = True
            return None
        try:
            resp = json.loads(line)
        except json.JSONDecodeError:
            return {"__badjson__": line[:200], "__id__": r["id"]}
        resp["__id__"] = r["id"]
        return resp

    def close(self):
        self.dead = True
        try:
            self.proc.stdin.close()
        except Exception:
            pass
        try:
            self.proc.wait(timeout=5)
        except Exception:
            self.proc.kill()
        self.err_f.close()

    def stderr_tail(self, n=1500):
        try:
            return self.err_path.read_text(encoding="utf-8", errors="replace")[-n:]
        except OSError:
            return ""


def check_payload_invariants(resp, src_lines, viol):
    """步级 payload 内部不变量（非独立 oracle）。"""
    res = resp.get("result") or {}
    arr = res.get("payloads")
    if not isinstance(arr, list):
        return
    prev = None
    for p in arr:
        if not isinstance(p, dict):
            viol.append(("payload_not_object", str(p)[:80]))
            continue
        si = p.get("step_index")
        if not isinstance(si, int):
            viol.append(("step_index_not_int", str(si)))
        elif prev is not None and si < prev:
            viol.append(("step_index_not_monotonic", f"{prev}->{si}"))
        elif isinstance(si, int):
            prev = si
        cl = p.get("code_line")
        if isinstance(cl, int) and src_lines and not (0 <= cl <= src_lines + 2):
            viol.append(("code_line_out_of_range", f"line={cl} src_lines={src_lines}"))
        for v in (p.get("local_vars") or []):
            nm = v.get("name") if isinstance(v, dict) else None
            if isinstance(nm, str) and nm and not IDENT.match(nm):
                viol.append(("local_var_bad_name", nm))
        for ps in (p.get("pointer_snapshots") or []):
            st = ps.get("status") if isinstance(ps, dict) else None
            if isinstance(st, str) and st not in VALID_STATUS:
                viol.append(("pointer_status_unknown", st))
        lbl = p.get("semantic_label")
        if lbl is not None and not isinstance(lbl, str):
            viol.append(("semantic_label_not_str", str(lbl)))


def part_a(seed: int, ops: int, do_rss: bool) -> dict:
    rng = random.Random(seed)
    programs = []
    for name in CURATED:
        p = CASES_DIR / f"{name}.c"
        if p.exists():
            programs.append((name, p.read_text(encoding="utf-8")))
    programs.append(("long_loop_5000", LONG_PROG))

    out = {"ops_total": 0, "reqs": 0, "dead_sessions": 0, "badjson": 0,
           "protocol_errors": 0, "state_errors": 0, "invariant_violations": [],
           "per_program": [], "rss_samples": [], "status_hist": {}}
    for name, src in programs:
        s = Serve(f"a_{name}")
        viol = []
        n_ops = 0
        max_step_seen = -1
        max_collected = -1
        rss0 = commit_mb(s.proc.pid) if do_rss else -1
        rss_peak = rss0
        src_lines = len(src.splitlines())
        got = s.req("compile", {"source": src})
        if got is None:
            out["dead_sessions"] += 1
            out["per_program"].append({"program": name, "died_at": "compile",
                                       "stderr": s.stderr_tail()})
            s.close()
            continue
        got = s.req("step.begin")
        if got is None:
            out["dead_sessions"] += 1
            out["per_program"].append({"program": name, "died_at": "step.begin",
                                       "stderr": s.stderr_tail()})
            s.close()
            continue
        for _ in range(ops):
            n_ops += 1
            op = rng.choices(
                ["step.next", "seek", "payload.get", "breakpoints.set",
                 "memory.regions", "output.delta", "config.set", "input.feed"],
                weights=[55, 8, 12, 5, 10, 4, 3, 3])[0]
            params = None
            if op == "seek":
                params = {"step": rng.choice([0, 1, 5, 50, 200, 1000, 4999, 50000])}
            elif op == "payload.get":
                start = rng.choice([0, 1, 10, max_step_seen if max_step_seen > 0 else 0])
                params = {"start": start, "end": start + rng.choice([1, 10, 500, 5000])}
            elif op == "breakpoints.set":
                params = {"lines": rng.sample(range(1, max(2, src_lines + 1)),
                                              k=min(3, max(1, src_lines)))}
            elif op == "output.delta":
                params = {"cursor": rng.choice([0, 1, 100])}
            elif op == "config.set":
                params = {"max_steps": rng.choice([1000, 100000, 10000000])}
            elif op == "input.feed":
                params = {"text": "1 2 3\n"}
            resp = s.req(op, params)
            if resp is None:
                out["dead_sessions"] += 1
                out["per_program"].append({
                    "program": name, "died_after_ops": n_ops, "died_at": op,
                    "stderr": s.stderr_tail()})
                break
            out["reqs"] += 1
            if "__badjson__" in resp:
                out["badjson"] += 1
                continue
            if resp.get("id") != resp.get("__id__"):
                viol.append(("id_mismatch", f"{resp.get('id')} != {resp.get('__id__')}"))
            if ("result" in resp) == ("error" in resp):
                viol.append(("frame_shape", json.dumps(resp, ensure_ascii=False)[:120]))
            if "error" in resp:
                kind = (resp["error"] or {}).get("kind")
                if kind == "protocol":
                    out["protocol_errors"] += 1
                else:
                    out["state_errors"] += 1
            if op == "step.next":
                check_payload_invariants(resp, src_lines, viol)
                res = resp.get("result") or {}
                for p in (res.get("payloads") or []):
                    if isinstance(p, dict) and isinstance(p.get("step_index"), int):
                        max_step_seen = max(max_step_seen, p["step_index"])
                if isinstance(res.get("max_collected_step"), int):
                    max_collected = max(max_collected, res["max_collected_step"])
            elif op == "payload.get":
                res = resp.get("result") or {}
                if isinstance(res.get("max_collected_step"), int):
                    max_collected = max(max_collected, res["max_collected_step"])
            if do_rss and n_ops % 25 == 0:
                c = commit_mb(s.proc.pid)
                if c > 0:
                    rss_peak = max(rss_peak, c)
                    out["rss_samples"].append({"program": name, "op": n_ops, "commit_mb": c})
        out["ops_total"] += n_ops
        out["per_program"].append({
            "program": name, "ops": n_ops, "alive": s.proc.poll() is None,
            "max_step_seen": max_step_seen, "max_collected_step": max_collected,
            "commit_mb_start": rss0, "commit_mb_peak": rss_peak,
            "commit_mb_peak_psapi": peak_commit_mb(s.proc.pid) if do_rss else -1,
        })
        out["invariant_violations"].extend([{"program": name, "kind": k, "detail": d}
                                            for k, d in viol])
        s.close()
    return out


MALFORMED = [
    '{"id":1,"method":"ping"}',                      # 合法对照
    'not json at all',
    '{"id":2,"method":',                             # 截断
    '{"id":3}',                                      # 缺 method
    '{"id":4,"method":123}',                         # method 类型错
    '{"id":5,"method":"ping","params":[]}',          # params 类型错
    '{"id":6,"method":"no.such.method"}',            # 未知方法
    '{"id":7,"method":"compile"}',                   # 缺 source
    '{"id":8,"method":"compile","params":{"source":123}}',
    '{"id":9,"method":"payload.get","params":{"start":0,"end":-1}}',
    '{"id":10,"method":"payload.get","params":{"start":-2147483648,"end":2147483647}}',
    '{"id":11,"method":"payload.get","params":{"start":0,"end":-9223372036854775808}}',
    '{"id":12,"method":"seek","params":{"step":-2147483648}}',
    '{"id":13,"method":"seek","params":{"step":999999999999}}',
    '{"id":14,"method":"seek","params":{}}',
    '{"id":15,"method":"breakpoints.set","params":{"lines":[-1,0,2147483647]}}',
    '{"id":16,"method":"config.set","params":{"max_steps":-1}}',
    '{"id":17,"method":"config.set","params":{"quarantine_budget":-99999999}}',
    '{"id":18,"method":"config.set","params":{"call_depth_limit":0}}',
    '{"id":19,"method":"session.reset"}',
    '{"id":20,"method":"step.begin"}',
    '{"id":21,"method":"step.next"}',                # 未编译就步进
    '{"id":22,"method":"memory.regions"}',
    '{"id":23,"method":"run","params":{"argv":["a"]*10000}}',
    '{"id":24,"method":"compile","params":{"source":"' + "int x=" * 2000 + '1;"}}',
    '{"id":25,"method":"compile","params":{"source":"#define A(x) x x\\nA(A(A(A(1))))"}}',
    '[' * 200 + ']' * 200,                           # 深嵌套 JSON
    '{"id":26,"method":"ping","params":{"nested":' + '[' * 500 + ']' * 500 + '}}',
    '{"id":27,"method":"semantic_labels"}',
    '{"id":28,"method":"shutdown"}',
]


def part_b() -> dict:
    s = Serve("b_fuzz")
    out = {"inputs": len(MALFORMED), "responses": 0, "badjson_out": 0,
           "died": False, "died_at": None, "stderr": "", "responses_detail": []}
    # 先做一次正常编译+步进，让 payload.get 的负参路径真的可达
    src = LONG_PROG
    s.req("compile", {"source": src})
    s.req("step.begin")
    for i, line in enumerate(MALFORMED, 1):
        if s.proc.poll() is not None:
            out["died"] = True
            out["died_at"] = f"before input #{i}: {line[:60]}"
            break
        try:
            s.proc.stdin.write(line + "\n")
            s.proc.stdin.flush()
            resp_line = s.proc.stdout.readline()
        except Exception as e:
            out["died"] = True
            out["died_at"] = f"input #{i} write/read: {e}"
            break
        if not resp_line:
            out["died"] = True
            out["died_at"] = f"input #{i} (no response): {line[:80]}"
            break
        out["responses"] += 1
        try:
            r = json.loads(resp_line)
            out["responses_detail"].append({"i": i, "ok": r.get("ok"),
                                            "kind": (r.get("error") or {}).get("kind")})
        except json.JSONDecodeError:
            out["badjson_out"] += 1
            out["responses_detail"].append({"i": i, "raw": resp_line[:120]})
    out["alive_at_end"] = s.proc.poll() is None
    out["stderr"] = s.stderr_tail(2000)
    s.close()
    return out


def main() -> int:
    seed, ops, do_rss = 20260912, 200, "--rss" in sys.argv
    if "--seed" in sys.argv:
        seed = int(sys.argv[sys.argv.index("--seed") + 1])
    if "--ops" in sys.argv:
        ops = int(sys.argv[sys.argv.index("--ops") + 1])
    print(f"cide_cli: {CLI}\nseed={seed} ops/程序={ops} rss采样={do_rss}\n")
    a = part_a(seed, ops, do_rss)
    print("== A 随机交互序列 ==")
    print(f"  请求总数 {a['reqs']}  会话死亡 {a['dead_sessions']}  非法响应 {a['badjson']}")
    print(f"  protocol 错误 {a['protocol_errors']}  state 错误 {a['state_errors']}")
    print(f"  不变量违反 {len(a['invariant_violations'])} 条")
    for v in a["invariant_violations"][:20]:
        print(f"    - [{v['program']}] {v['kind']}: {v['detail']}")
    for pp in a["per_program"]:
        print(f"  {pp.get('program'):22s} {json.dumps({k: v for k, v in pp.items() if k != 'program'}, ensure_ascii=False)}")
    b = part_b()
    print("\n== B 恶意输入 fuzz ==")
    print(f"  输入 {b['inputs']}  得到响应 {b['responses']}  输出非法 JSON {b['badjson_out']}")
    print(f"  serve 进程在 fuzz 中死亡: {b['died']}  位置: {b['died_at']}")
    print(f"  fuzz 结束后仍存活: {b['alive_at_end']}")
    if b["stderr"].strip():
        print("  stderr 尾部:")
        print("   " + b["stderr"].replace("\n", "\n   ")[-1200:])
    out = {"seed": seed, "ops": ops, "part_a": a, "part_b": b}
    p = HERE / "interaction_probe.json"
    p.write_text(json.dumps(out, ensure_ascii=False, indent=1), encoding="utf-8")
    print(f"\nJSON 已写出: {p}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
