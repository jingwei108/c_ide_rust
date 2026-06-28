#!/usr/bin/env python3
"""
test_templates.py — 模板静态一致性测试框架

职责：
1. 扫描 templates/ 目录，校验每个模板目录结构合法
2. 校验 meta.yaml 与 source.c/source.cpp 中的参数占位符一致
3. 校验 CideFlutter/assets/templates/index.json 与 templates/ 源文件同步
4. 校验 assets 中每个模板源码文件存在且与源码一致

用法：
    python scripts/test_templates.py

返回码：
    0 — 全部通过
    1 — 存在不一致或缺失
"""

import json
import re
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.parent
TPL_DIR = PROJECT_ROOT / "templates"
ASSETS_DIR = PROJECT_ROOT / "CideFlutter/assets/templates"
INDEX_PATH = ASSETS_DIR / "index.json"

PARAM_RE = re.compile(r'/\*__PARAM_(\w+)__\*/')


def error(msg: str) -> int:
    print(f"[FAIL] {msg}")
    return 1


def info(msg: str):
    print(f"[PASS] {msg}")


def load_meta_yaml(path: Path) -> dict:
    """极简 YAML 解析，只读取本项目的 meta.yaml 格式。"""
    data = {"params": {}, "tutorial": {"steps": []}, "knowledge_nodes": []}
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()
    i = 0
    current_param = None
    current_step = None
    in_params = False
    in_tutorial = False
    in_knowledge = False

    while i < len(lines):
        line = lines[i]
        stripped = line.strip()
        if not stripped or stripped.startswith('#'):
            i += 1
            continue

        if stripped.startswith('key:') and not in_params and not in_tutorial and not in_knowledge:
            data['key'] = stripped.split(':', 1)[1].strip()
        elif stripped.startswith('name:') and not in_params and not in_tutorial and not in_knowledge:
            data['name'] = stripped.split(':', 1)[1].strip()
        elif stripped.startswith('category:') and not in_params and not in_tutorial and not in_knowledge:
            data['category'] = stripped.split(':', 1)[1].strip()
        elif stripped == 'params:':
            in_params = True
            in_tutorial = False
            in_knowledge = False
        elif stripped == 'tutorial:':
            in_params = False
            in_tutorial = True
            in_knowledge = False
        elif stripped == 'knowledge_nodes:':
            in_knowledge = True
            in_tutorial = False
            in_params = False
        elif stripped.startswith('- ') and not stripped.startswith('- title:'):
            val = stripped[2:].strip()
            if in_knowledge:
                data['knowledge_nodes'].append(val)
        elif stripped.startswith('- title:'):
            in_tutorial = True
            in_knowledge = False
            current_step = {"title": stripped.split(':', 1)[1].strip()}
            data['tutorial']['steps'].append(current_step)
            current_param = None
        elif stripped.startswith('description:') and current_step is not None:
            current_step['description'] = stripped.split(':', 1)[1].strip()
        elif stripped.startswith('anchor:') and current_step is not None:
            current_step['anchor'] = stripped.split(':', 1)[1].strip()
        elif re.match(r'^\w+:$', stripped) and not in_tutorial and not in_knowledge:
            current_param = stripped[:-1]
            data['params'][current_param] = {}
        elif stripped.startswith('label:') and current_param is not None:
            data['params'][current_param]['label'] = stripped.split(':', 1)[1].strip()
        elif stripped.startswith('type:') and current_param is not None:
            data['params'][current_param]['type'] = stripped.split(':', 1)[1].strip()
        elif stripped.startswith('default:') and current_param is not None:
            data['params'][current_param]['default'] = stripped.split(':', 1)[1].strip()

        i += 1

    return data


def validate_source_templates() -> int:
    """校验 templates/ 源目录下的每个模板。"""
    failures = 0
    if not TPL_DIR.exists():
        return error(f"templates directory not found: {TPL_DIR}")

    dirs = sorted([d for d in TPL_DIR.iterdir() if d.is_dir()])
    if not dirs:
        return error("no template directories found")

    for d in dirs:
        key = d.name
        meta_path = d / "meta.yaml"
        source_c = d / "source.c"
        source_cpp = d / "source.cpp"

        if not meta_path.exists():
            failures += error(f"{key}: missing meta.yaml")
            continue

        if source_c.exists():
            source_path = source_c
            ext = "c"
        elif source_cpp.exists():
            source_path = source_cpp
            ext = "cpp"
        else:
            failures += error(f"{key}: missing source.c or source.cpp")
            continue

        try:
            meta = load_meta_yaml(meta_path)
        except Exception as e:
            failures += error(f"{key}: failed to parse meta.yaml: {e}")
            continue

        # 校验关键字段
        for field in ("key", "name", "category"):
            if not meta.get(field):
                failures += error(f"{key}: meta.yaml missing '{field}'")

        if meta.get("key") != key:
            failures += error(
                f"{key}: meta.yaml key mismatch (expected {key}, got {meta.get('key')})"
            )

        # 校验源码中的参数占位符与 meta.yaml 声明一致
        source = source_path.read_text(encoding="utf-8")
        placeholder_keys = set(PARAM_RE.findall(source))
        declared_keys = set(meta.get("params", {}).keys())

        missing_params = placeholder_keys - declared_keys
        if missing_params:
            failures += error(
                f"{key}: placeholders without params in meta.yaml: {sorted(missing_params)}"
            )

        unused_params = declared_keys - placeholder_keys
        if unused_params:
            failures += error(
                f"{key}: params without placeholders in source: {sorted(unused_params)}"
            )

        # 校验每个 param 都有 default 值（用于 shadow 用例生成）
        for pkey, pmeta in meta.get("params", {}).items():
            if "default" not in pmeta:
                failures += error(f"{key}: param '{pkey}' missing default value")

        # 校验教程步骤描述非空
        for idx, step in enumerate(meta.get("tutorial", {}).get("steps", [])):
            if not step.get("title"):
                failures += error(f"{key}: tutorial step {idx} missing title")

        if failures == 0:
            info(f"{key} ({ext}): meta/params/tutorial OK")

    return failures


def validate_assets_sync() -> int:
    """校验 CideFlutter/assets/templates/ 与 templates/ 源目录同步。"""
    failures = 0
    if not INDEX_PATH.exists():
        return error(f"Flutter index not found: {INDEX_PATH}")

    try:
        index = json.loads(INDEX_PATH.read_text(encoding="utf-8"))
    except Exception as e:
        return error(f"failed to parse {INDEX_PATH}: {e}")

    index_keys = {entry["key"] for entry in index.get("templates", [])}
    source_keys = {d.name for d in TPL_DIR.iterdir() if d.is_dir()}

    missing_in_index = source_keys - index_keys
    if missing_in_index:
        failures += error(
            f"templates missing in index.json: {sorted(missing_in_index)}"
        )

    extra_in_index = index_keys - source_keys
    if extra_in_index:
        failures += error(
            f"index.json entries without template source: {sorted(extra_in_index)}"
        )

    for entry in index.get("templates", []):
        key = entry.get("key")
        if not key:
            failures += error("index.json entry missing key")
            continue

        source_path = ASSETS_DIR / f"{key}.c"
        cpp_path = ASSETS_DIR / f"{key}.cpp"
        if not source_path.exists() and not cpp_path.exists():
            failures += error(f"{key}: asset source file missing in {ASSETS_DIR}")
            continue

        asset_path = cpp_path if cpp_path.exists() else source_path
        source_path_expected = TPL_DIR / key / ("source.cpp" if cpp_path.exists() else "source.c")

        if source_path_expected.exists():
            expected = source_path_expected.read_bytes()
            actual = asset_path.read_bytes()
            if expected != actual:
                failures += error(
                    f"{key}: asset source is out of sync with templates/{key}"
                )
        else:
            failures += error(
                f"{key}: expected source template {source_path_expected} not found"
            )

    if failures == 0:
        info(f"assets sync OK ({len(index_keys)} templates)")

    return failures


def main() -> int:
    failures = 0
    print("=== Template source validation ===")
    failures += validate_source_templates()
    print("\n=== Flutter assets sync validation ===")
    failures += validate_assets_sync()

    print()
    if failures:
        print(f"RESULT: {failures} failure(s). Run `python scripts/sync_templates.py` if assets are out of sync.")
        return 1
    print("RESULT: all template checks passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
