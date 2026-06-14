//! 跨 compiler/session/vm 共享的基础类型与工具函数。
//!
//! 设计原则：
//! - `source_loc` 不依赖任何上层模块，供 `compiler::ast` 和 `vm::instruction` re-export。
//! - `func_meta` / `symbol` / `type_utils` 依赖 `compiler::ast::Type`，
//!   但 `compiler::ast` 只 re-export `source_loc`，避免循环依赖。

pub mod func_meta;
pub mod source_loc;
pub mod symbol;
pub mod type_utils;

pub use func_meta::FuncMeta;
pub use source_loc::SourceLoc;
pub use symbol::Symbol;
pub use type_utils::base_kind;
