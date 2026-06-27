use cide_ast::{AccessSpec, Expr, Param, TemplateParam, Templateable, Type, VTable};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct VarSymbol {
    pub(crate) ty: Type,
    #[allow(dead_code)]
    pub(crate) is_global: bool,
    pub(crate) is_extern: bool,
    pub(crate) is_static: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct FuncSymbol {
    pub(crate) return_type: Type,
    pub(crate) param_types: Vec<Type>,
    pub(crate) is_variadic: bool,
    /// 用户参数的默认表达式（this 指针等编译器插入参数不含默认值）。
    pub(crate) param_defaults: Vec<Option<Expr>>,
}

#[derive(Debug, Clone)]
pub(crate) struct StructSymbol {
    pub(crate) fields: Vec<(Type, String)>,
}

#[derive(Debug, Clone)]
pub(crate) struct MethodSig {
    pub(crate) ret: Type,
    pub(crate) param_types: Vec<Type>,
    pub(crate) param_defaults: Vec<Option<Expr>>,
    pub(crate) is_virtual: bool,
    #[allow(dead_code)]
    pub(crate) is_static: bool,
    #[allow(dead_code)]
    pub(crate) is_explicit: bool,
    pub(crate) is_const: bool,
    pub(crate) access: AccessSpec,
}

#[derive(Debug, Clone)]
pub(crate) struct ClassSymbol {
    pub(crate) fields: Vec<(Type, String, AccessSpec)>,
    pub(crate) static_fields: Vec<(Type, String, AccessSpec)>,
    /// Method overloads keyed by the un-mangled method name. Each vector holds all
    /// overloads (including constructors and destructors) sharing that name.
    pub(crate) methods: HashMap<String, Vec<MethodSig>>,
    #[allow(dead_code)]
    pub(crate) base: Option<String>,
    pub(crate) vtable: Option<VTable>,
    pub(crate) size: i32,
    /// True if the class contains pointer/reference/RValueRef fields or class fields
    /// that themselves have resources. Triggers implicit move ctor generation.
    pub(crate) has_resource: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct TemplateSymbol {
    pub(crate) params: Vec<TemplateParam>,
    pub(crate) decl: Templateable,
}

#[derive(Debug, Clone)]
pub(crate) struct LambdaInfo {
    pub(crate) id: u64,
    pub(crate) captures: Vec<(String, Type, bool)>, // (name, type, is_by_reference)
    pub(crate) params: Vec<Param>,
    pub(crate) body: cide_ast::Stmt,
    pub(crate) loc: cide_ast::SourceLoc,
}
