//! AST 声明节点。

use super::expr::Expr;
use super::stmt::Stmt;
use super::types::Type;
use crate::shared::source_loc::SourceLoc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Param {
    pub ty: Type,
    pub name: String,
    pub loc: SourceLoc,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FuncDecl {
    pub loc: SourceLoc,
    pub return_type: Type,
    pub name: String,
    pub params: Vec<Param>,
    pub body: Option<Stmt>,
    pub is_static: bool,
    pub is_extern: bool,
    pub source_file: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StructField {
    pub ty: Type,
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StructDecl {
    pub loc: SourceLoc,
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GlobalDecl {
    pub loc: SourceLoc,
    pub ty: Type,
    pub name: String,
    pub init: Option<Expr>,
    pub is_static: bool,
    pub is_extern: bool,
    pub source_file: String,
}

// ============================================================================
// C++ Declaration Nodes
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AccessSpec {
    Public,
    Private,
    Protected,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ClassMember {
    Field {
        name: String,
        ty: Type,
        access: AccessSpec,
        is_static: bool,
    },
    Method {
        name: String,
        ret: Type,
        params: Vec<Param>,
        body: Option<Stmt>,
        is_virtual: bool,
        access: AccessSpec,
        is_static: bool,
        is_const: bool,
    },
    Constructor {
        params: Vec<Param>,
        body: Option<Stmt>,
        is_default: bool,
        access: AccessSpec,
        is_explicit: bool,
    },
    Destructor {
        body: Option<Stmt>,
        access: AccessSpec,
        is_virtual: bool,
    },
    NestedStruct {
        decl: StructDecl,
        access: AccessSpec,
    },
    NestedClass {
        decl: ClassDecl,
        access: AccessSpec,
    },
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct VTable {
    pub entries: Vec<(String, Type)>, // (method_name, function_type)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClassDecl {
    pub loc: SourceLoc,
    pub name: String,
    pub base: Option<String>,
    pub members: Vec<ClassMember>,
    pub vtable: Option<VTable>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TemplateParam {
    pub name: String,
    pub loc: SourceLoc,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Templateable {
    Func(Box<FuncDecl>),
    Class(Box<ClassDecl>),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TemplateDecl {
    pub loc: SourceLoc,
    pub params: Vec<TemplateParam>,
    pub decl: Templateable,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TemplateInstantiation {
    pub loc: SourceLoc,
    pub base: String,
    pub args: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CaptureMode {
    ByValue(String),
    ByReference(String),
    Implicit,
}

// ============================================================================
// Program Root
// ============================================================================

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ProgramNode {
    pub structs: Vec<StructDecl>,
    pub unions: Vec<StructDecl>,
    pub globals: Vec<GlobalDecl>,
    pub funcs: Vec<FuncDecl>,
    // === C++ 新增 ===
    pub classes: Vec<ClassDecl>,
    pub templates: Vec<TemplateDecl>,
    pub template_instantiations: Vec<TemplateInstantiation>,
}
