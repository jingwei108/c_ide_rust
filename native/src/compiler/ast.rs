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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Void { is_const: bool },
    Int { is_unsigned: bool, is_const: bool },
    Char { is_unsigned: bool, is_const: bool },
    Float { is_const: bool },
    Double { is_const: bool },
    LongLong { is_unsigned: bool, is_const: bool },
    Pointer {
        base_kind: TypeKind,
        name: String,
        is_unsigned: bool,
        is_const: bool,
    },
    Array {
        base_kind: TypeKind,
        name: String,
        array_size: i32,
        dims: Vec<i32>,
        is_unsigned: bool,
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
    pub fn pointer(base: TypeKind, name: impl Into<String>) -> Self {
        Type::Pointer { base_kind: base, name: name.into(), is_unsigned: false, is_const: false }
    }
    pub fn unsigned_pointer(base: TypeKind, name: impl Into<String>) -> Self {
        Type::Pointer { base_kind: base, name: name.into(), is_unsigned: true, is_const: false }
    }
    pub fn array(base: TypeKind, name: impl Into<String>, dims: Vec<i32>) -> Self {
        let array_size = if dims.is_empty() { 0 } else { dims.iter().map(|&d| if d > 0 { d } else { 1 }).product() };
        Type::Array { base_kind: base, name: name.into(), array_size, dims, is_unsigned: false, is_const: false }
    }
    pub fn struct_type(name: impl Into<String>) -> Self {
        Type::Struct { name: name.into(), is_const: false }
    }
    pub fn union_type(name: impl Into<String>) -> Self {
        Type::Union { name: name.into(), is_const: false }
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
            Type::Struct { .. } => TypeKind::Struct,
            Type::Union { .. } => TypeKind::Union,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Type::Pointer { name, .. } | Type::Array { name, .. } | Type::Struct { name, .. } | Type::Union { name, .. } => name.as_str(),
            Type::Void { .. } => "void",
            Type::Int { .. } => "int",
            Type::Char { .. } => "char",
            Type::Float { .. } => "float",
            Type::Double { .. } => "double",
            Type::LongLong { .. } => "long long",
        }
    }

    pub fn array_size(&self) -> i32 {
        match self { Type::Array { array_size, .. } => *array_size, _ => 0 }
    }

    pub fn base_kind(&self) -> TypeKind {
        match self { Type::Pointer { base_kind, .. } | Type::Array { base_kind, .. } => *base_kind, _ => TypeKind::Void }
    }

    pub fn dims(&self) -> &[i32] {
        match self { Type::Array { dims, .. } => dims.as_slice(), _ => &[] }
    }

    pub fn is_unsigned(&self) -> bool {
        match self {
            Type::Int { is_unsigned, .. } | Type::Char { is_unsigned, .. } | Type::LongLong { is_unsigned, .. } | Type::Pointer { is_unsigned, .. } | Type::Array { is_unsigned, .. } => *is_unsigned,
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
            Type::Array { base_kind, name, dims, is_unsigned, is_const, .. } => {
                if dims.len() <= 1 {
                    Self::from_base_kind(*base_kind, name.clone())
                } else {
                    let mut new_dims = dims.clone();
                    new_dims.remove(0);
                    let new_array_size = new_dims.iter().map(|&d| if d > 0 { d } else { 1 }).product();
                    Type::Array { base_kind: *base_kind, name: name.clone(), array_size: new_array_size, dims: new_dims, is_unsigned: *is_unsigned, is_const: *is_const }
                }
            }
            _ => self.clone(),
        }
    }

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
                    "char" => TypeKind::Char,
                    "float" => TypeKind::Float,
                    "void" => TypeKind::Void,
                    _ => TypeKind::Int,
                };
                Type::Pointer { base_kind: inferred_base, name, is_unsigned: false, is_const: false }
            }
            TypeKind::Array => Type::Array { base_kind: TypeKind::Void, name, array_size: 0, dims: vec![], is_unsigned: false, is_const: false },
        }
    }

    fn format_string(&self) -> String {
        match self.kind() {
            TypeKind::Void => "void".to_string(),
            TypeKind::Int => "int".to_string(),
            TypeKind::Char => "char".to_string(),
            TypeKind::Float => "float".to_string(),
            TypeKind::Double => "double".to_string(),
            TypeKind::LongLong => "long long".to_string(),
            TypeKind::Union => format!("union {}", self.name()),
            TypeKind::Pointer => {
                let base = match self.base_kind() {
                    TypeKind::Struct => format!("struct {}", self.name()),
                    TypeKind::Union => format!("union {}", self.name()),
                    TypeKind::Char => "char".to_string(),
                    TypeKind::Float => "float".to_string(),
                    TypeKind::Void => "void".to_string(),
                    _ => "int".to_string(),
                };
                format!("{}*", base)
            }
            TypeKind::Array => {
                let mut base = match self.base_kind() {
                    TypeKind::Struct => format!("struct {}", self.name()),
                    TypeKind::Char => "char".to_string(),
                    _ => "int".to_string(),
                };
                let dims = self.dims();
                if !dims.is_empty() {
                    for d in dims {
                        base.push_str(&format!("[{}]", d));
                    }
                    base
                } else if self.array_size() > 0 {
                    format!("{}[{}]", base, self.array_size())
                } else {
                    format!("{}[]", base)
                }
            }
            TypeKind::Struct => format!("struct {}", self.name()),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format_string())
    }
}

// 兼容 serde：保持与旧 struct 格式一致的 JSON 序列化
impl serde::Serialize for Type {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(7))?;
        let kind = self.kind();
        let name = self.name();
        let array_size = self.array_size();
        let base_kind = self.base_kind();
        let dims = self.dims().to_vec();
        let is_unsigned = self.is_unsigned();
        let is_const = self.is_const();
        map.serialize_entry("kind", &kind)?;
        map.serialize_entry("name", name)?;
        map.serialize_entry("array_size", &array_size)?;
        map.serialize_entry("base_kind", &base_kind)?;
        map.serialize_entry("dims", &dims)?;
        map.serialize_entry("is_unsigned", &is_unsigned)?;
        map.serialize_entry("is_const", &is_const)?;
        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for Type {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct TypeHelper {
            kind: TypeKind,
            #[serde(default)]
            name: String,
            #[serde(default)]
            array_size: i32,
            #[serde(default)]
            base_kind: TypeKind,
            #[serde(default)]
            dims: Vec<i32>,
            #[serde(default)]
            is_unsigned: bool,
            #[serde(default)]
            is_const: bool,
        }

        let helper = TypeHelper::deserialize(deserializer)?;
        Ok(match helper.kind {
            TypeKind::Void => Type::Void { is_const: helper.is_const },
            TypeKind::Int => Type::Int { is_unsigned: helper.is_unsigned, is_const: helper.is_const },
            TypeKind::Char => Type::Char { is_unsigned: helper.is_unsigned, is_const: helper.is_const },
            TypeKind::Float => Type::Float { is_const: helper.is_const },
            TypeKind::Double => Type::Double { is_const: helper.is_const },
            TypeKind::LongLong => Type::LongLong { is_unsigned: helper.is_unsigned, is_const: helper.is_const },
            TypeKind::Pointer => Type::Pointer { base_kind: helper.base_kind, name: helper.name, is_unsigned: helper.is_unsigned, is_const: helper.is_const },
            TypeKind::Array => Type::Array { base_kind: helper.base_kind, name: helper.name, array_size: helper.array_size, dims: helper.dims, is_unsigned: helper.is_unsigned, is_const: helper.is_const },
            TypeKind::Struct => Type::Struct { name: helper.name, is_const: helper.is_const },
            TypeKind::Union => Type::Union { name: helper.name, is_const: helper.is_const },
        })
    }
}

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
    Index { array: Box<Expr>, index: Box<Expr>, loc: SourceLoc, ty: Type },
    Member { object: Box<Expr>, member: String, loc: SourceLoc, ty: Type },
    Assign { op: AssignOp, left: Box<Expr>, right: Box<Expr>, loc: SourceLoc, ty: Type },
    Ternary { cond: Box<Expr>, then_branch: Box<Expr>, else_branch: Box<Expr>, loc: SourceLoc, ty: Type },
    Sizeof { target_type: Option<Type>, operand: Option<Box<Expr>>, loc: SourceLoc, ty: Type },
    Cast { expr: Box<Expr>, target_type: Type, loc: SourceLoc, ty: Type },
    InitList { elements: Vec<Expr>, loc: SourceLoc, ty: Type },
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
