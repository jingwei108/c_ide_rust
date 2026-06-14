use crate::session::Session;
use crate::unified::types::StepMeta;
use crate::vm::core::CideVM;
use crate::vm::snapshot::VMSnapshot;

/// 检查点管理器：定期保存 VM 快照，用于 Seek 恢复。
///
/// 支持两种快照模式：
/// - **全量快照**：每 `full_every` 个检查点保存一次完整 1MB 内存。
/// - **增量快照**：其余检查点仅保存自上一个全量检查点以来被修改的 4KB 页。
///
/// 增量快照可将单检查点内存从 ~1MB 降至典型 ~50-200KB（取决于程序行为），
/// 在 50 个检查点上限下整体内存占用从 50MB 降至约 5-10MB。
pub struct CheckpointManager {
    pub checkpoints: Vec<(i32, VMSnapshot)>,
    /// 固定间隔（步数），保底策略。
    pub interval: i32,
    /// 智能检查点：在控制流边界（函数调用/返回/循环/交换/内存操作）额外保存。
    pub smart_mode: bool,
    /// 最大检查点数量，防止长程序运行时内存无限增长。
    pub max_checkpoints: usize,
    /// 每 N 个检查点强制一个全量基准。
    pub full_every: usize,
}

impl CheckpointManager {
    pub fn new(interval: i32) -> Self {
        Self {
            checkpoints: Vec::new(),
            interval,
            smart_mode: true,
            max_checkpoints: 50,
            full_every: 5,
        }
    }

    /// 判断当前步是否需要保存检查点。
    ///
    /// 固定间隔保底 + 智能模式在控制流边界触发。
    pub fn should_checkpoint(&self, step: i32, meta: &StepMeta) -> bool {
        // 固定间隔保底
        if step % self.interval == 0 {
            return true;
        }

        if !self.smart_mode {
            return false;
        }

        // 智能策略：函数调用、返回、循环边界、数组交换、内存操作
        let label = &meta.semantic_label;
        let is_significant = label.starts_with("调用 ")
            || label == "返回"
            || label == "内存分配"
            || label == "释放内存"
            || label.contains("交换")
            || label.starts_with("循环");

        if !is_significant {
            return false;
        }

        // 避免检查点过于密集：距离上一个检查点至少 interval/4 步
        let min_gap = self.interval.max(4) / 4;
        if let Some((last_step, _)) = self.checkpoints.last() {
            if step - *last_step < min_gap {
                return false;
            }
        }

        true
    }

    /// 保存检查点。超过上限时自动移除最旧的检查点。
    ///
    /// 调用者负责在保存前通过 `should_checkpoint` 判断是否需要保存。
    pub fn save(&mut self, step: i32, vm: &mut CideVM, session: &Session) {
        let is_full = self.checkpoints.is_empty() || self.checkpoints.len().is_multiple_of(self.full_every);

        let snap = if is_full {
            vm.clear_dirty_pages();
            vm.snapshot(session)
        } else {
            let base_step = self
                .checkpoints
                .iter()
                .rev()
                .find(|(_, s)| matches!(s.memory, crate::vm::snapshot::MemoryImage::Full(_)))
                .map(|(s, _)| *s)
                .unwrap_or(0);
            vm.snapshot_incremental(session, base_step)
        };

        self.checkpoints.push((step, snap));

        // 移除最旧检查点；如果移除的是全量基准，需要把下一个全量之前的增量全删掉，
        // 否则增量会 dangling。简化处理：一直删到第一个是全量为止。
        while self.checkpoints.len() > self.max_checkpoints {
            let removed_is_full = matches!(self.checkpoints[0].1.memory, crate::vm::snapshot::MemoryImage::Full(_));
            self.checkpoints.remove(0);
            if !removed_is_full {
                // 如果删掉的是增量，继续删到下一个全量，保证链头是全量基准
                while !self.checkpoints.is_empty()
                    && !matches!(self.checkpoints[0].1.memory, crate::vm::snapshot::MemoryImage::Full(_))
                {
                    self.checkpoints.remove(0);
                }
            }
        }
    }

    /// 找到不超过 target 的最近检查点，并重建为可直接恢复的全量快照。
    pub fn nearest(&self, target: i32) -> Option<(i32, VMSnapshot)> {
        let idx = self.checkpoints.iter().rposition(|(s, _)| *s <= target)?;
        let (step, snap) = &self.checkpoints[idx];

        match &snap.memory {
            crate::vm::snapshot::MemoryImage::Full(_) => Some((*step, snap.clone())),
            crate::vm::snapshot::MemoryImage::Delta { base_step, .. } => {
                // 找到基础全量检查点
                let base_idx = self.checkpoints.iter().rposition(|(s, snap)| {
                    *s <= *base_step && matches!(snap.memory, crate::vm::snapshot::MemoryImage::Full(_))
                })?;
                let base_snap = &self.checkpoints[base_idx].1;

                // 从 base 到 target 之间的所有增量应用到基础内存
                let mut full_memory = match &base_snap.memory {
                    crate::vm::snapshot::MemoryImage::Full(m) => m.clone(),
                    _ => return None, // 不应该发生
                };

                for (_, intermediate) in &self.checkpoints[base_idx + 1..=idx] {
                    intermediate.memory.apply_to(&mut full_memory);
                }

                let mut reconstructed = snap.clone();
                reconstructed.memory = crate::vm::snapshot::MemoryImage::Full(full_memory);
                Some((*step, reconstructed))
            }
        }
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
