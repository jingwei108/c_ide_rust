# -*- coding: utf-8 -*-
"""驱动侧宿主内存独立测量（Windows psapi；psutil 在本机不可安装，改用同一 API 直接读取）。

口径与 `scripts/replay/debug_p3_leak.py` 一致：
  * PrivateMemorySize64 ≈ PagefileUsage（提交内存，含页面文件）；
  * WorkingSetSize（物理驻留）。
**只从驱动侧读被测进程**，不采信被测代码自报的任何数字。

用法：from winmem import commit_mb, working_set_mb
"""
import ctypes
from ctypes import wintypes


class _PROCESS_MEMORY_COUNTERS(ctypes.Structure):
    _fields_ = [
        ("cb", wintypes.DWORD),
        ("PageFaultCount", wintypes.DWORD),
        ("PeakWorkingSetSize", ctypes.c_size_t),
        ("WorkingSetSize", ctypes.c_size_t),
        ("QuotaPeakPagedPoolUsage", ctypes.c_size_t),
        ("QuotaPagedPoolUsage", ctypes.c_size_t),
        ("QuotaPeakNonPagedPoolUsage", ctypes.c_size_t),
        ("QuotaNonPagedPoolUsage", ctypes.c_size_t),
        ("PagefileUsage", ctypes.c_size_t),
        ("PeakPagefileUsage", ctypes.c_size_t),
    ]


PROCESS_QUERY_LIMITED_INFORMATION = 0x1000


def _counters(pid: int):
    h = ctypes.windll.kernel32.OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, False, pid)
    if not h:
        h = ctypes.windll.kernel32.OpenProcess(0x0400, False, pid)
    if not h:
        return None
    try:
        c = _PROCESS_MEMORY_COUNTERS()
        c.cb = ctypes.sizeof(c)
        ok = ctypes.windll.psapi.GetProcessMemoryInfo(h, ctypes.byref(c), ctypes.sizeof(c))
        return c if ok else None
    finally:
        ctypes.windll.kernel32.CloseHandle(h)


def commit_mb(pid: int) -> float:
    c = _counters(pid)
    return round(c.PagefileUsage / 1048576.0, 1) if c else -1.0


def working_set_mb(pid: int) -> float:
    c = _counters(pid)
    return round(c.WorkingSetSize / 1048576.0, 1) if c else -1.0


def peak_commit_mb(pid: int) -> float:
    c = _counters(pid)
    return round(c.PeakPagefileUsage / 1048576.0, 1) if c else -1.0
