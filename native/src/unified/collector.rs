use crate::session::Session;
use crate::unified::algorithm_steps::infer_algorithm_step;
use crate::unified::types::{
    AccessedVar, ApiFrameInfo, ApiVariableSnapshot, PointerSnapshot, PointerStatus, StepPayload,
};
use crate::vm::core::CideVM;

/// 每步数据收集器：从 VM 和 Session 中提取轻量 `StepPayload`。
pub struct StepCollector;

impl StepCollector {
    pub fn collect(vm: &mut CideVM, session: &Session, step_index: i32) -> StepPayload {
        let code_line = vm.get_current_line();
        let func_name = vm.get_call_stack().last().map(|f| f.func_name.clone()).unwrap_or_default();

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
        let heatmap_count = session.runtime.heatmap.line_counts.get(&code_line).copied().unwrap_or(0);

        let semantic_label = infer_semantic_label(code_line, &local_vars, &func_name, session);
        let algorithm_step = infer_algorithm_step(code_line, &local_vars, &func_name, session);

        let accessed_vars = vm
            .get_last_accessed_vars()
            .iter()
            .map(|a| AccessedVar {
                name: a.name.clone(),
                access_type: match a.access_type {
                    crate::vm::core::AccessType::Read => "Read".to_string(),
                    crate::vm::core::AccessType::Write => "Write".to_string(),
                },
            })
            .collect();

        let array_snapshots = vm.get_array_snapshots();
        let pointer_snapshots = collect_pointer_snapshots(vm, session, &local_vars);

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
            accessed_vars,
            array_snapshots,
            pointer_snapshots,
            algorithm_step,
            root_cause_hint: None,
        }
    }
}

/// 从变量快照中提取指针变量，并判断其状态。
fn collect_pointer_snapshots(
    _vm: &CideVM,
    session: &Session,
    local_vars: &[ApiVariableSnapshot],
) -> Vec<PointerSnapshot> {
    let mut result = Vec::new();
    for v in local_vars {
        if !is_pointer_type(&v.ty_name) {
            continue;
        }
        let target_addr = match parse_addr(&v.value) {
            Some(a) => a,
            None => continue,
        };

        let status = if target_addr == 0 {
            PointerStatus::Null
        } else if !(crate::vm::core::NULL_TRAP_SIZE..crate::vm::core::MEM_SIZE).contains(&target_addr) {
            PointerStatus::Dangling
        } else if is_freed_heap(&session.memory.regions, target_addr) {
            PointerStatus::Freed
        } else {
            PointerStatus::Valid
        };

        let target_name = find_var_name_at_addr(local_vars, target_addr);

        result.push(PointerSnapshot {
            name: v.name.clone(),
            addr: v.addr,
            ty_name: v.ty_name.clone(),
            target_addr,
            target_name,
            status,
        });
    }
    result
}

fn is_pointer_type(ty_name: &str) -> bool {
    ty_name.contains('*') || ty_name.contains("Pointer")
}

fn parse_addr(value: &str) -> Option<u32> {
    // 支持 "0x1234" 和十进制 "4660"
    if value.starts_with("0x") || value.starts_with("0X") {
        u32::from_str_radix(&value[2..], 16).ok()
    } else {
        value
            .parse::<i64>()
            .ok()
            .and_then(|n| if n >= 0 { Some(n as u32) } else { None })
    }
}

fn is_freed_heap(regions: &[crate::session::MemoryRegion], addr: u32) -> bool {
    regions.iter().any(|r| r.is_heap && r.addr == addr && r.is_freed)
}

fn find_var_name_at_addr(local_vars: &[ApiVariableSnapshot], addr: u32) -> String {
    for v in local_vars {
        if v.addr == addr {
            return v.name.clone();
        }
    }
    String::new()
}

fn format_value(v: &crate::session::VariableSnapshot) -> String {
    use crate::compiler::ast::TypeKind;
    match v.ty.kind() {
        TypeKind::Double => {
            let bits = v.value as u64;
            let f = f64::from_bits(bits);
            format!("{:.15}", f).trim_end_matches('0').trim_end_matches('.').to_string()
        }
        TypeKind::Float => {
            let bits = v.value as u32;
            let f = f32::from_bits(bits);
            format!("{:.7}", f).trim_end_matches('0').trim_end_matches('.').to_string()
        }
        _ => v.value.to_string(),
    }
}

/// 推断语义标签：根据源码行内容 + 局部变量值生成教学友好的描述。
fn infer_semantic_label(
    code_line: i32,
    local_vars: &[ApiVariableSnapshot],
    func_name: &str,
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
        .and_then(|u| u.source.lines().nth((code_line - 1) as usize).map(|s| s.trim()))
        .unwrap_or("");

    // 提取循环变量（i, j, k, idx, index, m, n, left, right, mid, low, high, pivot）
    let loop_vars: Vec<(String, i32)> = local_vars
        .iter()
        .filter_map(|v| {
            if matches!(
                v.name.as_str(),
                "i" | "j"
                    | "k"
                    | "idx"
                    | "index"
                    | "m"
                    | "n"
                    | "left"
                    | "right"
                    | "mid"
                    | "low"
                    | "high"
                    | "pivot"
                    | "gap"
            ) {
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

    // 检测递归调用
    let is_recursive = !func_name.is_empty() && func_name != "main" && source_line.contains(&format!("{}(", func_name));

    // 检测普通函数调用（排除控制流关键字）
    let is_func_call = source_line.contains('(')
        && !source_line.starts_with("if ")
        && !source_line.starts_with("while ")
        && !source_line.starts_with("for ")
        && !source_line.starts_with("switch ")
        && !source_line.starts_with("return ")
        && !source_line.starts_with("//")
        && !source_line.starts_with("/*");

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
    } else if is_recursive {
        format!("递归调用 {}", func_name)
    } else if source_line.starts_with("printf") || source_line.starts_with("scanf") {
        let func = if source_line.starts_with("printf") {
            "printf"
        } else {
            "scanf"
        };
        format!("调用 {}", func)
    } else if source_line.starts_with("return") {
        "返回".to_string()
    } else if source_line.contains("malloc") || source_line.contains("calloc") {
        "内存分配".to_string()
    } else if source_line.contains("free(") {
        "释放内存".to_string()
    } else if source_line.contains("getchar") || source_line.contains("putchar") {
        "IO 操作".to_string()
    } else if source_line.contains("qsort(") {
        "调用 qsort".to_string()
    } else if is_func_call {
        // 尝试提取函数名
        if let Some(func) = extract_called_func(source_line) {
            format!("调用 {}", func)
        } else {
            "函数调用".to_string()
        }
    } else {
        format!("第 {} 行", code_line)
    }
}

/// 从函数调用语句中提取函数名。
fn extract_called_func(line: &str) -> Option<String> {
    let trimmed = line.trim();
    // 跳过赋值部分："x = foo(...)" → "foo(...)"
    let after_assign = if let Some(pos) = trimmed.find("=") {
        trimmed[pos + 1..].trim()
    } else {
        trimmed
    };
    // 提取函数名："foo(bar, baz)" → "foo"
    if let Some(paren_pos) = after_assign.find('(') {
        let name = after_assign[..paren_pos].trim();
        if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Some(name.to_string());
        }
    }
    None
}
