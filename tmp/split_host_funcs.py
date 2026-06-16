#!/usr/bin/env python3
"""Incremental split of native/src/vm/host_funcs.rs into vm/host/*.rs."""
import os
import re
import subprocess
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
NATIVE_DIR = os.path.join(ROOT, 'native')
HOST_FUNCS = os.path.join(NATIVE_DIR, 'src', 'vm', 'host_funcs.rs')
HOST_DIR = os.path.join(NATIVE_DIR, 'src', 'vm', 'host')

# (start_line_1_based, descriptive_name, target_module)
# start line must be the first line of the item (including attributes).
ITEMS = [
    (5, 'read_cbytes', 'utils'),
    (14, 'read_cstring', 'utils'),
    (20, 'current_time_millis_1', 'utils'),
    (29, 'current_time_millis_2', 'utils'),
    (38, 'parse_format_spec', 'utils'),
    (113, 'parse_format_specs', 'utils'),
    (130, 'apply_width', 'utils'),
    (149, 'trim_trailing_zeros_and_dot', 'utils'),
    (160, 'format_g', 'utils'),
    (200, 'format_printf_string', 'utils'),
    (302, 'execute_host_func', 'parent'),
    (413, 'host_output', 'parent'),
    (418, 'host_step', 'parent'),
    (427, 'host_malloc', 'memory'),
    (487, 'host_free', 'memory'),
    (520, 'host_printf_n', 'io'),
    (539, 'parse_scanf_specs', 'utils'),
    (613, 'read_float_token', 'utils'),
    (631, 'host_scanf_n', 'io'),
    (744, 'host_strlen', 'string'),
    (750, 'host_strdup', 'string'),
    (820, 'host_strcpy', 'string'),
    (860, 'host_strcmp', 'string'),
    (888, 'MemorySlice', 'utils'),
    (892, 'impl_MemorySlice', 'utils'),
    (898, 'host_ungetc', 'io'),
    (905, 'host_getchar', 'io'),
    (951, 'host_putchar', 'io'),
    (956, 'host_rand', 'misc'),
    (963, 'host_srand', 'misc'),
    (968, 'host_memset', 'string'),
    (992, 'host_exit', 'misc'),
    (997, 'host_strcat', 'string'),
    (1041, 'host_strncpy', 'string'),
    (1063, 'host_memcpy', 'string'),
    (1086, 'host_memmove', 'string'),
    (1109, 'host_abs', 'math'),
    (1116, 'host_sin', 'math'),
    (1121, 'host_cos', 'math'),
    (1126, 'host_sqrt', 'math'),
    (1131, 'host_pow', 'math'),
    (1137, 'host_atan', 'math'),
    (1142, 'host_log', 'math'),
    (1147, 'host_exp', 'math'),
    (1152, 'host_isdigit', 'misc'),
    (1157, 'host_isalpha', 'misc'),
    (1168, 'host_islower', 'misc'),
    (1173, 'host_isupper', 'misc'),
    (1178, 'host_tolower', 'misc'),
    (1187, 'host_toupper', 'misc'),
    (1196, 'host_isspace', 'misc'),
    (1213, 'host_isalnum', 'misc'),
    (1220, 'host_isprint', 'misc'),
    (1225, 'host_iscntrl', 'misc'),
    (1230, 'host_isxdigit', 'misc'),
    (1238, 'host_isgraph', 'misc'),
    (1243, 'host_ispunct', 'misc'),
    (1252, 'host_isblank', 'misc'),
    (1257, 'host_atoi', 'misc'),
    (1282, 'host_fprintf_n', 'io'),
    (1302, 'host_realloc', 'memory'),
    (1482, 'MAX_QSORT_DEPTH', 'misc'),
    (1484, 'host_qsort', 'misc'),
    (1560, 'host_fopen', 'file'),
    (1612, 'host_fread', 'file'),
    (1625, 'host_fwrite', 'file'),
    (1638, 'host_fclose', 'file'),
    (1647, 'host_feof', 'file'),
    (1654, 'host_fgets', 'file'),
    (1664, 'host_fputs', 'file'),
    (1676, 'read_fd_from_stream', 'utils'),
    (1690, 'host_puts', 'io'),
    (1697, 'host_calloc', 'memory'),
    (1737, 'MAX_BSEARCH_DEPTH', 'misc'),
    (1739, 'host_bsearch', 'misc'),
    (1822, 'host_sprintf', 'io'),
    (1840, 'host_snprintf', 'io'),
    (1862, 'host_sscanf', 'io'),
    (1977, 'host_fgetc', 'file'),
    (1986, 'host_fputc', 'file'),
    (1996, 'host_fseek', 'file'),
    (2007, 'host_ftell', 'file'),
    (2016, 'host_rewind', 'file'),
    (2026, 'host_strncat', 'string'),
    (2050, 'host_strncmp', 'string'),
    (2081, 'host_memcmp', 'string'),
    (2106, 'host_strchr', 'string'),
    (2128, 'host_strrchr', 'string'),
    (2147, 'host_strstr', 'string'),
    (2165, 'host_memchr', 'string'),
    (2185, 'host_atof', 'misc'),
    (2192, 'host_atol', 'misc'),
    (2219, 'host_tan', 'math'),
    (2224, 'host_log10', 'math'),
    (2229, 'host_fabs', 'math'),
    (2234, 'host_ceil', 'math'),
    (2239, 'host_floor', 'math'),
    (2244, 'host_round', 'math'),
    (2249, 'host_fmod', 'math'),
    (2255, 'host_asin', 'math'),
    (2260, 'host_acos', 'math'),
    (2265, 'host_atan2', 'math'),
    (2271, 'host_sinh', 'math'),
    (2276, 'host_cosh', 'math'),
    (2281, 'host_tanh', 'math'),
    (2286, 'host_llabs', 'math'),
    (2291, 'host_abort', 'misc'),
    (2296, 'set_errno', 'utils'),
    (2305, 'host_strtol', 'misc'),
    (2354, 'host_strtod', 'misc'),
    (2398, 'host_strerror', 'misc'),
    (2419, 'host_fflush', 'file'),
    (2433, 'host_perror', 'file'),
    (2444, 'host_clearerr', 'file'),
    (2452, 'host_time', 'misc'),
    (2458, 'host_clock', 'misc'),
    (2465, 'host_cide_assert_fail', 'misc'),
    (2470, 'host_set_array_guard', 'misc'),
    (2478, 'host_clear_array_guard', 'misc'),
    (2482, 'host_remove', 'file'),
    (2491, 'host_rename', 'file'),
    (2502, 'host_strpbrk', 'string'),
    (2516, 'host_strspn', 'string'),
    (2532, 'host_strcspn', 'string'),
]

PARENT_NAMES = {'execute_host_func', 'host_output', 'host_step'}


def read_lines(path):
    with open(path, 'r', encoding='utf-8') as f:
        return f.read().splitlines()


def extract_items(lines):
    """Return list of dicts with keys name, module, start, end, text_lines."""
    starts = [x[0] for x in ITEMS]
    result = []
    for idx, (start, name, module) in enumerate(ITEMS):
        # 1-based start -> 0-based index
        s = start - 1
        if idx + 1 < len(ITEMS):
            e = ITEMS[idx + 1][0] - 1  # line before next item
        else:
            e = len(lines)
        text_lines = lines[s:e]
        result.append({
            'name': name,
            'module': module,
            'start': start,
            'end': e,
            'text_lines': text_lines,
        })
    return result


def apply_module_replacements(text_lines):
    """Make paths relative to the new host_funcs parent re-exports."""
    out = []
    for line in text_lines:
        line = line.replace('super::instruction::SourceLoc', 'SourceLoc')
        line = line.replace('super::core::NULL_TRAP_SIZE', 'NULL_TRAP_SIZE')
        line = line.replace('super::core::FreedRegionInfo', 'FreedRegionInfo')
        line = line.replace('crate::session::FreeBlock', 'FreeBlock')
        line = line.replace('crate::vm::core::ArrayConstructionGuard', 'ArrayConstructionGuard')
        out.append(line)
    return out


def make_utils_pubcrate(text_lines):
    """Make helper functions/trait crate-visible so the glob re-export works."""
    out = []
    for line in text_lines:
        if line.startswith('pub trait MemorySlice'):
            line = line.replace('pub trait MemorySlice', 'pub(crate) trait MemorySlice', 1)
        elif line.startswith('fn '):
            line = 'pub(crate) fn ' + line[3:]
        out.append(line)
    return out


def make_hosts_public(text_lines):
    """Ensure all host_xxx functions are public so the glob re-export works."""
    out = []
    for line in text_lines:
        if line.startswith('fn host_'):
            line = 'pub fn host_' + line[8:]
        out.append(line)
    return out


def build_host_funcs(items_by_module, completed_modules):
    """Build host_funcs.rs content for the current completed_modules set."""
    header_lines = [
        'use super::core::CideVM;',
        'use super::host_func_id;',
        'use crate::session::{MemoryRegion, Session};',
        '',
        'pub(crate) use super::core::{ArrayConstructionGuard, FreedRegionInfo, NULL_TRAP_SIZE};',
        'pub(crate) use super::instruction::SourceLoc;',
        'pub(crate) use crate::session::FreeBlock;',
        '',
    ]
    for mod in completed_modules:
        header_lines.append(f'#[path = "host/{mod}.rs"]')
        header_lines.append(f'mod {mod};')
    for mod in completed_modules:
        if mod == 'utils':
            header_lines.append(f'pub(crate) use {mod}::*;')
        else:
            header_lines.append(f'pub use {mod}::*;')
    header_lines.append('')

    out_lines = list(header_lines)

    # Remaining items (not moved yet) in original order
    for it in ITEMS:
        name = it[1]
        module = it[2]
        if module == 'parent':
            continue
        if module in completed_modules:
            continue
        # find item text
        item = items_by_module[module][name]
        out_lines.extend(item['text_lines'])

    # Parent items at the end
    parent_items = [items_by_module['parent'][n] for n in PARENT_NAMES]
    for item in parent_items:
        out_lines.extend(item['text_lines'])

    return '\n'.join(out_lines) + '\n'


def write_file(path, content):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'w', encoding='utf-8') as f:
        f.write(content)


def run_cargo_check():
    print('    -> cargo check ...')
    proc = subprocess.run(
        ['cargo', 'check'],
        cwd=NATIVE_DIR,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        encoding='utf-8',
        errors='replace',
    )
    if proc.returncode != 0:
        print(proc.stdout)
        raise SystemExit('cargo check failed')
    print('    -> cargo check OK')


def git_commit(message):
    print(f'    -> committing: {message}')
    subprocess.run(['git', 'add', '-A'], cwd=ROOT, check=True)
    subprocess.run(['git', 'commit', '-m', message], cwd=ROOT, check=True)


def main():
    print(f'Reading {HOST_FUNCS}')
    lines = read_lines(HOST_FUNCS)
    all_items = extract_items(lines)

    # Build lookup: module -> name -> item
    items_by_module = {}
    for item in all_items:
        items_by_module.setdefault(item['module'], {})[item['name']] = item

    # Backup original
    backup_path = HOST_FUNCS + '.orig'
    if not os.path.exists(backup_path):
        write_file(backup_path, '\n'.join(lines) + '\n')

    os.makedirs(HOST_DIR, exist_ok=True)

    order = ['utils', 'memory', 'string', 'io', 'file', 'math', 'misc']
    completed = []

    for mod in order:
        print(f'\n=== Extracting module: {mod} ===')
        module_items = [it for it in all_items if it['module'] == mod]
        # Build module file content
        module_lines = []
        if mod == 'utils':
            module_lines.append('use super::CideVM;')
        else:
            module_lines.append('use super::*;')
        module_lines.append('')
        for item in module_items:
            replaced = apply_module_replacements(item['text_lines'])
            if mod == 'utils':
                replaced = make_utils_pubcrate(replaced)
            else:
                replaced = make_hosts_public(replaced)
            module_lines.extend(replaced)
        module_path = os.path.join(HOST_DIR, f'{mod}.rs')
        write_file(module_path, '\n'.join(module_lines) + '\n')

        completed.append(mod)
        host_funcs_content = build_host_funcs(items_by_module, completed)
        write_file(HOST_FUNCS, host_funcs_content)

        run_cargo_check()
        git_commit(
            f'refactor(vm): P1 拆分 host_funcs.rs — 提取 {mod} 子模块\n\n'
            f'- 迁移相关 host 函数到 vm/host/{mod}.rs\n'
            '- cargo check 通过'
        )

    print('\n=== All modules extracted ===')
    print(f'Final {HOST_FUNCS} written.')


if __name__ == '__main__':
    main()
