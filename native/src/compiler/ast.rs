#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TypeKind {
    Void,
    Int,
    Char,
    Float,
    Pointer,
    Array,
    Struct,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Type {
    pub kind: TypeKind,
    pub name: String,
    pub array_size: i32,
    pub base_kind: TypeKind,
    pub dims: Vec<i32>,
    pub is_unsigned: bool,
    pub is_const: bool,
}

impl Default for Type {
    fn default() -> Self {
        Self {
            kind: TypeKind::Void,
            name: String::new(),
            array_size: 0,
            base_kind: TypeKind::Void,
            dims: Vec::new(),
            is_unsigned: false,
            is_const: false,
        }
    }
}

impl Type {
    pub fn int() -> Self {
        Self { kind: TypeKind::Int, ..Self::default() }
    }
    pub fn unsigned_int() -> Self {
        Self { kind: TypeKind::Int, is_unsigned: true, ..Self::default() }
    }
    pub fn char() -> Self {
        Self { kind: TypeKind::Char, ..Self::default() }
    }
    pub fn float() -> Self {
        Self { kind: TypeKind::Float, ..Self::default() }
    }
    pub fn void() -> Self {
        Self { kind: TypeKind::Void, ..Self::default() }
    }
    pub fn pointer(base: TypeKind, name: impl Into<String>) -> Self {
        Self { kind: TypeKind::Pointer, base_kind: base, name: name.into(), ..Self::default() }
    }
    pub fn unsigned_pointer(base: TypeKind, name: impl Into<String>) -> Self {
        Self { kind: TypeKind::Pointer, base_kind: base, name: name.into(), is_unsigned: true, ..Self::default() }
    }
    pub fn array(base: TypeKind, name: impl Into<String>, dims: Vec<i32>) -> Self {
        let array_size = if dims.is_empty() { 0 } else { dims.iter().map(|&d| if d > 0 { d } else { 1 }).product() };
        Self { kind: TypeKind::Array, base_kind: base, name: name.into(), array_size, dims, ..Self::default() }
    }
    pub fn struct_type(name: impl Into<String>) -> Self {
        Self { kind: TypeKind::Struct, name: name.into(), ..Self::default() }
    }

    pub fn is_scalar(&self) -> bool {
        matches!(self.kind, TypeKind::Int | TypeKind::Char | TypeKind::Float)
    }
    pub fn is_pointer(&self) -> bool {
        matches!(self.kind, TypeKind::Pointer)
    }
    pub fn is_array(&self) -> bool {
        matches!(self.kind, TypeKind::Array)
    }
    pub fn is_struct(&self) -> bool {
        matches!(self.kind, TypeKind::Struct)
    }
    pub fn is_void(&self) -> bool {
        matches!(self.kind, TypeKind::Void)
    }

    pub fn total_elements(&self) -> i32 {
        if !self.is_array() { return 1; }
        if !self.dims.is_empty() {
            let has_negative = self.dims.iter().any(|&d| d < 0);
            if has_negative && self.array_size > 0 {
                return self.array_size;
            }
            self.dims.iter().map(|&d| if d > 0 { d } else { 1 }).product()
        } else if self.array_size > 0 {
            self.array_size
        } else {
            1
        }
    }

    pub fn subscript_type(&self) -> Self {
        if !self.is_array() { return self.clone(); }
        if self.dims.len() <= 1 {
            let mut t = Self { kind: self.base_kind.clone(), name: self.name.clone(), ..Self::default() };
            if self.base_kind == TypeKind::Pointer {
                t.base_kind = match self.name.as_str() {
                    "char" => TypeKind::Char,
                    "float" => TypeKind::Float,
                    "void" => TypeKind::Void,
                    _ => TypeKind::Int,
                };
            }
            return t;
        }
        let mut t = self.clone();
        t.dims.remove(0);
        t.array_size = t.total_elements();
        t
    }

    fn format_string(&self) -> String {
        match self.kind {
            TypeKind::Void => "void".to_string(),
            TypeKind::Int => "int".to_string(),
            TypeKind::Char => "char".to_string(),
            TypeKind::Float => "float".to_string(),
            TypeKind::Pointer => {
                let base = match self.base_kind {
                    TypeKind::Struct => format!("struct {}", self.name),
                    TypeKind::Char => "char".to_string(),
                    TypeKind::Float => "float".to_string(),
                    TypeKind::Void => "void".to_string(),
                    _ => "int".to_string(),
                };
                format!("{}*", base)
            }
            TypeKind::Array => {
                let mut base = match self.base_kind {
                    TypeKind::Struct => format!("struct {}", self.name),
                    TypeKind::Char => "char".to_string(),
                    _ => "int".to_string(),
                };
                if !self.dims.is_empty() {
                    for d in &self.dims {
                        base.push_str(&format!("[{}]", d));
                    }
                    base
                } else if self.array_size > 0 {
                    format!("{}[{}]", base, self.array_size)
                } else {
                    format!("{}[]", base)
                }
            }
            TypeKind::Struct => format!("struct {}", self.name),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format_string())
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

impl Expr {
    pub fn loc(&self) -> &SourceLoc {
        match self {
            Expr::Binary { loc, .. } => loc,
            Expr::Unary { loc, .. } => loc,
            Expr::Literal { loc, .. } => loc,
            Expr::FloatLiteral { loc, .. } => loc,
            Expr::StringLiteral { loc, .. } => loc,
            Expr::Identifier { loc, .. } => loc,
            Expr::Call { loc, .. } => loc,
            Expr::Index { loc, .. } => loc,
            Expr::Member { loc, .. } => loc,
            Expr::Assign { loc, .. } => loc,
            Expr::Ternary { loc, .. } => loc,
            Expr::Sizeof { loc, .. } => loc,
            Expr::Cast { loc, .. } => loc,
            Expr::InitList { loc, .. } => loc,
        }
    }
    pub fn ty(&self) -> &Type {
        match self {
            Expr::Binary { ty, .. } => ty,
            Expr::Unary { ty, .. } => ty,
            Expr::Literal { ty, .. } => ty,
            Expr::FloatLiteral { ty, .. } => ty,
            Expr::StringLiteral { ty, .. } => ty,
            Expr::Identifier { ty, .. } => ty,
            Expr::Call { ty, .. } => ty,
            Expr::Index { ty, .. } => ty,
            Expr::Member { ty, .. } => ty,
            Expr::Assign { ty, .. } => ty,
            Expr::Ternary { ty, .. } => ty,
            Expr::Sizeof { ty, .. } => ty,
            Expr::Cast { ty, .. } => ty,
            Expr::InitList { ty, .. } => ty,
        }
    }
    pub fn set_ty(&mut self, new_ty: Type) {
        match self {
            Expr::Binary { ty, .. } => *ty = new_ty,
            Expr::Unary { ty, .. } => *ty = new_ty,
            Expr::Literal { ty, .. } => *ty = new_ty,
            Expr::FloatLiteral { ty, .. } => *ty = new_ty,
            Expr::StringLiteral { ty, .. } => *ty = new_ty,
            Expr::Identifier { ty, .. } => *ty = new_ty,
            Expr::Call { ty, .. } => *ty = new_ty,
            Expr::Index { ty, .. } => *ty = new_ty,
            Expr::Member { ty, .. } => *ty = new_ty,
            Expr::Assign { ty, .. } => *ty = new_ty,
            Expr::Ternary { ty, .. } => *ty = new_ty,
            Expr::Sizeof { ty, .. } => *ty = new_ty,
            Expr::Cast { ty, .. } => *ty = new_ty,
            Expr::InitList { ty, .. } => *ty = new_ty,
        }
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
    pub globals: Vec<GlobalDecl>,
    pub funcs: Vec<FuncDecl>,
}
