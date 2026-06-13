#!/usr/bin/env python3
"""预编译 runtime_libc 为嵌入产物。

开发流程：
1. 修改 native/runtime_libc/src/*.c
2. 运行 python scripts/precompile_bytecode_libc.py
3. git add native/src/vm/bytecode_libc_data.json native/src/vm/bytecode_libc_index.rs
4. git commit

CI 检查：
    python scripts/precompile_bytecode_libc.py --check
"""

import argparse
import json
import math
import os
import subprocess
import sys

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
NATIVE_DIR = os.path.join(PROJECT_ROOT, "native")
RUNTIME_LIBC_SRC_DIRS = [
    os.path.join(NATIVE_DIR, "runtime_libc", "src"),
    os.path.join(NATIVE_DIR, "runtime_libc", "cide"),
]
VM_DIR = os.path.join(NATIVE_DIR, "src", "vm")
OUTPUT_JSON = os.path.join(VM_DIR, "bytecode_libc_data.json")
OUTPUT_RS = os.path.join(VM_DIR, "bytecode_libc_index.rs")
LAYOUT_JSON = os.path.join(
    NATIVE_DIR, "src", "compiler", "cpp_frontend", "builtin_layout_data.json"
)


def find_cide_cli() -> str:
    """查找 cide_cli 可执行文件路径。"""
    candidates = [
        os.path.join(NATIVE_DIR, "target", "release", "cide_cli.exe"),
        os.path.join(NATIVE_DIR, "target", "release", "cide_cli"),
        os.path.join(NATIVE_DIR, "target", "debug", "cide_cli.exe"),
        os.path.join(NATIVE_DIR, "target", "debug", "cide_cli"),
    ]
    for c in candidates:
        if os.path.exists(c):
            return c
    return ""


def build_cide_cli() -> str:
    """构建 cide_cli，返回可执行文件路径。"""
    print("Building cide_cli...")
    subprocess.run(
        ["cargo", "build", "--release", "--bin", "cide_cli"],
        cwd=NATIVE_DIR,
        check=True,
    )
    exe = find_cide_cli()
    if not exe:
        raise RuntimeError("cide_cli build succeeded but executable not found")
    print(f"  -> {exe}")
    return exe


def precompile(exe: str) -> dict:
    """调用 cide_cli export 预编译 runtime_libc。"""
    sources = []
    for src_dir in RUNTIME_LIBC_SRC_DIRS:
        if os.path.isdir(src_dir):
            sources.extend(
                os.path.join(src_dir, f)
                for f in os.listdir(src_dir)
                if f.endswith(".c") or f.endswith(".cpp")
            )
    sources = sorted(sources)
    # Stage 2b: .cpp files now contain full C++ implementations,
    # and legacy .c container implementations are being removed.
    source_paths = sources

    print(f"Precompiling {len(source_paths)} files:")
    for p in source_paths:
        print(f"  - {os.path.basename(p)}")

    cmd = [exe, "export"] + source_paths + ["--builtin-libc", "-o", OUTPUT_JSON]
    subprocess.run(cmd, check=True)

    with open(OUTPUT_JSON, "r", encoding="utf-8") as f:
        data = json.load(f)

    # 将 code 中的 Call/CallPtr operand 从原始索引重定位为固定索引，
    # 避免 setup_vm 同时注册原始索引和固定索引时发生冲突。
    raw_func_index = data["func_index"]
    base_index = 1000
    sorted_raw = sorted(raw_func_index.items(), key=lambda x: x[1])
    raw_to_fixed = {raw: base_index + i for i, (_, raw) in enumerate(sorted_raw)}

    for inst in data["code"]:
        if inst.get("op") in ("Call", "CallPtr"):
            raw = inst.get("operand", 0)
            inst["operand"] = raw_to_fixed.get(raw, raw)

    data["func_index"] = {name: raw_to_fixed[raw] for name, raw in raw_func_index.items()}

    validate_precompiled(data)

    print(f"  code_len: {data['code_len']}")
    print(f"  func_count: {len(data['func_index'])}")
    print(f"  globals_size: {data['globals_size']}")
    return data


def validate_precompiled(data: dict) -> None:
    """校验预编译产物符合 Stage 2b 要求：

    1. runtime_libc/cide/ 下不应存在旧 .c 实现。
    2. builtin_layout 中注册的方法必须在产物中可用。
    3. 不应残留旧 C 风格函数名（如 cide_vec_init_int）。
    """
    cide_dir = os.path.join(NATIVE_DIR, "runtime_libc", "cide")
    legacy_c_files = [
        f for f in os.listdir(cide_dir) if f.endswith(".c")
    ] if os.path.isdir(cide_dir) else []
    if legacy_c_files:
        raise RuntimeError(
            f"发现遗留 C 容器实现: {legacy_c_files}. "
            "Stage 2b 要求 runtime_libc/cide/ 只保留 .cpp 实现。"
        )

    if not os.path.exists(LAYOUT_JSON):
        raise RuntimeError(f"布局文件不存在: {LAYOUT_JSON}")

    with open(LAYOUT_JSON, "r", encoding="utf-8") as f:
        layout = json.load(f)

    func_index = data["func_index"]
    missing = []
    for cide_name, cls in layout["classes"].items():
        method_map = layout.get("method_map", {}).get(cide_name, {})
        for method, mangled in method_map.items():
            if mangled not in func_index:
                missing.append(f"{cide_name}.{method} -> {mangled}")
    if missing:
        raise RuntimeError(
            "以下内置容器方法未在预编译产物中找到:\n  " + "\n  ".join(missing)
        )

    # 旧 C 风格函数名黑名单（Stage 2b 之前的命名）
    old_prefixes = (
        "cide_vec_init_",
        "cide_vec_push_",
        "cide_vec_pop_",
        "cide_vec_get_",
        "cide_vec_size_",
        "cide_vec_destroy_",
        "cide_vec_clear_",
        "cide_list_init_",
        "cide_list_push_",
        "cide_list_pop_",
        "cide_list_get_",
        "cide_list_size_",
        "cide_list_destroy_",
        "cide_list_clear_",
        "cide_string_init",
        "cide_string_push_",
        "cide_string_pop_",
        "cide_string_get_",
        "cide_string_size",
        "cide_string_destroy",
        "cide_string_clear",
        "cide_string_c_str",
    )
    stale = [name for name in func_index if name.startswith(old_prefixes)]
    if stale:
        raise RuntimeError(
            "预编译产物中残留旧 C 风格函数名:\n  " + "\n  ".join(stale)
        )

    print("  validation: ok")


def generate_index_rs(data: dict) -> str:
    """生成 bytecode_libc_index.rs。"""
    func_index = data["func_index"]
    base_index = min(func_index.values()) if func_index else 1000
    globals_reserved = max(1024, math.ceil(data["globals_size"] / 1024) * 1024)
    code_len = data["code_len"]

    # 预编译阶段已将 func_index 重定位为固定索引，直接复用。
    # 按固定索引排序
    sorted_funcs = sorted(func_index.items(), key=lambda x: x[1])

    lines = [
        "// AUTO-GENERATED by scripts/precompile_bytecode_libc.py",
        "// DO NOT EDIT MANUALLY",
        "//",
        "// To regenerate:",
        "//   python scripts/precompile_bytecode_libc.py",
        "",
        f"pub const BYTECODE_LIBC_CODE_LEN: usize = {code_len};",
        f"pub const BYTECODE_LIBC_BASE_INDEX: i32 = {base_index};",
        f"pub const BYTECODE_LIBC_GLOBALS_RESERVED: u32 = {globals_reserved};",
        f"pub const BYTECODE_LIBC_FUNC_COUNT: usize = {len(func_index)};",
        "",
        "/// Bytecode Libc 中所有可用的函数名（供索引查询使用）。",
        "/// 注意：并非所有函数都默认走 Bytecode 路径，",
        "/// 实际路径由 `host_func_id::BYTECODE_LIBC_PURE_FUNCS` 控制。",
        "pub const BYTECODE_LIBC_ALL_FUNCS: &[&str] = &[",
    ]
    for name, _ in sorted_funcs:
        lines.append(f'    "{name}",')
    lines.append("];")
    lines.append("")
    lines.append("/// 将函数名解析为 Bytecode Libc 固定索引。")
    lines.append("pub fn bytecode_libc_index(name: &str) -> Option<i32> {")
    lines.append("    match name {")
    for name, idx in sorted_funcs:
        lines.append(f'        "{name}" => Some({idx}),')
    lines.append("        _ => None,")
    lines.append("    }")
    lines.append("}")
    lines.append("")
    lines.append("/// 判断函数是否在 Bytecode Libc 预编译产物中存在。")
    lines.append("pub fn is_bytecode_libc(name: &str) -> bool {")
    lines.append("    bytecode_libc_index(name).is_some()")
    lines.append("}")
    lines.append("")

    return "\n".join(lines) + "\n"


def write_outputs(data: dict) -> None:
    """写入 JSON 和 Rust 常量文件。"""
    with open(OUTPUT_JSON, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
    print(f"Written: {OUTPUT_JSON}")

    rs_content = generate_index_rs(data)
    with open(OUTPUT_RS, "w", encoding="utf-8") as f:
        f.write(rs_content)
    print(f"Written: {OUTPUT_RS}")


def check_up_to_date() -> bool:
    """检查预编译产物是否与 runtime_libc 源码同步。"""
    if not os.path.exists(OUTPUT_JSON) or not os.path.exists(OUTPUT_RS):
        return False

    json_mtime = os.path.getmtime(OUTPUT_JSON)
    for src_dir in RUNTIME_LIBC_SRC_DIRS:
        if not os.path.isdir(src_dir):
            continue
        for fname in os.listdir(src_dir):
            if fname.endswith(".c") or fname.endswith(".cpp") or fname.endswith(".h"):
                fpath = os.path.join(src_dir, fname)
                if os.path.getmtime(fpath) > json_mtime:
                    return False
    return True


def main() -> int:
    parser = argparse.ArgumentParser(description="Precompile runtime_libc Bytecode")
    parser.add_argument(
        "--check",
        action="store_true",
        help="Check if precompiled artifacts are up-to-date (CI mode)",
    )
    args = parser.parse_args()

    if args.check:
        if check_up_to_date():
            print("Precompiled artifacts are up-to-date.")
            return 0
        else:
            print(
                "ERROR: Precompiled artifacts are out-of-date. "
                "Run: python scripts/precompile_bytecode_libc.py",
                file=sys.stderr,
            )
            return 1

    # 清理旧产物，避免残留索引与旧函数名
    for f in (OUTPUT_JSON, OUTPUT_RS):
        if os.path.exists(f):
            os.remove(f)
            print(f"  removed: {f}")

    exe = find_cide_cli()
    if not exe:
        exe = build_cide_cli()

    data = precompile(exe)
    write_outputs(data)
    print("Done.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
