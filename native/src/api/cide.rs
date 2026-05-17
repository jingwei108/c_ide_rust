use flutter_rust_bridge::frb;

// ========== FRB 友好数据结构 ==========

// 与 session.rs 完全一致的类型直接 re-export，消除重复定义
#[frb]
pub use crate::session::{
    AlgorithmMatch, CompileResult, Diagnostic, MemoryRegion, RunResult, StepResult, StepStatus,
    TraceEntry, VisEvent,
};

#[frb]
#[derive(Debug, Clone)]
pub struct VariableSnapshot {
    pub name: String,
    pub addr: u32,
    pub is_local: bool,
    pub ty_name: String,
    pub value: String,
}

#[frb]
#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub offset: i32,
}

// ========== 转换辅助函数 ==========

fn convert_variable(v: crate::session::VariableSnapshot) -> VariableSnapshot {
    let value_str = if v.ty.kind == crate::compiler::ast::TypeKind::Double {
        let bits = v.value as u64;
        let f = f64::from_bits(bits);
        format!("{:.15}", f).trim_end_matches('0').trim_end_matches('.').to_string()
    } else if v.ty.kind == crate::compiler::ast::TypeKind::Float {
        let bits = v.value as u32;
        let f = f32::from_bits(bits);
        format!("{:.7}", f).trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        v.value.to_string()
    };
    VariableSnapshot {
        name: v.name,
        addr: v.addr,
        is_local: v.is_local,
        ty_name: format!("{:?}", v.ty),
        value: value_str,
    }
}

// ========== 公开 API ==========

#[frb]
pub fn compile(source: String) -> CompileResult {
    crate::flutter_bridge::compile(source)
}

#[frb]
pub fn run_code() -> RunResult {
    crate::flutter_bridge::run_code()
}

#[frb]
pub fn step_next() -> StepResult {
    crate::flutter_bridge::step_next()
}

#[frb]
pub fn get_diagnostics() -> Vec<Diagnostic> {
    crate::flutter_bridge::get_diagnostics()
}

#[frb]
pub fn get_algorithm_matches() -> Vec<AlgorithmMatch> {
    crate::flutter_bridge::get_algorithm_matches()
}

#[frb]
pub fn get_variables() -> Vec<VariableSnapshot> {
    crate::flutter_bridge::get_variables()
        .into_iter()
        .map(convert_variable)
        .collect()
}

#[frb]
pub fn get_memory_regions() -> Vec<MemoryRegion> {
    crate::flutter_bridge::get_memory_regions()
}

#[frb]
pub fn get_memory_size() -> u32 {
    crate::flutter_bridge::get_memory_size()
}

#[frb]
pub fn get_callstack() -> Vec<TraceEntry> {
    crate::flutter_bridge::get_callstack()
}

#[frb]
pub fn get_output() -> String {
    crate::flutter_bridge::get_output()
}

#[frb]
pub fn get_current_line() -> i32 {
    crate::flutter_bridge::get_current_line()
}

#[frb]
pub fn is_waiting_input() -> bool {
    crate::flutter_bridge::is_waiting_input()
}

#[frb]
pub fn add_breakpoint(line: i32) {
    crate::flutter_bridge::add_breakpoint(line);
}

#[frb]
pub fn clear_breakpoints() {
    crate::flutter_bridge::clear_breakpoints();
}

#[frb]
pub fn set_breakpoints(lines: Vec<i32>) {
    crate::flutter_bridge::set_breakpoints(lines);
}

#[frb]
pub fn set_input(input: String) {
    crate::flutter_bridge::set_input(input);
}

#[frb]
pub fn provide_input_line(line: String) {
    crate::flutter_bridge::provide_input_line(line);
}

#[frb]
pub fn get_vis_events() -> Vec<VisEvent> {
    crate::flutter_bridge::get_vis_events()
}

#[frb]
pub fn clear_vis_events() {
    crate::flutter_bridge::clear_vis_events();
}

#[frb]
pub fn read_memory(addr: u32, count: u32) -> Vec<i32> {
    crate::flutter_bridge::read_memory(addr, count)
}

#[frb]
pub fn get_struct_fields(name: String) -> Vec<StructField> {
    crate::flutter_bridge::get_struct_fields(name)
        .into_iter()
        .map(|(n, offset)| StructField { name: n, offset })
        .collect()
}

#[frb]
pub fn reset_session() {
    crate::flutter_bridge::reset_session();
}
