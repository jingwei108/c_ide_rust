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


def ensure_abi(dll) -> None:
    """校验 DLL 是否提供结构化输出入口；缺失即抛出（fail fast）。"""
    missing = [name for name in REQUIRED_SYMBOLS if not hasattr(dll, name)]
    if missing:
        raise RuntimeError(_ABI_HINT.format(missing=", ".join(missing)))


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
