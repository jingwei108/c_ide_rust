// 生产代码 unwrap/expect 已收敛并均附 SAFETY 注释；测试代码使用 unwrap/expect 属于常见实践，
// 通过 cfg_attr(test) 统一豁免，避免在每个测试用例上重复 #[allow]。
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

#[cfg(target_arch = "wasm32")]
extern crate console_error_panic_hook;

/// WASM 模块加载时自动初始化 panic hook，将 Rust panic 信息输出到浏览器控制台。
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn wasm_start() {
    console_error_panic_hook::set_once();
}

pub mod api;
pub mod capi;
pub mod compiler;
pub mod diagnostics;
pub mod engine;
pub mod flutter_bridge;
// FRB 生成文件会在构建时重新生成，其 unwrap/expect 由生成器控制，项目级 lint 不约束生成代码。
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */
pub mod session;
pub mod shared;
pub mod unified;
pub mod vm;
