use crate::session::Session;
use crate::unified::types::{ApiFrameInfo, ApiVariableSnapshot, StepPayload};
use crate::vm::vm::CideVM;

/// 每步数据收集器：从 VM 和 Session 中提取轻量 `StepPayload`。
pub struct StepCollector;

impl StepCollector {
    pub fn collect(vm: &mut CideVM, session: &Session, step_index: i32) -> StepPayload {
        let code_line = vm.get_current_line();
        let func_name = vm
            .get_call_stack()
            .last()
            .map(|f| f.func_name.clone())
            .unwrap_or_default();

        let local_vars = vm
            .get_variable_snapshot()
            .into_iter()
            .map(|v| {
                let value_str = format_value(&v);
                ApiVariableSnapshot {
                    name: v.name,
                    addr: v.addr,
                    is_local: v.is_local,
                    ty_name: format!("{:?}", v.ty),
                    value: value_str,
                }
            })
            .collect();

        let call_stack = vm
            .get_call_stack()
            .iter()
            .map(|f| ApiFrameInfo {
                func_name: f.func_name.clone(),
                return_line: 0, // MVP 阶段简化
            })
            .collect();

        // 取出该步产生的可视化事件
        let vis_events = vm.take_vis_events();

        let heatmap_line = code_line;
        let heatmap_count = session
            .runtime
            .heatmap
            .line_counts
            .get(&code_line)
            .copied()
            .unwrap_or(0);

        let semantic_label = if code_line > 0 {
            format!("第 {} 行", code_line)
        } else {
            String::new()
        };

        StepPayload {
            step_index,
            code_line,
            func_name,
            semantic_label,
            local_vars,
            call_stack,
            vis_events,
            heatmap_line,
            heatmap_count,
        }
    }
}

fn format_value(v: &crate::session::VariableSnapshot) -> String {
    use crate::compiler::ast::TypeKind;
    match v.ty.kind() {
        TypeKind::Double => {
            let bits = v.value as u64;
            let f = f64::from_bits(bits);
            format!("{:.15}", f)
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        }
        TypeKind::Float => {
            let bits = v.value as u32;
            let f = f32::from_bits(bits);
            format!("{:.7}", f)
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        }
        _ => v.value.to_string(),
    }
}
