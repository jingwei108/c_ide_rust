use std::collections::HashMap;
use std::sync::LazyLock;

static KLIB_TYPE_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("vector<int>", "cide_vec_int");
    m.insert("vector<float>", "cide_vec_float");
    m.insert("vector<char>", "cide_vec_char");
    m.insert("list<int>", "cide_list_int");
    m.insert("string", "cide_string");
    m
});

static KLIB_METHOD_MAP: LazyLock<HashMap<(&'static str, &'static str), &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(("cide_vec_int", "push_back"), "cide_vec_push_int");
    m.insert(("cide_vec_int", "pop_back"), "cide_vec_pop_int");
    m.insert(("cide_vec_int", "size"), "cide_vec_size_int");
    m.insert(("cide_vec_int", "get"), "cide_vec_get_int");
    m.insert(("cide_vec_int", "clear"), "cide_vec_clear_int");
    m.insert(("cide_vec_int", "destroy"), "cide_vec_destroy_int");
    m.insert(("cide_vec_float", "push_back"), "cide_vec_push_float");
    m.insert(("cide_vec_float", "pop_back"), "cide_vec_pop_float");
    m.insert(("cide_vec_float", "size"), "cide_vec_size_float");
    m.insert(("cide_vec_float", "get"), "cide_vec_get_float");
    m.insert(("cide_vec_float", "clear"), "cide_vec_clear_float");
    m.insert(("cide_vec_float", "destroy"), "cide_vec_destroy_float");
    m.insert(("cide_vec_char", "push_back"), "cide_vec_push_char");
    m.insert(("cide_vec_char", "pop_back"), "cide_vec_pop_char");
    m.insert(("cide_vec_char", "size"), "cide_vec_size_char");
    m.insert(("cide_vec_char", "get"), "cide_vec_get_char");
    m.insert(("cide_vec_char", "clear"), "cide_vec_clear_char");
    m.insert(("cide_vec_char", "destroy"), "cide_vec_destroy_char");
    m.insert(("cide_string", "push_back"), "cide_string_push_back");
    m.insert(("cide_string", "pop_back"), "cide_string_pop_back");
    m.insert(("cide_string", "size"), "cide_string_size");
    m.insert(("cide_string", "get"), "cide_string_get");
    m.insert(("cide_string", "clear"), "cide_string_clear");
    m.insert(("cide_string", "destroy"), "cide_string_destroy");
    m
});

/// 将 C++ 标准类型名映射到 Cide 容器库的内部类型名。
/// 例如：`vector<int>` → `cide_vec_int`
pub fn cpp_type_to_cide(name: &str) -> Option<&'static str> {
    KLIB_TYPE_MAP.get(name).copied()
}

/// 检查给定的类型名是否是内置容器类型。
pub fn is_builtin_container(name: &str) -> bool {
    KLIB_TYPE_MAP.contains_key(name)
}

/// 将内置容器的方法调用映射为 Cide 容器库函数名。
/// 例如：`("cide_vec_int", "push_back")` → `"cide_vec_push_int"`
pub fn map_container_method(class_name: &str, method: &str) -> Option<&'static str> {
    KLIB_METHOD_MAP.get(&(class_name, method)).copied()
}
