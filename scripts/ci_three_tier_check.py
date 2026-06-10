#!/usr/bin/env python3
"""
三层契约验证 CI 脚本（Phase F）

目标：
1. 独立运行 Phase A/B/C/E 测试，确保三层契约在 CI 中被显式验证。
2. 生成综合报告，记录每层契约的通过状态。
3. 检查 *_FAILURES.md 的一致性：
   - 若文档中标记为 KNOWN_FAILURE / KNOWN_DIVERGENCE 的测试现在通过了，
     提示更新文档。
   - 若测试失败了但文档中没有对应记录，提示添加记录。
4. 报告输出到 reports/three_tier_report.md，并作为 CI artifact 上传。

使用方式：
    python scripts/ci_three_tier_check.py
"""

import os
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

# ─── 配置 ────────────────────────────────────────────────────────────────────

NATIVE_DIR = Path("native")
REPORTS_DIR = Path("reports")
TESTS_DIR = NATIVE_DIR / "tests"

TIER_TESTS = [
    ("Phase A", "host_contract_tests", "HOST_CONTRACT_FAILURES.md"),
    ("Phase B", "bytecode_libc_consistency", "BYTECODE_LIBC_FAILURES.md"),
    ("Phase C", "differential_stress", "DIFFERENTIAL_FAILURES.md"),
    ("Phase E", "fuzz_stress_test", "FUZZ_FAILURES.md"),
    ("K&R / E2E / LeetCode", "cide_e2e", ["KR_FAILURES.md", "E2E_FAILURES.md", "LEETCODE_FAILURES.md"]),
    ("C++ Parser", "parser_cpp_unit_test", "CPP_FAILURES.md"),
    ("C++ TypeChecker", "typeck_cpp_unit_test", "CPP_FAILURES.md"),
    ("C++ BytecodeGen", "bytecode_gen_cpp_unit_test", "CPP_FAILURES.md"),
    ("C++ Dogfooding", "cpp_dogfooding_test", "DOGFOODING_FAILURES.md"),
]

# 已知为设计决策、不需要修复的差异（不视为 CI 失败）
EXPECTED_DIVERGENCES = {
    "test_diff_abs",
    "test_diff_atoi",
}


# ─── 数据类 ──────────────────────────────────────────────────────────────────

@dataclass
class TierResult:
    phase: str
    test_file: str
    failures_md: list
    passed: bool
    tests_run: int = 0
    tests_passed: int = 0
    tests_failed: int = 0
    stdout: str = ""
    stderr: str = ""
    failed_tests: list = None

    def __post_init__(self):
        if self.failed_tests is None:
            self.failed_tests = []
        if isinstance(self.failures_md, str):
            self.failures_md = [self.failures_md]


# ─── 测试运行 ─────────────────────────────────────────────────────────────────


_test_cache: dict[str, subprocess.CompletedProcess] = {}


def run_cargo_test(test_file: str) -> subprocess.CompletedProcess:
    """运行指定的 cargo integration test（带缓存，避免同一 test_file 重复运行）。"""
    if test_file not in _test_cache:
        _test_cache[test_file] = _run_cargo_test_uncached(test_file)
    return _test_cache[test_file]


def _run_cargo_test_uncached(test_file: str) -> subprocess.CompletedProcess:
    """运行指定的 cargo integration test。"""
    cmd = [
        "cargo",
        "test",
        "--test",
        test_file,
        "--",
        "--test-threads=1",
    ]
    return subprocess.run(
        cmd,
        cwd=NATIVE_DIR,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def parse_test_output(output: str) -> dict:
    """从 cargo test 输出中提取统计信息。"""
    stats = {
        "run": 0,
        "passed": 0,
        "failed": 0,
        "ignored": 0,
        "failed_names": [],
    }

    # 匹配 "test result: ok. 38 passed; 0 failed; 0 ignored"
    result_line = re.search(
        r"test result:\s+(ok|FAILED)\.\s+(\d+)\s+passed;\s+(\d+)\s+failed;\s+(\d+)\s+ignored",
        output,
    )
    if result_line:
        stats["passed"] = int(result_line.group(2))
        stats["failed"] = int(result_line.group(3))
        stats["ignored"] = int(result_line.group(4))
        stats["run"] = stats["passed"] + stats["failed"]

    # 匹配失败的测试名 "test xxx ... FAILED"
    for m in re.finditer(r"^test\s+(\S+)\s+\.\.\.\s+FAILED", output, re.MULTILINE):
        stats["failed_names"].append(m.group(1))

    return stats


# ─── 文档一致性检查 ───────────────────────────────────────────────────────────


def extract_md_status(md_path: Path) -> dict:
    """从 *_FAILURES.md 中提取状态信息。"""
    if not md_path.exists():
        return {"missing": True, "entries": []}

    content = md_path.read_text(encoding="utf-8")
    entries = []

    # 提取带有 ~~删除线~~ 的标题，通常表示"已修复"
    for m in re.finditer(r"^###\s+~~(.+?)~~\s*→\s*已修复", content, re.MULTILINE):
        entries.append({
            "title": m.group(1).strip(),
            "status": "FIXED",
        })

    # 提取 KNOWN_FAILURE / KNOWN_DIVERGENCE
    for m in re.finditer(r"^##\s+(已知失败|KNOWN_FAILURE|已知偏差|KNOWN_DIVERGENCE)", content, re.MULTILINE):
        section = m.group(1).strip()
        # 提取该 section 下的所有三级标题
        section_start = m.end()
        next_section = re.search(r"^##\s+", content[section_start:], re.MULTILINE)
        section_end = section_start + next_section.start() if next_section else len(content)
        section_text = content[section_start:section_end]
        for tm in re.finditer(r"^###\s+(.+)$", section_text, re.MULTILINE):
            entries.append({
                "title": tm.group(1).strip(),
                "status": "KNOWN" if "偏差" not in section else "DIVERGENCE",
            })

    return {"missing": False, "entries": entries}


def check_consistency(result: TierResult) -> list:
    """检查测试结果与 *_FAILURES.md 的一致性，返回问题列表。"""
    issues = []
    for failures_md in result.failures_md:
        md_path = TESTS_DIR / failures_md
        md_info = extract_md_status(md_path)

        if md_info["missing"]:
            issues.append(f"缺少失败记录文件: {failures_md}")
            continue

        # 检查 KNOWN_FAILURE 是否仍然失败
        # 简化处理：如果整个测试文件通过了，但文档中仍有未标记为 FIXED 的 KNOWN 条目，提醒更新
        # 注意：KNOWN_DIVERGENCE（设计决策导致的偏差）不视为需要修复的故障，测试通过是正常的
        if result.passed:
            known_entries = [e for e in md_info["entries"] if e["status"] == "KNOWN"]
            if known_entries:
                titles = ", ".join(e["title"][:40] for e in known_entries)
                issues.append(
                    f"Tests all passed, but {failures_md} still has {len(known_entries)} un-fixed KNOWN entries: {titles}"
                )
        else:
            # 测试有失败，检查是否都已在文档中记录
            # 由于文档是自由文本，这里只能做粗略提醒
            fixed_count = len([e for e in md_info["entries"] if e["status"] == "FIXED"])
            issues.append(
                f"Tests have failures. Ensure all are recorded in {failures_md} (currently {fixed_count} FIXED records)"
            )

    return issues


# ─── 报告生成 ─────────────────────────────────────────────────────────────────


def generate_report(results: list, consistency_issues: dict) -> str:
    """生成 Markdown 报告。"""
    lines = [
        "# 三层契约验证报告（Three Tier Test Report）",
        "",
        f"生成时间: {__import__('datetime').datetime.now().isoformat()}",
        "",
        "> 本报告由 CI 自动生成，对应 Phase F 要求。",
        "",
        "## 摘要",
        "",
        "| 阶段 | 测试文件 | 状态 | 通过 | 失败 | 忽略 |",
        "|------|----------|------|------|------|------|",
    ]

    all_passed = True
    for r in results:
        status = "✅ PASS" if r.passed else "❌ FAIL"
        if not r.passed:
            all_passed = False
        lines.append(
            f"| {r.phase} | `{r.test_file}` | {status} | {r.tests_passed} | {r.tests_failed} | {r.tests_run - r.tests_passed - r.tests_failed} |"
        )

    lines.extend([
        "",
        "## 一致性检查",
        "",
    ])

    has_issues = False
    for r in results:
        issues = consistency_issues.get(r.phase, [])
        if issues:
            has_issues = True
            lines.append(f"### {r.phase}")
            lines.append("")
            for issue in issues:
                lines.append(f"- ⚠️ {issue}")
            lines.append("")

    if not has_issues:
        lines.append("✅ 所有失败记录文档与测试结果一致。")
        lines.append("")

    lines.extend([
        "## 详细输出",
        "",
    ])

    for r in results:
        lines.append(f"### {r.phase}: {r.test_file}")
        lines.append("")
        if r.failed_tests:
            lines.append("**失败的测试:**")
            for name in r.failed_tests:
                lines.append(f"- `{name}`")
            lines.append("")
        lines.append("```")
        # 截取最后的 800 字符，避免报告过长
        tail = (r.stdout + "\n" + r.stderr)[-800:]
        lines.append(tail)
        lines.append("```")
        lines.append("")

    return "\n".join(lines) + "\n"


# ─── 主流程 ───────────────────────────────────────────────────────────────────


def main() -> int:
    REPORTS_DIR.mkdir(parents=True, exist_ok=True)

    results = []
    consistency_issues = {}

    print("=" * 60)
    print("Three Tier Verification Start")
    print("=" * 60)

    for phase, test_file, failures_md in TIER_TESTS:
        print(f"\n>> {phase}: cargo test --test {test_file}")
        proc = run_cargo_test(test_file)
        stats = parse_test_output(proc.stdout + proc.stderr)

        passed = stats["failed"] == 0
        result = TierResult(
            phase=phase,
            test_file=test_file,
            failures_md=failures_md,
            passed=passed,
            tests_run=stats["run"],
            tests_passed=stats["passed"],
            tests_failed=stats["failed"],
            stdout=proc.stdout,
            stderr=proc.stderr,
            failed_tests=stats["failed_names"],
        )
        results.append(result)

        status = "[PASS]" if passed else "[FAIL]"
        print(f"   {status} — {stats['passed']} passed, {stats['failed']} failed")

        # 一致性检查
        issues = check_consistency(result)
        consistency_issues[phase] = issues
        if issues:
            for issue in issues:
                print(f"   [WARN] {issue}")

    # 生成报告
    report_md = generate_report(results, consistency_issues)
    report_path = REPORTS_DIR / "three_tier_report.md"
    report_path.write_text(report_md, encoding="utf-8")
    print(f"\n[REPORT] Generated: {report_path}")

    # 最终判定
    all_passed = all(r.passed for r in results)
    if all_passed:
        print("\n[SUCCESS] All three tier tests passed!")
        return 0
    else:
        print("\n[FAILED] Some three tier tests failed. See report and *_FAILURES.md.")
        return 1


if __name__ == "__main__":
    sys.exit(main())
