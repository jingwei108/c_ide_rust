#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Void,
    Int,
    Char,
    Pointer,
    Array,
    Struct,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Type {
    pub kind: TypeKind,
    pub name: String,
    pub array_size: i32,
    pub base_kind: TypeKind,
    pub dims: Vec<i32>,
}

impl Default for Type {
    fn default() -> Self {
        Self {
            kind: TypeKind::Void,
            name: String::new(),
            array_size: 0,
            base_kind: TypeKind::Void,
            dims: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SourceLoc {
    pub line: i32,
    pub column: i32,
}

impl Default for SourceLoc {
    fn default() -> Self {
        Self { line: 1, column: 1 }
    }
}
