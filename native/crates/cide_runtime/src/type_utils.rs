use cide_ast::{Type, TypeKind};

/// 解引用一层指针/数组，得到其元素（或指向）的标量类型。
pub fn base_kind(ty: &Type) -> TypeKind {
    match ty {
        Type::Pointer { pointee, .. } => pointee.kind(),
        Type::Array { element, .. } => base_kind(element),
        _ => ty.kind(),
    }
}

/// 只解引用一层，用于数组元素类型判断和单级指针解引用。
pub fn immediate_base_kind(ty: &Type) -> TypeKind {
    match ty {
        Type::Pointer { pointee, .. } => pointee.kind(),
        Type::Array { element, .. } => element.kind(),
        _ => ty.kind(),
    }
}
