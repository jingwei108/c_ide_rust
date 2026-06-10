use crate::compiler::ast::Type;
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct ClassLayout {
    pub size: i32,
    pub fields: Vec<(String, Type)>,
    pub methods: Vec<MethodSig>,
}

#[derive(Debug, Clone)]
pub struct MethodSig {
    pub name: String,
    pub params: Vec<Type>,
    pub ret: Type,
    pub is_virtual: bool,
}

fn builtin_layouts() -> HashMap<String, ClassLayout> {
    let mut m = HashMap::new();

    // vector_int
    m.insert(
        "cide_vec_int".to_string(),
        ClassLayout {
            size: 12,
            fields: vec![
                ("n".to_string(), Type::int()),
                ("m".to_string(), Type::int()),
                ("a".to_string(), Type::pointer_to(Type::int())),
            ],
            methods: vec![
                MethodSig { name: "push_back".to_string(), params: vec![Type::int()], ret: Type::void(), is_virtual: false },
                MethodSig { name: "pop_back".to_string(), params: vec![], ret: Type::int(), is_virtual: false },
                MethodSig { name: "size".to_string(), params: vec![], ret: Type::int(), is_virtual: false },
                MethodSig { name: "capacity".to_string(), params: vec![], ret: Type::int(), is_virtual: false },
                MethodSig { name: "front".to_string(), params: vec![], ret: Type::int(), is_virtual: false },
                MethodSig { name: "back".to_string(), params: vec![], ret: Type::int(), is_virtual: false },
                MethodSig { name: "get".to_string(), params: vec![Type::int()], ret: Type::int(), is_virtual: false },
                MethodSig { name: "pop_front".to_string(), params: vec![], ret: Type::void(), is_virtual: false },
                MethodSig { name: "clear".to_string(), params: vec![], ret: Type::void(), is_virtual: false },
                MethodSig { name: "destroy".to_string(), params: vec![], ret: Type::void(), is_virtual: false },
            ],
        },
    );

    // vector_float
    m.insert(
        "cide_vec_float".to_string(),
        ClassLayout {
            size: 12,
            fields: vec![
                ("n".to_string(), Type::int()),
                ("m".to_string(), Type::int()),
                ("a".to_string(), Type::pointer_to(Type::float())),
            ],
            methods: vec![
                MethodSig { name: "push_back".to_string(), params: vec![Type::float()], ret: Type::void(), is_virtual: false },
                MethodSig { name: "pop_back".to_string(), params: vec![], ret: Type::float(), is_virtual: false },
                MethodSig { name: "size".to_string(), params: vec![], ret: Type::int(), is_virtual: false },
                MethodSig { name: "capacity".to_string(), params: vec![], ret: Type::int(), is_virtual: false },
                MethodSig { name: "front".to_string(), params: vec![], ret: Type::float(), is_virtual: false },
                MethodSig { name: "back".to_string(), params: vec![], ret: Type::float(), is_virtual: false },
                MethodSig { name: "get".to_string(), params: vec![Type::int()], ret: Type::float(), is_virtual: false },
                MethodSig { name: "pop_front".to_string(), params: vec![], ret: Type::void(), is_virtual: false },
                MethodSig { name: "clear".to_string(), params: vec![], ret: Type::void(), is_virtual: false },
                MethodSig { name: "destroy".to_string(), params: vec![], ret: Type::void(), is_virtual: false },
            ],
        },
    );

    // vector_char
    m.insert(
        "cide_vec_char".to_string(),
        ClassLayout {
            size: 12,
            fields: vec![
                ("n".to_string(), Type::int()),
                ("m".to_string(), Type::int()),
                ("a".to_string(), Type::pointer_to(Type::char())),
            ],
            methods: vec![
                MethodSig { name: "push_back".to_string(), params: vec![Type::char()], ret: Type::void(), is_virtual: false },
                MethodSig { name: "pop_back".to_string(), params: vec![], ret: Type::char(), is_virtual: false },
                MethodSig { name: "size".to_string(), params: vec![], ret: Type::int(), is_virtual: false },
                MethodSig { name: "capacity".to_string(), params: vec![], ret: Type::int(), is_virtual: false },
                MethodSig { name: "front".to_string(), params: vec![], ret: Type::char(), is_virtual: false },
                MethodSig { name: "back".to_string(), params: vec![], ret: Type::char(), is_virtual: false },
                MethodSig { name: "get".to_string(), params: vec![Type::int()], ret: Type::char(), is_virtual: false },
                MethodSig { name: "c_str".to_string(), params: vec![], ret: Type::pointer_to(Type::char()), is_virtual: false },
                MethodSig { name: "pop_front".to_string(), params: vec![], ret: Type::void(), is_virtual: false },
                MethodSig { name: "clear".to_string(), params: vec![], ret: Type::void(), is_virtual: false },
                MethodSig { name: "destroy".to_string(), params: vec![], ret: Type::void(), is_virtual: false },
            ],
        },
    );

    // string
    m.insert(
        "cide_string".to_string(),
        ClassLayout {
            size: 12,
            fields: vec![
                ("n".to_string(), Type::int()),
                ("m".to_string(), Type::int()),
                ("s".to_string(), Type::pointer_to(Type::char())),
            ],
            methods: vec![
                MethodSig { name: "push_back".to_string(), params: vec![Type::char()], ret: Type::void(), is_virtual: false },
                MethodSig { name: "pop_back".to_string(), params: vec![], ret: Type::char(), is_virtual: false },
                MethodSig { name: "size".to_string(), params: vec![], ret: Type::int(), is_virtual: false },
                MethodSig { name: "capacity".to_string(), params: vec![], ret: Type::int(), is_virtual: false },
                MethodSig { name: "front".to_string(), params: vec![], ret: Type::char(), is_virtual: false },
                MethodSig { name: "back".to_string(), params: vec![], ret: Type::char(), is_virtual: false },
                MethodSig { name: "get".to_string(), params: vec![Type::int()], ret: Type::char(), is_virtual: false },
                MethodSig { name: "c_str".to_string(), params: vec![], ret: Type::pointer_to(Type::char()), is_virtual: false },
                MethodSig { name: "pop_front".to_string(), params: vec![], ret: Type::void(), is_virtual: false },
                MethodSig { name: "clear".to_string(), params: vec![], ret: Type::void(), is_virtual: false },
                MethodSig { name: "destroy".to_string(), params: vec![], ret: Type::void(), is_virtual: false },
            ],
        },
    );

    // list_int
    m.insert(
        "cide_list_int".to_string(),
        ClassLayout {
            size: 12,
            fields: vec![
                ("head".to_string(), Type::pointer_to(Type::void())),
                ("tail".to_string(), Type::pointer_to(Type::void())),
                ("n".to_string(), Type::int()),
            ],
            methods: vec![
                MethodSig { name: "push_back".to_string(), params: vec![Type::int()], ret: Type::void(), is_virtual: false },
                MethodSig { name: "push_front".to_string(), params: vec![Type::int()], ret: Type::void(), is_virtual: false },
                MethodSig { name: "pop_back".to_string(), params: vec![], ret: Type::int(), is_virtual: false },
                MethodSig { name: "size".to_string(), params: vec![], ret: Type::int(), is_virtual: false },
                MethodSig { name: "front".to_string(), params: vec![], ret: Type::int(), is_virtual: false },
                MethodSig { name: "back".to_string(), params: vec![], ret: Type::int(), is_virtual: false },
                MethodSig { name: "get".to_string(), params: vec![Type::int()], ret: Type::int(), is_virtual: false },
                MethodSig { name: "pop_front".to_string(), params: vec![], ret: Type::void(), is_virtual: false },
                MethodSig { name: "destroy".to_string(), params: vec![], ret: Type::void(), is_virtual: false },
            ],
        },
    );

    m
}

static BUILTIN_LAYOUTS: LazyLock<HashMap<String, ClassLayout>> = LazyLock::new(builtin_layouts);

pub fn builtin_class_layout(name: &str) -> Option<ClassLayout> {
    BUILTIN_LAYOUTS.get(name).cloned()
}

pub fn builtin_method_sig(class_name: &str, method_name: &str) -> Option<MethodSig> {
    BUILTIN_LAYOUTS.get(class_name)?.methods.iter().find(|m| m.name == method_name).cloned()
}
