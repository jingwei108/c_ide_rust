//! 统一模式快照差分比较辅助函数。

use crate::session::VisEvent;
use crate::unified::types::{AccessedVar, ApiFrameInfo, ArraySnapshot, PointerSnapshot};

pub fn frame_lists_equal(a: &[ApiFrameInfo], b: &[ApiFrameInfo]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .all(|(x, y)| x.func_name == y.func_name && x.return_line == y.return_line)
}

pub fn vis_events_equal(a: &[VisEvent], b: &[VisEvent]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).all(|(x, y)| {
        x.ty == y.ty
            && x.line == y.line
            && x.extra0 == y.extra0
            && x.extra1 == y.extra1
            && x.extra2 == y.extra2
            && x.context == y.context
    })
}

pub fn accessed_vars_equal(a: &[AccessedVar], b: &[AccessedVar]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .all(|(x, y)| x.name == y.name && x.access_type == y.access_type)
}

pub fn arrays_equal(a: &ArraySnapshot, b: &ArraySnapshot) -> bool {
    a.name == b.name && a.element_ty == b.element_ty && a.elements == b.elements
}

pub fn pointers_equal(a: &PointerSnapshot, b: &PointerSnapshot) -> bool {
    a.name == b.name
        && a.addr == b.addr
        && a.ty_name == b.ty_name
        && a.target_addr == b.target_addr
        && a.target_name == b.target_name
        && a.status == b.status
}
