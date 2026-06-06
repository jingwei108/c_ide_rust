#!/usr/bin/env python3
"""
从 shadow_verify.py 中提取 SHADOW_CASES 并生成 .c 文件到 native/tests/cases/ 目录。
同时修改 shadow_verify.py，支持从文件加载用例。
"""

import re
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.parent
SHADOW_PY_DIR = PROJECT_ROOT / "native/tests/shadow_verification"
SHADOW_PY = SHADOW_PY_DIR / "shadow_verify.py"
BASELINE_DIR = PROJECT_ROOT / "native/tests/cases/baseline"
GOLDEN_DIR = PROJECT_ROOT / "native/tests/cases_golden"
TPL_GEN_DIR = PROJECT_ROOT / "native/tests/cases_template_generated"


def extract_cases_via_import():
    """通过导入 shadow_verify 模块获取 SHADOW_CASES"""
    if str(SHADOW_PY_DIR) not in sys.path:
        sys.path.insert(0, str(SHADOW_PY_DIR))
    if str(PROJECT_ROOT / "native") not in sys.path:
        sys.path.insert(0, str(PROJECT_ROOT / "native"))
    
    import shadow_verify
    cases = []
    for case in shadow_verify.SHADOW_CASES:
        cases.append((case.name, case.source, case.category))
    return cases


def write_case_files(cases):
    BASELINE_DIR.mkdir(parents=True, exist_ok=True)
    TPL_GEN_DIR.mkdir(parents=True, exist_ok=True)
    GOLDEN_DIR.mkdir(parents=True, exist_ok=True)
    
    baseline_count = 0
    gap_count = 0
    
    for name, src, cat in cases:
        if cat == "baseline":
            target_dir = BASELINE_DIR
            baseline_count += 1
        else:
            target_dir = PROJECT_ROOT / "native/tests/cases/gap"
            target_dir.mkdir(parents=True, exist_ok=True)
            gap_count += 1
        
        # 写入 .c 文件，添加注释标记
        c_path = target_dir / f"{name}.c"
        content = f"// @category: {cat}\n{src}\n"
        c_path.write_text(content, encoding="utf-8")
    
    print(f"已生成 {baseline_count} 个 baseline 用例到 {BASELINE_DIR}")
    print(f"已生成 {gap_count} 个 gap 用例到 {PROJECT_ROOT / 'native/tests/cases/gap'}")


def patch_shadow_verify():
    """修改 shadow_verify.py，添加 load_case_files() 并在 main() 中使用"""
    text = SHADOW_PY.read_text(encoding="utf-8")
    
    # 如果已经 patch 过，跳过
    if "load_case_files" in text:
        print("shadow_verify.py 已包含 load_case_files，跳过 patch")
        return
    
    # 在 CLANG_HEADER 之后、SHADOW_CASES 之前插入 load_case_files 函数
    insert_marker = "# ===== 测试用例库 ====="
    insert_pos = text.find(insert_marker)
    if insert_pos == -1:
        print("WARNING: 无法找到插入点，跳过 shadow_verify.py patch")
        return
    
    load_func = '''

def load_case_files() -> List[ShadowCase]:
    """从 .c 文件加载用例，替代硬编码 SHADOW_CASES 列表"""
    cases = []
    for root in [
        Path("tests/cases/baseline"),
        Path("tests/cases/gap"),
    ]:
        root_path = NATIVE_DIR / root
        if not root_path.exists():
            continue
        for path in root_path.glob("*.c"):
            source = path.read_text(encoding="utf-8")
            # 提取 @category 注释
            cat_match = re.search(r'@category:\\s*(\\S+)', source)
            category = cat_match.group(1) if cat_match else "baseline"
            # 移除注释标记，保留纯源码
            lines = source.splitlines()
            clean_lines = []
            for line in lines:
                stripped = line.strip()
                if stripped.startswith("// @"):
                    continue
                clean_lines.append(line)
            clean_source = "\\n".join(clean_lines)
            cases.append(ShadowCase(
                name=path.stem,
                source=clean_source,
                category=category
            ))
    return cases

# 保持向后兼容：优先从文件加载，回退到硬编码列表
try:
    FILE_CASES = load_case_files()
except Exception as e:
    print(f"从文件加载用例失败: {e}, 使用硬编码列表")
    FILE_CASES = None

'''
    
    text = text[:insert_pos] + load_func + text[insert_pos:]
    
    # 修改 main() 中使用 FILE_CASES
    text = text.replace(
        "for i, case in enumerate(SHADOW_CASES, 1):",
        "CASES = FILE_CASES if FILE_CASES else SHADOW_CASES\n    for i, case in enumerate(CASES, 1):"
    )
    text = text.replace(
        'f"[{i}/{len(SHADOW_CASES)}] {case.name} ({case.category})"',
        'f"[{i}/{len(CASES)}] {case.name} ({case.category})"'
    )
    
    # 备份原文件
    backup = SHADOW_PY.with_suffix(".py.bak")
    backup.write_text(SHADOW_PY.read_text(encoding="utf-8"), encoding="utf-8")
    print(f"已备份原文件到 {backup}")
    
    SHADOW_PY.write_text(text, encoding="utf-8")
    print("已修改 shadow_verify.py 支持文件加载")


if __name__ == "__main__":
    if not SHADOW_PY.exists():
        print(f"ERROR: 找不到 {SHADOW_PY}")
        sys.exit(1)
    
    cases = extract_cases_via_import()
    print(f"从 shadow_verify.py 提取了 {len(cases)} 个用例")
    
    write_case_files(cases)
    patch_shadow_verify()
    print("Phase 1 完成！")
