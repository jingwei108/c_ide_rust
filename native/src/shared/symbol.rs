use crate::compiler::ast::Type;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Symbol {
    pub name: String,
    pub addr: u32,
    pub is_local: bool,
    pub ty: Type,
    pub scope_depth: i32,
    pub func_name: String,
    /// 声明处的源码行号（0 = 未知/不适用）。见 `cide_runtime::Symbol::decl_line`。
    pub decl_line: i32,
}
