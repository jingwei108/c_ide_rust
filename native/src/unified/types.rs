use flutter_rust_bridge::frb;
use crate::unified::root_cause::RootCauseHint;

/// 算法步骤语义快照（用于前端步骤标注）。
#[frb]
#[derive(Debug, Clone)]
pub struct AlgorithmStepSnapshot {
    pub algorithm_name: String,
    pub display_name: String,
    pub phase: String,
    pub description: String,
}

/// 每步的轻量数据包，传输到 Flutter 前端作为 FrameCache。
#[frb]
#[derive(Debug, Clone)]
pub struct StepPayload {
    pub step_index: i32,
    pub code_line: i32,
    pub func_name: String,
    pub semantic_label: String,
    pub algorithm_step: Option<AlgorithmStepSnapshot>,
    pub local_vars: Vec<ApiVariableSnapshot>,
    pub call_stack: Vec<ApiFrameInfo>,
    pub vis_events: Vec<crate::session::VisEvent>,
    pub heatmap_line: i32,
    pub heatmap_count: u64,
    pub accessed_vars: Vec<AccessedVar>,
    pub array_snapshots: Vec<ArraySnapshot>,
    pub pointer_snapshots: Vec<PointerSnapshot>,
    pub root_cause_hint: Option<RootCauseHint>,
}

/// 指针变量快照（用于指针追踪动画）。
#[frb]
#[derive(Debug, Clone)]
pub struct PointerSnapshot {
    pub name: String,
    pub addr: u32,
    pub ty_name: String,
    pub target_addr: u32,
    pub target_name: String,
    pub status: PointerStatus,
}

/// 指针状态。
#[frb]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerStatus {
    /// 指向有效内存（栈、全局、已分配堆）。
    Valid,
    /// 指向已释放的堆内存。
    Freed,
    /// NULL 指针。
    Null,
    /// 悬空指针（越界或未初始化）。
    Dangling,
}

/// 当前步访问的变量（用于变量级高亮）。
#[frb]
#[derive(Debug, Clone)]
pub struct AccessedVar {
    pub name: String,
    pub access_type: String, // "Read" | "Write"
}

/// FRB 友好的变量快照（ty 已格式化为字符串）。
#[frb]
#[derive(Debug, Clone)]
pub struct ApiVariableSnapshot {
    pub name: String,
    pub addr: u32,
    pub is_local: bool,
    pub ty_name: String,
    pub value: String,
}

/// 数组变量快照（用于算法可视化条形图）。
#[frb]
#[derive(Debug, Clone)]
pub struct ArraySnapshot {
    pub name: String,
    pub element_ty: String,
    pub elements: Vec<String>,
}

/// FRB 友好的调用帧信息。
#[frb]
#[derive(Debug, Clone)]
pub struct ApiFrameInfo {
    pub func_name: String,
    pub return_line: i32,
}

/// 语义元数据（用于进度条标签和智能检查点）。
#[frb]
#[derive(Debug, Clone)]
pub struct StepMeta {
    pub code_line: i32,
    pub func_name: String,
    pub loop_depth: i32,
    pub semantic_label: String,
}

/// 调试摘要（悬浮球零延迟）。
#[frb]
#[derive(Debug, Clone)]
pub struct DebugSummary {
    pub local_vars: Vec<ApiVariableSnapshot>,
    pub call_stack: Vec<ApiFrameInfo>,
    pub output_len: i32,
}

/// 执行热力图增量。
#[frb]
#[derive(Debug, Clone)]
pub struct HeatmapDelta {
    pub line: i32,
    pub count: u64,
}

/// 编译并启动统一模式的返回结果。
#[frb]
#[derive(Debug, Clone)]
pub struct UnifiedRunResult {
    pub success: bool,
    pub error: Option<String>,
    pub total_steps: i32,
    pub finished: bool,
}

/// 批量自动执行的返回结果。
#[frb]
#[derive(Debug, Clone)]
pub struct AutoStepResult {
    pub payloads: Vec<StepPayload>,
    pub finished: bool,
    pub trapped: bool,
    pub waiting_input: bool,
    pub paused: bool,
    pub current_line: i32,
    pub trap_message: Option<String>,
}

/// Seek 到指定步的返回结果。
#[frb]
#[derive(Debug, Clone)]
pub struct SeekResult {
    pub success: bool,
    pub payload: Option<StepPayload>,
    pub error: Option<String>,
}

/// 执行热力图数据。
#[frb]
#[derive(Debug, Clone)]
pub struct HeatmapData {
    pub line_counts: Vec<(i32, u64)>,
    pub max_count: u64,
}
