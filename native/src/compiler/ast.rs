#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum TypeKind {
    #[default]
    Void,
    Int,
    Char,
    Float,
    Double,
    LongLong,
    Pointer,
    Array,
    Struct,
    Union,
    Function,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Type {
    Void { is_const: bool },
    Int { is_unsigned: bool, is_const: bool },
    Char { is_unsigned: bool, is_const: bool },
    Float { is_const: bool },
    Double { is_const: bool },
    LongLong { is_unsigned: bool, is_const: bool },
    Pointer {
        pointee: Box<Type>,
        is_const: bool,
    },
    Array {
        element: Box<Type>,
        array_size: i32,
        dims: Vec<i32>,
        is_const: bool,
    },
    Function {
        return_type: Box<Type>,
        param_types: Vec<Type>,
        is_const: bool,
    },
    Struct { name: String, is_const: bool },
    Union { name: String, is_const: bool },
}

impl Default for Type {
    fn default() -> Self {
        Type::Void { is_const: false }
    }
}

impl Type {
    pub fn int() -> Self {
        Type::Int { is_unsigned: false, is_const: false }
    }
    pub fn unsigned_int() -> Self {
        Type::Int { is_unsigned: true, is_const: false }
    }
    pub fn char() -> Self {
        Type::Char { is_unsigned: false, is_const: false }
    }
    pub fn float() -> Self {
        Type::Float { is_const: false }
    }
    pub fn double() -> Self {
        Type::Double { is_const: false }
    }
    pub fn long_long() -> Self {
        Type::LongLong { is_unsigned: false, is_const: false }
    }
    pub fn void() -> Self {
        Type::Void { is_const: false }
    }
    pub fn pointer_to(pointee: Type) -> Self {
        Type::Pointer { pointee: Box::new(pointee), is_const: false }
    }
    pub fn array_of(element: Type, dims: Vec<i32>) -> Self {
        let array_size = if dims.is_empty() { 0 } else { dims.iter().map(|&d| if d > 0 { d } else { 1 }).product() };
        Type::Array { element: Box::new(element), array_size, dims, is_const: false }
    }
    pub fn struct_type(name: impl Into<String>) -> Self {
        Type::Struct { name: name.into(), is_const: false }
    }
    pub fn union_type(name: impl Into<String>) -> Self {
        Type::Union { name: name.into(), is_const: false }
    }
    pub fn function(return_type: Type, param_types: Vec<Type>) -> Self {
        Type::Function { return_type: Box::new(return_type), param_types, is_const: false }
    }
    pub fn function_pointer(return_type: Type, param_types: Vec<Type>) -> Self {
        Type::Pointer { pointee: Box::new(Type::Function { return_type: Box::new(return_type), param_types, is_const: false }), is_const: false }
    }

    // 兼容访问器
    pub fn kind(&self) -> TypeKind {
        match self {
            Type::Void { .. } => TypeKind::Void,
            Type::Int { .. } => TypeKind::Int,
            Type::Char { .. } => TypeKind::Char,
            Type::Float { .. } => TypeKind::Float,
            Type::Double { .. } => TypeKind::Double,
            Type::LongLong { .. } => TypeKind::LongLong,
            Type::Pointer { .. } => TypeKind::Pointer,
            Type::Array { .. } => TypeKind::Array,
            Type::Function { .. } => TypeKind::Function,
            Type::Struct { .. } => TypeKind::Struct,
            Type::Union { .. } => TypeKind::Union,
        }
    }

    /// 返回类型的核心名称。对 Struct/Union 返回原始名称；对 Pointer/Array 递归返回；
    /// 对基础类型返回关键字。返回值的生命周期与 self 绑定。
    pub fn name(&self) -> &str {
        match self {
            Type::Struct { name, .. } | Type::Union { name, .. } => name.as_str(),
            Type::Pointer { pointee, .. } => pointee.name(),
            Type::Array { element, .. } => element.name(),
            Type::Void { .. } => "void",
            Type::Int { .. } => "int",
            Type::Char { .. } => "char",
            Type::Float { .. } => "float",
            Type::Double { .. } => "double",
            Type::LongLong { .. } => "long long",
            Type::Function { .. } => "fn",
        }
    }

    pub fn array_size(&self) -> i32 {
        match self { Type::Array { array_size, .. } => *array_size, _ => 0 }
    }

    pub fn dims(&self) -> &[i32] {
        match self { Type::Array { dims, .. } => dims.as_slice(), _ => &[] }
    }

    pub fn is_unsigned(&self) -> bool {
        match self {
            Type::Int { is_unsigned, .. } | Type::Char { is_unsigned, .. } | Type::LongLong { is_unsigned, .. } => *is_unsigned,
            _ => false,
        }
    }

    pub fn is_const(&self) -> bool {
        match self {
            Type::Void { is_const } => *is_const,
            Type::Int { is_const, .. } => *is_const,
            Type::Char { is_const, .. } => *is_const,
            Type::Float { is_const } => *is_const,
            Type::Double { is_const } => *is_const,
            Type::LongLong { is_const, .. } => *is_const,
            Type::Pointer { is_const, .. } => *is_const,
            Type::Array { is_const, .. } => *is_const,
            Type::Function { is_const, .. } => *is_const,
            Type::Struct { is_const, .. } => *is_const,
            Type::Union { is_const, .. } => *is_const,
        }
    }

    pub fn set_const(&mut self, value: bool) {
        match self {
            Type::Void { is_const } => *is_const = value,
            Type::Int { is_const, .. } => *is_const = value,
            Type::Char { is_const, .. } => *is_const = value,
            Type::Float { is_const } => *is_const = value,
            Type::Double { is_const } => *is_const = value,
            Type::LongLong { is_const, .. } => *is_const = value,
            Type::Pointer { is_const, .. } => *is_const = value,
            Type::Array { is_const, .. } => *is_const = value,
            Type::Function { is_const, .. } => *is_const = value,
            Type::Struct { is_const, .. } => *is_const = value,
            Type::Union { is_const, .. } => *is_const = value,
        }
    }

    pub fn is_scalar(&self) -> bool {
        matches!(self.kind(), TypeKind::Int | TypeKind::Char | TypeKind::Float | TypeKind::Double | TypeKind::LongLong)
    }
    pub fn is_pointer(&self) -> bool {
        matches!(self.kind(), TypeKind::Pointer)
    }
    pub fn is_function_pointer(&self) -> bool {
        matches!(self, Type::Pointer { pointee, .. } if matches!(pointee.as_ref(), Type::Function { .. }))
    }
    pub fn is_array(&self) -> bool {
        matches!(self.kind(), TypeKind::Array)
    }
    pub fn is_struct(&self) -> bool {
        matches!(self.kind(), TypeKind::Struct)
    }
    pub fn is_union(&self) -> bool {
        matches!(self.kind(), TypeKind::Union)
    }
    pub fn is_void(&self) -> bool {
        matches!(self.kind(), TypeKind::Void)
    }

    /// 递归获取数组的最内层元素类型。对非数组类型返回自身克隆。
    pub fn innermost_element_type(&self) -> Self {
        match self {
            Type::Array { element, .. } => element.innermost_element_type(),
            _ => self.clone(),
        }
    }

    pub fn total_elements(&self) -> i32 {
        if !self.is_array() { return 1; }
        let dims = self.dims();
        if !dims.is_empty() {
            let has_negative = dims.iter().any(|&d| d < 0);
            if has_negative && self.array_size() > 0 {
                return self.array_size();
            }
            dims.iter().map(|&d| if d > 0 { d } else { 1 }).product()
        } else if self.array_size() > 0 {
            self.array_size()
        } else {
            1
        }
    }

    pub fn subscript_type(&self) -> Self {
        if !self.is_array() { return self.clone(); }
        match self {
            Type::Array { element, dims, is_const, .. } => {
                if dims.len() <= 1 {
                    *element.clone()
                } else {
                    let mut new_dims = dims.clone();
                    new_dims.remove(0);
                    let new_array_size = new_dims.iter().map(|&d| if d > 0 { d } else { 1 }).product();
                    Type::Array { element: element.clone(), array_size: new_array_size, dims: new_dims, is_const: *is_const }
                }
            }
            _ => self.clone(),
        }
    }

    /// 从 TypeKind 和可选名称重建基础类型。仅用于旧代码兼容路径。
    pub fn from_base_kind(base_kind: TypeKind, name: String) -> Self {
        match base_kind {
            TypeKind::Void => Type::Void { is_const: false },
            TypeKind::Int => Type::Int { is_unsigned: false, is_const: false },
            TypeKind::Char => Type::Char { is_unsigned: false, is_const: false },
            TypeKind::Float => Type::Float { is_const: false },
            TypeKind::Double => Type::Double { is_const: false },
            TypeKind::LongLong => Type::LongLong { is_unsigned: false, is_const: false },
            TypeKind::Struct => Type::Struct { name, is_const: false },
            TypeKind::Union => Type::Union { name, is_const: false },
            TypeKind::Pointer => {
                let inferred_base = match name.as_str() {
                    "char" => Type::Char { is_unsigned: false, is_const: false },
                    "float" => Type::Float { is_const: false },
                    "void" => Type::Void { is_const: false },
                    _ => Type::Int { is_unsigned: false, is_const: false },
                };
                Type::Pointer { pointee: Box::new(inferred_base), is_const: false }
            }
            TypeKind::Array => Type::Array { element: Box::new(Type::Void { is_const: false }), array_size: 0, dims: vec![], is_const: false },
            TypeKind::Function => Type::Function { return_type: Box::new(Type::int()), param_types: vec![], is_const: false },
        }
    }

    fn format_string(&self) -> String {
        match self {
            Type::Void { .. } => "void".to_string(),
            Type::Int { .. } => "int".to_string(),
            Type::Char { .. } => "char".to_string(),
            Type::Float { .. } => "float".to_string(),
            Type::Double { .. } => "double".to_string(),
            Type::LongLong { .. } => "long long".to_string(),
            Type::Struct { name, .. } => format!("struct {}", name),
            Type::Union { name, .. } => format!("union {}", name),
            Type::Pointer { pointee, .. } => format!("{}*", pointee.format_string()),
            Type::Array { element, dims, array_size, .. } => {
                let mut base = element.format_string();
                if !dims.is_empty() {
                    for d in dims {
                        base.push_str(&format!("[{}]", d));
                    }
                    base
                } else if *array_size > 0 {
                    format!("{}[{}]", base, array_size)
                } else {
                    format!("{}[]", base)
                }
            }
            Type::Function { return_type, param_types, .. } => {
                let mut params = String::new();
                for (i, p) in param_types.iter().enumerate() {
                    if i > 0 { params.push_str(", "); }
                    params.push_str(&p.format_string());
                }
                format!("{} (*)({})", return_type.format_string(), params)
            }
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format_string())
    }
}

// Type 的 serde 由 #[derive(Serialize, Deserialize)] 自动生成嵌套 JSON 格式。
// 本项目处于开发期，无需兼容旧 flat 格式。

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct SourceLoc {
    pub line: i32,
    pub column: i32,
}

// ============================================================================
// Expression Nodes
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
    BitAnd, BitOr, BitXor, Shl, Shr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Neg, Not, BitNot, Addr, Deref, PreInc, PreDec, PostInc, PostDec,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignOp {
    Assign, AddAssign, SubAssign, MulAssign, DivAssign, ModAssign,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr>, loc: SourceLoc, ty: Type },
    Unary { op: UnaryOp, operand: Box<Expr>, loc: SourceLoc, ty: Type },
    Literal { value: i32, loc: SourceLoc, ty: Type },
    FloatLiteral { value: f64, loc: SourceLoc, ty: Type },
    LongLiteral { value: i64, loc: SourceLoc, ty: Type },
    StringLiteral { value: String, loc: SourceLoc, ty: Type },
    Identifier { name: String, loc: SourceLoc, ty: Type },
    Call { name: String, args: Vec<Expr>, loc: SourceLoc, ty: Type },
    CallPtr { callee: Box<Expr>, args: Vec<Expr>, loc: SourceLoc, ty: Type },
    Index { array: Box<Expr>, index: Box<Expr>, loc: SourceLoc, ty: Type },
    Member { object: Box<Expr>, member: String, loc: SourceLoc, ty: Type },
    Assign { op: AssignOp, left: Box<Expr>, right: Box<Expr>, loc: SourceLoc, ty: Type },
    Ternary { cond: Box<Expr>, then_branch: Box<Expr>, else_branch: Box<Expr>, loc: SourceLoc, ty: Type },
    Sizeof { target_type: Option<Type>, operand: Option<Box<Expr>>, loc: SourceLoc, ty: Type },
    Cast { expr: Box<Expr>, target_type: Type, loc: SourceLoc, ty: Type },
    InitList { elements: Vec<Expr>, loc: SourceLoc, ty: Type },
}

impl Default for Expr {
    fn default() -> Self {
        Expr::Literal { value: 0, loc: SourceLoc::default(), ty: Type::default() }
    }
}

macro_rules! expr_field {
    ($self:expr, $field:ident) => {
        match $self {
            Expr::Binary { $field, .. } => $field,
            Expr::Unary { $field, .. } => $field,
            Expr::Literal { $field, .. } => $field,
            Expr::FloatLiteral { $field, .. } => $field,
            Expr::LongLiteral { $field, .. } => $field,
            Expr::StringLiteral { $field, .. } => $field,
            Expr::Identifier { $field, .. } => $field,
            Expr::Call { $field, .. } => $field,
            Expr::CallPtr { $field, .. } => $field,
            Expr::Index { $field, .. } => $field,
            Expr::Member { $field, .. } => $field,
            Expr::Assign { $field, .. } => $field,
            Expr::Ternary { $field, .. } => $field,
            Expr::Sizeof { $field, .. } => $field,
            Expr::Cast { $field, .. } => $field,
            Expr::InitList { $field, .. } => $field,
        }
    };
}

impl Expr {
    pub fn loc(&self) -> &SourceLoc {
        expr_field!(self, loc)
    }
    pub fn ty(&self) -> &Type {
        expr_field!(self, ty)
    }
    pub fn set_ty(&mut self, new_ty: Type) {
        *expr_field!(self, ty) = new_ty;
    }
}

// ============================================================================
// Statement Nodes
// ============================================================================

#[derive(Debug, Clone)]
pub enum Stmt {
    Block { stmts: Vec<Stmt>, loc: SourceLoc },
    VarDecl { var_type: Type, name: String, init: Option<Expr>, extra_vars: Vec<(Type, String, Option<Expr>)>, loc: SourceLoc },
    Expr { expr: Expr, loc: SourceLoc },
    If { cond: Expr, then_stmt: Box<Stmt>, else_stmt: Option<Box<Stmt>>, loc: SourceLoc },
    While { cond: Expr, body: Box<Stmt>, loc: SourceLoc },
    DoWhile { body: Box<Stmt>, cond: Expr, loc: SourceLoc },
    For { init: Option<Box<Stmt>>, cond: Option<Expr>, step: Option<Expr>, body: Box<Stmt>, loc: SourceLoc },
    Return { value: Option<Expr>, loc: SourceLoc },
    Break { loc: SourceLoc },
    Continue { loc: SourceLoc },
    Switch { cond: Expr, body: Box<Stmt>, loc: SourceLoc },
    Case { label: Option<Expr>, stmt: Box<Stmt>, loc: SourceLoc },
}

// ============================================================================
// Declaration Nodes
// ============================================================================

#[derive(Debug, Clone)]
pub struct Param {
    pub ty: Type,
    pub name: String,
    pub loc: SourceLoc,
}

#[derive(Debug, Clone)]
pub struct FuncDecl {
    pub loc: SourceLoc,
    pub return_type: Type,
    pub name: String,
    pub params: Vec<Param>,
    pub body: Option<Stmt>,
    pub is_static: bool,
    pub source_file: String,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub ty: Type,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct StructDecl {
    pub loc: SourceLoc,
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone)]
pub struct GlobalDecl {
    pub loc: SourceLoc,
    pub ty: Type,
    pub name: String,
    pub init: Option<Expr>,
    pub is_static: bool,
    pub source_file: String,
}

// ============================================================================
// Program Root
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct ProgramNode {
    pub structs: Vec<StructDecl>,
    pub unions: Vec<StructDecl>,
    pub globals: Vec<GlobalDecl>,
    pub funcs: Vec<FuncDecl>,
}

use std::collections::HashMap;

/// 获取数组类型的最底层元素类型。
pub fn base_element_type(ty: &Type) -> &Type {
    match ty {
        Type::Array { element, .. } => base_element_type(element),
        _ => ty,
    }
}

/// 根据类型定义计算类型的字节大小。
/// 与 `bytecode_gen::type_size` 和 `compile_pipeline::type_size` 保持同一语义。
pub fn compute_type_size(
    ty: &Type,
    struct_defs: &HashMap<String, Vec<StructField>>,
    union_defs: &HashMap<String, Vec<StructField>>,
) -> i32 {
    match ty.kind() {
        TypeKind::Void => 0,
        TypeKind::Int => 4,
        TypeKind::Char => 1,
        TypeKind::Float => 4,
        TypeKind::Double | TypeKind::LongLong => 8,
        TypeKind::Pointer | TypeKind::Function => 4,
        TypeKind::Array => {
            let elem_count = ty.total_elements();
            let base_elem = base_element_type(ty);
            let elem_size = compute_type_size(base_elem, struct_defs, union_defs);
            elem_count * elem_size
        }
        TypeKind::Struct => {
            struct_defs.get(ty.name()).map(|f| {
                f.iter().map(|field| compute_type_size(&field.ty, struct_defs, union_defs)).sum()
            }).unwrap_or(0)
        }
        TypeKind::Union => {
            union_defs.get(ty.name()).map(|f| {
                f.iter().map(|field| compute_type_size(&field.ty, struct_defs, union_defs)).max().unwrap_or(0)
            }).unwrap_or(0)
        }
    }
}
