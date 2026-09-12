# -*- coding: utf-8 -*-
"""seek 内存累积判定：峰值 vs 常驻（区分"瞬时尖峰"与"真泄漏"）。

resource_longrun.py 已实测 seek 重放的峰值提交内存斜率（~1.2-2.3MB/千步）。
本探针回答更关键的问题：**同一会话内反复 seek 之后，内存是否回落？**
  * 若每次 seek 后 commit 单调抬升 → 累积泄漏（复现事故形态：磁盘被页面文件占满）；
  * 若 seek 完成后回落 → 只是瞬时窗口放大（仍是风险，但性质不同）。

同时测 malloc/free 的**耗时增长指数**（50k/100k/200k/400k），对既有文档
"O(N²)" 的说法做独立复核（本裁定实测结论可能与既有文档不一致，按诚实纪律记录）。

用法：python scripts/core_asset_verdict/seek_accumulation.py [--cap-mb 1200]
输出：seek_accumulation.json。标注 [动态实测]。
"""
import io
import json
import subprocess
import sys
import threading
import time
from pathlib import Path

sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from winmem import commit_mb  # noqa: E402

ROOT = HERE.parent.parent
CLI = ROOT / "native/target/release/cide_cli.exe"
WORK = HERE / ".longrun"
WORK.mkdir(exist_ok=True)
CAP = 1200.0


class Session:
    def __init__(self, cap=CAP):
        self.p = subprocess.Popen([str(CLI), "serve"], stdin=subprocess.PIPE,
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                  text=True, encoding="utf-8", errors="replace")
        self.rid = 0
        self.cap = cap
        self.killed = False
        self.samples = []
        self.stop = False
        self.t = threading.Thread(target=self._watch, daemon=True)
        self.t.start()

    def _watch(self):
        while not self.stop and self.p.poll() is None:
            c = commit_mb(self.p.pid)
            if c > 0:
                self.samples.append(c)
                if c > self.cap:
                    self.killed = True
                    subprocess.run(["taskkill", "/PID", str(self.p.pid), "/F"],
                                   capture_output=True)
                    return
            time.sleep(0.03)

    def req(self, method, params=None):
        if self.p.poll() is not None:
            return None
        self.rid += 1
        r = {"id": self.rid, "method": method}
        if params is not None:
            r["params"] = params
        self.p.stdin.write(json.dumps(r) + "\n")
        self.p.stdin.flush()
        line = self.p.stdout.readline()
        return json.loads(line) if line else None

    def commit(self):
        return commit_mb(self.p.pid)

    def close(self):
        self.stop = True
        try:
            self.p.stdin.close()
        except Exception:
            pass
        try:
            self.p.wait(timeout=5)
        except Exception:
            self.p.kill()
        err = ""
        try:
            err = self.p.stderr.read()[-400:]
        except Exception:
            pass
        return err


def seek_accumulation():
    iters = 50000
    src = ('#include <stdio.h>\nint main(){int s=0;\n'
           f'for(int i=0;i<{iters};i++){{s+=i%7;}}\nprintf("%d\\n",s);return 0;}}\n')
    s = Session()
    out = {"iters": iters, "steps_est": iters * 10, "timeline": []}
    s.req("compile", {"source": src})
    s.req("step.begin")
    targets = [100000, 300000, 450000, 200000, 100000, 450000]
    for t in targets:
        before = s.commit()
        t0 = time.time()
        r = s.req("seek", {"step": t})
        after = s.commit()
        time.sleep(0.3)
        settled = s.commit()
        out["timeline"].append({
            "seek": t, "before_mb": before, "after_mb": after, "settled_mb": settled,
            "wall_s": round(time.time() - t0, 2),
            "success": (r or {}).get("result", {}).get("success") if r else None,
        })
        print(f"  seek({t:>7}) before={before:8.1f}MB after={after:8.1f}MB "
              f"settled={settled:8.1f}MB ({out['timeline'][-1]['wall_s']}s)", flush=True)
    out["killed_by_watchdog"] = s.killed
    out["peak_commit_mb"] = max(s.samples) if s.samples else -1
    out["stderr_tail"] = s.close()
    return out


def malloc_timing():
    out = []
    for n in (50000, 100000, 200000, 400000):
        src = ('#include <stdio.h>\n#include <stdlib.h>\nint main(){\n'
               f'for(int i=0;i<{n};i++){{int* p=(int*)malloc(16); *p=i; free(p);}}\n'
               'printf("done\\n");return 0;}\n')
        f = WORK / f"mt_{n}.c"
        f.write_text(src, encoding="utf-8")
        t0 = time.time()
        p = subprocess.run([str(CLI), "run", str(f)], capture_output=True, text=True,
                           encoding="utf-8", errors="replace", timeout=900)
        wall = time.time() - t0
        out.append({"n": n, "wall_s": round(wall, 2), "exit": p.returncode,
                    "stdout_tail": (p.stdout or "")[-40:]})
        print(f"  malloc n={n:>7} wall={wall:6.2f}s exit={p.returncode}", flush=True)
    if len(out) >= 2:
        import math
        a, b = out[0], out[-1]
        out.append({"exponent_estimate": round(
            math.log(b["wall_s"] / a["wall_s"]) / math.log(b["n"] / a["n"]), 3),
            "basis": f"{a['n']}->{b['n']}"})
    return out


def main() -> int:
    print("== A. 同一会话反复 seek：峰值 vs 常驻 ==")
    a = seek_accumulation()
    print(f"  看门狗触发={a['killed_by_watchdog']}  峰值提交={a['peak_commit_mb']}MB")
    print("== B. malloc/free 耗时增长 ==")
    b = malloc_timing()
    p = HERE / "seek_accumulation.json"
    p.write_text(json.dumps({"seek": a, "malloc_timing": b}, ensure_ascii=False, indent=1),
                 encoding="utf-8")
    print(f"\nJSON 已写出: {p}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
