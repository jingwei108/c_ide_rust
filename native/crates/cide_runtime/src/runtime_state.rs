use std::collections::HashMap;
use std::ffi::CString;

use cide_ast::Type;

/// 执行轨迹条目基础数据：`cide_native` 会定义带 `#[frb]` 的同名包装。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TraceEntryData {
    pub line: i32,
    pub operation: String,
}

/// 可视化事件基础数据：`cide_native` 会定义带 `#[frb]` 的同名包装。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VisEventData {
    pub ty: i32,
    pub line: i32,
    pub extra0: i32,
    pub extra1: i32,
    pub extra2: i32,
    pub context: String,
}

/// 变量快照基础数据：`cide_native` 会定义带 `#[frb]` 的同名包装。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VariableSnapshotData {
    pub name: String,
    pub addr: u32,
    pub is_local: bool,
    pub ty: Type,
    pub value: i64,
}

/// 执行路径热力图：记录每行源代码被执行的次数。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ExecutionHeatmap {
    pub line_counts: HashMap<i32, u64>,
}

impl ExecutionHeatmap {
    pub fn record(&mut self, line: i32) {
        if line > 0 {
            *self.line_counts.entry(line).or_insert(0) += 1;
        }
    }

    pub fn max_count(&self) -> u64 {
        self.line_counts.values().copied().max().unwrap_or(0)
    }

    pub fn clear(&mut self) {
        self.line_counts.clear();
    }
}

/// 输入模式：交互式或批量。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum InputMode {
    #[default]
    Interactive,
    Batch,
}

/// 运行时状态：记录 VM 执行过程中的输出、trace、输入、变量快照等。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RuntimeState {
    pub error: String,
    /// 最近一次 `cide_get_runtime_error` 返回的 C 字符串缓存，避免返回 `String` 内部指针导致悬垂。
    pub last_error_cstring: Option<CString>,
    pub error_buffer: String,
    pub output_lines: Vec<String>,
    pub running: bool,
    pub trace: Vec<TraceEntryData>,
    pub current_line: i32,
    pub input_lines: Vec<String>,
    pub input_index: usize,
    pub step_mode: bool,
    pub step_count: i32,
    pub variable_snapshot: Vec<VariableSnapshotData>,
    pub vis_event_cache: Vec<VisEventData>,
    pub rand_seed: u32,
    pub input_char_offset: usize,
    pub waiting_input: bool,
    pub heatmap: ExecutionHeatmap,
    pub input_mode: InputMode,
    pub ungetc_char: Option<i32>,
    /// 命令行参数个数（供 `main(int argc, char *argv[])` 使用）。
    pub argc: i32,
    /// 命令行参数字符串数组（供 `main(int argc, char *argv[])` 使用）。
    pub argv: Vec<String>,
}

impl RuntimeState {
    /// 拼接所有输出片段为完整输出字符串。
    /// output_lines 中每个元素是独立的输出单元（如一次 printf/putchar 调用产生的字节），
    /// 不再自动添加换行符；换行由格式字符串或 puts 等函数显式提供，与 C 标准一致。
    pub fn output(&self) -> String {
        self.output_lines.join("")
    }
}
