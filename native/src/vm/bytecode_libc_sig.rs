use crate::compiler::ast::Type;

/// 返回 Bytecode Libc 函数的签名 (return_type, param_types)。
pub fn bytecode_libc_sig(name: &str) -> Option<(Type, Vec<Type>)> {
    let void = Type::void();
    let int = Type::int();
    let float = Type::float();
    let char = Type::char();
    let int_ptr = Type::pointer_to(Type::int());
    let float_ptr = Type::pointer_to(Type::float());
    let char_ptr = Type::pointer_to(Type::char());
    match name {
        // list<int>
        "cide_list_init_int" => Some((void.clone(), vec![int_ptr.clone()])),
        "cide_list_push_back_int" => Some((void.clone(), vec![int_ptr.clone(), int.clone()])),
        "cide_list_push_front_int" => Some((void.clone(), vec![int_ptr.clone(), int.clone()])),
        "cide_list_pop_back_int" => Some((int.clone(), vec![int_ptr.clone()])),
        "cide_list_size_int" => Some((int.clone(), vec![int_ptr.clone()])),
        "cide_list_get_int" => Some((int.clone(), vec![int_ptr.clone(), int.clone()])),
        "cide_list_destroy_int" => Some((void.clone(), vec![int_ptr.clone()])),
        // sort
        "cide_sort_int_swap" => Some((void.clone(), vec![int_ptr.clone(), int_ptr.clone()])),
        "cide_sort_int_qsort" => Some((void.clone(), vec![int_ptr.clone(), int.clone(), int.clone()])),
        "cide_sort_int" => Some((void.clone(), vec![int_ptr.clone(), int.clone()])),
        // string
        "cide_string_init" => Some((void.clone(), vec![Type::pointer_to(Type::void())])),
        "cide_string_push_back" => Some((void.clone(), vec![Type::pointer_to(Type::void()), char.clone()])),
        "cide_string_pop_back" => Some((void.clone(), vec![Type::pointer_to(Type::void())])),
        "cide_string_size" => Some((int.clone(), vec![Type::pointer_to(Type::void())])),
        "cide_string_get" => Some((char.clone(), vec![Type::pointer_to(Type::void()), int.clone()])),
        "cide_string_clear" => Some((void.clone(), vec![Type::pointer_to(Type::void())])),
        "cide_string_destroy" => Some((void.clone(), vec![Type::pointer_to(Type::void())])),
        // vec<char>
        "cide_vec_init_char" => Some((void.clone(), vec![char_ptr.clone()])),
        "cide_vec_push_char" => Some((void.clone(), vec![char_ptr.clone(), char.clone()])),
        "cide_vec_pop_char" => Some((char.clone(), vec![char_ptr.clone()])),
        "cide_vec_size_char" => Some((int.clone(), vec![char_ptr.clone()])),
        "cide_vec_get_char" => Some((char.clone(), vec![char_ptr.clone(), int.clone()])),
        "cide_vec_clear_char" => Some((void.clone(), vec![char_ptr.clone()])),
        "cide_vec_destroy_char" => Some((void.clone(), vec![char_ptr.clone()])),
        // vec<float>
        "cide_vec_init_float" => Some((void.clone(), vec![float_ptr.clone()])),
        "cide_vec_push_float" => Some((void.clone(), vec![float_ptr.clone(), float.clone()])),
        "cide_vec_pop_float" => Some((float.clone(), vec![float_ptr.clone()])),
        "cide_vec_size_float" => Some((int.clone(), vec![float_ptr.clone()])),
        "cide_vec_get_float" => Some((float.clone(), vec![float_ptr.clone(), int.clone()])),
        "cide_vec_clear_float" => Some((void.clone(), vec![float_ptr.clone()])),
        "cide_vec_destroy_float" => Some((void.clone(), vec![float_ptr.clone()])),
        // vec<int>
        "cide_vec_init_int" => Some((void.clone(), vec![int_ptr.clone()])),
        "cide_vec_push_int" => Some((void.clone(), vec![int_ptr.clone(), int.clone()])),
        "cide_vec_pop_int" => Some((int.clone(), vec![int_ptr.clone()])),
        "cide_vec_size_int" => Some((int.clone(), vec![int_ptr.clone()])),
        "cide_vec_get_int" => Some((int.clone(), vec![int_ptr.clone(), int.clone()])),
        "cide_vec_clear_int" => Some((void.clone(), vec![int_ptr.clone()])),
        "cide_vec_destroy_int" => Some((void.clone(), vec![int_ptr.clone()])),
        // ctype
        "isdigit" | "isalpha" | "islower" | "isupper" | "isspace" | "isalnum" | "isprint" | "iscntrl" | "isxdigit" | "tolower" | "toupper" => Some((int.clone(), vec![int.clone()])),
        // misc
        "abs" => Some((int.clone(), vec![int.clone()])),
        "atoi" => Some((int.clone(), vec![Type::pointer_to(Type::char())])),
        "srand" => Some((void.clone(), vec![int.clone()])),
        "rand" => Some((int.clone(), vec![])),
        "strlen" => Some((int.clone(), vec![Type::pointer_to(Type::char())])),
        "strcmp" => Some((int.clone(), vec![Type::pointer_to(Type::char()), Type::pointer_to(Type::char())])),
        "strcpy" => Some((void.clone(), vec![Type::pointer_to(Type::char()), Type::pointer_to(Type::char())])),
        "strcat" => Some((void.clone(), vec![Type::pointer_to(Type::char()), Type::pointer_to(Type::char())])),
        "strncpy" => Some((void.clone(), vec![Type::pointer_to(Type::char()), Type::pointer_to(Type::char()), int.clone()])),
        "memcpy" => Some((void.clone(), vec![Type::pointer_to(Type::void()), Type::pointer_to(Type::void()), int.clone()])),
        "memmove" => Some((void.clone(), vec![Type::pointer_to(Type::void()), Type::pointer_to(Type::void()), int.clone()])),
        _ => None,
    }
}
