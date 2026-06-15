use flutter_rust_bridge::frb;

// ========== FRB 友好数据结构 ==========

// 与 session.rs 完全一致的类型直接 re-export，消除重复定义
#[frb]
pub use crate::session::{
    AlgorithmMatch, CompileResult, Diagnostic, HeapStats, MemoryFragment, MemoryRegion, RunResult, StepResult,
    StepStatus, TraceEntry, VisEvent,
};

// 统一模式类型 re-export
#[frb]
pub use crate::diagnostics::knowledge_graph::{ActivatedConcept, ConceptEdge, ConceptNode, NeighborConcept};

#[frb]
pub use crate::diagnostics::learning_path::{LearningPath, PathStep};

#[frb]
pub use crate::diagnostics::misconception_patterns::{CompileRecord, DetectedMisconception, MisconceptionPattern};

#[frb]
pub fn detect_misconceptions(history: Vec<CompileRecord>) -> Vec<DetectedMisconception> {
    crate::diagnostics::misconception_patterns::detect_misconceptions(history)
}

#[frb]
pub fn recommend_learning_paths(detected: Vec<DetectedMisconception>) -> Vec<LearningPath> {
    crate::diagnostics::learning_path::recommend_learning_paths(detected)
}

#[frb]
pub fn activate_concepts_from_error(error_code: i32) -> Vec<ActivatedConcept> {
    crate::diagnostics::knowledge_graph::activate_from_error(error_code)
}

#[frb]
pub fn activate_concepts_from_ast(features: Vec<String>) -> Vec<ActivatedConcept> {
    crate::diagnostics::knowledge_graph::activate_from_ast(features)
}

#[frb]
pub fn find_prerequisite_path(target_id: String) -> Vec<ConceptNode> {
    crate::diagnostics::knowledge_graph::find_prerequisite_path(target_id)
}

#[frb]
pub fn get_all_concept_nodes() -> Vec<ConceptNode> {
    crate::diagnostics::knowledge_graph::get_all_concept_nodes()
}

#[frb]
pub fn get_all_concept_edges() -> Vec<ConceptEdge> {
    crate::diagnostics::knowledge_graph::get_all_concept_edges()
}

#[frb]
pub use crate::unified::root_cause::RootCauseHint;

// 智能补全 v2
#[frb]
#[derive(Debug, Clone)]
pub struct CompletionCandidate {
    pub label: String,
    pub kind: String,
    pub detail: String,
    pub documentation: String,
    pub insert_text: String,
    pub sort_text: String,
}

#[frb]
pub fn get_completion_candidates(source: String, line: i32, column: i32, prefix: String) -> Vec<CompletionCandidate> {
    crate::flutter_bridge::get_completion_candidates(source, line, column, prefix)
        .into_iter()
        .map(|c| CompletionCandidate {
            label: c.label,
            kind: c.kind.as_str().to_string(),
            detail: c.detail,
            documentation: c.documentation,
            insert_text: c.insert_text,
            sort_text: c.sort_text,
        })
        .collect()
}

// P3: Code Understanding Layer
#[frb]
pub use crate::compiler::intent::{CodeIntent, IntentScore};

#[frb]
pub fn infer_intent_from_source(source: String) -> Vec<IntentScore> {
    let (tokens, _) = crate::compiler::lexer::Lexer::new(&source).tokenize();
    let (program, _) = crate::compiler::parser::Parser::new(tokens).parse();
    let program = match program {
        Some(p) => p,
        None => return Vec::new(),
    };
    let mut all = Vec::new();
    for func in &program.funcs {
        let mut scores = crate::compiler::intent::infer_intent(func);
        // annotate with function name
        for s in &mut scores {
            s.reasons.insert(0, format!("函数: {}", func.name));
        }
        all.extend(scores);
    }
    all
}

#[frb]
pub use crate::unified::types::{
    AlgorithmStepSnapshot, ApiFrameInfo, ApiVariableSnapshot, AutoStepResult, HeatmapData, HeatmapDelta,
    PointerSnapshot, PointerStatus, SeekResult, StepMeta, StepPayload, UnifiedRunResult,
};

// Stream 批量传输优化类型
#[frb]
pub use crate::unified::stream::{
    AccessedVarRef, AlgorithmStepSnapshotRef, ApiFrameInfoRef, ApiVarSnapshotRef, ArraySnapshotRef, PointerSnapshotRef,
    StepPayloadDelta, StepPayloadRef, StepStreamBatch, VarDelta,
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
    let value_str = if v.ty.kind() == crate::compiler::ast::TypeKind::Double {
        let bits = v.value as u64;
        let f = f64::from_bits(bits);
        format!("{:.15}", f).trim_end_matches('0').trim_end_matches('.').to_string()
    } else if v.ty.kind() == crate::compiler::ast::TypeKind::Float {
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
#[derive(Debug, Clone)]
pub struct CodeFile {
    pub filename: String,
    pub source: String,
}

#[frb]
pub fn compile_multi(files: Vec<CodeFile>) -> CompileResult {
    let files = files
        .into_iter()
        .map(|f| crate::session::CodeFile {
            filename: f.filename,
            source: f.source,
        })
        .collect();
    crate::flutter_bridge::compile_multi(files)
}

#[frb]
pub fn compile_and_run_multi(files: Vec<CodeFile>) -> UnifiedRunResult {
    let files = files
        .into_iter()
        .map(|f| crate::session::CodeFile {
            filename: f.filename,
            source: f.source,
        })
        .collect();
    crate::flutter_bridge::compile_and_run_multi(files)
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

/// 根据诊断中的结构化修复信息，对源码执行替换/插入/删除。
/// fix_kind: 0=None, 1=ReplaceText, 2=InsertText, 3=DeleteText, 4=ManualHint
#[frb]
pub fn apply_fix(source: String, diag: Diagnostic) -> Option<String> {
    match diag.fix_kind {
        1 => apply_replace(&source, &diag),
        2 => apply_insert(&source, &diag),
        3 => apply_delete(&source, &diag),
        _ => None,
    }
}

fn apply_replace(source: &str, diag: &Diagnostic) -> Option<String> {
    let mut lines: Vec<String> = source.lines().map(|s| s.to_string()).collect();
    let start_line = diag.replace_start_line as usize;
    let start_col = diag.replace_start_column as usize;
    let end_line = diag.replace_end_line as usize;
    let end_col = diag.replace_end_column as usize;

    if start_line == 0 || end_line == 0 || start_line > lines.len() || end_line > lines.len() {
        return None;
    }
    let start_idx = start_line - 1;
    let end_idx = end_line - 1;
    if start_col > lines[start_idx].len()
        || end_col > lines[end_idx].len()
        || start_col > end_col && start_idx == end_idx
    {
        return None;
    }

    let before = lines[start_idx][..start_col].to_string();
    let after = lines[end_idx][end_col..].to_string();
    let mut new_line = before;
    new_line.push_str(&diag.replacement_text);
    new_line.push_str(&after);

    lines.drain(start_idx..=end_idx);
    lines.insert(start_idx, new_line);
    Some(lines.join("\n"))
}

fn apply_insert(source: &str, diag: &Diagnostic) -> Option<String> {
    let mut lines: Vec<String> = source.lines().map(|s| s.to_string()).collect();
    let start_line = diag.replace_start_line as usize;
    let start_col = diag.replace_start_column as usize;

    if start_line == 0 || start_line > lines.len() || start_col > lines[start_line - 1].len() {
        return None;
    }

    lines[start_line - 1].insert_str(start_col, &diag.replacement_text);
    Some(lines.join("\n"))
}

fn apply_delete(source: &str, diag: &Diagnostic) -> Option<String> {
    let mut lines: Vec<String> = source.lines().map(|s| s.to_string()).collect();
    let start_line = diag.replace_start_line as usize;
    let start_col = diag.replace_start_column as usize;
    let end_line = diag.replace_end_line as usize;
    let end_col = diag.replace_end_column as usize;

    if start_line == 0 || end_line == 0 || start_line > lines.len() || end_line > lines.len() {
        return None;
    }
    let start_idx = start_line - 1;
    let end_idx = end_line - 1;
    if start_col > lines[start_idx].len()
        || end_col > lines[end_idx].len()
        || start_col > end_col && start_idx == end_idx
    {
        return None;
    }

    if start_idx == end_idx {
        lines[start_idx].replace_range(start_col..end_col, "");
    } else {
        let before = lines[start_idx][..start_col].to_string();
        let after = lines[end_idx][end_col..].to_string();
        let mut new_line = before;
        new_line.push_str(&after);
        lines.drain(start_idx..=end_idx);
        lines.insert(start_idx, new_line);
    }
    Some(lines.join("\n"))
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
pub fn get_memory_fragments() -> Vec<MemoryFragment> {
    crate::flutter_bridge::get_memory_fragments()
}

#[frb]
pub fn get_heap_stats() -> HeapStats {
    crate::flutter_bridge::get_heap_stats()
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

// ========== 统一模式 FRB API ==========

#[frb]
pub fn compile_and_run(source: String) -> UnifiedRunResult {
    crate::flutter_bridge::compile_and_run(source)
}

#[frb]
pub fn run_auto_steps(batch_size: i32) -> AutoStepResult {
    crate::flutter_bridge::run_auto_steps(batch_size)
}

#[frb]
pub fn seek_to_step(target: i32) -> SeekResult {
    crate::flutter_bridge::seek_to_step(target)
}

#[frb]
pub fn step_next_unified() -> Option<StepPayload> {
    crate::flutter_bridge::step_next_unified()
}

#[frb]
pub fn pause_execution() {
    crate::flutter_bridge::pause_execution();
}

#[frb]
pub fn resume_execution() {
    crate::flutter_bridge::resume_execution();
}

#[frb]
pub fn get_heatmap() -> HeatmapData {
    crate::flutter_bridge::get_heatmap()
}

#[frb]
pub fn get_step_payloads(start: i32, end: i32) -> Vec<StepPayload> {
    crate::flutter_bridge::get_step_payloads(start, end)
}

#[frb]
pub fn get_frame_cache_start_step() -> i32 {
    crate::flutter_bridge::get_frame_cache_start_step()
}

/// Stream 模式批量自动执行（batch_size=100）。
/// Dart 端订阅返回的 Stream，Rust 在后台线程中循环执行并推送批次。
#[frb]
pub fn run_auto_steps_stream(sink: crate::frb_generated::StreamSink<StepStreamBatch>, batch_size: i32) {
    crate::flutter_bridge::run_auto_steps_stream(sink, batch_size);
}

#[frb]
pub fn continue_from_step(step: i32) -> UnifiedRunResult {
    crate::flutter_bridge::continue_from_step(step)
}

#[frb]
pub fn create_session() -> u64 {
    crate::flutter_bridge::create_session()
}

#[frb]
pub fn destroy_session(session_id: u64) {
    crate::flutter_bridge::destroy_session(session_id);
}

#[frb]
pub fn set_current_session_id(session_id: u64) {
    crate::flutter_bridge::set_current_session_id(session_id);
}

#[frb]
pub fn get_current_session_id() -> u64 {
    crate::flutter_bridge::get_current_session_id()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Diagnostic;

    fn diag_replace(line: i32, start_line: i32, start_col: i32, end_line: i32, end_col: i32, text: &str) -> Diagnostic {
        Diagnostic {
            line,
            column: 1,
            error_code: 2005,
            severity: 0,
            message: "test".to_string(),
            fix_suggestion: "add semicolon".to_string(),
            fix_kind: 1,
            replace_start_line: start_line,
            replace_start_column: start_col,
            replace_end_line: end_line,
            replace_end_column: end_col,
            replacement_text: text.to_string(),
            filename: "main.c".to_string(),
        }
    }

    fn diag_insert(line: i32, col: i32, text: &str) -> Diagnostic {
        Diagnostic {
            line,
            column: 1,
            error_code: 2005,
            severity: 0,
            message: "test".to_string(),
            fix_suggestion: "insert".to_string(),
            fix_kind: 2,
            replace_start_line: line,
            replace_start_column: col,
            replace_end_line: line,
            replace_end_column: col,
            replacement_text: text.to_string(),
            filename: "main.c".to_string(),
        }
    }

    fn diag_delete(start_line: i32, start_col: i32, end_line: i32, end_col: i32) -> Diagnostic {
        Diagnostic {
            line: start_line,
            column: 1,
            error_code: 2005,
            severity: 0,
            message: "test".to_string(),
            fix_suggestion: "delete".to_string(),
            fix_kind: 3,
            replace_start_line: start_line,
            replace_start_column: start_col,
            replace_end_line: end_line,
            replace_end_column: end_col,
            replacement_text: String::new(),
            filename: "main.c".to_string(),
        }
    }

    #[test]
    fn test_apply_fix_replace_single_line() {
        let source = "int main()\n    return 0\n}".to_string();
        let diag = diag_replace(2, 2, 12, 2, 12, ";");
        assert_eq!(apply_fix(source, diag).unwrap(), "int main()\n    return 0;\n}");
    }

    #[test]
    fn test_apply_fix_replace_multi_line() {
        let source = "int main() {\n    int a = 1\n    return a\n}".to_string();
        let diag = diag_replace(2, 2, 13, 3, 12, ";\n    return a;");
        assert_eq!(
            apply_fix(source, diag).unwrap(),
            "int main() {\n    int a = 1;\n    return a;\n}"
        );
    }

    #[test]
    fn test_apply_fix_insert() {
        let source = "int main()\n    return 0\n}".to_string();
        let diag = diag_insert(2, 12, ";");
        assert_eq!(apply_fix(source, diag).unwrap(), "int main()\n    return 0;\n}");
    }

    #[test]
    fn test_apply_fix_delete() {
        let source = "int main() {\n    int a = 1;;\n    return a;\n}".to_string();
        let diag = diag_delete(2, 14, 2, 15);
        assert_eq!(
            apply_fix(source, diag).unwrap(),
            "int main() {\n    int a = 1;\n    return a;\n}"
        );
    }

    #[test]
    fn test_apply_fix_no_fix_kind_returns_none() {
        let source = "int main() {}".to_string();
        let diag = Diagnostic {
            line: 1,
            column: 1,
            error_code: 3023,
            severity: 0,
            message: "undeclared".to_string(),
            fix_suggestion: String::new(),
            fix_kind: 0,
            replace_start_line: 0,
            replace_start_column: 0,
            replace_end_line: 0,
            replace_end_column: 0,
            replacement_text: String::new(),
            filename: "main.c".to_string(),
        };
        assert!(apply_fix(source, diag).is_none());
    }
}
