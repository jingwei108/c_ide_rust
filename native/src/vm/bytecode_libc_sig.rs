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
        // list<int> (Stage 2b mangled method names)
        "cide_list_int__push_back" => Some((void.clone(), vec![int_ptr.clone(), int.clone()])),
        "cide_list_int__push_front" => Some((void.clone(), vec![int_ptr.clone(), int.clone()])),
        "cide_list_int__pop_back" => Some((int.clone(), vec![int_ptr.clone()])),
        "cide_list_int__size" => Some((int.clone(), vec![int_ptr.clone()])),
        "cide_list_int__front" => Some((int.clone(), vec![int_ptr.clone()])),
        "cide_list_int__back" => Some((int.clone(), vec![int_ptr.clone()])),
        "cide_list_int__pop_front" => Some((void.clone(), vec![int_ptr.clone()])),
        "cide_list_int__get" => Some((int.clone(), vec![int_ptr.clone(), int.clone()])),
        "cide_list_int__clear" => Some((void.clone(), vec![int_ptr.clone()])),
        "__dtor__cide_list_int" => Some((void.clone(), vec![int_ptr.clone()])),
        // sort
        "cide_sort_int_swap" => Some((void.clone(), vec![int_ptr.clone(), int_ptr.clone()])),
        "cide_sort_int_qsort" => Some((void.clone(), vec![int_ptr.clone(), int.clone(), int.clone()])),
        "cide_sort_int" => Some((void.clone(), vec![int_ptr.clone(), int.clone()])),
        // string (Stage 2b mangled method names)
        "cide_string__push_back" => Some((void.clone(), vec![Type::pointer_to(Type::void()), char.clone()])),
        "cide_string__pop_back" => Some((char.clone(), vec![Type::pointer_to(Type::void())])),
        "cide_string__size" => Some((int.clone(), vec![Type::pointer_to(Type::void())])),
        "cide_string__capacity" => Some((int.clone(), vec![Type::pointer_to(Type::void())])),
        "cide_string__get" => Some((char.clone(), vec![Type::pointer_to(Type::void()), int.clone()])),
        "cide_string__c_str" => Some((Type::pointer_to(Type::char()), vec![Type::pointer_to(Type::void())])),
        "cide_string__front" => Some((char.clone(), vec![Type::pointer_to(Type::void())])),
        "cide_string__back" => Some((char.clone(), vec![Type::pointer_to(Type::void())])),
        "cide_string__pop_front" => Some((void.clone(), vec![Type::pointer_to(Type::void())])),
        "cide_string__clear" => Some((void.clone(), vec![Type::pointer_to(Type::void())])),
        "__dtor__cide_string" => Some((void.clone(), vec![Type::pointer_to(Type::void())])),
        // vec<char> (Stage 2b mangled method names)
        "cide_vec_char__push_back" => Some((void.clone(), vec![char_ptr.clone(), char.clone()])),
        "cide_vec_char__pop_back" => Some((char.clone(), vec![char_ptr.clone()])),
        "cide_vec_char__size" => Some((int.clone(), vec![char_ptr.clone()])),
        "cide_vec_char__capacity" => Some((int.clone(), vec![char_ptr.clone()])),
        "cide_vec_char__front" => Some((char.clone(), vec![char_ptr.clone()])),
        "cide_vec_char__back" => Some((char.clone(), vec![char_ptr.clone()])),
        "cide_vec_char__pop_front" => Some((void.clone(), vec![char_ptr.clone()])),
        "cide_vec_char__get" => Some((char.clone(), vec![char_ptr.clone(), int.clone()])),
        "cide_vec_char__clear" => Some((void.clone(), vec![char_ptr.clone()])),
        "__dtor__cide_vec_char" => Some((void.clone(), vec![char_ptr.clone()])),
        // vec<float> (Stage 2b mangled method names)
        "cide_vec_float__push_back" => Some((void.clone(), vec![float_ptr.clone(), float.clone()])),
        "cide_vec_float__pop_back" => Some((float.clone(), vec![float_ptr.clone()])),
        "cide_vec_float__size" => Some((int.clone(), vec![float_ptr.clone()])),
        "cide_vec_float__capacity" => Some((int.clone(), vec![float_ptr.clone()])),
        "cide_vec_float__front" => Some((float.clone(), vec![float_ptr.clone()])),
        "cide_vec_float__back" => Some((float.clone(), vec![float_ptr.clone()])),
        "cide_vec_float__pop_front" => Some((void.clone(), vec![float_ptr.clone()])),
        "cide_vec_float__get" => Some((float.clone(), vec![float_ptr.clone(), int.clone()])),
        "cide_vec_float__clear" => Some((void.clone(), vec![float_ptr.clone()])),
        "__dtor__cide_vec_float" => Some((void.clone(), vec![float_ptr.clone()])),
        // vec<int> (Stage 2b mangled method names)
        "cide_vec_int__push_back" => Some((void.clone(), vec![int_ptr.clone(), int.clone()])),
        "cide_vec_int__pop_back" => Some((int.clone(), vec![int_ptr.clone()])),
        "cide_vec_int__size" => Some((int.clone(), vec![int_ptr.clone()])),
        "cide_vec_int__capacity" => Some((int.clone(), vec![int_ptr.clone()])),
        "cide_vec_int__front" => Some((int.clone(), vec![int_ptr.clone()])),
        "cide_vec_int__back" => Some((int.clone(), vec![int_ptr.clone()])),
        "cide_vec_int__pop_front" => Some((void.clone(), vec![int_ptr.clone()])),
        "cide_vec_int__get" => Some((int.clone(), vec![int_ptr.clone(), int.clone()])),
        "cide_vec_int__clear" => Some((void.clone(), vec![int_ptr.clone()])),
        "__dtor__cide_vec_int" => Some((void.clone(), vec![int_ptr.clone()])),
        // ctype
        "isdigit" | "isalpha" | "islower" | "isupper" | "isspace" | "isalnum" | "isprint" | "iscntrl" | "isxdigit"
        | "tolower" | "toupper" => Some((int.clone(), vec![int.clone()])),
        // misc
        "abs" => Some((int.clone(), vec![int.clone()])),
        "atoi" => Some((int.clone(), vec![Type::pointer_to(Type::char())])),
        "srand" => Some((void.clone(), vec![int.clone()])),
        "rand" => Some((int.clone(), vec![])),
        "strlen" => Some((int.clone(), vec![Type::pointer_to(Type::char())])),
        "strcmp" => Some((
            int.clone(),
            vec![Type::pointer_to(Type::char()), Type::pointer_to(Type::char())],
        )),
        "strcpy" => Some((
            void.clone(),
            vec![Type::pointer_to(Type::char()), Type::pointer_to(Type::char())],
        )),
        "strcat" => Some((
            void.clone(),
            vec![Type::pointer_to(Type::char()), Type::pointer_to(Type::char())],
        )),
        "strncpy" => Some((
            void.clone(),
            vec![Type::pointer_to(Type::char()), Type::pointer_to(Type::char()), int.clone()],
        )),
        "memcpy" => Some((
            void.clone(),
            vec![Type::pointer_to(Type::void()), Type::pointer_to(Type::void()), int.clone()],
        )),
        "memmove" => Some((
            void.clone(),
            vec![Type::pointer_to(Type::void()), Type::pointer_to(Type::void()), int.clone()],
        )),
        _ => None,
    }
}
