//! Execution-trace slicing and root-cause inference (P0).
//!
//! When a trap occurs in unified mode, `TraceAnalyzer` looks back through the
//! collected `StepPayload` history, slices variable timelines, and produces a
//! human-readable `RootCauseHint`.

pub mod bounds;
pub mod div_zero;
pub mod double_free;
pub mod null_deref;
pub mod use_after_free;
pub mod utils;

#[cfg(test)]
mod tests;

use crate::session::Session;
use crate::unified::root_cause::RootCauseHint;
use crate::unified::trace_analyzer::bounds::analyze_bounds;
use crate::unified::trace_analyzer::div_zero::analyze_div_zero;
use crate::unified::trace_analyzer::double_free::analyze_double_free;
use crate::unified::trace_analyzer::null_deref::analyze_null_deref;
use crate::unified::trace_analyzer::use_after_free::analyze_use_after_free;
use crate::unified::types::StepPayload;

pub struct TraceAnalyzer;

impl TraceAnalyzer {
    /// Analyze a runtime trap and return a structured root-cause hint.
    ///
    /// * `steps`       – all previously collected steps (including the trap step).
    /// * `trap_step`   – index (in `steps`) of the step where the trap happened.
    /// * `trap_message`– VM error message (e.g. bounds, UAF, div-by-zero).
    /// * `session`     – current session (for source code inspection).
    pub fn analyze_trap(
        steps: &[StepPayload],
        trap_step: usize,
        trap_message: &str,
        session: &Session,
    ) -> Option<RootCauseHint> {
        if trap_message.contains("数组越界") {
            analyze_bounds(trap_message, steps, trap_step, session)
        } else if trap_message.contains("Use-After-Free") || trap_message.contains("E3060") {
            analyze_use_after_free(trap_message, steps, trap_step)
        } else if trap_message.contains("Double-Free") || trap_message.contains("E3061") {
            analyze_double_free(trap_message, steps, trap_step)
        } else if trap_message.contains("除零") || trap_message.contains("除以零") {
            analyze_div_zero(trap_message, steps, trap_step, session)
        } else if trap_message.contains("NULL") || trap_message.contains("null") {
            analyze_null_deref(trap_message, steps, trap_step)
        } else {
            None
        }
    }
}
