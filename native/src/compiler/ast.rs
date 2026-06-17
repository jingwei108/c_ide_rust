//! AST 节点定义。
//!
//! 为降低单文件认知负荷，按类型大类拆分为子模块：
//! - `types`: 类型系统（TypeKind、Type）
//! - `expr`: 表达式节点
//! - `stmt`: 语句节点
//! - `decl`: 声明节点（含 C++ 类/模板）

use std::collections::HashMap;

pub mod decl;
pub mod expr;
pub mod stmt;
pub mod types;

pub use crate::shared::source_loc::SourceLoc;

// 公共类型统一重新导出，保持外部 use 路径不变。
pub use decl::{
    AccessSpec, CaptureMode, ClassDecl, ClassMember, FuncDecl, GlobalDecl, Param, ProgramNode, StructDecl, StructField,
    TemplateDecl, TemplateInstantiation, TemplateParam, Templateable, VTable,
};
pub use expr::{AssignOp, BinaryOp, Designator, Expr, InitElement, UnaryOp};
pub use stmt::{CatchClause, Stmt};
pub use types::{Type, TypeKind};

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
    class_size_map: &HashMap<String, i32>,
) -> i32 {
    match ty.kind() {
        TypeKind::Void => 0,
        TypeKind::Int => 4,
        TypeKind::Char => 1,
        TypeKind::Float => 4,
        TypeKind::Double | TypeKind::LongLong => 8,
        TypeKind::Pointer | TypeKind::Function => 4,
        TypeKind::Array => {
            if ty.is_vla() {
                // VLA variable itself is stored as a pointer on stack
                return 4;
            }
            let elem_count = ty.total_elements();
            let base_elem = base_element_type(ty);
            let elem_size = compute_type_size(base_elem, struct_defs, union_defs, class_size_map);
            elem_count * elem_size
        }
        TypeKind::Struct => struct_defs
            .get(ty.name())
            .map(|f| {
                f.iter()
                    .map(|field| compute_type_size(&field.ty, struct_defs, union_defs, class_size_map))
                    .sum()
            })
            .unwrap_or(0),
        TypeKind::Union => union_defs
            .get(ty.name())
            .map(|f| {
                f.iter()
                    .map(|field| compute_type_size(&field.ty, struct_defs, union_defs, class_size_map))
                    .max()
                    .unwrap_or(0)
            })
            .unwrap_or(0),
        TypeKind::Class => class_size_map.get(ty.name()).copied().unwrap_or(0),
        TypeKind::Reference | TypeKind::RValueRef => 4, // reference is a pointer under the hood
        TypeKind::Auto => 0,                            // should not appear in codegen before TypeChecker replaces it
        TypeKind::TemplateId => 0,                      // should be resolved to Class before codegen
    }
}
