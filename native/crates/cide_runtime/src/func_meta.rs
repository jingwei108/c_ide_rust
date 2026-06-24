use cide_ast::Type;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FuncMeta {
    pub ip: usize,
    /// 参数总 word 数（以 4-byte words 计），供 Call 指令弹栈使用。
    pub arg_count: i32,
    /// 参数个数（供 call_user_function 使用，与总 word 数不同）。
    pub param_count: i32,
    pub local_count: i32,
    pub param_sizes: Vec<i32>,
    /// 函数返回类型（codegen 使用；session/vm 中保持默认 Void 即可）。
    #[serde(default)]
    pub return_type: Type,
}
