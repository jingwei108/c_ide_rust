use crate::session::Session;
use crate::unified::checkpoint::CheckpointManager;
use crate::unified::collector::StepCollector;
use crate::unified::types::{AutoStepResult, SeekResult, StepMeta, StepPayload};
use crate::vm::vm::{CideVM, StepResult};

/// 统一模式引擎：整合检查点管理、数据收集和批量执行。
///
/// 不直接持有 VM 或 Session，而是在方法调用时通过参数传入，
/// 以便与 `flutter_bridge.rs` 的 Session 管理模式兼容。
pub struct UnifiedEngine {
    pub checkpoints: CheckpointManager,
    pub frame_cache: Vec<StepPayload>,
    pub max_steps: i32,
    pub is_paused: bool,
    pub is_cancelled: bool,
}

impl Default for UnifiedEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl UnifiedEngine {
    pub fn new() -> Self {
        Self {
            checkpoints: CheckpointManager::new(20),
            frame_cache: Vec::new(),
            max_steps: 100_000,
            is_paused: false,
            is_cancelled: false,
        }
    }

    pub fn reset(&mut self) {
        self.checkpoints.clear();
        self.frame_cache.clear();
        self.is_paused = false;
        self.is_cancelled = false;
    }

    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    pub fn resume(&mut self) {
        self.is_paused = false;
    }

    pub fn cancel(&mut self) {
        self.is_cancelled = true;
    }

    /// 批量自动执行，返回收集到的 StepPayload 列表。
    ///
    /// 调用者应确保 VM 已初始化（`setup_vm` 已调用）。
    pub fn run_batch(
        &mut self,
        vm: &mut CideVM,
        session: &mut Session,
        batch_size: i32,
    ) -> Result<AutoStepResult, String> {
        let mut payloads = Vec::new();
        let mut finished = false;
        let mut trapped = false;
        let mut waiting_input = false;

        let mut trap_message: Option<String> = None;

        for _ in 0..batch_size {
            if self.is_paused || self.is_cancelled {
                break;
            }

            let step = vm.get_executed_steps();

            // 检查点保存
            let meta = StepMeta {
                code_line: vm.get_current_line(),
                func_name: vm
                    .get_call_stack()
                    .last()
                    .map(|f| f.func_name.clone())
                    .unwrap_or_default(),
                loop_depth: 0,
                semantic_label: String::new(),
            };
            if self.checkpoints.should_checkpoint(step, &meta) {
                self.checkpoints.save(step, vm, session);
            }

            // 执行前快照：用于 Trap 时自动回退
            let pre_step_snap = vm.snapshot(session);

            // 执行一步
            match vm.step(session) {
                StepResult::Ok => {
                    let payload = StepCollector::collect(vm, session, step);
                    payloads.push(payload);
                }
                StepResult::Paused => {
                    let payload = StepCollector::collect(vm, session, step);
                    payloads.push(payload);
                    self.is_paused = true;
                }
                StepResult::WaitingInput => {
                    let payload = StepCollector::collect(vm, session, step);
                    payloads.push(payload);
                    waiting_input = true;
                    break;
                }
                StepResult::Finished => {
                    let payload = StepCollector::collect(vm, session, step);
                    payloads.push(payload);
                    finished = true;
                    break;
                }
                StepResult::Trap => {
                    // 自动回退到上一步状态
                    vm.restore(&pre_step_snap, session);
                    let payload = StepCollector::collect(vm, session, step);
                    payloads.push(payload);
                    trapped = true;
                    trap_message = Some(vm.get_error().to_string());
                    break;
                }
            }

            if step >= self.max_steps {
                return Err(
                    "执行步数超过限制（10,000 步），可能存在无限循环。".to_string()
                );
            }
        }

        self.frame_cache.extend(payloads.clone());

        Ok(AutoStepResult {
            payloads,
            finished,
            trapped,
            waiting_input,
            paused: self.is_paused,
            current_line: vm.get_current_line(),
            trap_message,
        })
    }

    /// Seek 到指定步。
    ///
    /// 如果目标步已在 `frame_cache` 中，直接返回；
    /// 否则从最近检查点恢复 VM 并正向重放。
    pub fn seek_to(
        &mut self,
        target: i32,
        vm: &mut CideVM,
        session: &mut Session,
    ) -> SeekResult {
        // 目标已在缓存中
        if let Some(payload) = self.frame_cache.get(target as usize) {
            return SeekResult {
                success: true,
                payload: Some(payload.clone()),
                error: None,
            };
        }

        // 找到最近检查点
        let (checkpoint_step, snap) = match self.checkpoints.nearest(target) {
            Some(v) => v,
            None => {
                return SeekResult {
                    success: false,
                    payload: None,
                    error: Some("没有可用的检查点".to_string()),
                };
            }
        };

        // 恢复 VM 状态
        vm.restore(snap, session);

        // 正向重放到目标步
        for step in checkpoint_step..target {
            if self.is_cancelled {
                return SeekResult {
                    success: false,
                    payload: None,
                    error: Some("执行已取消".to_string()),
                };
            }
            match vm.step(session) {
                StepResult::Ok | StepResult::Paused => {
                    let payload = StepCollector::collect(vm, session, step);
                    if step as usize == self.frame_cache.len() {
                        self.frame_cache.push(payload);
                    }
                }
                StepResult::WaitingInput => {
                    let payload = StepCollector::collect(vm, session, step);
                    if step as usize == self.frame_cache.len() {
                        self.frame_cache.push(payload.clone());
                    }
                    return SeekResult {
                        success: true,
                        payload: Some(payload),
                        error: None,
                    };
                }
                StepResult::Finished => {
                    let payload = StepCollector::collect(vm, session, step);
                    if step as usize == self.frame_cache.len() {
                        self.frame_cache.push(payload.clone());
                    }
                    return SeekResult {
                        success: true,
                        payload: Some(payload),
                        error: None,
                    };
                }
                StepResult::Trap => {
                    return SeekResult {
                        success: false,
                        payload: None,
                        error: Some(format!("运行时错误：{}", vm.get_error())),
                    };
                }
            }
        }

        // 截断 frame_cache，丢弃 target 之后的旧数据（如果存在）
        self.frame_cache.truncate((target + 1) as usize);

        // 返回目标步的 payload
        match self.frame_cache.get(target as usize).cloned() {
            Some(payload) => SeekResult {
                success: true,
                payload: Some(payload),
                error: None,
            },
            None => SeekResult {
                success: false,
                payload: None,
                error: Some("无法获取目标步的 payload".to_string()),
            },
        }
    }

    /// 获取指定范围的 StepPayload（用于前端 FrameCache 批量回填）。
    pub fn get_payloads(&self, start: i32, end: i32) -> Vec<StepPayload> {
        let start = start.max(0) as usize;
        let end = (end as usize).min(self.frame_cache.len());
        if start < end {
            self.frame_cache[start..end].to_vec()
        } else {
            Vec::new()
        }
    }

    /// 获取当前已收集的最大步数。
    pub fn max_collected_step(&self) -> i32 {
        self.frame_cache.len().saturating_sub(1) as i32
    }
}
