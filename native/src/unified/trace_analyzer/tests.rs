//! Unit tests for trace analysis helpers.

use crate::unified::trace_analyzer::bounds::{build_bounds_hint, infer_bounds_category, BoundsCategory};
use crate::unified::trace_analyzer::utils::{
    extract_array_size, extract_assignment_rhs, extract_increment, format_history_vals, is_likely_loop_var,
    parse_alloc_freed_lines, parse_bounds_message, slice_variable_history, LoopInfo,
};
use crate::unified::types::{ApiVariableSnapshot, StepPayload};

#[test]
fn test_parse_bounds_message() {
    let msg = "🚫 数组越界：你访问了 arr[5]，但数组 'arr' 只有 5 个元素，有效索引是 0~4。";
    let (name, idx) = parse_bounds_message(msg).unwrap();
    assert_eq!(name, "arr");
    assert_eq!(idx, 5);
    assert_eq!(extract_array_size(msg), Some(5));
}

#[test]
fn test_parse_alloc_freed_lines() {
    let msg =
        "💥 Use-After-Free (E3060)：你正在读取一块已经在第 10 行被 free 的内存（由第 3 行的 malloc/realloc 分配）。";
    let (alloc, freed) = parse_alloc_freed_lines(msg).unwrap();
    assert_eq!(alloc, 3);
    assert_eq!(freed, 10);
}

#[test]
fn test_is_likely_loop_var() {
    assert!(is_likely_loop_var("i"));
    assert!(is_likely_loop_var("idx"));
    assert!(!is_likely_loop_var("total"));
}

#[test]
fn test_slice_variable_history() {
    let steps = vec![
        StepPayload {
            step_index: 0,
            code_line: 1,
            func_name: "main".into(),
            semantic_label: "".into(),
            algorithm_step: None,
            local_vars: vec![ApiVariableSnapshot {
                name: "i".into(),
                addr: 0,
                is_local: true,
                ty_name: "int".into(),
                value: "0".into(),
            }],
            call_stack: vec![],
            vis_events: vec![],
            heatmap_line: 1,
            heatmap_count: 0,
            accessed_vars: vec![],
            array_snapshots: vec![],
            pointer_snapshots: vec![],
            root_cause_hint: None,
        },
        StepPayload {
            step_index: 1,
            code_line: 1,
            func_name: "main".into(),
            semantic_label: "".into(),
            algorithm_step: None,
            local_vars: vec![ApiVariableSnapshot {
                name: "i".into(),
                addr: 0,
                is_local: true,
                ty_name: "int".into(),
                value: "1".into(),
            }],
            call_stack: vec![],
            vis_events: vec![],
            heatmap_line: 1,
            heatmap_count: 0,
            accessed_vars: vec![],
            array_snapshots: vec![],
            pointer_snapshots: vec![],
            root_cause_hint: None,
        },
    ];
    let hist = slice_variable_history(&steps, "i", 1, 3);
    assert_eq!(hist.len(), 2);
    assert_eq!(hist[0].value, "0");
    assert_eq!(hist[1].value, "1");
}

#[test]
fn test_infer_off_by_one() {
    // History: i goes 0→1→2→3→4→5, array_size=5, accessed=5, source has <=
    let hist = vec![0, 1, 2, 3, 4, 5];
    let loop_info = LoopInfo {
        lines: vec![3],
        has_le: true,
        has_ge: false,
        start_val: Some(0),
        increment: Some(1),
    };
    let cat = infer_bounds_category(&hist, 5, 5, &loop_info);
    assert_eq!(cat, BoundsCategory::OffByOne);

    let (msg, fix, fix_line, fix_desc) = build_bounds_hint(cat, "i", "arr", 5, 5, &loop_info, &hist);
    assert!(msg.contains("<="));
    assert!(msg.contains("第 3 行"));
    assert_eq!(fix, "ChangeLeToLt");
    assert_eq!(fix_line, Some(3));
    assert_eq!(fix_desc, Some("将 <= 改为 <".to_string()));
}

#[test]
fn test_infer_wrong_init() {
    // History: i goes 1→2→3→4→5, array_size=5, accessed=5 (should have been 0..4)
    let hist = vec![1, 2, 3, 4, 5];
    let loop_info = LoopInfo {
        lines: vec![3],
        has_le: false,
        has_ge: false,
        start_val: Some(1),
        increment: Some(1),
    };
    let cat = infer_bounds_category(&hist, 5, 5, &loop_info);
    assert_eq!(cat, BoundsCategory::WrongInit);

    let (msg, fix, fix_line, fix_desc) = build_bounds_hint(cat, "i", "arr", 5, 5, &loop_info, &hist);
    assert!(msg.contains("从 1 开始"));
    assert!(msg.contains("改为 0"));
    assert_eq!(fix, "FixLoopStart");
    assert_eq!(fix_line, Some(3));
    assert_eq!(fix_desc, Some("将 i 的初始值 1 改为 0".to_string()));
}

#[test]
fn test_infer_wrong_increment() {
    // History: i goes 0→2→4→6, array_size=5, accessed=6 (step=2 skips over)
    let hist = vec![0, 2, 4, 6];
    let loop_info = LoopInfo {
        lines: vec![3, 5],
        has_le: false,
        has_ge: false,
        start_val: Some(0),
        increment: Some(2),
    };
    let cat = infer_bounds_category(&hist, 6, 5, &loop_info);
    assert_eq!(cat, BoundsCategory::WrongIncrement);

    let (msg, fix, fix_line, fix_desc) = build_bounds_hint(cat, "i", "arr", 6, 5, &loop_info, &hist);
    assert!(msg.contains("步长是 2"));
    assert_eq!(fix, "FixLoopIncrement");
    assert_eq!(fix_line, Some(3));
    assert_eq!(fix_desc, Some("将步长 2 改为 1".to_string()));
}

#[test]
fn test_infer_uninitialized_index() {
    let hist: Vec<i32> = vec![];
    let loop_info = LoopInfo::default();
    let cat = infer_bounds_category(&hist, 99, 5, &loop_info);
    assert_eq!(cat, BoundsCategory::UninitializedIndex);

    let (msg, fix, fix_line, fix_desc) = build_bounds_hint(cat, "idx", "arr", 99, 5, &loop_info, &hist);
    assert!(msg.contains("未初始化"));
    assert_eq!(fix, "InitVariable");
    assert_eq!(fix_line, None);
    assert_eq!(fix_desc, Some("初始化 'idx'".to_string()));
}

#[test]
fn test_extract_assignment_rhs() {
    assert_eq!(extract_assignment_rhs("int i = 0;", "i"), Some(0));
    assert_eq!(extract_assignment_rhs("int i = 5;", "i"), Some(5));
    assert_eq!(extract_assignment_rhs("i=3", "i"), Some(3));
    assert_eq!(extract_assignment_rhs("j = 10", "i"), None);
}

#[test]
fn test_extract_increment() {
    assert_eq!(extract_increment("i++;", "i"), Some(1));
    assert_eq!(extract_increment("++i;", "i"), Some(1));
    assert_eq!(extract_increment("i += 2;", "i"), Some(2));
    assert_eq!(extract_increment("i = i + 3;", "i"), Some(3));
    assert_eq!(extract_increment("j += 2;", "i"), None);
}

#[test]
fn test_format_history_vals_short() {
    assert_eq!(format_history_vals(&[1, 2, 3]), "1 → 2 → 3");
}

#[test]
fn test_format_history_vals_long() {
    assert_eq!(format_history_vals(&[1, 2, 3, 4, 5]), "1 → ... → 5");
}
