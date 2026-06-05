use std::collections::HashMap;

use flutter_rust_bridge::frb;

use crate::session::VisEvent;
use crate::unified::root_cause::RootCauseHint;
use crate::unified::types::{
    AlgorithmStepSnapshot, ApiFrameInfo, ApiVariableSnapshot, ArraySnapshot, PointerSnapshot,
    PointerStatus, StepPayload,
};

/// 符号表索引。
pub type SymIdx = i32;

/// 使用符号表的变量快照（不变数据 dedup）。
#[frb]
#[derive(Debug, Clone)]
pub struct ApiVarSnapshotRef {
    pub name_idx: SymIdx,
    pub addr: u32,
    pub is_local: bool,
    pub ty_name_idx: SymIdx,
    pub value: String,
}

/// 差分变量：仅包含值发生变化的变量。
#[frb]
#[derive(Debug, Clone)]
pub struct VarDelta {
    pub name_idx: SymIdx,
    pub value: String,
}

/// 使用符号表的指针快照。
#[frb]
#[derive(Debug, Clone)]
pub struct PointerSnapshotRef {
    pub name_idx: SymIdx,
    pub addr: u32,
    pub ty_name_idx: SymIdx,
    pub target_addr: u32,
    pub target_name_idx: SymIdx,
    pub status: PointerStatus,
}

/// 使用符号表的数组快照。
#[frb]
#[derive(Debug, Clone)]
pub struct ArraySnapshotRef {
    pub name_idx: SymIdx,
    pub element_ty_idx: SymIdx,
    pub elements: Vec<String>,
}

/// 使用符号表的访问变量。
#[frb]
#[derive(Debug, Clone)]
pub struct AccessedVarRef {
    pub name_idx: SymIdx,
    pub access_type_idx: SymIdx,
}

/// 使用符号表的调用帧。
#[frb]
#[derive(Debug, Clone)]
pub struct ApiFrameInfoRef {
    pub func_name_idx: SymIdx,
    pub return_line: i32,
}

/// 使用符号表的算法步骤。
#[frb]
#[derive(Debug, Clone)]
pub struct AlgorithmStepSnapshotRef {
    pub algorithm_name_idx: SymIdx,
    pub display_name_idx: SymIdx,
    pub phase_idx: SymIdx,
    pub description_idx: SymIdx,
}

/// 使用符号表的 StepPayload（基准快照）。
#[frb]
#[derive(Debug, Clone)]
pub struct StepPayloadRef {
    pub step_index: i32,
    pub code_line: i32,
    pub func_name_idx: SymIdx,
    pub semantic_label_idx: SymIdx,
    pub algorithm_step: Option<AlgorithmStepSnapshotRef>,
    pub local_vars: Vec<ApiVarSnapshotRef>,
    pub call_stack: Vec<ApiFrameInfoRef>,
    pub vis_events: Vec<VisEvent>,
    pub heatmap_line: i32,
    pub heatmap_count: u64,
    pub accessed_vars: Vec<AccessedVarRef>,
    pub array_snapshots: Vec<ArraySnapshotRef>,
    pub pointer_snapshots: Vec<PointerSnapshotRef>,
    pub root_cause_hint: Option<RootCauseHint>,
}

/// 差分 StepPayload（基于前一个 StepPayloadRef 或 StepPayloadDelta）。
#[frb]
#[derive(Debug, Clone)]
pub struct StepPayloadDelta {
    pub step_index: i32,
    pub code_line: i32,
    pub func_name_idx: SymIdx,
    pub semantic_label_idx: SymIdx,
    pub algorithm_step: Option<AlgorithmStepSnapshotRef>,
    /// 值发生变化的变量。
    pub var_deltas: Vec<VarDelta>,
    /// 新出现的变量。
    pub new_vars: Vec<ApiVarSnapshotRef>,
    /// 消失的变量名索引。
    pub removed_var_name_indices: Vec<SymIdx>,
    pub call_stack: Vec<ApiFrameInfoRef>,
    pub vis_events: Vec<VisEvent>,
    pub heatmap_line: i32,
    pub heatmap_count: u64,
    pub accessed_vars: Vec<AccessedVarRef>,
    pub array_snapshots: Vec<ArraySnapshotRef>,
    pub pointer_snapshots: Vec<PointerSnapshotRef>,
    pub root_cause_hint: Option<RootCauseHint>,
}

/// Stream 批量传输单元。
///
/// 编码规则：
/// - `base_payloads` 包含每 batch 的第 1 个完整快照（step 0）。
/// - `deltas` 包含后续步的差分数据（基于前一步的局部变量状态）。
/// - `symbol_table` 全局去重字符串池。
#[frb]
#[derive(Debug, Clone)]
pub struct StepStreamBatch {
    pub symbol_table: Vec<String>,
    pub base_payloads: Vec<StepPayloadRef>,
    pub deltas: Vec<StepPayloadDelta>,
    pub finished: bool,
    pub trapped: bool,
    pub waiting_input: bool,
    pub paused: bool,
    pub current_line: i32,
    pub trap_message: Option<String>,
}

// ==================== 编码器 ====================

struct SymbolTable {
    symbols: Vec<String>,
    index: HashMap<String, SymIdx>,
}

impl SymbolTable {
    fn new() -> Self {
        let mut s = Self {
            symbols: Vec::new(),
            index: HashMap::new(),
        };
        // 索引 0 预留为空字符串
        s.insert(String::new());
        s
    }

    fn insert(&mut self, s: String) -> SymIdx {
        if let Some(&idx) = self.index.get(&s) {
            return idx;
        }
        let idx = self.symbols.len() as SymIdx;
        self.symbols.push(s.clone());
        self.index.insert(s, idx);
        idx
    }

    fn into_vec(self) -> Vec<String> {
        self.symbols
    }
}

/// 将一组 StepPayload 编码为优化的 StepStreamBatch。
pub fn encode_payloads(payloads: &[StepPayload]) -> StepStreamBatch {
    if payloads.is_empty() {
        return StepStreamBatch {
            symbol_table: vec![String::new()],
            base_payloads: Vec::new(),
            deltas: Vec::new(),
            finished: false,
            trapped: false,
            waiting_input: false,
            paused: false,
            current_line: 0,
            trap_message: None,
        };
    }

    let mut sym = SymbolTable::new();

    // 编码第 0 个 payload 为 base
    let base = encode_step_payload(&payloads[0], &mut sym);
    let mut deltas = Vec::new();

    let mut prev_vars: HashMap<SymIdx, String> = HashMap::new();
    for v in &base.local_vars {
        prev_vars.insert(v.name_idx, v.value.clone());
    }

    // 对后续 payload 做差分编码
    let mut current_full_ref = base.clone();
    for i in 1..payloads.len() {
        let delta = encode_step_delta(&payloads[i - 1], &payloads[i], &mut sym, &prev_vars);

        // 增量更新 prev_vars：应用当前 delta 到 current_full_ref
        for d in &delta.var_deltas {
            if let Some(v) = current_full_ref.local_vars.iter_mut().find(|v| v.name_idx == d.name_idx) {
                v.value = d.value.clone();
            }
        }
        for v in &delta.new_vars {
            current_full_ref.local_vars.push(v.clone());
        }
        current_full_ref.local_vars.retain(|v| !delta.removed_var_name_indices.contains(&v.name_idx));

        prev_vars.clear();
        for v in &current_full_ref.local_vars {
            prev_vars.insert(v.name_idx, v.value.clone());
        }

        deltas.push(delta);
    }

    StepStreamBatch {
        symbol_table: sym.into_vec(),
        base_payloads: vec![base],
        deltas,
        finished: false,
        trapped: false,
        waiting_input: false,
        paused: false,
        current_line: payloads.last().map(|p| p.code_line).unwrap_or(0),
        trap_message: None,
    }
}

fn encode_step_payload(payload: &StepPayload, sym: &mut SymbolTable) -> StepPayloadRef {
    StepPayloadRef {
        step_index: payload.step_index,
        code_line: payload.code_line,
        func_name_idx: sym.insert(payload.func_name.clone()),
        semantic_label_idx: sym.insert(payload.semantic_label.clone()),
        algorithm_step: payload.algorithm_step.as_ref().map(|a| AlgorithmStepSnapshotRef {
            algorithm_name_idx: sym.insert(a.algorithm_name.clone()),
            display_name_idx: sym.insert(a.display_name.clone()),
            phase_idx: sym.insert(a.phase.clone()),
            description_idx: sym.insert(a.description.clone()),
        }),
        local_vars: payload
            .local_vars
            .iter()
            .map(|v| ApiVarSnapshotRef {
                name_idx: sym.insert(v.name.clone()),
                addr: v.addr,
                is_local: v.is_local,
                ty_name_idx: sym.insert(v.ty_name.clone()),
                value: v.value.clone(),
            })
            .collect(),
        call_stack: payload
            .call_stack
            .iter()
            .map(|f| ApiFrameInfoRef {
                func_name_idx: sym.insert(f.func_name.clone()),
                return_line: f.return_line,
            })
            .collect(),
        vis_events: payload.vis_events.clone(),
        heatmap_line: payload.heatmap_line,
        heatmap_count: payload.heatmap_count,
        accessed_vars: payload
            .accessed_vars
            .iter()
            .map(|a| AccessedVarRef {
                name_idx: sym.insert(a.name.clone()),
                access_type_idx: sym.insert(a.access_type.clone()),
            })
            .collect(),
        array_snapshots: payload
            .array_snapshots
            .iter()
            .map(|a| ArraySnapshotRef {
                name_idx: sym.insert(a.name.clone()),
                element_ty_idx: sym.insert(a.element_ty.clone()),
                elements: a.elements.clone(),
            })
            .collect(),
        pointer_snapshots: payload
            .pointer_snapshots
            .iter()
            .map(|p| PointerSnapshotRef {
                name_idx: sym.insert(p.name.clone()),
                addr: p.addr,
                ty_name_idx: sym.insert(p.ty_name.clone()),
                target_addr: p.target_addr,
                target_name_idx: sym.insert(p.target_name.clone()),
                status: p.status,
            })
            .collect(),
        root_cause_hint: payload.root_cause_hint.clone(),
    }
}

fn encode_step_delta(
    prev: &StepPayload,
    curr: &StepPayload,
    sym: &mut SymbolTable,
    prev_vars: &HashMap<SymIdx, String>,
) -> StepPayloadDelta {
    let curr_vars: HashMap<String, &ApiVariableSnapshot> = curr
        .local_vars
        .iter()
        .map(|v| (v.name.clone(), v))
        .collect();
    let _prev_var_names: std::collections::HashSet<String> =
        prev.local_vars.iter().map(|v| v.name.clone()).collect();

    let mut var_deltas = Vec::new();
    let mut new_vars = Vec::new();
    let mut removed_var_name_indices = Vec::new();

    for v in &curr.local_vars {
        let name_idx = sym.insert(v.name.clone());
        if let Some(prev_val) = prev_vars.get(&name_idx) {
            if prev_val != &v.value {
                var_deltas.push(VarDelta {
                    name_idx,
                    value: v.value.clone(),
                });
            }
        } else {
            new_vars.push(ApiVarSnapshotRef {
                name_idx,
                addr: v.addr,
                is_local: v.is_local,
                ty_name_idx: sym.insert(v.ty_name.clone()),
                value: v.value.clone(),
            });
        }
    }

    for v in &prev.local_vars {
        let name_idx = sym.insert(v.name.clone());
        if !curr_vars.contains_key(&v.name) {
            removed_var_name_indices.push(name_idx);
        }
    }

    StepPayloadDelta {
        step_index: curr.step_index,
        code_line: curr.code_line,
        func_name_idx: sym.insert(curr.func_name.clone()),
        semantic_label_idx: sym.insert(curr.semantic_label.clone()),
        algorithm_step: curr.algorithm_step.as_ref().map(|a| AlgorithmStepSnapshotRef {
            algorithm_name_idx: sym.insert(a.algorithm_name.clone()),
            display_name_idx: sym.insert(a.display_name.clone()),
            phase_idx: sym.insert(a.phase.clone()),
            description_idx: sym.insert(a.description.clone()),
        }),
        var_deltas,
        new_vars,
        removed_var_name_indices,
        call_stack: curr
            .call_stack
            .iter()
            .map(|f| ApiFrameInfoRef {
                func_name_idx: sym.insert(f.func_name.clone()),
                return_line: f.return_line,
            })
            .collect(),
        vis_events: curr.vis_events.clone(),
        heatmap_line: curr.heatmap_line,
        heatmap_count: curr.heatmap_count,
        accessed_vars: curr
            .accessed_vars
            .iter()
            .map(|a| AccessedVarRef {
                name_idx: sym.insert(a.name.clone()),
                access_type_idx: sym.insert(a.access_type.clone()),
            })
            .collect(),
        array_snapshots: curr
            .array_snapshots
            .iter()
            .map(|a| ArraySnapshotRef {
                name_idx: sym.insert(a.name.clone()),
                element_ty_idx: sym.insert(a.element_ty.clone()),
                elements: a.elements.clone(),
            })
            .collect(),
        pointer_snapshots: curr
            .pointer_snapshots
            .iter()
            .map(|p| PointerSnapshotRef {
                name_idx: sym.insert(p.name.clone()),
                addr: p.addr,
                ty_name_idx: sym.insert(p.ty_name.clone()),
                target_addr: p.target_addr,
                target_name_idx: sym.insert(p.target_name.clone()),
                status: p.status,
            })
            .collect(),
        root_cause_hint: curr.root_cause_hint.clone(),
    }
}

// ==================== 解码器 ====================

/// 将 StepStreamBatch 解码为完整的 StepPayload 列表。
pub fn decode_batch(batch: &StepStreamBatch) -> Vec<StepPayload> {
    if batch.base_payloads.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    let base = &batch.base_payloads[0];
    result.push(decode_step_payload_ref(base, &batch.symbol_table));

    let mut current_vars: HashMap<SymIdx, ApiVariableSnapshot> = HashMap::new();
    for v in &base.local_vars {
        current_vars.insert(
            v.name_idx,
            ApiVariableSnapshot {
                name: get_sym(&batch.symbol_table, v.name_idx),
                addr: v.addr,
                is_local: v.is_local,
                ty_name: get_sym(&batch.symbol_table, v.ty_name_idx),
                value: v.value.clone(),
            },
        );
    }

    // 维护变量出现顺序（base 顺序 + 后续新增追加到末尾）
    let mut var_order: Vec<SymIdx> = base.local_vars.iter().map(|v| v.name_idx).collect();

    for delta in &batch.deltas {
        // 应用 var_deltas
        for d in &delta.var_deltas {
            if let Some(var) = current_vars.get_mut(&d.name_idx) {
                var.value = d.value.clone();
            }
        }
        // 添加新变量
        for v in &delta.new_vars {
            current_vars.insert(
                v.name_idx,
                ApiVariableSnapshot {
                    name: get_sym(&batch.symbol_table, v.name_idx),
                    addr: v.addr,
                    is_local: v.is_local,
                    ty_name: get_sym(&batch.symbol_table, v.ty_name_idx),
                    value: v.value.clone(),
                },
            );
        }
        // 移除消失的变量
        for &idx in &delta.removed_var_name_indices {
            current_vars.remove(&idx);
        }

        // 更新 var_order：移除已消失的，追加新出现的
        var_order.retain(|idx| !delta.removed_var_name_indices.contains(idx));
        for v in &delta.new_vars {
            if !var_order.contains(&v.name_idx) {
                var_order.push(v.name_idx);
            }
        }

        // 按 var_order 构建 local_vars
        let local_vars: Vec<ApiVariableSnapshot> = var_order
            .iter()
            .filter_map(|idx| current_vars.get(idx).cloned())
            .collect();

        result.push(StepPayload {
            step_index: delta.step_index,
            code_line: delta.code_line,
            func_name: batch.symbol_table[delta.func_name_idx as usize].clone(),
            semantic_label: batch.symbol_table[delta.semantic_label_idx as usize].clone(),
            algorithm_step: delta.algorithm_step.as_ref().map(|a| AlgorithmStepSnapshot {
                algorithm_name: batch.symbol_table[a.algorithm_name_idx as usize].clone(),
                display_name: batch.symbol_table[a.display_name_idx as usize].clone(),
                phase: batch.symbol_table[a.phase_idx as usize].clone(),
                description: batch.symbol_table[a.description_idx as usize].clone(),
            }),
            local_vars,
            call_stack: delta
                .call_stack
                .iter()
                .map(|f| ApiFrameInfo {
                    func_name: batch.symbol_table[f.func_name_idx as usize].clone(),
                    return_line: f.return_line,
                })
                .collect(),
            vis_events: delta.vis_events.clone(),
            heatmap_line: delta.heatmap_line,
            heatmap_count: delta.heatmap_count,
            accessed_vars: delta
                .accessed_vars
                .iter()
                .map(|a| crate::unified::types::AccessedVar {
                    name: batch.symbol_table[a.name_idx as usize].clone(),
                    access_type: batch.symbol_table[a.access_type_idx as usize].clone(),
                })
                .collect(),
            array_snapshots: delta
                .array_snapshots
                .iter()
                .map(|a| ArraySnapshot {
                    name: batch.symbol_table[a.name_idx as usize].clone(),
                    element_ty: batch.symbol_table[a.element_ty_idx as usize].clone(),
                    elements: a.elements.clone(),
                })
                .collect(),
            pointer_snapshots: delta
                .pointer_snapshots
                .iter()
                .map(|p| PointerSnapshot {
                    name: batch.symbol_table[p.name_idx as usize].clone(),
                    addr: p.addr,
                    ty_name: batch.symbol_table[p.ty_name_idx as usize].clone(),
                    target_addr: p.target_addr,
                    target_name: batch.symbol_table[p.target_name_idx as usize].clone(),
                    status: p.status,
                })
                .collect(),
            root_cause_hint: delta.root_cause_hint.clone(),
        });
    }

    result
}

fn decode_step_payload_ref(base: &StepPayloadRef, sym: &[String]) -> StepPayload {
    StepPayload {
        step_index: base.step_index,
        code_line: base.code_line,
        func_name: get_sym(sym, base.func_name_idx),
        semantic_label: get_sym(sym, base.semantic_label_idx),
        algorithm_step: base.algorithm_step.as_ref().map(|a| AlgorithmStepSnapshot {
            algorithm_name: get_sym(sym, a.algorithm_name_idx),
            display_name: get_sym(sym, a.display_name_idx),
            phase: get_sym(sym, a.phase_idx),
            description: get_sym(sym, a.description_idx),
        }),
        local_vars: base
            .local_vars
            .iter()
            .map(|v| ApiVariableSnapshot {
                name: get_sym(sym, v.name_idx),
                addr: v.addr,
                is_local: v.is_local,
                ty_name: get_sym(sym, v.ty_name_idx),
                value: v.value.clone(),
            })
            .collect(),
        call_stack: base
            .call_stack
            .iter()
            .map(|f| ApiFrameInfo {
                func_name: get_sym(sym, f.func_name_idx),
                return_line: f.return_line,
            })
            .collect(),
        vis_events: base.vis_events.clone(),
        heatmap_line: base.heatmap_line,
        heatmap_count: base.heatmap_count,
        accessed_vars: base
            .accessed_vars
            .iter()
            .map(|a| crate::unified::types::AccessedVar {
                name: get_sym(sym, a.name_idx),
                access_type: get_sym(sym, a.access_type_idx),
            })
            .collect(),
        array_snapshots: base
            .array_snapshots
            .iter()
            .map(|a| ArraySnapshot {
                name: get_sym(sym, a.name_idx),
                element_ty: get_sym(sym, a.element_ty_idx),
                elements: a.elements.clone(),
            })
            .collect(),
        pointer_snapshots: base
            .pointer_snapshots
            .iter()
            .map(|p| PointerSnapshot {
                name: get_sym(sym, p.name_idx),
                addr: p.addr,
                ty_name: get_sym(sym, p.ty_name_idx),
                target_addr: p.target_addr,
                target_name: get_sym(sym, p.target_name_idx),
                status: p.status,
            })
            .collect(),
        root_cause_hint: base.root_cause_hint.clone(),
    }
}

#[inline]
fn get_sym(sym: &[String], idx: SymIdx) -> String {
    sym.get(idx as usize).cloned().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_payload(step: i32, vars: Vec<ApiVariableSnapshot>) -> StepPayload {
        StepPayload {
            step_index: step,
            code_line: step + 1,
            func_name: "main".to_string(),
            semantic_label: format!("step {}", step),
            algorithm_step: None,
            local_vars: vars,
            call_stack: vec![ApiFrameInfo { func_name: "main".to_string(), return_line: 0 }],
            vis_events: Vec::new(),
            heatmap_line: step + 1,
            heatmap_count: 1,
            accessed_vars: Vec::new(),
            array_snapshots: Vec::new(),
            pointer_snapshots: Vec::new(),
            root_cause_hint: None,
        }
    }

    fn var(name: &str, ty: &str, value: &str) -> ApiVariableSnapshot {
        ApiVariableSnapshot {
            name: name.to_string(),
            addr: 0,
            is_local: true,
            ty_name: ty.to_string(),
            value: value.to_string(),
        }
    }

    #[test]
    fn test_empty_payloads() {
        let batch = encode_payloads(&[]);
        assert!(batch.base_payloads.is_empty());
        assert!(batch.deltas.is_empty());
        assert_eq!(batch.symbol_table.len(), 1); // 空字符串
    }

    #[test]
    fn test_single_payload_roundtrip() {
        let payloads = vec![make_payload(0, vec![var("i", "int", "0"), var("n", "int", "5")])];
        let batch = encode_payloads(&payloads);
        let decoded = decode_batch(&batch);
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].step_index, 0);
        assert_eq!(decoded[0].local_vars.len(), 2);
        assert_eq!(decoded[0].local_vars[0].name, "i");
        assert_eq!(decoded[0].local_vars[0].value, "0");
    }

    #[test]
    fn test_delta_encoding_roundtrip() {
        let payloads = vec![
            make_payload(0, vec![var("i", "int", "0"), var("n", "int", "5")]),
            make_payload(1, vec![var("i", "int", "1"), var("n", "int", "5")]),
            make_payload(2, vec![var("i", "int", "2"), var("n", "int", "5")]),
        ];
        let batch = encode_payloads(&payloads);
        assert_eq!(batch.base_payloads.len(), 1);
        assert_eq!(batch.deltas.len(), 2);

        // 验证差分：只有 i 变化，n 不变
        assert_eq!(batch.deltas[0].var_deltas.len(), 1);
        assert_eq!(batch.deltas[0].var_deltas[0].value, "1");
        assert!(batch.deltas[0].new_vars.is_empty());
        assert!(batch.deltas[0].removed_var_name_indices.is_empty());

        let decoded = decode_batch(&batch);
        assert_eq!(decoded.len(), 3);
        assert_eq!(decoded[1].local_vars[0].value, "1");
        assert_eq!(decoded[2].local_vars[0].value, "2");
        assert_eq!(decoded[2].local_vars[1].value, "5"); // n 未变
    }

    #[test]
    fn test_variable_add_remove_roundtrip() {
        let payloads = vec![
            make_payload(0, vec![var("i", "int", "0")]),
            make_payload(1, vec![var("i", "int", "1"), var("j", "int", "10")]),
            make_payload(2, vec![var("j", "int", "11")]),
        ];
        let batch = encode_payloads(&payloads);
        assert_eq!(batch.deltas[0].new_vars.len(), 1); // j 新增
        assert_eq!(batch.deltas[1].removed_var_name_indices.len(), 1); // i 移除
        assert_eq!(batch.deltas[1].var_deltas.len(), 1); // j 变化

        let decoded = decode_batch(&batch);
        assert_eq!(decoded.len(), 3);
        assert_eq!(decoded[0].local_vars.len(), 1);
        assert_eq!(decoded[1].local_vars.len(), 2);
        assert_eq!(decoded[2].local_vars.len(), 1);
        assert_eq!(decoded[2].local_vars[0].name, "j");
        assert_eq!(decoded[2].local_vars[0].value, "11");
    }

    #[test]
    fn test_symbol_table_dedup() {
        let payloads = vec![
            make_payload(0, vec![var("i", "int", "0")]),
            make_payload(1, vec![var("i", "int", "1")]),
        ];
        let batch = encode_payloads(&payloads);
        // func_name "main", semantic_label "step 0"/"step 1", name "i", ty "int", value "0"/"1"
        // 只有 "main" 和 "i" 和 "int" 是重复的，应该被去重
        let sym_set: std::collections::HashSet<&String> = batch.symbol_table.iter().collect();
        assert_eq!(sym_set.len(), batch.symbol_table.len()); // 无重复
    }
}


