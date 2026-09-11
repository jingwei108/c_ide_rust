//! StepPayload Schema v0.1 冻结测试（对应 `docs/spec/STEP_PAYLOAD_SCHEMA_V0_1.md`）。
//!
//! 目的：schema "定稿"必须有可执行的锚点 —— 字段名、枚举字面量、窗口常量一旦
//! 被无意改动，本测试立即失败，迫使改动方走版本化流程（字段只增不改语义）。
//!
//! 断言对象是 **capi 出口的 JSON**，不是 Rust 结构体：协议契约必须在出口成立。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeSet;
use std::ffi::{c_char, CStr, CString};

use cide_native::capi;
use cide_native::session::Session;
use cide_native::unified::types::{PointerStatus, StepPayload};

/// schema v0.1 顶层 14 字段（`docs/spec/STEP_PAYLOAD_SCHEMA_V0_1.md` §1）。
const TOP_LEVEL_FIELDS: [&str; 14] = [
    "step_index",
    "code_line",
    "func_name",
    "semantic_label",
    "algorithm_step",
    "local_vars",
    "call_stack",
    "vis_events",
    "heatmap_line",
    "heatmap_count",
    "accessed_vars",
    "array_snapshots",
    "pointer_snapshots",
    "root_cause_hint",
];

unsafe fn take_string(p: *mut c_char) -> String {
    if p.is_null() {
        return String::new();
    }
    let s = CStr::from_ptr(p).to_string_lossy().to_string();
    capi::cide_free_string(p);
    s
}

fn compile_session(source: &str) -> *mut Session {
    unsafe {
        let session = capi::cide_session_create();
        assert!(!session.is_null(), "session 创建失败");
        let fname = CString::new("main.c").unwrap();
        let src = CString::new(source).unwrap();
        capi::cide_compile_unit(session, fname.as_ptr(), src.as_ptr());
        let diag = take_string(capi::cide_compile_json(session));
        let v: serde_json::Value = serde_json::from_str(&diag).unwrap();
        assert_eq!(v["ok"], true, "示例程序应编译成功: {}", diag);
        session
    }
}

fn step_until(session: *mut Session, steps: usize) -> Vec<serde_json::Value> {
    let mut collected = Vec::new();
    unsafe {
        // 统一模式必须显式初始化（cide_step_begin 装载 VM + 建初始检查点）
        assert_eq!(
            capi::cide_step_begin(session),
            0,
            "统一模式会话初始化失败（会话须已编译成功）"
        );
        for _ in 0..steps {
            let s = take_string(capi::cide_step_next_json(session));
            let v: serde_json::Value = serde_json::from_str(&s).unwrap();
            for p in v["payloads"].as_array().cloned().unwrap_or_default() {
                collected.push(p);
            }
            if v["finished"] == true || v["trapped"] == true {
                break;
            }
        }
    }
    collected
}

fn keys_of(value: &serde_json::Value) -> BTreeSet<String> {
    value
        .as_object()
        .expect("应为 JSON 对象")
        .keys()
        .cloned()
        .collect()
}

const SAMPLE: &str = r#"
#include <stdio.h>
int main() {
    int arr[3] = {1, 2, 3};
    int *p = arr;
    int s = 0;
    for (int i = 0; i < 3; i++) { s += arr[i]; }
    *p = 5;
    printf("%d", s);
    return s;
}
"#;

#[test]
fn test_step_payload_top_level_fields_frozen() {
    let session = compile_session(SAMPLE);
    let payloads = step_until(session, 60);
    unsafe { capi::cide_session_destroy(session) };

    assert!(!payloads.is_empty(), "应至少收集到一个 payload");
    let expected: BTreeSet<String> = TOP_LEVEL_FIELDS.iter().map(|s| s.to_string()).collect();
    for p in &payloads {
        let actual = keys_of(p);
        assert_eq!(
            actual, expected,
            "StepPayload 顶层字段集合与 schema v0.1 不一致（新增字段须走版本化流程）：{}",
            p
        );
    }
}

#[test]
fn test_substructure_fields_frozen() {
    let session = compile_session(SAMPLE);
    let payloads = step_until(session, 60);
    unsafe { capi::cide_session_destroy(session) };

    let mut saw_local = false;
    let mut saw_array = false;
    let mut saw_pointer = false;
    let mut saw_call_frame = false;
    let mut saw_accessed = false;

    for p in &payloads {
        for v in p["local_vars"].as_array().cloned().unwrap_or_default() {
            saw_local = true;
            assert_eq!(
                keys_of(&v),
                ["addr", "is_local", "name", "ty_name", "value"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<BTreeSet<_>>(),
                "ApiVariableSnapshot 字段与 schema 不一致: {}",
                v
            );
        }
        for f in p["call_stack"].as_array().cloned().unwrap_or_default() {
            saw_call_frame = true;
            assert_eq!(
                keys_of(&f),
                ["func_name", "return_line"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<BTreeSet<_>>(),
                "ApiFrameInfo 字段与 schema 不一致: {}",
                f
            );
        }
        for a in p["accessed_vars"].as_array().cloned().unwrap_or_default() {
            saw_accessed = true;
            assert_eq!(
                keys_of(&a),
                ["access_type", "name"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<BTreeSet<_>>(),
                "AccessedVar 字段与 schema 不一致: {}",
                a
            );
            let ty = a["access_type"].as_str().unwrap_or("");
            assert!(
                ty == "Read" || ty == "Write",
                "access_type 只允许 Read/Write（大小写敏感），实际: {}",
                ty
            );
        }
        for arr in p["array_snapshots"].as_array().cloned().unwrap_or_default() {
            saw_array = true;
            assert_eq!(
                keys_of(&arr),
                ["element_ty", "elements", "name"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<BTreeSet<_>>(),
                "ArraySnapshot 字段与 schema 不一致: {}",
                arr
            );
        }
        for ptr in p["pointer_snapshots"].as_array().cloned().unwrap_or_default() {
            saw_pointer = true;
            assert_eq!(
                keys_of(&ptr),
                ["addr", "name", "status", "target_addr", "target_name", "ty_name"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<BTreeSet<_>>(),
                "PointerSnapshot 字段与 schema 不一致: {}",
                ptr
            );
            let status = ptr["status"].as_str().unwrap_or("");
            assert!(
                ["Valid", "Freed", "Null", "Dangling"].contains(&status),
                "pointer status 必须落在四状态枚举内，实际: {}",
                status
            );
        }
    }

    assert!(saw_local, "样本程序应产生变量快照");
    assert!(saw_call_frame, "样本程序应产生调用栈");
    assert!(saw_accessed, "样本程序应产生 accessed_vars");
    assert!(saw_array, "样本程序应产生数组快照");
    assert!(saw_pointer, "样本程序应产生指针快照");
}

#[test]
fn test_pointer_status_enum_literals_frozen() {
    // 枚举字面量是协议的一部分：改名 = 破坏性变更
    for (status, literal) in [
        (PointerStatus::Valid, "Valid"),
        (PointerStatus::Freed, "Freed"),
        (PointerStatus::Null, "Null"),
        (PointerStatus::Dangling, "Dangling"),
    ] {
        let v = serde_json::to_value(status).unwrap();
        assert_eq!(v, serde_json::json!(literal), "PointerStatus 字面量被改动");
    }
}

#[test]
fn test_frame_cache_window_2000_frames_with_20pct_trim() {
    // 窗口语义：2000 帧上限、超限丢最早 20%（§4）
    let session = compile_session(
        "#include <stdio.h>\nint main(){ int s = 0; for (int i = 0; i < 900; i++) { s = s + i; } printf(\"%d\", s); return 0; }\n",
    );
    let steps = step_until(session, 20_000).len();
    assert!(steps > 2000, "样本应产生 >2000 步以触发窗口裁剪，实际 {}", steps);

    unsafe {
        let s = take_string(capi::cide_get_step_payloads_json(session, 0, i32::MAX));
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        let window = v["payloads"].as_array().expect("payloads 必须是数组");
        let cache_start = v["cache_start_step"].as_i64().unwrap_or(-1);

        assert!(
            window.len() <= 2000,
            "frame_cache 窗口不得超过 2000 帧，实际 {}",
            window.len()
        );
        assert!(cache_start > 0, "窗口应已滑动，cache_start_step 实际 {}", cache_start);
        // 越窗查询被静默裁剪：请求 [0, MAX) 只返回窗口内子集
        let first_step = window[0]["step_index"].as_i64().unwrap_or(-1);
        assert_eq!(
            first_step, cache_start,
            "窗口首帧步号必须等于 cache_start_step"
        );
        capi::cide_session_destroy(session);
    }
}

#[test]
fn test_step_payload_serializes_every_field() {
    // 类型链完整性：StepPayload 必须能整体序列化（不丢字段）
    let payload = StepPayload {
        step_index: 7,
        code_line: 3,
        func_name: "main".to_string(),
        semantic_label: "第 3 行".to_string(),
        algorithm_step: None,
        local_vars: Vec::new(),
        call_stack: Vec::new(),
        vis_events: Vec::new(),
        heatmap_line: 3,
        heatmap_count: 1,
        accessed_vars: Vec::new(),
        array_snapshots: Vec::new(),
        pointer_snapshots: Vec::new(),
        root_cause_hint: None,
    };
    let v = serde_json::to_value(&payload).unwrap();
    assert_eq!(keys_of(&v).len(), TOP_LEVEL_FIELDS.len());
}
