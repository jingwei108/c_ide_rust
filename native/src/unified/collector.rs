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

        let local_vars: Vec<ApiVariableSnapshot> = vm
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

        let semantic_label = infer_semantic_label(code_line, &local_vars, session);

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

/// 推断语义标签：根据源码行内容 + 局部变量值生成教学友好的描述。
fn infer_semantic_label(
    code_line: i32,
    local_vars: &[ApiVariableSnapshot],
    session: &Session,
) -> String {
    if code_line <= 0 {
        return String::new();
    }

    // 获取当前源码行
    let source_line = session
        .compile
        .compile_units
        .first()
        .and_then(|u| {
            u.source
                .lines()
                .nth((code_line - 1) as usize)
                .map(|s| s.trim())
        })
        .unwrap_or("");

    // 提取循环变量（i, j, k, idx, index, m, n）
    let loop_vars: Vec<(String, i32)> = local_vars
        .iter()
        .filter_map(|v| {
            if matches!(v.name.as_str(), "i" | "j" | "k" | "idx" | "index" | "m" | "n" | "left" | "right" | "mid" | "low" | "high") {
                v.value.parse::<i32>().ok().map(|val| (v.name.clone(), val))
            } else {
                None
            }
        })
        .collect();

    let loop_depth = loop_vars.len() as i32;

    // 检测交换模式：包含 temp + arr[ / a[ + 赋值
    let is_swap = source_line.contains("temp")
        && (source_line.contains("arr[") || source_line.contains("a["))
        && source_line.contains("=");

    // 检测递归调用：函数调用自身
    let _is_recursive_call = {
        let _func_name = if let Some(_frame) = local_vars.first() {
            // 无法直接从 local_vars 获取函数名，简化处理
            ""
        } else {
            ""
        };
        !_func_name.is_empty() && source_line.contains(_func_name)
    };

    // 生成语义标签
    if is_swap && loop_depth >= 1 {
        let i_val = loop_vars.first().map(|(_, v)| *v).unwrap_or(0);
        format!("交换 arr[{}]↔arr[{}]", i_val, i_val + 1)
    } else if loop_depth >= 1 {
        let iter_str = loop_vars
            .iter()
            .map(|(name, val)| format!("{}={}", name, val))
            .collect::<Vec<_>>()
            .join(", ");
        format!("循环 {}", iter_str)
    } else if source_line.starts_with("printf") || source_line.starts_with("scanf") {
        let func = if source_line.starts_with("printf") {
            "printf"
        } else {
            "scanf"
        };
        format!("调用 {}", func)
    } else if source_line.starts_with("return") {
        format!("返回")
    } else if source_line.contains("malloc") || source_line.contains("calloc") {
        format!("内存分配")
    } else if source_line.contains("free(") {
        format!("释放内存")
    } else {
        format!("第 {} 行", code_line)
    }
}
