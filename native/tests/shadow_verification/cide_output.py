#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""cide_native DLL 输出的结构化读取（E-P1-5）。

背景
----
引擎此前把「程序自己的 stdout」与「引擎附注」（"程序运行完成，返回值：N"、内存泄漏
检测报告、教学安全提示）追加进同一条字节流，驱动只能靠文本正则把附注洗掉。清洗规则
在 Python / Rust 多处各写一份、语义互不一致（全局替换 vs 行内截断、`>=30` vs `==30`
个等号、是否丢空行），并且会在教学程序自己打印同类文本时**误删真实输出**，导出假阳性
`output_gap`。

现在引擎按通道打标（`OutputKind::Stdout / Stderr / Note`），本模块是驱动侧读取输出的
**唯一入口**：

* :func:`read_program_stdout` —— 纯程序 stdout，与 Clang golden 比对的唯一合法来源；
* :func:`read_engine_notes` —— 引擎附注，用于展示/诊断，**不得**进入 stdout 比对。

纪律：任何驱动都**不得**再对输出文本做正则清洗。缺少新符号（ABI < 1.1.0）时 fail fast，
而不是退回旧清洗规则——否则"完全匹配"的口径又会分叉。
"""

import ctypes
import subprocess
from pathlib import Path

#: 仓库根（本文件位于 native/tests/shadow_verification/）。
_REPO_ROOT = Path(__file__).resolve().parents[3]

#: ABI 1.1.0 起提供的结构化输出入口。
REQUIRED_SYMBOLS = (
    "cide_get_program_output_length",
    "cide_get_program_output",
    "cide_get_engine_notes_length",
    "cide_get_engine_notes",
)

_ABI_HINT = (
    "当前 cide_native DLL 缺少 {missing}（需要 ABI >= 1.1.0 的结构化输出通道）。\n"
    "请重建引擎后重跑：cd native && cargo build --release"
    "（或 python native/tests/shadow_verification/shadow_verify.py --rebuild）。\n"
    "不要退回文本清洗：E-P1-5 已废除该口径。"
)


def _git_short_head():
    """当前提交短哈希；git 不可用（导出源码、无 .git）时返回 None。"""
    try:
        out = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"],
            capture_output=True,
            text=True,
            cwd=str(_REPO_ROOT),
        )
    except OSError:
        return None
    return out.stdout.strip() if out.returncode == 0 else None


def _ensure_fresh_artifacts(dll) -> None:
    """产物新鲜度校验（fail fast）：DLL 版本串必须含当前 HEAD 短哈希。

    动机（实测踩坑）：影子验证读的是 `native/target/release/cide_native.dll`，
    源码或提交变了而产物没重建时，全部用例会在"验证陈旧二进制"的情况下绿过去 ——
    这比失败更坏，因为它制造的是**假绿**。故与 REQUIRED_SYMBOLS 缺失同一口径：
    宁可 fail fast。git 不可用时跳过本项（ABI 校验已兜住最硬的兼容性）。
    """
    head = _git_short_head()
    if not head or not hasattr(dll, "cide_engine_version"):
        return
    # 注意：restype 必须是 c_void_p 而非 c_char_p —— 后者会把返回值变成 Python bytes
    # 副本，既拿不到原指针（无法按 rust-alloc 契约释放），又会把副本地址交给
    # cide_free_string 触发**非法释放**。
    dll.cide_engine_version.restype = ctypes.c_void_p
    dll.cide_free_string.argtypes = [ctypes.c_void_p]
    raw = dll.cide_engine_version()
    version = ctypes.cast(raw, ctypes.c_char_p).value.decode("utf-8", "replace") if raw else ""
    if raw:
        dll.cide_free_string(raw)
    if head not in version:
        raise RuntimeError(
            f"引擎产物不是当前提交构建的：cide_engine_version()={version!r} 不含 HEAD {head}。\n"
            "影子验证读的是 native/target/release/cide_native.dll，"
            "请先 `cd native && cargo build --release` ——\n"
            "否则会在陈旧二进制上得到假绿（实测：HEAD 前进后产物仍是上一个哈希）。"
        )


def ensure_abi(dll) -> None:
    """校验 DLL 是否提供结构化输出入口；缺失即抛出（fail fast）。

    同时做**产物新鲜度**校验（版本串含当前 HEAD），防止在陈旧二进制上假绿。
    """
    missing = [name for name in REQUIRED_SYMBOLS if not hasattr(dll, name)]
    if missing:
        raise RuntimeError(_ABI_HINT.format(missing=", ".join(missing)))
    _ensure_fresh_artifacts(dll)


def _read(dll, length_fn: str, copy_fn: str, session) -> str:
    length = getattr(dll, length_fn)(session)
    if length <= 0:
        return ""
    buf = ctypes.create_string_buffer(length + 1)
    getattr(dll, copy_fn)(session, buf, length + 1)
    return buf.value.decode("utf-8", errors="replace")


def read_program_stdout(dll, session) -> str:
    """纯程序 stdout（不含引擎附注、不含 stderr），已 strip 两端空白。

    strip 与 Clang 侧的金标准提取保持对称，避免"尾部换行差异"被算成 output_gap。
    """
    return _read(dll, "cide_get_program_output_length", "cide_get_program_output", session).strip()


def read_engine_notes(dll, session) -> str:
    """引擎附注原文（运行完成提示 / 泄漏报告 / 教学提示）。"""
    return _read(dll, "cide_get_engine_notes_length", "cide_get_engine_notes", session)
