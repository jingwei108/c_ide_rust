use crate::session::Session;
use crate::unified::types::StepMeta;
use crate::vm::snapshot::VMSnapshot;
use crate::vm::vm::CideVM;

/// 检查点管理器：定期保存 VM 全量快照，用于 Seek 恢复。
pub struct CheckpointManager {
    pub checkpoints: Vec<(i32, VMSnapshot)>,
    pub interval: i32,
    pub smart_mode: bool,
}

impl CheckpointManager {
    pub fn new(interval: i32) -> Self {
        Self {
            checkpoints: Vec::new(),
            interval,
            smart_mode: true,
        }
    }

    /// 判断当前步是否需要保存检查点。
    ///
    /// MVP 阶段仅使用固定间隔；智能模式在后续迭代中启用。
    pub fn should_checkpoint(&self, step: i32, _meta: &StepMeta) -> bool {
        if step % self.interval == 0 {
            return true;
        }
        // TODO: 智能检查点（循环边界、函数调用、数组交换）
        false
    }

    /// 保存检查点。
    pub fn save(&mut self, step: i32, vm: &CideVM, session: &Session) {
        self.checkpoints.push((step, vm.snapshot(session)));
    }

    /// 找到不超过 target 的最近检查点。
    pub fn nearest(&self, target: i32) -> Option<(i32, &VMSnapshot)> {
        self.checkpoints
            .iter()
            .rfind(|(s, _)| *s <= target)
            .map(|(step, snap)| (*step, snap))
    }

    /// 清除所有检查点。
    pub fn clear(&mut self) {
        self.checkpoints.clear();
    }

    /// 获取当前保存的检查点数量。
    pub fn len(&self) -> usize {
        self.checkpoints.len()
    }

    pub fn is_empty(&self) -> bool {
        self.checkpoints.is_empty()
    }
}
