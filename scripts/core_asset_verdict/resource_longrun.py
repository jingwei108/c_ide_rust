# -*- coding: utf-8 -*-
"""资源长跑探针：宿主 RSS 由**驱动侧**采样（psapi），不信被测代码自报。

**已被 Go 版取代（2026-09-12，D5 探针集）**：`go run scripts/core_asset_verdict/resource_longrun.go`
（结构/机制对账一致；数值为测量值，双轨不要求相等）。本文件保留为**双轨对照基准**。

覆盖三条"宿主资源域"路径（路线图 U2 的对象），全部用规模 realism 的参数
（真实教学语料上界方向，而非几十步小程序）：

  1. seek 重放放大：N 步程序 + 一次远距 seek → 采样提交内存随步数增长的斜率
     （即事故 INCIDENT-2026-09-SEEK-REPLAY-LEAK 的机制复现）。
  2. malloc/free 循环：regions 登记表只增不删（R4）→ 内存与耗时随 N 的变化
     （耗时若超线性即 O(N²) 扫描的实证）。
  3. putchar 1M：output_chunks 每字符一个 String（R11）。

安全：所有子进程带硬内存看门狗（默认 1500MB 即击杀并记录），避免重演 63.6GB 事故。

用法：python scripts/core_asset_verdict/resource_longrun.py [--cap-mb 1500]
输出：resource_longrun.json + 控制台摘要。标注 [动态实测]。
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
from winmem import commit_mb, peak_commit_mb  # noqa: E402

ROOT = HERE.parent.parent
CLI = ROOT / "native/target/release/cide_cli.exe"
if not CLI.exists():
    CLI = ROOT / "native/target/debug/cide_cli.exe"
WORK = HERE / ".longrun"
WORK.mkdir(exist_ok=True)


class Sampler:
    """驱动侧采样线程 + 硬看门狗。"""

    def __init__(self, pid: int, cap_mb: float):
        self.pid, self.cap_mb = pid, cap_mb
        self.peak = 0.0
        self.samples = []
        self.killed = False
        self.stop = False
        self.t = threading.Thread(target=self._run, daemon=True)
        self.t.start()

    def _run(self):
        while not self.stop:
            c = commit_mb(self.pid)
            if c > 0:
                self.peak = max(self.peak, c)
                self.samples.append(c)
                if c > self.cap_mb and not self.killed:
                    self.killed = True
                    subprocess.run(["taskkill", "/PID", str(self.pid), "/F"],
                                   capture_output=True)
                    return
            time.sleep(0.05)

    def finish(self):
        self.stop = True
        self.t.join(timeout=2)
        return self.peak


def sample_run(cmd, timeout, cap_mb, stdin_lines=None):
    """跑一个子进程（非交互），驱动侧采样其提交内存。"""
    t0 = time.time()
    p = subprocess.Popen(cmd, stdin=subprocess.PIPE if stdin_lines else subprocess.DEVNULL,
                         stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                         text=True, encoding="utf-8", errors="replace")
    s = Sampler(p.pid, cap_mb)
    try:
        out, err = p.communicate(
            input=("\n".join(stdin_lines) + "\n") if stdin_lines else None, timeout=timeout)
    except subprocess.TimeoutExpired:
        p.kill()
        out, err = p.communicate()
    peak = s.finish()
    return {"wall_s": round(time.time() - t0, 2), "exit": p.returncode,
            "commit_peak_mb": peak, "peak_psapi_mb": peak_commit_mb(p.pid) if p.poll() is not None else -1,
            "killed_by_watchdog": s.killed,
            "stdout_tail": (out or "")[-200:], "stderr_tail": (err or "")[-200:]}


def seek_scaling(cap_mb: float) -> list:
    """seek 重放放大：步数 N → 提交内存斜率。"""
    out = []
    for iters in (5000, 20000, 50000):
        src = ('#include <stdio.h>\nint main(){int s=0;\n'
               f'for(int i=0;i<{iters};i++){{s+=i%7;}}\n'
               'printf("%d\\n",s);return 0;}\n')
        n_est = iters * 10  # 经验值：每轮约 10 条 VM 步
        f = WORK / f"seek_{iters}.c"
        f.write_text(src, encoding="utf-8")
        # 用 serve：compile → step.begin → seek(远距但仍在程序内)
        reqs = [
            {"id": 1, "method": "compile", "params": {"source": src}},
            {"id": 2, "method": "step.begin"},
            {"id": 3, "method": "seek", "params": {"step": n_est - 100}},
        ]
        payload = "\n".join(json.dumps(r) for r in reqs) + "\n"
        r = sample_run([str(CLI), "serve"], timeout=300, cap_mb=cap_mb,
                       stdin_lines=payload.splitlines())
        r.update({"case": f"seek_iters{iters}", "est_steps": n_est,
                  "mb_per_1k_steps": round(r["commit_peak_mb"] / max(1, n_est) * 1000, 3)})
        out.append(r)
        print(f"  seek iters={iters} (~{n_est} 步) 峰值提交 {r['commit_peak_mb']}MB  "
              f"{r['mb_per_1k_steps']}MB/千步  墙钟 {r['wall_s']}s  "
              f"watchdog={r['killed_by_watchdog']}", flush=True)
    return out


def malloc_scaling(cap_mb: float) -> list:
    out = []
    for n in (100_000, 300_000, 1_000_000):
        src = ('#include <stdio.h>\n#include <stdlib.h>\nint main(){\n'
               f'for(int i=0;i<{n};i++){{int* p=(int*)malloc(16); *p=i; free(p);}}\n'
               'printf("done\\n");return 0;}\n')
        f = WORK / f"malloc_{n}.c"
        f.write_text(src, encoding="utf-8")
        r = sample_run([str(CLI), "run", str(f)], timeout=600, cap_mb=cap_mb)
        r.update({"case": f"malloc_{n}", "n": n})
        out.append(r)
        print(f"  malloc n={n:>9} 峰值提交 {r['commit_peak_mb']}MB  墙钟 {r['wall_s']}s  "
              f"exit={r['exit']}  watchdog={r['killed_by_watchdog']}", flush=True)
    return out


def putchar_run(cap_mb: float) -> dict:
    src = ('#include <stdio.h>\nint main(){for(int i=0;i<1000000;i++)putchar(97);\n'
           'printf("\\n");return 0;}\n')
    f = WORK / "putchar_1m.c"
    f.write_text(src, encoding="utf-8")
    r = sample_run([str(CLI), "run", str(f)], timeout=600, cap_mb=cap_mb)
    r["case"] = "putchar_1m"
    print(f"  putchar 1M 峰值提交 {r['commit_peak_mb']}MB  墙钟 {r['wall_s']}s  "
          f"watchdog={r['killed_by_watchdog']}", flush=True)
    return r


def main() -> int:
    cap = 1500.0
    if "--cap-mb" in sys.argv:
        cap = float(sys.argv[sys.argv.index("--cap-mb") + 1])
    print(f"cide_cli: {CLI}\n看门狗上限 {cap}MB\n")
    print("== 1. seek 重放放大 ==")
    a = seek_scaling(cap)
    print("== 2. malloc/free (regions 登记表) ==")
    b = malloc_scaling(cap)
    print("== 3. putchar 1M (output_chunks) ==")
    c = putchar_run(cap)
    out = {"cap_mb": cap, "seek": a, "malloc": b, "putchar": c}
    p = HERE / "resource_longrun.json"
    p.write_text(json.dumps(out, ensure_ascii=False, indent=1), encoding="utf-8")
    print(f"\nJSON 已写出: {p}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
