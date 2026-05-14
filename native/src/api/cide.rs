use flutter_rust_bridge::frb;

// ========== FRB 友好数据结构 ==========

#[frb]
#[derive(Debug, Clone)]
pub struct CompileResult {
    pub success: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub algorithm_matches: Vec<AlgorithmMatch>,
}

#[frb]
#[derive(Debug, Clone)]
pub struct RunResult {
    pub success: bool,
    pub output: String,
    pub waiting_input: bool,
    pub error: Option<String>,
}

#[frb]
#[derive(Debug, Clone)]
pub struct StepResult {
    pub status: StepStatus,
    pub current_line: i32,
    pub output: String,
    pub waiting_input: bool,
}

#[frb]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Paused,
    WaitingInput,
    Finished,
    Trap,
}

#[frb]
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub line: i32,
    pub column: i32,
    pub error_code: i32,
    pub severity: i32,
    pub message: String,
    pub fix_suggestion: String,
    pub fix_kind: i32,
    pub replace_start_line: i32,
    pub replace_start_column: i32,
    pub replace_end_line: i32,
    pub replace_end_column: i32,
    pub replacement_text: String,
}

#[frb]
#[derive(Debug, Clone)]
pub struct AlgorithmMatch {
    pub name: String,
    pub display_name: String,
    pub func_name: String,
    pub confidence: i32,
    pub suggestion: String,
    pub line: i32,
    pub vis_events: Vec<VisEvent>,
}

#[frb]
#[derive(Debug, Clone)]
pub struct VisEvent {
    pub ty: i32,
    pub line: i32,
    pub extra0: i32,
    pub extra1: i32,
    pub extra2: i32,
    pub context: String,
}

#[frb]
#[derive(Debug, Clone)]
pub struct VariableSnapshot {
    pub name: String,
    pub addr: u32,
    pub is_local: bool,
    pub ty_name: String,
    pub value: i32,
}

#[frb]
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub addr: u32,
    pub size: i32,
    pub name: String,
    pub ty: String,
    pub is_heap: bool,
    pub is_freed: bool,
}

#[frb]
#[derive(Debug, Clone)]
pub struct TraceEntry {
    pub line: i32,
    pub operation: String,
}

#[frb]
#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub offset: i32,
}

// ========== 转换辅助函数 ==========

fn convert_diagnostic(d: crate::session::Diagnostic) -> Diagnostic {
    Diagnostic {
        line: d.line,
        column: d.column,
        error_code: d.error_code,
        severity: d.severity,
        message: d.message,
        fix_suggestion: d.fix_suggestion,
        fix_kind: d.fix_kind,
        replace_start_line: d.replace_start_line,
        replace_start_column: d.replace_start_column,
        replace_end_line: d.replace_end_line,
        replace_end_column: d.replace_end_column,
        replacement_text: d.replacement_text,
    }
}

fn convert_algorithm_match(m: crate::session::AlgorithmMatch) -> AlgorithmMatch {
    AlgorithmMatch {
        name: m.name,
        display_name: m.display_name,
        func_name: m.func_name,
        confidence: m.confidence,
        suggestion: m.suggestion,
        line: m.line,
        vis_events: m.vis_events.into_iter().map(|(line, ty, ctx)| VisEvent { line, ty, extra0: 0, extra1: 0, extra2: 0, context: ctx }).collect(),
    }
}

fn convert_step_status(s: crate::flutter_bridge::StepStatus) -> StepStatus {
    match s {
        crate::flutter_bridge::StepStatus::Paused => StepStatus::Paused,
        crate::flutter_bridge::StepStatus::WaitingInput => StepStatus::WaitingInput,
        crate::flutter_bridge::StepStatus::Finished => StepStatus::Finished,
        crate::flutter_bridge::StepStatus::Trap => StepStatus::Trap,
    }
}

fn convert_step_result(r: crate::flutter_bridge::StepResult) -> StepResult {
    StepResult {
        status: convert_step_status(r.status),
        current_line: r.current_line,
        output: r.output,
        waiting_input: r.waiting_input,
    }
}

fn convert_variable(v: crate::session::VariableSnapshot) -> VariableSnapshot {
    VariableSnapshot {
        name: v.name,
        addr: v.addr,
        is_local: v.is_local,
        ty_name: format!("{:?}", v.ty),
        value: v.value,
    }
}

fn convert_memory_region(r: crate::session::MemoryRegion) -> MemoryRegion {
    MemoryRegion {
        addr: r.addr,
        size: r.size,
        name: r.name,
        ty: r.ty,
        is_heap: r.is_heap,
        is_freed: r.is_freed,
    }
}

fn convert_trace_entry(t: crate::session::TraceEntry) -> TraceEntry {
    TraceEntry {
        line: t.line,
        operation: t.operation,
    }
}

fn convert_compile_result(r: crate::flutter_bridge::CompileResult) -> CompileResult {
    CompileResult {
        success: r.success,
        diagnostics: r.diagnostics.into_iter().map(convert_diagnostic).collect(),
        algorithm_matches: r.algorithm_matches.into_iter().map(convert_algorithm_match).collect(),
    }
}

// ========== 公开 API ==========

#[frb]
pub fn compile(source: String) -> CompileResult {
    convert_compile_result(crate::flutter_bridge::compile(source))
}

#[frb]
pub fn run_code() -> RunResult {
    let r = crate::flutter_bridge::run_code();
    RunResult {
        success: r.success,
        output: r.output,
        waiting_input: r.waiting_input,
        error: r.error,
    }
}

#[frb]
pub fn step_next() -> StepResult {
    convert_step_result(crate::flutter_bridge::step_next())
}

#[frb]
pub fn get_diagnostics() -> Vec<Diagnostic> {
    crate::flutter_bridge::get_diagnostics()
        .into_iter()
        .map(convert_diagnostic)
        .collect()
}

#[frb]
pub fn get_algorithm_matches() -> Vec<AlgorithmMatch> {
    crate::flutter_bridge::get_algorithm_matches()
        .into_iter()
        .map(convert_algorithm_match)
        .collect()
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
        .into_iter()
        .map(convert_memory_region)
        .collect()
}

#[frb]
pub fn get_callstack() -> Vec<TraceEntry> {
    crate::flutter_bridge::get_callstack()
        .into_iter()
        .map(convert_trace_entry)
        .collect()
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
        .into_iter()
        .map(|v| VisEvent {
            line: v.line,
            ty: v.ty,
            extra0: v.extra[0],
            extra1: v.extra[1],
            extra2: v.extra[2],
            context: String::new(),
        })
        .collect()
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
