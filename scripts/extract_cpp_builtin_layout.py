#!/usr/bin/env python3
"""
提取 Cide 内置 C++ 容器接口声明，生成 builtin_layout_data.json。
输入: native/runtime_libc/cide/*.cpp
输出: native/src/compiler/cpp_frontend/builtin_layout_data.json
"""

import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

# 相对于项目根目录
INPUT_DIR = Path("native/runtime_libc/cide")
OUTPUT_PATH = Path("native/src/compiler/cpp_frontend/builtin_layout_data.json")

# 类型到 size 的映射（与 Cide VM 32/64 位模型对齐，当前用 4 字节指针）
TYPE_SIZE = {
    "int": 4,
    "float": 4,
    "char": 1,
    "double": 8,
    "void": 1,      # void 本身不占空间，但 void* 占 4
    "void*": 4,
    "int*": 4,
    "float*": 4,
    "char*": 4,
    "double*": 4,
}

# C++ 类名 → cide 前缀/命名规则
CLASS_RULES = {
    "vector": ("vec", True),   # (prefix, has_type_suffix)
    "list":   ("list", True),
    "string": ("string", False),
}


def strip_comments(text: str) -> str:
    """移除 C/C++ 风格的 // 和 /* */ 注释。"""
    # 移除 // 行尾注释
    text = re.sub(r'//.*?$', '', text, flags=re.MULTILINE)
    # 移除 /* */ 块注释
    text = re.sub(r'/\*.*?\*/', '', text, flags=re.DOTALL)
    return text


def parse_type_size(ty: str) -> int:
    """根据类型字符串计算简单 size。"""
    ty = ty.strip()
    if ty in TYPE_SIZE:
        return TYPE_SIZE[ty]
    if ty.endswith("*"):
        return 4
    # 未知类型 fallback
    return 4


def cpp_name_to_cide(cpp_name: str) -> str:
    """
    将 C++ 类型名推导为 Cide 内部类型名。
    e.g. vector<int> -> cide_vec_int
         string -> cide_string
    """
    m = re.match(r'^(\w+)<(\w+)>$', cpp_name)
    if m:
        cls, inst = m.group(1), m.group(2)
        prefix, has_suffix = CLASS_RULES.get(cls, (cls, True))
        if has_suffix:
            return f"cide_{prefix}_{inst}"
        return f"cide_{prefix}"
    # 非模板类
    prefix, _ = CLASS_RULES.get(cpp_name, (cpp_name, False))
    return f"cide_{prefix}"


# 历史遗留：vector 的 push_back/pop_back 在 C 函数名中省略了 _back
VEC_METHOD_MAP = {
    "push_back": "push",
    "pop_back": "pop",
}


def derive_method_c_name(cide_class: str, method: str, inst_type: str | None) -> str:
    """
    根据实际 type_map.rs 中的映射规则推导 C 函数名。
    """
    if cide_class == "cide_string":
        return f"cide_string_{method}"
    if cide_class.startswith("cide_vec_"):
        mapped = VEC_METHOD_MAP.get(method, method)
        return f"cide_vec_{mapped}_{inst_type}"
    if cide_class.startswith("cide_list_"):
        return f"cide_list_{method}_{inst_type}"
    # fallback
    if inst_type:
        return f"{cide_class}_{method}_{inst_type}"
    return f"{cide_class}_{method}"


def extract_explicit_instantiations(text: str) -> list[tuple[str, str, str]]:
    """
    提取显式模板实例化。
    返回: [(cpp_name, class_name, inst_type), ...]
    e.g. ("vector<int>", "vector", "int")
    """
    results = []
    # template class vector<int>;
    for m in re.finditer(r'template\s+class\s+(\w+)<(\w+)>\s*;', text):
        cls, inst = m.group(1), m.group(2)
        results.append((f"{cls}<{inst}>", cls, inst))
    return results


def extract_class_definition(text: str, class_name: str) -> str | None:
    """
    从文本中提取 class/struct 定义体（大括号内内容）。
    支持 template<class T> class Foo { ... };
    """
    # 匹配 template...class class_name { ... };
    pattern = rf'(?:template\s*<\s*class\s+\w+\s*>\s*)?\bclass\s+{re.escape(class_name)}\s*\{{'
    m = re.search(pattern, text)
    if not m:
        return None

    start = m.end() - 1  # 指向 '{'
    brace_count = 0
    i = start
    while i < len(text):
        if text[i] == '{':
            brace_count += 1
        elif text[i] == '}':
            brace_count -= 1
            if brace_count == 0:
                return text[start + 1:i]
        i += 1
    return None


def parse_fields(body: str, inst_type: str | None) -> list[dict]:
    """解析 class 体内的字段声明。"""
    fields = []
    # 简单正则：匹配 type name;  或 type* name;
    # 避开方法括号
    for m in re.finditer(r'\b(\w+(?:\s*\*)?)\s+(\w+)\s*;', body):
        ty, name = m.group(1).strip(), m.group(2).strip()
        # 跳过在方法参数列表内的匹配（简单启发：前面有 ( ）
        before = body[:m.start()]
        if before.rstrip().endswith('('):
            continue
        # 替换模板参数 T
        if inst_type:
            ty = re.sub(r'\bT\b', inst_type, ty)
            ty = re.sub(r'\bT\s*\*', inst_type + '*', ty)
        fields.append({"name": name, "type": ty})
    return fields


def parse_methods(body: str, inst_type: str | None) -> list[dict]:
    """解析 public 方法声明。"""
    methods = []
    # 先按 public:/private:/protected: 分段
    parts = re.split(r'\b(public|private|protected)\s*:', body)
    current_access = "private"
    for part in parts:
        part = part.strip()
        if part in ("public", "private", "protected"):
            current_access = part
            continue
        if current_access != "public":
            continue

        # 逐句提取（以 ; 结尾）
        stmts = [s.strip() for s in part.split(';') if s.strip()]
        for stmt in stmts:
            # 跳过字段（无括号）
            if '(' not in stmt:
                continue
            # 析构函数
            dm = re.match(r'~\s*(\w+)\s*\(\s*\)', stmt)
            if dm:
                methods.append({
                    "name": "destroy",
                    "params": [],
                    "ret": "void",
                    "is_virtual": False,
                })
                continue

            # 构造函数: ClassName()
            cm = re.match(r'(\w+)\s*\(\s*\)', stmt)
            if cm:
                ctor_name = cm.group(1)
                # 如果 stmt 前面没有返回类型关键字，且名字与类名匹配，则是构造函数
                prefix = stmt[:stmt.find('(')].strip()
                if prefix == ctor_name and ctor_name not in ("void", "int", "float", "char", "double", "bool"):
                    continue  # 构造函数不加入列表

            # 普通方法: ret name(params)
            mm = re.match(r'((?:\w+(?:\s*\*)?\s+)?)(\w+)\s*\(([^)]*)\)', stmt)
            if mm:
                ret_raw = mm.group(1).strip()
                name = mm.group(2).strip()
                params_raw = mm.group(3).strip()

                # 跳过非方法
                if name in ("return", "if", "while", "for"):
                    continue

                # 处理返回类型
                if not ret_raw:
                    ret = "void"
                else:
                    ret = ret_raw
                    if inst_type:
                        ret = re.sub(r'\bT\b', inst_type, ret)
                        ret = re.sub(r'\bT\s*\*', inst_type + '*', ret)

                # 处理参数
                params = []
                if params_raw:
                    for p in params_raw.split(','):
                        p = p.strip()
                        if not p:
                            continue
                        # 提取类型（忽略参数名）
                        pm = re.match(r'(\w+(?:\s*\*)?)\s+(?:\w+)', p)
                        if pm:
                            pty = pm.group(1).strip()
                        else:
                            pty = p
                        if inst_type:
                            pty = re.sub(r'\bT\b', inst_type, pty)
                            pty = re.sub(r'\bT\s*\*', inst_type + '*', pty)
                        params.append(pty)

                methods.append({
                    "name": name,
                    "params": params,
                    "ret": ret,
                    "is_virtual": False,
                })

    return methods


def process_cpp_file(path: Path) -> dict:
    """处理单个 .cpp 文件，返回 {cide_name: class_data, ...}"""
    text = path.read_text(encoding='utf-8')
    text = strip_comments(text)

    insts = extract_explicit_instantiations(text)
    results = {}

    # 如果没有显式实例化（如 string），尝试直接提取 class
    if not insts:
        cm = re.search(r'\bclass\s+(\w+)\s*\{', text)
        if cm:
            cls_name = cm.group(1)
            body = extract_class_definition(text, cls_name)
            if body:
                cpp_name = cls_name
                cide_name = cpp_name_to_cide(cpp_name)
                fields = parse_fields(body, None)
                methods = parse_methods(body, None)
                size = sum(parse_type_size(f["type"]) for f in fields)
                # 方法映射
                method_map = {}
                for m in methods:
                    c_fn = derive_method_c_name(cide_name, m["name"], None)
                    method_map[m["name"]] = c_fn
                results[cide_name] = {
                    "cpp_name": cpp_name,
                    "source_file": str(path).replace("\\", "/"),
                    "size": size,
                    "fields": fields,
                    "methods": methods,
                    "method_map": method_map,
                }
        return results

    # 有显式实例化的模板类
    for cpp_name, cls_name, inst_type in insts:
        body = extract_class_definition(text, cls_name)
        if not body:
            continue
        cide_name = cpp_name_to_cide(cpp_name)
        fields = parse_fields(body, inst_type)
        methods = parse_methods(body, inst_type)
        size = sum(parse_type_size(f["type"]) for f in fields)
        method_map = {}
        for m in methods:
            c_fn = derive_method_c_name(cide_name, m["name"], inst_type)
            method_map[m["name"]] = c_fn
        results[cide_name] = {
            "cpp_name": cpp_name,
            "source_file": str(path).replace("\\", "/"),
            "size": size,
            "fields": fields,
            "methods": methods,
            "method_map": method_map,
        }

    return results


def main():
    all_classes = {}
    all_method_map = {}

    for cpp_path in sorted(INPUT_DIR.glob("*.cpp")):
        classes = process_cpp_file(cpp_path)
        for cide_name, data in classes.items():
            all_classes[cide_name] = {
                "cpp_name": data["cpp_name"],
                "source_file": data["source_file"],
                "size": data["size"],
                "fields": data["fields"],
                "methods": data["methods"],
            }
            all_method_map[cide_name] = data["method_map"]

    output = {
        "version": 1,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "classes": all_classes,
        "method_map": all_method_map,
    }

    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    with open(OUTPUT_PATH, 'w', encoding='utf-8') as f:
        json.dump(output, f, indent=2, ensure_ascii=False)

    print(f"Generated {OUTPUT_PATH} with {len(all_classes)} classes.")


if __name__ == "__main__":
    main()
