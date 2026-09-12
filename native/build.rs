//! 构建期注入 git 短哈希 → `CIDE_GIT_HASH`（`cide_engine_version()` 消费，
//! S5 A4 / 回放纪律 #2 的版本锚定依赖：版本串必须含锚定 commit）。
use std::process::Command;

fn main() {
    let hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=CIDE_GIT_HASH={}", hash);
    // 哈希随每次提交变化：任何源文件变更都重跑（无 rerun-if 指令的默认行为即此）
}
