// TODO(#D08): 当前测试代码与 FRB 生成代码中存在大量 unwrap/expect，待逐步收敛后移除全局豁免。
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
