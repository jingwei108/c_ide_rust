#![allow(clippy::unwrap_used, clippy::expect_used)]

use cide_native::engine::compile_pipeline::{run_multi_file_pipeline, setup_vm};
use cide_native::engine::session_ops::{execute_run, reset_runtime};
use cide_native::session::{CompileUnit, Session};
use std::time::Instant;

fn bench_source(source: &str) -> (f64, f64, i32) {
    // JIT 版本
    let mut session = Session::default();
    session.compile.compile_units.push(CompileUnit {
        filename: "main.c".to_string(),
        source: source.to_string(),
    });
    reset_runtime(&mut session);
    let units = session.compile.compile_units.clone();
    run_multi_file_pipeline(&mut session, units, false).expect("compile");

    let start = Instant::now();
    let result = execute_run(&mut session);
    let jit_time = start.elapsed().as_secs_f64();
    let ret = result.expect("run").0;
    let _jit_steps = session.vm.as_ref().map(|vm| vm.jit_stats().steps_accelerated).unwrap_or(0);

    // 纯解释版本：通过清空 jit_traces 来禁用 JIT
    let mut session2 = Session::default();
    session2.compile.compile_units.push(CompileUnit {
        filename: "main.c".to_string(),
        source: source.to_string(),
    });
    reset_runtime(&mut session2);
    let units2 = session2.compile.compile_units.clone();
    run_multi_file_pipeline(&mut session2, units2, false).expect("compile");
    {
        let mut vm = session2.vm.take().unwrap_or_default();
        setup_vm(&mut vm, &session2);
        cide_native::engine::session_ops::inject_preset_files(&mut vm, &mut session2);
        // 禁用 JIT：清空所有 trace
        vm.jit_traces_mut().clear();
        let start2 = Instant::now();
        let ret2 = vm.run(&mut session2.as_vm_context());
        let interp_time = start2.elapsed().as_secs_f64();
        session2.vm = Some(vm);
        assert_eq!(ret, ret2, "JIT and interpreter should produce same return code");
        (jit_time, interp_time, ret)
    }
}

#[test]
fn bench_nested_loop_1k() {
    let source = r#"
#include <stdio.h>
int main() {
    int sum = 0;
    for (int i = 0; i < 1000; i++) {
        for (int j = 0; j < 1000; j++) {
            sum = sum + 1;
        }
    }
    printf("%d", sum);
    return 0;
}
"#;
    let (jit_time, interp_time, ret) = bench_source(source);
    println!(
        "[BENCH] nested_loop_1k: JIT={:.4}s, interp={:.4}s, speedup={:.2}x, ret={}",
        jit_time,
        interp_time,
        interp_time / jit_time.max(0.0001),
        ret
    );
}

#[test]
fn bench_factorial_recursive_10() {
    let source = r#"
#include <stdio.h>
int fact(int n) {
    if (n <= 1) return 1;
    return n * fact(n - 1);
}
int main() {
    printf("%d", fact(10));
    return 0;
}
"#;
    let (jit_time, interp_time, ret) = bench_source(source);
    println!(
        "[BENCH] factorial_recursive_10: JIT={:.4}s, interp={:.4}s, speedup={:.2}x, ret={}",
        jit_time,
        interp_time,
        interp_time / jit_time.max(0.0001),
        ret
    );
}

#[test]
fn bench_array_sum_200k() {
    let source = r#"
#include <stdio.h>
int main() {
    int sum = 0;
    for (int i = 0; i < 200000; i++) {
        sum = sum + 1;
    }
    printf("%d", sum);
    return 0;
}
"#;
    let (jit_time, interp_time, ret) = bench_source(source);
    println!(
        "[BENCH] array_sum_200k: JIT={:.4}s, interp={:.4}s, speedup={:.2}x, ret={}",
        jit_time,
        interp_time,
        interp_time / jit_time.max(0.0001),
        ret
    );
}
